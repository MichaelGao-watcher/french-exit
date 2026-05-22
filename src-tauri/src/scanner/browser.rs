use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Connection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// 浏览器扫描器
///
/// 负责检测 Chrome、Edge、Firefox 浏览器，扫描历史记录数据库。
/// Chrome 与 Edge 基于 Chromium 内核，共用相同 SQLite 结构；Firefox 独立实现。
pub struct BrowserScanner;

impl BrowserScanner {
    pub fn new() -> Self {
        BrowserScanner
    }

    // ── 浏览器检测 ───────────────────────────────────────────────

    /// 检测 Chrome 默认 Profile 路径
    /// 路径：`%LOCALAPPDATA%\Google\Chrome\User Data\Default`
    fn detect_chrome_path() -> Option<PathBuf> {
        let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
        let path = PathBuf::from(local_appdata)
            .join("Google")
            .join("Chrome")
            .join("User Data")
            .join("Default");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// 检测 Edge 默认 Profile 路径
    /// 路径：`%LOCALAPPDATA%\Microsoft\Edge\User Data\Default`
    fn detect_edge_path() -> Option<PathBuf> {
        let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
        let path = PathBuf::from(local_appdata)
            .join("Microsoft")
            .join("Edge")
            .join("User Data")
            .join("Default");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// 检测 Firefox 默认 Profile 目录（可测试版本，接受 appdata 路径）
    fn detect_firefox_profile_from(appdata: &Path) -> Option<PathBuf> {
        let firefox_dir = appdata.join("Mozilla").join("Firefox");
        let profiles_ini = firefox_dir.join("profiles.ini");
        if !profiles_ini.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&profiles_ini).ok()?;

        #[derive(Default)]
        struct ProfileInfo {
            path: Option<String>,
            is_relative: bool,
            is_default: bool,
        }

        let mut profiles: Vec<ProfileInfo> = Vec::new();
        let mut current = ProfileInfo::default();
        let mut in_profile = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("[Profile") {
                if in_profile {
                    profiles.push(std::mem::take(&mut current));
                }
                in_profile = true;
                current = ProfileInfo::default();
                current.is_relative = true; // Firefox 默认使用相对路径
            } else if in_profile {
                if let Some(val) = line.strip_prefix("Path=") {
                    current.path = Some(val.to_string());
                } else if let Some(val) = line.strip_prefix("IsRelative=") {
                    current.is_relative = val == "1";
                } else if line == "Default=1" {
                    current.is_default = true;
                }
            }
        }

        if in_profile {
            profiles.push(current);
        }

        // 优先返回 Default=1 的 Profile
        for profile in &profiles {
            if profile.is_default {
                if let Some(path) = &profile.path {
                    let full = if profile.is_relative {
                        firefox_dir.join(path)
                    } else {
                        PathBuf::from(path)
                    };
                    if full.exists() {
                        return Some(full);
                    }
                }
            }
        }

        // fallback：返回第一个存在的 Profile
        for profile in &profiles {
            if let Some(path) = &profile.path {
                let full = if profile.is_relative {
                    firefox_dir.join(path)
                } else {
                    PathBuf::from(path)
                };
                if full.exists() {
                    return Some(full);
                }
            }
        }

        None
    }

    /// 检测 Firefox 默认 Profile 目录
    /// 通过解析 `%APPDATA%\Mozilla\Firefox\profiles.ini` 获取 Default=1 的 Profile，
    /// 若无 Default 标记则取第一个存在的 Profile。
    fn detect_firefox_profile() -> Option<PathBuf> {
        let appdata = std::env::var("APPDATA").ok()?;
        Self::detect_firefox_profile_from(Path::new(&appdata))
    }

    // ── 时间转换 ─────────────────────────────────────────────────

    /// 将入职日期转换为 Chrome 时间（1601-01-01 00:00:00 UTC 以来的微秒数）
    fn naive_date_to_chrome_time(date: NaiveDate) -> i64 {
        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        (unix_secs + 11_644_473_600) * 1_000_000
    }

    /// Chrome 时间（微秒）→ DateTime<Local>
    fn chrome_time_to_datetime(chrome_time: i64) -> Option<DateTime<Local>> {
        let unix_secs = (chrome_time / 1_000_000) - 11_644_473_600;
        DateTime::from_timestamp(unix_secs, 0).map(|dt| dt.with_timezone(&Local))
    }

    /// 将入职日期转换为 Firefox 时间（1970-01-01 以来的微秒数）
    fn naive_date_to_firefox_time(date: NaiveDate) -> i64 {
        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        unix_secs * 1_000_000
    }

    /// Firefox 时间（微秒）→ DateTime<Local>
    fn firefox_time_to_datetime(firefox_time: i64) -> Option<DateTime<Local>> {
        let unix_secs = firefox_time / 1_000_000;
        DateTime::from_timestamp(unix_secs, 0).map(|dt| dt.with_timezone(&Local))
    }

    // ── 通用辅助 ─────────────────────────────────────────────────

    /// 基于 URL 生成唯一 ID
    fn generate_url_id(prefix: &str, url: &str) -> String {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("browser-{}-{:x}", prefix, hasher.finish())
    }

    /// 生成浏览器被占用时的提示性 TraceItem
    fn make_locked_item(browser_display: &str, path: &Path) -> TraceItem {
        TraceItem {
            id: format!(
                "browser-{}-locked",
                browser_display.to_lowercase().replace(' ', "-")
            ),
            category: TraceCategory::Browser,
            scanner_id: "scanner-browser".to_string(),
            name: format!("{} 历史记录（浏览器正在运行）", browser_display),
            path: Some(path.to_path_buf()),
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: Some("浏览器正在运行，无法读取完整记录".to_string()),
            suggested_action: Some(Action::Delete),
        }
    }

    // ── 历史记录扫描（Chromium 内核）──────────────────────────────

    /// 扫描 Chrome / Edge 的 History 数据库
    ///
    /// 读取 `urls` 表，按 `last_visit_time` 过滤入职日期之后的记录。
    /// 若数据库被浏览器进程锁定，返回包含 "被锁定" 的 Internal 错误，
    /// 由上层 `scan()` 转换为提示性 TraceItem。
    fn scan_chromium_history(
        path: &Path,
        start_date: NaiveDate,
        browser_prefix: &str,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let conn = Connection::open(path).map_err(|e| {
            // Chrome/Edge 正在运行时数据库会被锁定，捕获 SQLITE_BUSY 等错误
            if let rusqlite::Error::SqliteFailure(_, _) = e {
                ScanError::Internal(format!("{} 历史记录数据库被锁定", browser_prefix))
            } else {
                ScanError::Internal(format!(
                    "无法打开 {} 历史记录数据库: {}",
                    browser_prefix, e
                ))
            }
        })?;

        let min_time = Self::naive_date_to_chrome_time(start_date);

        // 先统计符合条件的记录总数，用于进度报告
        let total: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM urls WHERE last_visit_time > ?1",
                [min_time],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| ScanError::Internal(format!("查询计数失败: {}", e)))?
            as usize;

        if total == 0 {
            return Ok(Vec::new());
        }

        let mut stmt = conn
            .prepare(
                "SELECT url, title, last_visit_time \
                 FROM urls \
                 WHERE last_visit_time > ?1 \
                 ORDER BY last_visit_time DESC",
            )
            .map_err(|e| ScanError::Internal(format!("准备查询失败: {}", e)))?;

        let rows = stmt
            .query_map([min_time], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| ScanError::Internal(format!("执行查询失败: {}", e)))?;

        let mut items = Vec::with_capacity(total);
        let mut processed = 0;

        for row in rows {
            let (url, title, visit_time) =
                row.map_err(|e| ScanError::Internal(format!("读取行失败: {}", e)))?;

            let visit_datetime = Self::chrome_time_to_datetime(visit_time);
            let name = title.filter(|t| !t.is_empty()).unwrap_or_else(|| url.clone());
            // 标题截断到 100 个 Unicode 标量值，避免过长
            let name = if name.chars().count() > 100 {
                name.chars().take(100).collect()
            } else {
                name
            };

            items.push(TraceItem {
                id: Self::generate_url_id(browser_prefix, &url),
                category: TraceCategory::Browser,
                scanner_id: "scanner-browser".to_string(),
                name,
                path: Some(path.to_path_buf()),
                size_bytes: None,
                modified_at: visit_datetime,
                inferred: false,
                risk_note: Some("⚠️ 浏览器历史记录可能包含工作相关页面，建议清理".to_string()),
                suggested_action: Some(Action::Delete),
            });

            processed += 1;
            if processed % 100 == 0 {
                progress(ScanProgress {
                    scanner_id: "scanner-browser".to_string(),
                    current: processed,
                    total,
                    message: format!(
                        "已扫描 {} 条 {} 历史记录",
                        processed, browser_prefix
                    ),
                    global_percent: None,
                });
            }
        }

        Ok(items)
    }

    // ── 历史记录扫描（Firefox）───────────────────────────────────

    /// 扫描 Firefox 的 places.sqlite 数据库
    ///
    /// 读取 `moz_places` 表，按 `last_visit_date` 过滤入职日期之后的记录。
    fn scan_firefox_history(
        path: &Path,
        start_date: NaiveDate,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let conn = Connection::open(path).map_err(|e| {
            if let rusqlite::Error::SqliteFailure(_, _) = e {
                ScanError::Internal("firefox 历史记录数据库被锁定".to_string())
            } else {
                ScanError::Internal(format!(
                    "无法打开 Firefox 历史记录数据库: {}",
                    e
                ))
            }
        })?;

        let min_time = Self::naive_date_to_firefox_time(start_date);

        let total: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM moz_places WHERE last_visit_date > ?1",
                [min_time],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| ScanError::Internal(format!("查询计数失败: {}", e)))?
            as usize;

        if total == 0 {
            return Ok(Vec::new());
        }

        let mut stmt = conn
            .prepare(
                "SELECT url, title, last_visit_date \
                 FROM moz_places \
                 WHERE last_visit_date > ?1 \
                 ORDER BY last_visit_date DESC",
            )
            .map_err(|e| ScanError::Internal(format!("准备查询失败: {}", e)))?;

        let rows = stmt
            .query_map([min_time], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| ScanError::Internal(format!("执行查询失败: {}", e)))?;

        let mut items = Vec::with_capacity(total);
        let mut processed = 0;

        for row in rows {
            let (url, title, visit_time) =
                row.map_err(|e| ScanError::Internal(format!("读取行失败: {}", e)))?;

            let visit_datetime = Self::firefox_time_to_datetime(visit_time);
            let name = title.filter(|t| !t.is_empty()).unwrap_or_else(|| url.clone());
            let name = if name.chars().count() > 100 {
                name.chars().take(100).collect()
            } else {
                name
            };

            items.push(TraceItem {
                id: Self::generate_url_id("firefox", &url),
                category: TraceCategory::Browser,
                scanner_id: "scanner-browser".to_string(),
                name,
                path: Some(path.to_path_buf()),
                size_bytes: None,
                modified_at: visit_datetime,
                inferred: false,
                risk_note: Some("⚠️ 浏览器历史记录可能包含工作相关页面，建议清理".to_string()),
                suggested_action: Some(Action::Delete),
            });

            processed += 1;
            if processed % 100 == 0 {
                progress(ScanProgress {
                    scanner_id: "scanner-browser".to_string(),
                    current: processed,
                    total,
                    message: format!(
                        "已扫描 {} 条 Firefox 历史记录",
                        processed
                    ),
                    global_percent: None,
                });
            }
        }

        Ok(items)
    }
}

impl Scanner for BrowserScanner {
    fn id(&self) -> &'static str {
        "scanner-browser"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::Browser
    }

    fn display_name(&self) -> &'static str {
        "浏览器记录"
    }

    fn scan(
        &self,
        ctx: &ScanContext,
        _pause_rx: &tokio::sync::watch::Receiver<bool>,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut all_items = Vec::new();

        // 1. Chrome
        if let Some(chrome_dir) = Self::detect_chrome_path() {
            let history_path = chrome_dir.join("History");
            if history_path.exists() {
                match Self::scan_chromium_history(
                    &history_path,
                    ctx.start_date,
                    "chrome",
                    progress,
                ) {
                    Ok(items) => all_items.extend(items),
                    Err(ScanError::Internal(ref msg)) if msg.contains("被锁定") => {
                        all_items.push(Self::make_locked_item("Chrome", &history_path));
                    }
                    Err(_) => {
                        // 其他错误静默跳过，不中断整体扫描
                    }
                }
            }
        }

        // 2. Edge
        if let Some(edge_dir) = Self::detect_edge_path() {
            let history_path = edge_dir.join("History");
            if history_path.exists() {
                match Self::scan_chromium_history(
                    &history_path,
                    ctx.start_date,
                    "edge",
                    progress,
                ) {
                    Ok(items) => all_items.extend(items),
                    Err(ScanError::Internal(ref msg)) if msg.contains("被锁定") => {
                        all_items.push(Self::make_locked_item("Edge", &history_path));
                    }
                    Err(_) => {
                        // 其他错误静默跳过
                    }
                }
            }
        }

        // 3. Firefox
        if let Some(firefox_dir) = Self::detect_firefox_profile() {
            let places_path = firefox_dir.join("places.sqlite");
            if places_path.exists() {
                match Self::scan_firefox_history(&places_path, ctx.start_date, progress) {
                    Ok(items) => all_items.extend(items),
                    Err(ScanError::Internal(ref msg)) if msg.contains("被锁定") => {
                        all_items.push(Self::make_locked_item("Firefox", &places_path));
                    }
                    Err(_) => {
                        // 其他错误静默跳过
                    }
                }
            }
        }

        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_scanner_trait_compiles() {
        let scanner = BrowserScanner::new();
        assert_eq!(scanner.id(), "scanner-browser");
        assert_eq!(scanner.category(), TraceCategory::Browser);
        assert_eq!(scanner.display_name(), "浏览器记录");
    }

    #[test]
    fn test_chrome_time_conversion_roundtrip() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let chrome_time = BrowserScanner::naive_date_to_chrome_time(date);

        // 验证正向转换：Unix 时间戳 + 11644473600 秒，再转微秒
        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        assert_eq!(chrome_time, (unix_secs + 11_644_473_600) * 1_000_000);

        // 验证反向转换
        let dt = BrowserScanner::chrome_time_to_datetime(chrome_time).unwrap();
        assert_eq!(dt.date_naive(), date);
    }

    #[test]
    fn test_firefox_time_conversion_roundtrip() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let firefox_time = BrowserScanner::naive_date_to_firefox_time(date);

        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        assert_eq!(firefox_time, unix_secs * 1_000_000);

        let dt = BrowserScanner::firefox_time_to_datetime(firefox_time).unwrap();
        assert_eq!(dt.date_naive(), date);
    }

    #[test]
    fn test_generate_url_id_stable() {
        let id1 = BrowserScanner::generate_url_id("chrome", "https://example.com");
        let id2 = BrowserScanner::generate_url_id("chrome", "https://example.com");
        assert_eq!(id1, id2);
        assert!(id1.starts_with("browser-chrome-"));
    }

    // ── detect_firefox_profile_from 测试 ─────────────────────────

    #[test]
    fn test_detect_firefox_profile_default_relative() {
        let temp = tempfile::tempdir().unwrap();
        let firefox_dir = temp.path().join("Mozilla").join("Firefox");
        std::fs::create_dir_all(&firefox_dir).unwrap();

        let profile_dir = firefox_dir.join("profiles").join("abc123");
        std::fs::create_dir_all(&profile_dir).unwrap();

        let ini_content = "[Profile0]\nName=default\nIsRelative=1\nPath=profiles/abc123\nDefault=1\n";
        std::fs::write(firefox_dir.join("profiles.ini"), ini_content).unwrap();

        let result = BrowserScanner::detect_firefox_profile_from(temp.path());
        assert_eq!(result, Some(profile_dir));
    }

    #[test]
    fn test_detect_firefox_profile_fallback_first_existing() {
        let temp = tempfile::tempdir().unwrap();
        let firefox_dir = temp.path().join("Mozilla").join("Firefox");
        std::fs::create_dir_all(&firefox_dir).unwrap();

        let profile2 = firefox_dir.join("profiles").join("second");
        std::fs::create_dir_all(&profile2).unwrap();

        let ini_content = "[Profile0]\nName=missing\nIsRelative=1\nPath=profiles/missing\n\n[Profile1]\nName=second\nIsRelative=1\nPath=profiles/second\n";
        std::fs::write(firefox_dir.join("profiles.ini"), ini_content).unwrap();

        let result = BrowserScanner::detect_firefox_profile_from(temp.path());
        assert_eq!(result, Some(profile2));
    }

    #[test]
    fn test_detect_firefox_profile_absolute_path() {
        let temp = tempfile::tempdir().unwrap();
        let firefox_dir = temp.path().join("Mozilla").join("Firefox");
        std::fs::create_dir_all(&firefox_dir).unwrap();

        let abs_profile = temp.path().join("custom-profile");
        std::fs::create_dir_all(&abs_profile).unwrap();

        let ini_content = format!(
            "[Profile0]\nName=custom\nIsRelative=0\nPath={}\nDefault=1\n",
            abs_profile.display()
        );
        std::fs::write(firefox_dir.join("profiles.ini"), ini_content).unwrap();

        let result = BrowserScanner::detect_firefox_profile_from(temp.path());
        assert_eq!(result, Some(abs_profile));
    }

    #[test]
    fn test_detect_firefox_profile_no_ini_returns_none() {
        let temp = tempfile::tempdir().unwrap();
        let result = BrowserScanner::detect_firefox_profile_from(temp.path());
        assert_eq!(result, None);
    }

    // ── SQLite 历史记录扫描测试 ─────────────────────────────────

    #[test]
    fn test_scan_chromium_history_filters_by_date() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("History");

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE urls (url TEXT, title TEXT, last_visit_time INTEGER)",
            [],
        ).unwrap();

        let date_2024 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let chrome_time_2024 = BrowserScanner::naive_date_to_chrome_time(date_2024);
        conn.execute(
            "INSERT INTO urls (url, title, last_visit_time) VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://example.com", "Example", chrome_time_2024],
        ).unwrap();

        let date_2023 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let chrome_time_2023 = BrowserScanner::naive_date_to_chrome_time(date_2023);
        conn.execute(
            "INSERT INTO urls (url, title, last_visit_time) VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://old.com", "Old", chrome_time_2023],
        ).unwrap();

        let start_date = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
        let progress = |_p: ScanProgress| {};
        let items = BrowserScanner::scan_chromium_history(&db_path, start_date, "chrome", &progress).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Example");
        assert_eq!(items[0].category, TraceCategory::Browser);
        assert!(items[0].id.starts_with("browser-chrome-"));
        assert!(items[0].path.as_ref().unwrap().ends_with("History"));
    }

    #[test]
    fn test_scan_firefox_history_filters_by_date() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("places.sqlite");

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE moz_places (url TEXT, title TEXT, last_visit_date INTEGER)",
            [],
        ).unwrap();

        let date_2024 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let ff_time_2024 = BrowserScanner::naive_date_to_firefox_time(date_2024);
        conn.execute(
            "INSERT INTO moz_places (url, title, last_visit_date) VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://example.com", "Example", ff_time_2024],
        ).unwrap();

        let date_2023 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let ff_time_2023 = BrowserScanner::naive_date_to_firefox_time(date_2023);
        conn.execute(
            "INSERT INTO moz_places (url, title, last_visit_date) VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://old.com", "Old", ff_time_2023],
        ).unwrap();

        let start_date = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
        let progress = |_p: ScanProgress| {};
        let items = BrowserScanner::scan_firefox_history(&db_path, start_date, &progress).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Example");
        assert_eq!(items[0].category, TraceCategory::Browser);
        assert!(items[0].id.starts_with("browser-firefox-"));
    }

    #[test]
    fn test_scan_chromium_history_empty_db() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("History");

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE urls (url TEXT, title TEXT, last_visit_time INTEGER)",
            [],
        ).unwrap();

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let progress = |_p: ScanProgress| {};
        let items = BrowserScanner::scan_chromium_history(&db_path, start_date, "edge", &progress).unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_scan_chromium_history_title_truncation() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("History");

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE urls (url TEXT, title TEXT, last_visit_time INTEGER)",
            [],
        ).unwrap();

        let long_title = "a".repeat(200);
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let chrome_time = BrowserScanner::naive_date_to_chrome_time(date);
        conn.execute(
            "INSERT INTO urls (url, title, last_visit_time) VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://x.com", &long_title, chrome_time],
        ).unwrap();

        // start_date 必须早于数据日期，因为查询条件是 last_visit_time > start_date（严格大于）
        let start_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        let progress = |_p: ScanProgress| {};
        let items = BrowserScanner::scan_chromium_history(&db_path, start_date, "chrome", &progress).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name.chars().count(), 100);
    }

    // ── 辅助函数测试 ────────────────────────────────────────────

    #[test]
    fn test_make_locked_item() {
        let path = Path::new("C:\\History");
        let item = BrowserScanner::make_locked_item("Chrome", path);
        assert_eq!(item.id, "browser-chrome-locked");
        assert!(item.name.contains("Chrome"));
        assert!(item.risk_note.as_ref().unwrap().contains("正在运行"));
        assert_eq!(item.category, TraceCategory::Browser);
        assert_eq!(item.suggested_action, Some(Action::Delete));
    }
}
