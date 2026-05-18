use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

use crate::error::BackendError;
use crate::executor::Executor;
use crate::types::{Action, ExecutionResult, ExecutionStatus, TraceItem};

/// 打包执行器
///
/// 收集标记为 `Action::Pack` 的 `TraceItem`，最终打包为 `French-exit.zip`。
/// 支持条目去重、保留原始目录结构、加密文件检测（简化版）。
pub struct PackExecutor {
    output_dir: PathBuf,
    items: Mutex<Vec<TraceItem>>,
    seen_paths: Mutex<HashSet<PathBuf>>,
}

impl PackExecutor {
    /// 创建新的打包执行器
    ///
    /// # 参数
    /// - `output_dir`: zip 文件输出目录
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            items: Mutex::new(Vec::new()),
            seen_paths: Mutex::new(HashSet::new()),
        }
    }

    /// 计算所有待打包文件的总大小（字节）
    ///
    /// 优先使用 `TraceItem.size_bytes`，若不存在则通过 `fs::metadata` 读取。
    fn total_size(items: &[TraceItem]) -> u64 {
        items.iter().fold(0u64, |acc, item| {
            let size = item.size_bytes.unwrap_or_else(|| {
                item.path
                    .as_ref()
                    .and_then(|p| std::fs::metadata(p).ok().map(|m| m.len()))
                    .unwrap_or(0)
            });
            acc + size
        })
    }

    /// 将绝对路径转换为 zip 内的相对路径
    ///
    /// 去掉 `user_home` 前缀，保留剩余目录结构；若不在 `user_home` 下，则使用文件名本身。
    /// zip 内统一使用 `/` 作为路径分隔符。
    fn to_zip_path(path: &Path, user_home: &Path) -> String {
        match path.strip_prefix(user_home) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }

    /// 检查文件是否为加密文件（简化版）
    ///
    /// 通过扩展名判断：`.enc`、`.locked` 视为加密文件。
    /// 当前版本不实现回调确认机制，仅做检测标记。
    fn is_encrypted_file(path: &Path) -> bool {
        match path.extension() {
            Some(ext) => {
                let ext = ext.to_string_lossy().to_lowercase();
                ext == "enc" || ext == "locked"
            }
            None => false,
        }
    }

    /// 生成最终 zip 文件
    ///
    /// # 流程
    /// 1. 计算待打包文件总大小
    /// 2. （TODO）磁盘空间预检查
    /// 3. 创建 `French-exit.zip`
    /// 4. 遍历收集的条目，写入文件内容
    /// 5. 关闭 zip writer，返回文件路径
    pub fn finalize(&self) -> Result<PathBuf, BackendError> {
        let items = self
            .items
            .lock()
            .map_err(|e| BackendError::ExecutionError(format!("打包列表锁 poisoned: {}", e)))?;

        let _total_size = Self::total_size(&items);

        // TODO: 磁盘空间预检查
        // Windows 下需调用 GetDiskFreeSpaceExW，当前版本暂跳过，后续迭代补充。
        // let required = (_total_size as f64 * 1.1) as u64;

        // 确保输出目录存在
        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir)?;
        }

        let zip_path = self.output_dir.join("French-exit.zip");
        let zip_file = File::create(&zip_path)?;
        let mut zip_writer = ZipWriter::new(zip_file);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        // 获取用户主目录，用于计算 zip 内相对路径
        let user_home = std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        // 用于防止 zip 内出现重复路径
        let mut zip_paths: HashSet<String> = HashSet::new();

        for item in items.iter() {
            let Some(path) = &item.path else {
                continue;
            };

            if !path.exists() {
                continue;
            }

            if path.is_file() {
                let zip_path_str = Self::to_zip_path(path, &user_home);

                // 加密文件检测（简化版，不实现回调）
                if Self::is_encrypted_file(path) {
                    // TODO: 后续迭代实现加密文件回调确认机制
                }

                // 跳过 zip 内重复路径
                if !zip_paths.insert(zip_path_str.clone()) {
                    continue;
                }

                zip_writer
                    .start_file(&zip_path_str, options)
                    .map_err(|e| {
                        BackendError::ExecutionError(format!(
                            "zip start_file 失败 ({}): {}",
                            zip_path_str, e
                        ))
                    })?;

                let mut file = File::open(path)?;
                io::copy(&mut file, &mut zip_writer).map_err(|e| {
                    BackendError::ExecutionError(format!(
                        "写入 zip 失败 ({}): {}",
                        zip_path_str, e
                    ))
                })?;
            } else if path.is_dir() {
                // 递归打包目录内所有文件
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if !entry_path.is_file() {
                        continue;
                    }

                    let zip_path_str = Self::to_zip_path(entry_path, &user_home);

                    if !zip_paths.insert(zip_path_str.clone()) {
                        continue;
                    }

                    zip_writer
                        .start_file(&zip_path_str, options)
                        .map_err(|e| {
                            BackendError::ExecutionError(format!(
                                "zip start_file 失败 ({}): {}",
                                zip_path_str, e
                            ))
                        })?;

                    let mut file = File::open(entry_path)?;
                    io::copy(&mut file, &mut zip_writer).map_err(|e| {
                        BackendError::ExecutionError(format!(
                            "写入 zip 失败 ({}): {}",
                            zip_path_str, e
                        ))
                    })?;
                }
            }
        }

        zip_writer
            .finish()
            .map_err(|e| BackendError::ExecutionError(format!("zip finish 失败: {}", e)))?;

        Ok(zip_path)
    }
}

impl Executor for PackExecutor {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, BackendError> {
        let mut items = self
            .items
            .lock()
            .map_err(|e| BackendError::ExecutionError(format!("打包列表锁 poisoned: {}", e)))?;
        let mut seen = self
            .seen_paths
            .lock()
            .map_err(|e| BackendError::ExecutionError(format!("去重集合锁 poisoned: {}", e)))?;

        if let Some(ref path) = item.path {
            if seen.insert(path.clone()) {
                items.push(item.clone());
            }
        } else {
            // 无可操作路径的条目也加入列表（无去重依据）
            items.push(item.clone());
        }

        Ok(ExecutionResult {
            item_id: item.id.clone(),
            action: Action::Pack,
            status: ExecutionStatus::Success,
            detail: Some("已加入打包列表".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_deduplication() {
        let temp_dir = tempfile::tempdir().unwrap();
        let executor = PackExecutor::new(temp_dir.path().to_path_buf());

        let item1 = TraceItem {
            id: "a".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "file1".to_string(),
            path: Some(PathBuf::from("C:\\Users\\Test\\file1.txt")),
            size_bytes: Some(100),
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Pack),
        };

        let item2 = TraceItem {
            id: "b".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "file1 dup".to_string(),
            path: Some(PathBuf::from("C:\\Users\\Test\\file1.txt")),
            size_bytes: Some(100),
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Pack),
        };

        executor.execute(&item1).unwrap();
        executor.execute(&item2).unwrap();

        let items = executor.items.lock().unwrap();
        assert_eq!(items.len(), 1); // 去重后只剩 1 条
    }

    #[test]
    fn test_pack_executor_different_paths() {
        let temp_dir = tempfile::tempdir().unwrap();
        let executor = PackExecutor::new(temp_dir.path().to_path_buf());

        let item1 = TraceItem {
            id: "a".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "file1".to_string(),
            path: Some(PathBuf::from("C:\\Users\\Test\\file1.txt")),
            size_bytes: Some(100),
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Pack),
        };

        let item2 = TraceItem {
            id: "b".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "file2".to_string(),
            path: Some(PathBuf::from("C:\\Users\\Test\\file2.txt")),
            size_bytes: Some(200),
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Pack),
        };

        executor.execute(&item1).unwrap();
        executor.execute(&item2).unwrap();

        let items = executor.items.lock().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_to_zip_path_relative() {
        let user_home = Path::new("C:\\Users\\Test");
        let file = Path::new("C:\\Users\\Test\\Desktop\\file.txt");
        assert_eq!(PackExecutor::to_zip_path(file, user_home), "Desktop/file.txt");
    }

    #[test]
    fn test_to_zip_path_fallback() {
        let user_home = Path::new("C:\\Users\\Test");
        let file = Path::new("D:\\Other\\file.txt");
        assert_eq!(PackExecutor::to_zip_path(file, user_home), "file.txt");
    }

    #[test]
    fn test_is_encrypted_file() {
        assert!(PackExecutor::is_encrypted_file(Path::new("data.enc")));
        assert!(PackExecutor::is_encrypted_file(Path::new("data.locked")));
        assert!(!PackExecutor::is_encrypted_file(Path::new("data.txt")));
        assert!(!PackExecutor::is_encrypted_file(Path::new("data")));
    }

    #[test]
    fn test_total_size() {
        let items = vec![
            TraceItem {
                id: "a".to_string(),
                category: TraceCategory::FileSystem,
                scanner_id: "test".to_string(),
                name: "file1".to_string(),
                path: None,
                size_bytes: Some(1024),
                modified_at: None,
                inferred: false,
                risk_note: None,
                suggested_action: Some(Action::Pack),
            },
            TraceItem {
                id: "b".to_string(),
                category: TraceCategory::FileSystem,
                scanner_id: "test".to_string(),
                name: "file2".to_string(),
                path: None,
                size_bytes: Some(2048),
                modified_at: None,
                inferred: false,
                risk_note: None,
                suggested_action: Some(Action::Pack),
            },
        ];
        assert_eq!(PackExecutor::total_size(&items), 3072);
    }
}
