use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local, NaiveDate};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// 系统痕迹扫描器
///
/// 负责扫描 Windows 系统中可能包含个人痕迹的位置：
/// - 最近打开文档列表（%APPDATA%\Microsoft\Windows\Recent\）
/// - 用户级 Temp 文件夹
/// - 缩略图缓存（thumbcache_*.db）
pub struct SystemScanner;

impl SystemScanner {
    pub fn new() -> Self {
        SystemScanner
    }

    /// 获取文件修改日期（NaiveDate）
    fn modified_date(path: &Path) -> Option<NaiveDate> {
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        let datetime: DateTime<Local> = modified.into();
        Some(datetime.date_naive())
    }

    /// 获取文件修改时间（DateTime<Local>）
    fn modified_datetime(path: &Path) -> Option<DateTime<Local>> {
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        Some(modified.into())
    }

    /// 对文件名进行简单哈希，用于生成唯一 ID
    fn hash_filename(name: &str) -> String {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 扫描最近打开文档列表
    ///
    /// 遍历 `%APPDATA%\Microsoft\Windows\Recent\` 中的 `.lnk` 快捷方式文件，
    /// 按修改时间过滤（>= start_date），每个符合条件的 .lnk 生成一条 TraceItem。
    fn scan_recent_docs(
        recent_dir: &Path,
        start_date: NaiveDate,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut items = Vec::new();

        if !recent_dir.exists() || !recent_dir.is_dir() {
            return Ok(items);
        }

        // 收集所有 .lnk 文件
        let entries: Vec<_> = std::fs::read_dir(recent_dir)
            .map_err(ScanError::IoError)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.is_file()
                    && path
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("lnk"))
            })
            .collect();

        let total = entries.len();
        let mut processed = 0;
        let mut report_counter = 0;

        for entry in entries {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // 入职日期过滤：文件修改日期 >= start_date
            if let Some(file_date) = Self::modified_date(&path) {
                if file_date < start_date {
                    continue;
                }
            }

            let modified = Self::modified_datetime(&path);

            items.push(TraceItem {
                id: format!("recent-{}", file_name),
                category: TraceCategory::System,
                scanner_id: "scanner-system".to_string(),
                name: format!("最近文档: {}", file_name),
                path: Some(path),
                size_bytes: None,
                modified_at: modified,
                inferred: false,
                risk_note: Some("系统记录的最近打开文档列表".to_string()),
                suggested_action: Some(Action::Delete),
            });

            processed += 1;
            report_counter += 1;

            // 每处理 50 个文件报告一次进度
            if report_counter >= 50 {
                progress(ScanProgress {
                    scanner_id: "scanner-system".to_string(),
                    current: processed,
                    total,
                    message: format!("已扫描 {} 个最近文档", processed),
                    global_percent: None,
                });
                report_counter = 0;
            }
        }

        // 报告最终进度
        if processed > 0 {
            progress(ScanProgress {
                scanner_id: "scanner-system".to_string(),
                current: processed,
                total,
                message: format!("最近文档扫描完成，共 {} 个", processed),
                global_percent: None,
            });
        }

        Ok(items)
    }

    /// 扫描用户级 Temp 文件夹
    ///
    /// 只遍历 Temp 目录的直接子文件（不递归），按修改时间过滤（>= start_date）。
    fn scan_temp_dir(
        temp_dir: &Path,
        start_date: NaiveDate,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut items = Vec::new();

        if !temp_dir.exists() || !temp_dir.is_dir() {
            return Ok(items);
        }

        // 只遍历一层，避免 Temp 目录过深导致结果爆炸
        let entries: Vec<_> = std::fs::read_dir(temp_dir)
            .map_err(ScanError::IoError)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        let total = entries.len();
        let mut processed = 0;
        let mut report_counter = 0;

        for entry in entries {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // 入职日期过滤
            if let Some(file_date) = Self::modified_date(&path) {
                if file_date < start_date {
                    continue;
                }
            }

            let modified = Self::modified_datetime(&path);
            let size = entry.metadata().ok().map(|m| m.len());

            items.push(TraceItem {
                id: format!("temp-{}", Self::hash_filename(&file_name)),
                category: TraceCategory::System,
                scanner_id: "scanner-system".to_string(),
                name: format!("临时文件: {}", file_name),
                path: Some(path),
                size_bytes: size,
                modified_at: modified,
                inferred: false,
                risk_note: None,
                suggested_action: Some(Action::Delete),
            });

            processed += 1;
            report_counter += 1;

            // 每处理 50 个文件报告一次进度
            if report_counter >= 50 {
                progress(ScanProgress {
                    scanner_id: "scanner-system".to_string(),
                    current: processed,
                    total,
                    message: format!("已扫描 {} 个临时文件", processed),
                    global_percent: None,
                });
                report_counter = 0;
            }
        }

        // 报告最终进度
        if processed > 0 {
            progress(ScanProgress {
                scanner_id: "scanner-system".to_string(),
                current: processed,
                total,
                message: format!("临时文件扫描完成，共 {} 个", processed),
                global_percent: None,
            });
        }

        Ok(items)
    }

    /// 扫描缩略图缓存
    ///
    /// 检测 `%LOCALAPPDATA%\Microsoft\Windows\Explorer\` 下是否存在 `thumbcache_*.db` 文件，
    /// 若存在则返回单条汇总 TraceItem（整目录标记）。
    fn scan_thumbcache(local_appdata: &Path) -> Option<TraceItem> {
        let explorer_dir = local_appdata.join("Microsoft").join("Windows").join("Explorer");

        if !explorer_dir.exists() || !explorer_dir.is_dir() {
            return None;
        }

        // 检测是否存在 thumbcache_*.db 文件
        let has_thumbcache = std::fs::read_dir(&explorer_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .any(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.starts_with("thumbcache_") && name.ends_with(".db")
            });

        if !has_thumbcache {
            return None;
        }

        Some(TraceItem {
            id: "thumbcache".to_string(),
            category: TraceCategory::System,
            scanner_id: "scanner-system".to_string(),
            name: "缩略图缓存".to_string(),
            path: Some(explorer_dir),
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: Some("包含你浏览过的图片缩略图".to_string()),
            suggested_action: Some(Action::Delete),
        })
    }
}

impl Scanner for SystemScanner {
    fn id(&self) -> &'static str {
        "scanner-system"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::System
    }

    fn display_name(&self) -> &'static str {
        "系统痕迹"
    }

    fn scan(
        &self,
        ctx: &ScanContext,
        _pause_rx: &tokio::sync::watch::Receiver<bool>,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut all_items = Vec::new();

        // a) 扫描最近打开文档列表
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        if !appdata.is_empty() {
            let recent_dir = PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent");
            let recent_items = Self::scan_recent_docs(&recent_dir, ctx.start_date, progress)?;
            all_items.extend(recent_items);
        }

        // b) 扫描用户级 Temp 文件夹
        // 优先使用 ctx.temp_dir，若不可用则回退到 std::env::temp_dir()
        let temp_dir = if ctx.temp_dir.exists() {
            ctx.temp_dir.clone()
        } else {
            std::env::temp_dir()
        };
        let temp_items = Self::scan_temp_dir(&temp_dir, ctx.start_date, progress)?;
        all_items.extend(temp_items);

        // c) 扫描缩略图缓存
        let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        if !local_appdata.is_empty() {
            let local_appdata_path = PathBuf::from(&local_appdata);
            if let Some(thumb_item) = Self::scan_thumbcache(&local_appdata_path) {
                all_items.push(thumb_item);
            }
        }

        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_scanner_trait_compiles() {
        let scanner = SystemScanner::new();
        assert_eq!(scanner.id(), "scanner-system");
        assert_eq!(scanner.category(), TraceCategory::System);
        assert_eq!(scanner.display_name(), "系统痕迹");
    }

    #[test]
    fn test_recent_docs_lnk_filter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lnk_file = temp_dir.path().join("test.txt.lnk");
        std::fs::File::create(&lnk_file).unwrap();
        let non_lnk = temp_dir.path().join("readme.txt");
        std::fs::File::create(&non_lnk).unwrap();

        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let items = SystemScanner::scan_recent_docs(temp_dir.path(), start_date, &|_| {}).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].name.contains("test.txt.lnk"));
    }

    #[test]
    fn test_temp_dir_scan_non_recursive() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::File::create(temp_dir.path().join("file1.txt")).unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::File::create(temp_dir.path().join("subdir/file2.txt")).unwrap();

        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let items = SystemScanner::scan_temp_dir(temp_dir.path(), start_date, &|_| {}).unwrap();
        // 只扫描直接子文件，不递归
        assert_eq!(items.len(), 1);
        assert!(items[0].name.contains("file1.txt"));
    }

    #[test]
    fn test_hash_filename_stable() {
        let h1 = SystemScanner::hash_filename("test.lnk");
        let h2 = SystemScanner::hash_filename("test.lnk");
        assert_eq!(h1, h2);
        // 不同文件名应产生不同哈希（概率极高）
        let h3 = SystemScanner::hash_filename("other.lnk");
        assert_ne!(h1, h3);
    }
}
