use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local, NaiveDate};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
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
        let system_prefixes = [
            "c:\\windows",
            "c:\\program files",
            "c:\\program files (x86)",
            "c:\\programdata",
            "c:\\system32",
            "c:\\syswow64",
            "c:\\$recycle.bin",
            "c:\\recovery",
            "c:\\boot",
            "c:\\documents and settings",
            "c:\\intel",
            "c:\\perflogs",
            "c:\\drivers",
        ];
        for prefix in &system_prefixes {
            if path_str.starts_with(prefix) {
                return true;
            }
        }
        // 排除虚拟内存/休眠/交换文件
        if let Some(name) = path.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();
            if matches!(
                name_lower.as_str(),
                "pagefile.sys" | "hiberfil.sys" | "swapfile.sys"
            ) {
                return true;
            }
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

    /// 根据路径判断来源分类
    ///
    /// 个人目录（Desktop/Downloads/Documents）返回 personal_* 前缀，
    /// 其他位置返回 "other"。
    fn classify_source(path: &Path, user_home: &Path) -> String {
        let path_str = path.to_string_lossy().to_lowercase();
        let home_str = user_home.to_string_lossy().to_lowercase();
        if path_str.starts_with(&format!("{}\\desktop", home_str)) {
            "personal_desktop".to_string()
        } else if path_str.starts_with(&format!("{}\\downloads", home_str)) {
            "personal_downloads".to_string()
        } else if path_str.starts_with(&format!("{}\\documents", home_str)) {
            "personal_documents".to_string()
        } else {
            "other".to_string()
        }
    }

    /// 根据扩展名判断文件类型分类
    fn classify_file_type(path: &Path) -> String {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif" | "raw" | "heic"
            | "heif" | "ico" | "dds" | "svg" => "photo".to_string(),
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
            | "3gp" => "video".to_string(),
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" | "mid"
            | "midi" => "audio".to_string(),
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odp" | "ods" | "odt" | "rtf"
            | "csv" | "pdf" => "work_doc".to_string(),
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "tgz" | "iso" | "dmg" => {
                "archive".to_string()
            }
            "psd" | "ai" | "indd" | "sketch" | "fig" | "xd" | "afdesign" | "afphoto" | "kra"
            | "xcf" | "blend" | "max" | "dwg" | "dxf" => "design".to_string(),
            "exe" | "msi" | "appx" | "msix" | "dll" | "sys" | "com" | "scr" | "cpl" => {
                "executable".to_string()
            }
            "tmp" | "temp" | "bak" | "old" | "swp" | "swo" | "cache" | "log" => "temp".to_string(),
            _ => {
                // 代码文件
                if matches!(
                    ext.as_str(),
                    "js" | "ts" | "tsx" | "jsx" | "py" | "rs" | "java" | "c" | "cpp" | "h"
                        | "hpp" | "cs" | "go" | "rb" | "php" | "html" | "css" | "scss"
                        | "less" | "sql" | "sh" | "bat" | "ps1" | "vue" | "svelte" | "swift"
                        | "kt" | "dart" | "r" | "lua" | "zig" | "scala" | "ex" | "exs"
                        | "erl" | "hs" | "ml" | "clj"
                ) {
                    "code".to_string()
                } else {
                    "other".to_string()
                }
            }
        }
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
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
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
                source: "other".to_string(),
                file_type: "other".to_string(),
            });

            current += 1;
            progress(ScanProgress {
                scanner_id: "scanner-fs".to_string(),
                current,
                total,
                message: format!("发现微信聊天记录: {}", wxid),
                global_percent: None,
            });
        }

        Ok(items)
    }

    /// 扫描普通文件目录（Desktop / Downloads）
    ///
    /// 递归遍历，排除系统目录/自身目录/隐藏文件。
    ///
    /// - `apply_date_filter=true`：按入职日期过滤（仅修改日期 >= start_date 的文件）
    /// - `apply_date_filter=false`：不过滤（个人目录全量扫描）
    fn scan_directory(
        &self,
        ctx: &ScanContext,
        dir: &Path,
        apply_date_filter: bool,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
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

            // 来源分类与文件类型分类
            let source = Self::classify_source(path, &ctx.user_home);
            let file_type = Self::classify_file_type(path);

            // 入职日期过滤（仅当 apply_date_filter=true 且不是个人目录时）
            if apply_date_filter && !source.starts_with("personal_") {
                if let Some(file_date) = Self::modified_date(path) {
                    if file_date < ctx.start_date {
                        continue;
                    }
                }
            }

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
                source,
                file_type,
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
                    global_percent: None,
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
                global_percent: None,
            });
        }

        Ok(items)
    }

    /// 枚举所有可用盘符（C: 到 Z:）
    fn get_all_drives() -> Vec<PathBuf> {
        let mut drives = Vec::new();
        for letter in b'C'..=b'Z' {
            let path = format!("{}:\\", letter as char);
            if Path::new(&path).exists() {
                drives.push(PathBuf::from(path));
            }
        }
        drives
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
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut all_items = Vec::new();
        let mut scanned_paths: HashSet<PathBuf> = HashSet::new();

        // 步骤 1 & 2：优先扫描微信目录（RULE-03，无日期过滤）
        let wechat_items = self.scan_wechat(ctx, progress)?;
        all_items.extend(wechat_items);

        // 步骤 3：扫描个人目录（无日期过滤，位置优先）
        let personal_dirs = [
            ctx.user_home.join("Desktop"),
            ctx.user_home.join("Downloads"),
            ctx.user_home.join("Documents"),
        ];
        for dir in &personal_dirs {
            if dir.exists() && dir.is_dir() {
                let items = self.scan_directory(ctx, dir, false, progress)?;
                all_items.extend(items);
                // 记录已扫描的目录（用于去重）
                if let Ok(canonical) = std::fs::canonicalize(dir) {
                    scanned_paths.insert(canonical);
                }
            }
        }

        // 步骤 4：全盘扫描（有日期过滤），精细去重
        let drives = Self::get_all_drives();
        for drive in &drives {
            if Self::is_system_path(drive) {
                continue;
            }
            // 遍历盘符下一级子目录，跳过已扫描的个人目录
            for entry in walkdir::WalkDir::new(drive)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !entry.file_type().is_dir() {
                    continue;
                }
                // 跳过系统目录
                if Self::is_system_path(path) {
                    continue;
                }
                // 精细去重：检查是否在已扫描路径下
                if let Ok(canonical) = std::fs::canonicalize(path) {
                    if scanned_paths.iter().any(|p| canonical.starts_with(p)) {
                        continue;
                    }
                }
                let items = self.scan_directory(ctx, path, true, progress)?;
                all_items.extend(items);
            }
        }

        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use crate::types::{Action, ScanContext};
    use crate::scanner::ScanProgress;
    use chrono::Duration;

    fn make_scan_ctx(temp_dir: &tempfile::TempDir, start_date: NaiveDate) -> ScanContext {
        ScanContext {
            start_date,
            user_home: temp_dir.path().to_path_buf(),
            temp_dir: temp_dir.path().to_path_buf(),
        }
    }

    fn no_op_progress(_p: ScanProgress) {}

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

    #[test]
    fn test_scan_directory_finds_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("scan-target");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("file1.txt"), "a").unwrap();
        std::fs::write(dir.join("file2.txt"), "b").unwrap();
        std::fs::create_dir(dir.join("subdir")).unwrap();
        std::fs::write(dir.join("subdir").join("nested.txt"), "c").unwrap();

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_directory(&ctx, &dir, true, &no_op_progress).unwrap();

        assert_eq!(items.len(), 3, "应扫描到 3 个文件");
        let names: std::collections::HashSet<_> = items.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains("file1.txt"));
        assert!(names.contains("file2.txt"));
        assert!(names.contains("nested.txt"));

        for item in &items {
            assert_eq!(item.category, TraceCategory::FileSystem);
            assert_eq!(item.scanner_id, "scanner-fs");
            assert!(item.size_bytes.unwrap() > 0);
            assert_eq!(item.suggested_action, Some(Action::DeleteOrPack));
            assert!(item.path.as_ref().unwrap().to_string_lossy().contains("scan-target"));
        }
    }

    #[test]
    fn test_scan_directory_skips_hidden_and_system_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("scan-target");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("normal.txt"), "a").unwrap();
        std::fs::write(dir.join(".hidden"), "b").unwrap();
        std::fs::write(dir.join("pagefile.sys"), "c").unwrap();

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_directory(&ctx, &dir, true, &no_op_progress).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "normal.txt");
    }

    #[test]
    fn test_scan_directory_date_filter_excludes_old_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("scan-target");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("file.txt"), "a").unwrap();

        // start_date 设为明天，apply_date_filter=true，所有现有文件都应在入职前
        let tomorrow = Local::now().date_naive() + Duration::days(1);
        let ctx = make_scan_ctx(&temp_dir, tomorrow);
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_directory(&ctx, &dir, true, &no_op_progress).unwrap();

        assert_eq!(items.len(), 0, "入职日期后的文件应被过滤");
    }

    #[test]
    fn test_scan_directory_no_date_filter_includes_old_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("scan-target");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("file.txt"), "a").unwrap();

        // start_date 设为明天，但 apply_date_filter=false，不应过滤
        let tomorrow = Local::now().date_naive() + Duration::days(1);
        let ctx = make_scan_ctx(&temp_dir, tomorrow);
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_directory(&ctx, &dir, false, &no_op_progress).unwrap();

        assert_eq!(items.len(), 1, "apply_date_filter=false 时不应过滤文件");
    }

    #[test]
    fn test_scan_directory_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("empty");
        std::fs::create_dir(&dir).unwrap();

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_directory(&ctx, &dir, true, &no_op_progress).unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_scan_wechat_finds_wxid_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let wechat_dir = temp_dir.path().join("Documents").join("WeChat Files");
        std::fs::create_dir_all(&wechat_dir).unwrap();

        let wxid_dir = wechat_dir.join("wxid_abc123");
        std::fs::create_dir(&wxid_dir).unwrap();
        std::fs::write(wxid_dir.join("msg.db"), "dummy").unwrap();

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_wechat(&ctx, &no_op_progress).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].category, TraceCategory::Chat);
        assert_eq!(items[0].id, "wechat-wxid_abc123");
        assert!(items[0].name.contains("wxid_abc123"));
        assert_eq!(items[0].suggested_action, Some(Action::DeleteOrPack));
        assert_eq!(items[0].path, Some(wxid_dir));
    }

    #[test]
    fn test_scan_wechat_ignores_non_wxid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let wechat_dir = temp_dir.path().join("Documents").join("WeChat Files");
        std::fs::create_dir_all(&wechat_dir).unwrap();

        let other_dir = wechat_dir.join("All Users");
        std::fs::create_dir(&other_dir).unwrap();

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_wechat(&ctx, &no_op_progress).unwrap();

        assert_eq!(items.len(), 0, "非 wxid_ 目录应被忽略");
    }

    #[test]
    fn test_scan_wechat_no_dir_returns_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        // 不创建 WeChat Files 目录

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_wechat(&ctx, &no_op_progress).unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_classify_source_personal_dirs() {
        let user_home = Path::new("C:\\Users\\Test");
        assert_eq!(
            FileSystemScanner::classify_source(
                &Path::new("C:\\Users\\Test\\Desktop\\file.txt"),
                user_home
            ),
            "personal_desktop"
        );
        assert_eq!(
            FileSystemScanner::classify_source(
                &Path::new("C:\\Users\\Test\\Downloads\\doc.pdf"),
                user_home
            ),
            "personal_downloads"
        );
        assert_eq!(
            FileSystemScanner::classify_source(
                &Path::new("C:\\Users\\Test\\Documents\\notes.txt"),
                user_home
            ),
            "personal_documents"
        );
    }

    #[test]
    fn test_classify_source_other_dirs() {
        let user_home = Path::new("C:\\Users\\Test");
        assert_eq!(
            FileSystemScanner::classify_source(
                &Path::new("D:\\Projects\\work"),
                user_home
            ),
            "other"
        );
        assert_eq!(
            FileSystemScanner::classify_source(
                &Path::new("C:\\Users\\Test\\AppData\\cache"),
                user_home
            ),
            "other"
        );
    }

    #[test]
    fn test_classify_file_type_photos() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("photo.jpg")),
            "photo"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("image.PNG")),
            "photo"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("icon.svg")),
            "photo"
        );
    }

    #[test]
    fn test_classify_file_type_videos() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("video.mp4")),
            "video"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("clip.avi")),
            "video"
        );
    }

    #[test]
    fn test_classify_file_type_work_docs() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("report.docx")),
            "work_doc"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("data.xlsx")),
            "work_doc"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("slides.pptx")),
            "work_doc"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("document.pdf")),
            "work_doc"
        );
    }

    #[test]
    fn test_classify_file_type_code() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("main.rs")),
            "code"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("app.py")),
            "code"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("index.html")),
            "code"
        );
    }

    #[test]
    fn test_classify_file_type_archives() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("backup.zip")),
            "archive"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("archive.7z")),
            "archive"
        );
    }

    #[test]
    fn test_classify_file_type_temp() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("temp.tmp")),
            "temp"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("backup.bak")),
            "temp"
        );
    }

    #[test]
    fn test_classify_file_type_other() {
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("unknown.xyz")),
            "other"
        );
        assert_eq!(
            FileSystemScanner::classify_file_type(Path::new("noextension")),
            "other"
        );
    }

    #[test]
    fn test_scan_results_have_source_and_file_type() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("scan-target");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("photo.jpg"), "fake-image").unwrap();
        std::fs::write(dir.join("doc.pdf"), "fake-doc").unwrap();

        let ctx = make_scan_ctx(&temp_dir, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let scanner = FileSystemScanner::new();
        let items = scanner.scan_directory(&ctx, &dir, false, &no_op_progress).unwrap();

        assert_eq!(items.len(), 2);
        for item in &items {
            assert!(!item.source.is_empty(), "source 不应为空");
            assert!(!item.file_type.is_empty(), "file_type 不应为空");
        }

        let jpg_item = items.iter().find(|i| i.name == "photo.jpg").unwrap();
        assert_eq!(jpg_item.file_type, "photo");

        let pdf_item = items.iter().find(|i| i.name == "doc.pdf").unwrap();
        assert_eq!(pdf_item.file_type, "work_doc");
    }
}
