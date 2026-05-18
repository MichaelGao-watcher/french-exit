use crate::error::BackendError;
use crate::scanner::ScanProgress;
use crate::scanner::Scanner;
use crate::types::{ScanContext, TraceCategory, TraceItem};
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinSet;

/// 扫描器注册中心，管理所有 Scanner 实例并负责并行调度。
///
/// 持有 `Vec<Box<dyn Scanner>>`，提供注册、查询、按类别过滤和并行扫描能力。
pub struct ScannerRegistry {
    scanners: Vec<Box<dyn Scanner>>,
}

/// 扫描器引用包装，用于在 tokio 任务间安全传递引用。
///
/// # 安全性说明
/// `ScannerRegistry::scan_all` / `scan_by_category` 中，`self` 被借用直到所有
/// spawned 任务完成。因此 scanner 指针在整个任务生命周期内始终有效。
/// 此结构体不含生命周期参数，满足 `Send + 'static` 约束，可安全传入
/// `tokio::task::spawn_blocking`。
struct ScannerRef {
    ptr: *const dyn Scanner,
}

// SAFETY: Scanner trait 要求 Send + Sync，且指针有效性由 scan_all/scan_by_category 的借用保证。
unsafe impl Send for ScannerRef {}
unsafe impl Sync for ScannerRef {}

impl ScannerRef {
    fn as_scanner(&self) -> &dyn Scanner {
        // SAFETY: 见 ScannerRef 文档说明。指针在注册中心方法完成前始终有效。
        unsafe { &*self.ptr }
    }
}

impl ScannerRegistry {
    pub fn new() -> Self {
        Self {
            scanners: Vec::new(),
        }
    }

    /// 注册一个扫描器实例。
    pub fn register(&mut self, scanner: Box<dyn Scanner>) {
        self.scanners.push(scanner);
    }

    /// 获取所有已注册的扫描器。
    pub fn scanners(&self) -> &[Box<dyn Scanner>] {
        &self.scanners
    }

    /// 按类别过滤扫描器。
    pub fn scanners_by_category(&self, category: TraceCategory) -> Vec<&Box<dyn Scanner>> {
        self.scanners
            .iter()
            .filter(|s| {
                s.category() == category
            })
            .collect()
    }

    /// 并行执行所有已注册的扫描器，聚合结果。
    ///
    /// # 调度策略
    /// - 使用 `tokio::task::spawn_blocking` 将每个 Scanner 放到独立线程执行，
    ///   避免阻塞 async runtime（扫描为 IO 密集型同步操作）。
    /// - 通过 `tokio::sync::mpsc` 通道收集各扫描器的进度事件，实时回传给调用方。
    /// - 单个扫描器失败时记录错误信息，不影响其他扫描器继续执行。
    /// - 所有扫描器完成后，如有任何失败，返回 `BackendError::ScanError` 聚合错误。
    pub async fn scan_all(
        &self,
        ctx: &ScanContext,
        pause_rx: &watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, BackendError> {
        let indices: Vec<usize> = (0..self.scanners.len()).collect();
        self.scan_impl(&indices, ctx, pause_rx, progress).await
    }

    /// 只扫描指定类别的扫描器，逻辑同 `scan_all`。
    pub async fn scan_by_category(
        &self,
        category: TraceCategory,
        ctx: &ScanContext,
        pause_rx: &watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, BackendError> {
        let indices: Vec<usize> = self
            .scanners
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.category() == category
            })
            .map(|(i, _)| i)
            .collect();

        self.scan_impl(&indices, ctx, pause_rx, progress).await
    }

    /// 核心扫描实现：按索引列表并行调度扫描器。
    async fn scan_impl(
        &self,
        indices: &[usize],
        ctx: &ScanContext,
        pause_rx: &watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, BackendError> {
        if indices.is_empty() {
            return Ok(Vec::new());
        }

        // 内部进度通道：各扫描任务通过此通道发送进度，主任务实时转发给回调
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<ScanProgress>(128);

        let mut join_set = JoinSet::new();

        for &idx in indices {
            let scanner = &self.scanners[idx];
            let scanner_ref = ScannerRef {
                ptr: scanner.as_ref(),
            };
            let ctx = ctx.clone();
            let pause_rx = pause_rx.clone();
            let scanner_id = scanner.id().to_string();
            let p_tx = progress_tx.clone();

            // 将同步扫描任务放入 tokio 的阻塞线程池，避免占用 async worker 线程
            join_set.spawn_blocking(move || {
                // 每次扫描前检查暂停信号，若为 true 则循环等待
                while *pause_rx.borrow() {
                    std::thread::sleep(Duration::from_millis(100));
                }

                let progress_cb = |p: ScanProgress| {
                    let _ = p_tx.blocking_send(p);
                };

                let result = scanner_ref.as_scanner().scan(&ctx, &pause_rx, &progress_cb);
                (scanner_id, result)
            });
        }

        // 释放原始发送端，当所有任务完成后通道会自动关闭
        drop(progress_tx);

        let mut all_items = Vec::new();
        let mut error_msgs = Vec::new();

        // 并发等待任务完成 + 实时转发进度
        loop {
            tokio::select! {
                Some(p) = progress_rx.recv() => {
                    progress(p);
                }
                res = join_set.join_next() => {
                    match res {
                        Some(Ok((id, Ok(items)))) => {
                            all_items.extend(items);
                        }
                        Some(Ok((id, Err(e)))) => {
                            // 单点失败不中断：记录错误，继续等待其他扫描器
                            error_msgs.push(format!("[{}] {}", id, e));
                        }
                        Some(Err(join_err)) => {
                            error_msgs.push(format!("[任务异常] {}", join_err));
                        }
                        None => {
                            // 所有扫描任务已完成，Drain 剩余进度
                            while let Ok(p) = progress_rx.try_recv() {
                                progress(p);
                            }
                            break;
                        }
                    }
                }
                else => break,
            }
        }

        // 错误聚合：如有任何扫描器失败，返回聚合错误信息
        if !error_msgs.is_empty() {
            return Err(BackendError::ScanError(error_msgs.join("; ")));
        }

        Ok(all_items)
    }
}

impl Default for ScannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ScanError;
    use crate::types::Action;
    use chrono::{Local, NaiveDate};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tokio::sync::watch;

    /// 可复用的 mock Scanner，用于单元测试注入。
    struct MockScanner {
        id: &'static str,
        category: TraceCategory,
        display_name: &'static str,
        items: Vec<TraceItem>,
        should_fail: bool,
        progress_count: usize,
    }

    impl MockScanner {
        fn new(
            id: &'static str,
            category: TraceCategory,
            display_name: &'static str,
            item_count: usize,
            should_fail: bool,
            progress_count: usize,
        ) -> Self {
            let items = (0..item_count)
                .map(|i| TraceItem {
                    id: format!("{}-item-{}", id, i),
                    category,
                    scanner_id: id.to_string(),
                    name: format!("{} 痕迹 {}", display_name, i),
                    path: None,
                    size_bytes: None,
                    modified_at: Some(Local::now()),
                    inferred: false,
                    risk_note: None,
                    suggested_action: Some(Action::Delete),
                })
                .collect();
            Self {
                id,
                category,
                display_name,
                items,
                should_fail,
                progress_count,
            }
        }
    }

    impl Scanner for MockScanner {
        fn id(&self) -> &'static str {
            self.id
        }

        fn category(&self) -> TraceCategory {
            self.category
        }

        fn display_name(&self) -> &'static str {
            self.display_name
        }

        fn scan(
            &self,
            _ctx: &ScanContext,
            pause_rx: &watch::Receiver<bool>,
            progress: &dyn Fn(ScanProgress),
        ) -> Result<Vec<TraceItem>, ScanError> {
            // 处理暂停信号：若处于暂停状态则短暂等待
            if *pause_rx.borrow() {
                std::thread::sleep(Duration::from_millis(10));
            }

            for i in 0..self.progress_count {
                progress(ScanProgress {
                    scanner_id: self.id.to_string(),
                    current: i + 1,
                    total: self.progress_count,
                    message: format!("进度 {}/{}", i + 1, self.progress_count),
                });
            }

            if self.should_fail {
                Err(ScanError::Internal("模拟扫描失败".to_string()))
            } else {
                Ok(self.items.clone())
            }
        }
    }

    /// 构造一个空的 ScanContext，仅用于测试传参。
    fn dummy_context() -> ScanContext {
        ScanContext {
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            user_home: PathBuf::from("C:\\Users\\Test"),
            temp_dir: PathBuf::from("C:\\Temp"),
        }
    }

    /// 测试1：注册与查询
    /// 验证 ScannerRegistry 能正确注册扫描器，并按类别过滤返回。
    #[test]
    fn test_registry_register_and_query() {
        let mut registry = ScannerRegistry::new();
        registry.register(Box::new(MockScanner::new(
            "fs-1",
            TraceCategory::FileSystem,
            "文件扫描1",
            0,
            false,
            0,
        )));
        registry.register(Box::new(MockScanner::new(
            "chat-1",
            TraceCategory::Chat,
            "聊天扫描1",
            0,
            false,
            0,
        )));
        registry.register(Box::new(MockScanner::new(
            "env-1",
            TraceCategory::EnvVar,
            "环境变量扫描1",
            0,
            false,
            0,
        )));

        assert_eq!(registry.scanners().len(), 3, "应注册 3 个 Scanner");

        let fs_scanners = registry.scanners_by_category(TraceCategory::FileSystem);
        assert_eq!(fs_scanners.len(), 1, "FileSystem 类别应只有 1 个");
        assert_eq!(fs_scanners[0].id(), "fs-1");

        let chat_scanners = registry.scanners_by_category(TraceCategory::Chat);
        assert_eq!(chat_scanners.len(), 1);
        assert_eq!(chat_scanners[0].id(), "chat-1");
    }

    /// 测试2：scan_all 聚合结果（SR-08）
    /// 注入 3 个 mock Scanner，分别返回 2、3、5 个 TraceItem，
    /// 验证 scan_all 返回总数 10，且每个 item 的 scanner_id 正确。
    #[tokio::test]
    async fn test_scan_all_aggregates_results() {
        let mut registry = ScannerRegistry::new();
        registry.register(Box::new(MockScanner::new(
            "s1", TraceCategory::FileSystem, "S1", 2, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "s2", TraceCategory::Chat, "S2", 3, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "s3", TraceCategory::Browser, "S3", 5, false, 0,
        )));

        let ctx = dummy_context();
        let (_tx, rx) = watch::channel(false);

        let result = registry.scan_all(&ctx, &rx, &|_p| {}).await;
        assert!(result.is_ok(), "scan_all 应成功");

        let items = result.unwrap();
        assert_eq!(items.len(), 10, "总 item 数应为 2+3+5=10");

        let s1_count = items.iter().filter(|i| i.scanner_id == "s1").count();
        let s2_count = items.iter().filter(|i| i.scanner_id == "s2").count();
        let s3_count = items.iter().filter(|i| i.scanner_id == "s3").count();
        assert_eq!(s1_count, 2);
        assert_eq!(s2_count, 3);
        assert_eq!(s3_count, 5);

        // 验证每个 item 的 category 与 scanner 一致
        assert!(items.iter().filter(|i| i.scanner_id == "s1").all(|i| i.category == TraceCategory::FileSystem));
        assert!(items.iter().filter(|i| i.scanner_id == "s2").all(|i| i.category == TraceCategory::Chat));
        assert!(items.iter().filter(|i| i.scanner_id == "s3").all(|i| i.category == TraceCategory::Browser));
    }

    /// 测试3：单点失败不中断整体流程（SR-09）
    /// 第 2 个 Scanner 故意返回 Err，验证：
    /// - scan_all 最终返回 Err(BackendError::ScanError(...))
    /// - 错误信息中包含失败 Scanner 的 id
    /// - 错误信息中不包含成功 Scanner 的 id，说明聚合逻辑正确区分了成功与失败
    #[tokio::test]
    async fn test_scan_all_single_failure_not_interrupt() {
        let mut registry = ScannerRegistry::new();
        registry.register(Box::new(MockScanner::new(
            "ok1", TraceCategory::FileSystem, "OK1", 2, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "fail", TraceCategory::Chat, "FAIL", 3, true, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "ok2", TraceCategory::Browser, "OK2", 5, false, 0,
        )));

        let ctx = dummy_context();
        let (_tx, rx) = watch::channel(false);

        let result = registry.scan_all(&ctx, &rx, &|_p| {}).await;
        assert!(result.is_err(), "存在失败 Scanner，应返回 Err");

        let err_msg = match result {
            Err(BackendError::ScanError(msg)) => msg,
            other => panic!("期望 BackendError::ScanError，实际得到 {:?}", other),
        };

        assert!(
            err_msg.contains("fail"),
            "错误信息应包含失败 Scanner 的 id: {}",
            err_msg
        );
        assert!(
            !err_msg.contains("ok1"),
            "错误信息不应包含成功 Scanner ok1 的 id: {}",
            err_msg
        );
        assert!(
            !err_msg.contains("ok2"),
            "错误信息不应包含成功 Scanner ok2 的 id: {}",
            err_msg
        );
    }

    /// 测试4：按类别过滤扫描（SR-10）
    /// 注册 5 个 Scanner（2 FileSystem, 2 Chat, 1 EnvVar），
    /// 调用 scan_by_category(Chat)，验证只返回 Chat 类别的结果。
    #[tokio::test]
    async fn test_scan_by_category_filters_correctly() {
        let mut registry = ScannerRegistry::new();
        registry.register(Box::new(MockScanner::new(
            "fs-1", TraceCategory::FileSystem, "FS1", 1, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "fs-2", TraceCategory::FileSystem, "FS2", 1, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "chat-1", TraceCategory::Chat, "CHAT1", 1, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "chat-2", TraceCategory::Chat, "CHAT2", 1, false, 0,
        )));
        registry.register(Box::new(MockScanner::new(
            "env-1", TraceCategory::EnvVar, "ENV1", 1, false, 0,
        )));

        let ctx = dummy_context();
        let (_tx, rx) = watch::channel(false);

        let result = registry
            .scan_by_category(TraceCategory::Chat, &ctx, &rx, &|_p| {})
            .await;
        assert!(result.is_ok());

        let items = result.unwrap();
        assert_eq!(items.len(), 2, "Chat 类别应只有 2 个 Scanner 的结果");
        assert!(
            items.iter().all(|i| i.category == TraceCategory::Chat),
            "所有返回 item 的 category 应为 Chat"
        );
        assert!(
            items.iter().all(|i| i.scanner_id == "chat-1" || i.scanner_id == "chat-2"),
            "返回的 item 应仅来自 chat-1 或 chat-2"
        );
    }

    /// 测试5：进度回调触发验证
    /// 2 个 mock Scanner 各在 scan 中调用 3 次 progress 回调，
    /// 验证 scan_all 能正确收集并转发 6 个进度事件，且 scanner_id 正确。
    #[tokio::test]
    async fn test_scan_all_progress_callback_fired() {
        let mut registry = ScannerRegistry::new();
        registry.register(Box::new(MockScanner::new(
            "p1", TraceCategory::FileSystem, "P1", 0, false, 3,
        )));
        registry.register(Box::new(MockScanner::new(
            "p2", TraceCategory::Chat, "P2", 0, false, 3,
        )));

        let ctx = dummy_context();
        let (_tx, rx) = watch::channel(false);

        let progresses: Arc<Mutex<Vec<ScanProgress>>> = Arc::new(Mutex::new(Vec::new()));
        let progresses_clone = Arc::clone(&progresses);

        let result = registry
            .scan_all(&ctx, &rx, &move |p: ScanProgress| {
                progresses_clone.lock().unwrap().push(p);
            })
            .await;

        assert!(result.is_ok());

        let collected = progresses.lock().unwrap();
        assert_eq!(
            collected.len(),
            6,
            "应收到 6 个进度事件（2 scanner × 3 次）"
        );

        let p1_count = collected.iter().filter(|p| p.scanner_id == "p1").count();
        let p2_count = collected.iter().filter(|p| p.scanner_id == "p2").count();
        assert_eq!(p1_count, 3, "p1 应产生 3 个进度事件");
        assert_eq!(p2_count, 3, "p2 应产生 3 个进度事件");

        // 验证进度事件的 current/total 语义正确
        for p in collected.iter().filter(|p| p.scanner_id == "p1") {
            assert_eq!(p.total, 3);
            assert!(p.current >= 1 && p.current <= 3);
        }
    }
}
