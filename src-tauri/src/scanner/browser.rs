use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local, NaiveDate, TimeZone};
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

    /// 检测 Firefox 默认 Profile 目录
    /// 通过解析 `%APPDATA%\Mozilla\Firefox\profiles.ini` 获取 Default=1 的 Profile，
    /// 若无 Default 标记则取第一个存在的 Profile。
    fn detect_firefox_profile() -> Option<PathBuf> {
        let appdata = std::env::var("APPDATA").ok()?;
        let firefox_dir = PathBuf::from(&appdata).join("Mozilla").join("Firefox");
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

    // ── 时间转换 ─────────────────────────────────────────────────

    /// 将入职日期转换为 Chrome 时间（1601-01-01 00:00:00 UTC 以来的微秒数）
    fn naive_date_to_chrome_time(date: NaiveDate) -> i64 {
        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().timestamp();
        (unix_secs + 11_644_473_600) * 1_000_000
    }

    /// Chrome 时间（微秒）→ DateTime<Local>
    fn chrome_time_to_datetime(chrome_time: i64) -> Option<DateTime<Local>> {
        let unix_secs = (chrome_time / 1_000_000) - 11_644_473_600;
        match Local.timestamp_opt(unix_secs, 0) {
            chrono::LocalResult::Single(dt) => Some(dt),
            _ => None,
        }
    }

    /// 将入职日期转换为 Firefox 时间（1970-01-01 以来的微秒数）
    fn naive_date_to_firefox_time(date: NaiveDate) -> i64 {
        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().timestamp();
        unix_secs * 1_000_000
    }

    /// Firefox 时间（微秒）→ DateTime<Local>
    fn firefox_time_to_datetime(firefox_time: i64) -> Option<DateTime<Local>> {
        let unix_secs = firefox_time / 1_000_000;
        match Local.timestamp_opt(unix_secs, 0) {
            chrono::LocalResult::Single(dt) => Some(dt),
            _ => None,
        }
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
        progress: &dyn Fn(ScanProgress),
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
        progress: &dyn Fn(ScanProgress),
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
        progress: &dyn Fn(ScanProgress),
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
        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().timestamp();
        assert_eq!(chrome_time, (unix_secs + 11_644_473_600) * 1_000_000);

        // 验证反向转换
        let dt = BrowserScanner::chrome_time_to_datetime(chrome_time).unwrap();
        assert_eq!(dt.date_naive(), date);
    }

    #[test]
    fn test_firefox_time_conversion_roundtrip() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let firefox_time = BrowserScanner::naive_date_to_firefox_time(date);

        let unix_secs = date.and_hms_opt(0, 0, 0).unwrap().timestamp();
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
}
