use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::BackendError;
use crate::types::TraceItem;
use uuid::Uuid;

/// 临时数据管理器
///
/// 职责：管理 `%TEMP%/french-exit/{pid}/` 目录下的所有临时文件；
/// 扫描中间结果分批落盘；程序退出时自毁。
pub struct TempStore {
    root: PathBuf,
    batch_counter: AtomicU64,
}

impl TempStore {
    /// 构造函数，root 固定为 `%TEMP%/french-exit/{pid}/`
    pub fn new() -> Result<Self, BackendError> {
        let pid = std::process::id();
        let temp = std::env::temp_dir();
        let root = temp.join("french-exit").join(pid.to_string());
        fs::create_dir_all(&root)
            .map_err(|e| BackendError::StoreError(format!("创建临时目录失败: {}", e)))?;
        Ok(Self {
            root,
            batch_counter: AtomicU64::new(0),
        })
    }

    /// 测试专用构造函数，允许指定自定义根目录
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Result<Self, BackendError> {
        fs::create_dir_all(&root)
            .map_err(|e| BackendError::StoreError(format!("创建临时目录失败: {}", e)))?;
        Ok(Self {
            root,
            batch_counter: AtomicU64::new(0),
        })
    }

    /// 返回根目录路径
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 在 root 下创建带随机后缀的临时文件或子目录路径
    ///
    /// 命名规则：`{prefix}_{uuid}`
    /// 返回完整路径（不实际创建文件/目录，仅分配路径）
    pub fn allocate(&self, prefix: &str) -> PathBuf {
        let uuid = Uuid::new_v4();
        let name = format!("{}_{}", prefix, uuid);
        self.root.join(name)
    }

    /// 将一批 TraceItem 以 JSON Lines 格式落盘
    ///
    /// 输出路径：`root/results/scan_{batch_id}.jsonl`
    /// batch_id 从 0 开始递增，自动追加不覆盖
    pub fn save_scan_batch(&self, batch: &[TraceItem]) -> Result<(), BackendError> {
        let batch_id = self.batch_counter.fetch_add(1, Ordering::SeqCst);
        let results_dir = self.root.join("results");
        fs::create_dir_all(&results_dir)
            .map_err(|e| BackendError::StoreError(format!("创建结果目录失败: {}", e)))?;

        let path = results_dir.join(format!("scan_{}.jsonl", batch_id));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| BackendError::StoreError(format!("打开批次文件失败: {}", e)))?;

        for item in batch {
            let line = serde_json::to_string(item)
                .map_err(|e| BackendError::StoreError(format!("序列化失败: {}", e)))?;
            writeln!(file, "{}", line)
                .map_err(|e| BackendError::StoreError(format!("写入失败: {}", e)))?;
        }

        Ok(())
    }

    /// 按文件名顺序读取扫描结果，支持跨文件分页
    ///
    /// 内存友好：逐行读取，只加载需要的行
    pub fn load_scan_results(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<TraceItem>, BackendError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let results_dir = self.root.join("results");
        if !results_dir.exists() {
            return Ok(Vec::new());
        }

        // 读取目录并按 batch_id 排序
        let mut entries: Vec<_> = fs::read_dir(&results_dir)
            .map_err(|e| BackendError::StoreError(format!("读取结果目录失败: {}", e)))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "jsonl"))
            .collect();

        entries.sort_by_key(|e| parse_batch_id(&e.path()).unwrap_or(u64::MAX));

        let mut current_offset = 0usize;
        let mut result = Vec::with_capacity(limit.min(1024));

        for entry in entries {
            let path = entry.path();
            let file = File::open(&path)
                .map_err(|e| BackendError::StoreError(format!("打开文件失败: {}", e)))?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line =
                    line.map_err(|e| BackendError::StoreError(format!("读取行失败: {}", e)))?;
                if line.trim().is_empty() {
                    continue;
                }

                if current_offset >= offset {
                    let item: TraceItem = serde_json::from_str(&line)
                        .map_err(|e| BackendError::StoreError(format!("反序列化失败: {}", e)))?;
                    result.push(item);
                    if result.len() >= limit {
                        return Ok(result);
                    }
                }
                current_offset += 1;
            }
        }

        Ok(result)
    }

    /// 递归删除 root 下所有内容
    ///
    /// 安全校验：root 路径字符串中必须包含 "french-exit"（不区分大小写）
    pub fn self_destruct(&self) -> Result<(), BackendError> {
        let root_str = self.root.to_string_lossy();
        if !root_str.to_lowercase().contains("french-exit") {
            return Err(BackendError::StoreError(
                "防误删校验失败：路径不包含 french-exit 关键字".to_string(),
            ));
        }

        if !self.root.exists() {
            return Ok(());
        }

        // 递归删除 root 下的所有内容
        fn remove_all(path: &Path) -> Result<(), BackendError> {
            if path.is_dir() {
                for entry in fs::read_dir(path)
                    .map_err(|e| BackendError::StoreError(format!("读取目录失败: {}", e)))?
                {
                    let entry = entry
                        .map_err(|e| BackendError::StoreError(format!("读取目录项失败: {}", e)))?;
                    remove_all(&entry.path())?;
                }
                fs::remove_dir(path)
                    .map_err(|e| BackendError::StoreError(format!("删除目录失败: {}", e)))?;
            } else {
                fs::remove_file(path)
                    .map_err(|e| BackendError::StoreError(format!("删除文件失败: {}", e)))?;
            }
            Ok(())
        }

        for entry in fs::read_dir(&self.root)
            .map_err(|e| BackendError::StoreError(format!("读取根目录失败: {}", e)))?
        {
            let entry = entry
                .map_err(|e| BackendError::StoreError(format!("读取目录项失败: {}", e)))?;
            remove_all(&entry.path())?;
        }

        // 最后删除 root 本身
        fs::remove_dir(&self.root)
            .map_err(|e| BackendError::StoreError(format!("删除根目录失败: {}", e)))?;

        Ok(())
    }
}

impl Drop for TempStore {
    fn drop(&mut self) {
        let _ = self.self_destruct();
    }
}

/// 从文件名提取 batch_id，用于排序
fn parse_batch_id(path: &Path) -> Option<u64> {
    let stem = path.file_stem()?.to_str()?;
    let num_str = stem.strip_prefix("scan_")?;
    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, TraceCategory};
    use chrono::Local;
    use std::collections::HashSet;
    use std::panic;
    use std::thread;
    use tempfile::TempDir;

    /// TS-07：验证 self_destruct 完全清理嵌套结构
    #[test]
    fn test_self_destruct_cleans_nested_structure() {
        let temp_dir = TempDir::new().unwrap();
        let store = TempStore::with_root(temp_dir.path().join("french-exit")).unwrap();
        let root = store.root().to_path_buf();

        // 构造多层嵌套结构
        let nested = root.join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("file1.txt"), "hello").unwrap();
        fs::write(root.join("a").join("file2.txt"), "world").unwrap();
        fs::write(root.join("top.txt"), "top").unwrap();

        store.self_destruct().unwrap();

        // 验证 root 为空或不存在
        assert!(!root.exists() || root.read_dir().unwrap().next().is_none());
    }

    /// TS-08：验证 save_scan_batch + load_scan_results 分页一致性
    #[test]
    fn test_pagination_consistency_across_batches() {
        let temp_dir = TempDir::new().unwrap();
        let store = TempStore::with_root(temp_dir.path().join("french-exit")).unwrap();

        // 生成 1200 条 TraceItem，分 3 个 batch 写入（400 + 400 + 400）
        let total = 1200usize;
        let batch_size = 400usize;
        let mut all_items = Vec::with_capacity(total);

        for i in 0..total {
            all_items.push(TraceItem {
                id: format!("item-{}", i),
                category: TraceCategory::FileSystem,
                scanner_id: "test".to_string(),
                name: format!("Test Item {}", i),
                path: Some(PathBuf::from(format!("/tmp/{}", i))),
                size_bytes: Some(i as u64),
                modified_at: Some(Local::now()),
                inferred: false,
                risk_note: None,
                suggested_action: Some(Action::Delete),
                source: "other".to_string(),
                file_type: "other".to_string(),
            });
        }

        for chunk in all_items.chunks(batch_size) {
            store.save_scan_batch(chunk).unwrap();
        }

        // offset=0, limit=100
        let page1 = store.load_scan_results(0, 100).unwrap();
        assert_eq!(page1.len(), 100);
        assert_eq!(page1[0].id, "item-0");
        assert_eq!(page1[99].id, "item-99");

        // offset=500, limit=100（跨文件边界）
        let page2 = store.load_scan_results(500, 100).unwrap();
        assert_eq!(page2.len(), 100);
        assert_eq!(page2[0].id, "item-500");
        assert_eq!(page2[99].id, "item-599");

        // offset=1199, limit=10（接近末尾）
        let page3 = store.load_scan_results(1199, 10).unwrap();
        assert_eq!(page3.len(), 1);
        assert_eq!(page3[0].id, "item-1199");

        // offset=2000, limit=10（超出总量，应返回空）
        let page4 = store.load_scan_results(2000, 10).unwrap();
        assert!(page4.is_empty());

        // limit=0 返回空
        let page5 = store.load_scan_results(0, 0).unwrap();
        assert!(page5.is_empty());
    }

    /// TS-09：验证 Drop 在 panic 后仍被调用
    #[test]
    fn test_drop_called_after_panic() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("french-exit-drop-test");

        let result = panic::catch_unwind(|| {
            let store = TempStore::with_root(root.clone()).unwrap();
            // 写入一个文件证明目录存在
            fs::write(store.root().join("marker.txt"), "x").unwrap();
            // 故意 panic
            panic!("intentional panic");
        });

        assert!(result.is_err());
        // panic 后 Drop 应该被调用，目录应被清理
        assert!(!root.exists() || root.read_dir().unwrap().next().is_none());
    }

    /// TS-10：验证 allocate 在并发场景下不生成同名文件
    #[test]
    fn test_allocate_concurrent_no_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let store = TempStore::with_root(temp_dir.path().join("french-exit")).unwrap();
        let store = std::sync::Arc::new(store);

        let mut handles = Vec::new();
        for _ in 0..100 {
            let store_clone = store.clone();
            handles.push(thread::spawn(move || {
                store_clone.allocate("test")
            }));
        }

        let mut paths = HashSet::new();
        for handle in handles {
            let path = handle.join().unwrap();
            assert!(
                paths.insert(path.clone()),
                "发现重复路径: {:?}",
                path
            );
        }

        assert_eq!(paths.len(), 100);
    }

    /// 验证防误删校验：路径不包含 french-exit 时应失败
    #[test]
    fn test_self_destruct_safety_check() {
        let temp_dir = TempDir::new().unwrap();
        // 故意使用不包含 french-exit 的路径
        let store = TempStore::with_root(temp_dir.path().join("unsafe-dir")).unwrap();

        let result = store.self_destruct();
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("防误删校验失败"));
    }

    /// 验证构造函数 new() 能正确创建目录
    #[test]
    fn test_new_creates_directory() {
        let store = TempStore::new().unwrap();
        assert!(store.root().exists());
        assert!(store.root().is_dir());
    }

    /// 验证 allocate 返回的路径前缀正确
    #[test]
    fn test_allocate_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let store = TempStore::with_root(temp_dir.path().join("french-exit")).unwrap();

        let path = store.allocate("my_prefix");
        let file_name = path.file_name().unwrap().to_str().unwrap();
        assert!(file_name.starts_with("my_prefix_"));
        assert!(path.starts_with(store.root()));
    }

    /// 验证 save_scan_batch 写入后读取一致
    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let store = TempStore::with_root(temp_dir.path().join("french-exit")).unwrap();

        let items = vec![TraceItem {
            id: "id-1".to_string(),
            category: TraceCategory::Chat,
            scanner_id: "scanner".to_string(),
            name: "WeChat".to_string(),
            path: Some(PathBuf::from("/wechat")),
            size_bytes: Some(1024),
            modified_at: Some(Local::now()),
            inferred: true,
            risk_note: Some("高风险".to_string()),
            suggested_action: Some(Action::DeleteOrPack),
            source: "other".to_string(),
            file_type: "other".to_string(),
        }];

        store.save_scan_batch(&items).unwrap();
        let loaded = store.load_scan_results(0, 10).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "id-1");
        assert_eq!(loaded[0].name, "WeChat");
    }
}
