use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local, NaiveDate};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use walkdir::DirEntry;

/// 文件系统扫描器
///
/// 负责扫描 Desktop、Downloads 中的个人文件，并特殊处理微信聊天记录目录。
/// 微信目录会被整体标记为单条 TraceItem（不递归列出内部文件），符合 RULE-03。
pub struct FileSystemScanner;

/// French Exit 自身所在目录缓存（避免扫描到自身文件）
static SELF_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

impl FileSystemScanner {
    pub fn new() -> Self {
        FileSystemScanner
    }

    /// 获取 French Exit 自身所在目录
    fn self_dir() -> Option<&'static PathBuf> {
        SELF_DIR
            .get_or_init(|| std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())))
            .as_ref()
    }

    /// 判断路径是否为系统关键目录（禁止扫描）
    fn is_system_path(path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        // 排除 Windows 系统目录
        if path_str.starts_with("c:\\windows")
            || path_str.starts_with("c:\\program files")
            || path_str.starts_with("c:\\programdata")
        {
            return true;
        }
        false
    }

    /// 判断路径是否为 French Exit 自身所在目录
    fn is_self_directory(path: &Path) -> bool {
        if let Some(self_dir) = Self::self_dir() {
            if let Ok(canonical_self) = std::fs::canonicalize(self_dir) {
                if let Ok(canonical_path) = std::fs::canonicalize(path) {
                    return canonical_path.starts_with(&canonical_self);
                }
            }
        }
        false
    }

    /// 判断是否为系统隐藏文件/目录
    fn is_hidden(entry: &DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();
        name.starts_with('.')
    }

    /// 判断是否为已知系统文件（如虚拟内存文件）
    fn is_system_file(entry: &DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        matches!(name.as_str(), "pagefile.sys" | "hiberfil.sys" | "swapfile.sys")
    }

    /// 综合排除判断：遇到以下目录/文件直接跳过
    fn should_skip(entry: &DirEntry) -> bool {
        let path = entry.path();

        // 1. 系统目录
        if Self::is_system_path(path) {
            return true;
        }

        // 2. 自身目录
        if Self::is_self_directory(path) {
            return true;
        }

        // 3. 隐藏文件/目录
        if Self::is_hidden(entry) {
            return true;
        }

        // 4. 已知系统文件
        if Self::is_system_file(entry) {
            return true;
        }

        false
    }

    /// 递归计算目录总大小（字节）
    fn dir_size(path: &Path) -> u64 {
        walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
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

    /// 生成文件唯一 ID（基于路径的简单哈希）
    fn generate_file_id(path: &Path) -> String {
        let mut hasher = DefaultHasher::new();
        path.to_string_lossy().hash(&mut hasher);
        format!("fs-{:x}", hasher.finish())
    }

    /// 扫描微信聊天记录目录
    ///
    /// 探测 `%USERPROFILE%\Documents\WeChat Files`，对其下的 `wxid_xxx` 子目录
    /// 每个生成一条 TraceItem，category 为 Chat，suggested_action 为 DeleteOrPack。
    fn scan_wechat(
        &self,
        ctx: &ScanContext,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut items = Vec::new();
        let wechat_dir = ctx.user_home.join("Documents").join("WeChat Files");

        if !wechat_dir.exists() || !wechat_dir.is_dir() {
            return Ok(items);
        }

        // 遍历 WeChat Files 的直接子目录（通常是 wxid_xxx）
        let entries = std::fs::read_dir(&wechat_dir).map_err(ScanError::IoError)?;

        let mut current = 0;
        let total = entries.count();
        // 重新读取，因为 count() 消耗了迭代器
        let entries = std::fs::read_dir(&wechat_dir).map_err(ScanError::IoError)?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("wxid_") {
                continue;
            }

            let wxid = name.clone();
            let size = Self::dir_size(&path);
            let modified = Self::modified_datetime(&path);

            items.push(TraceItem {
                id: format!("wechat-{}", wxid),
                category: TraceCategory::Chat,
                scanner_id: "scanner-fs".to_string(),
                name: format!("微信聊天记录: {}", wxid),
                path: Some(path),
                size_bytes: Some(size),
                modified_at: modified,
                inferred: false,
                risk_note: Some("微信聊天记录属于私人内容，建议处理。".to_string()),
                suggested_action: Some(Action::DeleteOrPack), // RULE-03
            });

            current += 1;
            progress(ScanProgress {
                scanner_id: "scanner-fs".to_string(),
                current,
                total,
                message: format!("发现微信聊天记录: {}", wxid),
            });
        }

        Ok(items)
    }

    /// 扫描普通文件目录（Desktop / Downloads）
    ///
    /// 递归遍历，排除系统目录/自身目录/隐藏文件，过滤入职日期前的文件。
    fn scan_directory(
        &self,
        ctx: &ScanContext,
        dir: &Path,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut items = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(items);
        }

        // 先估算总文件数（粗略），用于进度条
        let total_estimate: usize = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        let mut processed = 0;
        let mut report_counter = 0;

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // 综合排除判断
            if Self::should_skip(&entry) {
                continue;
            }

            // 只处理文件
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            // 入职日期过滤：文件修改日期 >= start_date
            if let Some(file_date) = Self::modified_date(path) {
                if file_date < ctx.start_date {
                    continue;
                }
            }
            // 如果无法获取修改时间，默认保留（不过滤掉）

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_size = metadata.len();
            let modified = Self::modified_datetime(path);

            items.push(TraceItem {
                id: Self::generate_file_id(path),
                category: TraceCategory::FileSystem,
                scanner_id: "scanner-fs".to_string(),
                name: file_name,
                path: Some(path.to_path_buf()),
                size_bytes: Some(file_size),
                modified_at: modified,
                inferred: false,
                risk_note: None,
                suggested_action: Some(Action::DeleteOrPack),
            });

            processed += 1;
            report_counter += 1;

            // 每扫描 10 个文件报告一次进度，避免回调过于频繁
            if report_counter >= 10 {
                progress(ScanProgress {
                    scanner_id: "scanner-fs".to_string(),
                    current: processed.min(total_estimate),
                    total: total_estimate,
                    message: format!("已扫描 {} 个文件", processed),
                });
                report_counter = 0;
            }
        }

        // 最终进度
        if processed > 0 {
            progress(ScanProgress {
                scanner_id: "scanner-fs".to_string(),
                current: processed,
                total: total_estimate.max(processed),
                message: format!("目录扫描完成，共 {} 个文件", processed),
            });
        }

        Ok(items)
    }
}

impl Scanner for FileSystemScanner {
    fn id(&self) -> &'static str {
        "scanner-fs"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::FileSystem
    }

    fn display_name(&self) -> &'static str {
        "个人文件"
    }

    fn scan(
        &self,
        ctx: &ScanContext,
        _pause_rx: &tokio::sync::watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut all_items = Vec::new();

        // 步骤 1 & 2：优先扫描微信目录（RULE-03）
        let wechat_items = self.scan_wechat(ctx, progress)?;
        all_items.extend(wechat_items);

        // 步骤 3：扫描 Desktop 和 Downloads
        let scan_paths = [
            ctx.user_home.join("Desktop"),
            ctx.user_home.join("Downloads"),
        ];

        for path in &scan_paths {
            if !path.exists() {
                continue;
            }

            let items = self.scan_directory(ctx, path, progress)?;
            all_items.extend(items);
        }

        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_fs_scanner_trait_compiles() {
        let scanner = FileSystemScanner::new();
        assert_eq!(scanner.id(), "scanner-fs");
        assert_eq!(scanner.category(), TraceCategory::FileSystem);
        assert_eq!(scanner.display_name(), "个人文件");
    }

    #[test]
    fn test_fs_excludes_system_paths() {
        assert!(FileSystemScanner::is_system_path(Path::new("C:\\Windows")));
        assert!(FileSystemScanner::is_system_path(Path::new("C:\\Program Files")));
        assert!(FileSystemScanner::is_system_path(Path::new("C:\\ProgramData")));
        assert!(!FileSystemScanner::is_system_path(Path::new("C:\\Users\\Test\\Desktop")));
        assert!(!FileSystemScanner::is_system_path(Path::new("D:\\Projects")));
    }

    #[test]
    fn test_fs_modified_date_filter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // 验证能获取到修改日期
        let date = FileSystemScanner::modified_date(&file_path);
        assert!(date.is_some());

        // 验证修改日期为今天或更早（不可能为未来）
        let today = Local::now().date_naive();
        assert!(date.unwrap() <= today);
    }

    #[test]
    fn test_fs_generate_file_id_stable() {
        let path = Path::new("C:\\Users\\Test\\file.txt");
        let id1 = FileSystemScanner::generate_file_id(path);
        let id2 = FileSystemScanner::generate_file_id(path);
        assert_eq!(id1, id2);
        assert!(id1.starts_with("fs-"));
    }
}
