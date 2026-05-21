use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use ts_rs::TS;

use crate::error::BackendError;
use crate::executor::{delete::DeleteExecutor, pack::PackExecutor, preserve::PreserveExecutor, Executor};
use crate::reporter::Reporter;
use crate::resource::controller::ResourceController;
use crate::scanner::registry::ScannerRegistry;
use crate::store::temp_store::TempStore;
use crate::types::{Action, Decision, ExecutionReport, ExecutionResult, ExecutionStatus, ProgressEvent, ScanContext};

/// 会话状态机（FSM）
///
/// 状态流转规则由 Orchestrator::is_valid_transition 严格校验。
/// 任何非法转换都会返回 OrchestratorError，防止误操作。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SessionState {
    /// 初始状态，等待用户启动扫描
    Idle,
    /// 正在扫描中，持有本次扫描的唯一标识
    Scanning { scan_id: String },
    /// 扫描被用户暂停
    Paused,
    /// 扫描完成，等待用户确认
    Scanned { item_count: usize },
    /// 用户正在浏览结果并做决策
    Confirming,
    /// 正在执行用户决策（删除/打包/保留）
    Executing,
    /// 全部完成，持有执行报告
    Completed { report: ExecutionReport },
    /// 流程异常终止
    Failed { reason: String },
}

/// 流程调度器（M03）
///
/// Orchestrator 是 French Exit 的核心指挥模块，负责：
/// 1. 维护全局会话状态机（Idle → Scanning → Scanned → Confirming → Executing → Completed）
/// 2. 调度 ScannerRegistry 执行并行扫描
/// 3. 管理暂停/恢复信号
/// 4. 按用户决策分发到各 Executor
/// 5. 调用 Reporter 生成最终 HTML 报告
/// 6. 退出前触发 TempStore 自毁
///
/// 所有方法均为 `&self`，通过内部 Mutex / Arc 保证线程安全，满足 Tauri State 的 Send + Sync 要求。
pub struct Orchestrator {
    scanner_registry: Arc<ScannerRegistry>,
    temp_store: Arc<TempStore>,
    resource_controller: Arc<ResourceController>,
    delete_executor: Arc<DeleteExecutor>,
    pack_executor: Arc<PackExecutor>,
    preserve_executor: Arc<PreserveExecutor>,
    reporter: Reporter,
    state: Arc<std::sync::Mutex<SessionState>>,
    pause_tx: Arc<std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
    /// 当前扫描会话 ID，用于 Pause → Scanning 恢复时重建状态
    current_scan_id: Arc<std::sync::Mutex<Option<String>>>,
    /// 用户提交的决策清单，在 submit_decisions 时写入
    decisions: Arc<std::sync::Mutex<Vec<Decision>>>,
    /// 扫描进度发送通道（由 Commands 层注入）
    progress_tx: Arc<std::sync::Mutex<Option<tokio::sync::mpsc::Sender<ProgressEvent>>>>,
}

impl Orchestrator {
    /// 创建新的 Orchestrator 实例
    ///
    /// 各依赖模块由调用方（通常为 M02 commands 或 main 初始化代码）构造后注入。
    pub fn new(
        scanner_registry: ScannerRegistry,
        temp_store: Arc<TempStore>,
        resource_controller: Arc<ResourceController>,
        delete_executor: DeleteExecutor,
        pack_executor: PackExecutor,
        preserve_executor: PreserveExecutor,
    ) -> Self {
        Self {
            scanner_registry: Arc::new(scanner_registry),
            temp_store,
            resource_controller,
            delete_executor: Arc::new(delete_executor),
            pack_executor: Arc::new(pack_executor),
            preserve_executor: Arc::new(preserve_executor),
            reporter: Reporter,
            state: Arc::new(std::sync::Mutex::new(SessionState::Idle)),
            pause_tx: Arc::new(std::sync::Mutex::new(None)),
            current_scan_id: Arc::new(std::sync::Mutex::new(None)),
            decisions: Arc::new(std::sync::Mutex::new(Vec::new())),
            progress_tx: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// 读取当前状态
    pub fn current_state(&self) -> SessionState {
        self.state
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| SessionState::Failed {
                reason: "状态锁被污染".to_string(),
            })
    }

    /// 状态转换（带校验）
    ///
    /// 只有 is_valid_transition 返回 true 的转换才被允许。
    /// 非法转换返回 BackendError::OrchestratorError。
    pub fn transition_to(&self, new_state: SessionState) -> Result<(), BackendError> {
        let mut guard = self.state.lock().map_err(|e| {
            BackendError::OrchestratorError(format!("状态锁被污染: {:?}", e))
        })?;
        let old = guard.clone();
        if !Self::is_valid_transition(&old, &new_state) {
            return Err(BackendError::OrchestratorError(format!(
                "非法状态转换: {:?} -> {:?}",
                old, new_state
            )));
        }
        *guard = new_state;
        Ok(())
    }

    /// 校验状态转换是否合法
    ///
    /// 合法路径：
    /// - Idle → Scanning（开始扫描）
    /// - Scanning ↔ Paused（暂停/恢复）
    /// - Scanning → Scanned（扫描完成）
    /// - Scanned → Confirming（加载结果）
    /// - Confirming → Executing（提交决策）
    /// - Executing → Completed（执行完成）
    /// - Executing → Failed（执行异常）
    /// - Any → Idle（取消/重置）
    pub fn is_valid_transition(from: &SessionState, to: &SessionState) -> bool {
        matches!(
            (from, to),
            (SessionState::Idle, SessionState::Scanning { .. })
                | (SessionState::Scanning { .. }, SessionState::Paused)
                | (SessionState::Paused, SessionState::Scanning { .. })
                | (SessionState::Scanning { .. }, SessionState::Scanned { .. })
                | (SessionState::Scanned { .. }, SessionState::Confirming)
                | (SessionState::Confirming, SessionState::Executing)
                | (SessionState::Executing, SessionState::Completed { .. })
                | (SessionState::Executing, SessionState::Failed { .. })
                | (_, SessionState::Idle)
        )
    }

    /// 设置进度发送通道（由 Commands 层在扫描前注入）
    pub fn set_progress_tx(&self, tx: tokio::sync::mpsc::Sender<ProgressEvent>) {
        if let Ok(mut guard) = self.progress_tx.lock() {
            *guard = Some(tx);
        }
    }

    /// 启动扫描
    ///
    /// 1. 校验当前状态为 Idle，否则返回错误
    /// 2. 生成 scan_id（uuid v4）
    /// 3. 创建 watch::channel(false) 作为暂停信号
    /// 4. 保存 pause_tx 到字段，保存 scan_id 到 current_scan_id
    /// 5. 状态转为 Scanning
    /// 6. 在后台 tokio task 中执行 scanner_registry.scan_all
    /// 7. 扫描结果分批落盘到 TempStore（每批 500 条）
    /// 8. 完成后自动转为 Scanned（或 Failed）
    ///
    /// 扫描过程中，各 Scanner 的细粒度进度通过 `progress_tx` 实时推送到前端。
    pub fn start_scan(&self, ctx: ScanContext) -> Result<String, BackendError> {
        let scan_id = uuid::Uuid::new_v4().to_string();

        // 状态校验与转换：Idle → Scanning
        self.transition_to(SessionState::Scanning {
            scan_id: scan_id.clone(),
        })?;

        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        if let Ok(mut guard) = self.pause_tx.lock() {
            *guard = Some(pause_tx);
        }
        if let Ok(mut guard) = self.current_scan_id.lock() {
            *guard = Some(scan_id.clone());
        }

        // 克隆 Arc，供后台 task 使用（满足 'static 生命周期）
        let registry = Arc::clone(&self.scanner_registry);
        let store = Arc::clone(&self.temp_store);
        let state = Arc::clone(&self.state);
        let scan_id_for_task = scan_id.clone();
        let progress_tx = Arc::clone(&self.progress_tx);

        tokio::spawn(async move {
            // 细粒度进度回调：将 ScanProgress 转为 ProgressEvent 推送到前端
            let progress_callback = {
                let tx = progress_tx;
                move |p: crate::scanner::ScanProgress| {
                    if let Ok(guard) = tx.lock() {
                        if let Some(ref sender) = *guard {
                            let _ = sender.try_send(ProgressEvent::ScanProgress {
                                scanner_id: p.scanner_id,
                                current: p.current,
                                total: p.total,
                                message: p.message,
                            });
                        }
                    }
                }
            };

            match registry.scan_all(&ctx, &pause_rx, &progress_callback).await {
                Ok(items) => {
                    let count = items.len();
                    // 分批落盘，每批 500 条，避免单批次过大导致内存或 IO 压力
                    for chunk in items.chunks(500) {
                        if let Err(e) = store.save_scan_batch(chunk) {
                            log::error!("[{}] 保存扫描批次失败: {}", scan_id_for_task, e);
                            let _ = state.lock().map(|mut g| {
                                *g = SessionState::Failed {
                                    reason: format!("保存扫描结果失败: {}", e),
                                };
                            });
                            return;
                        }
                    }

                    // 扫描成功完成：Scanning → Scanned
                    let _ = state.lock().map(|mut g| {
                        *g = SessionState::Scanned { item_count: count };
                    });
                }
                Err(e) => {
                    log::error!("[{}] 扫描失败: {}", scan_id_for_task, e);
                    // 扫描异常：转为 Failed
                    let _ = state.lock().map(|mut g| {
                        *g = SessionState::Failed {
                            reason: format!("扫描失败: {}", e),
                        };
                    });
                }
            }
        });

        Ok(scan_id)
    }

    /// 暂停当前扫描会话
    ///
    /// 仅当状态为 Scanning 时有效。
    /// 通过 watch channel 发送 true，各 Scanner 在检查点轮询后进入等待。
    pub fn pause_session(&self) -> Result<(), BackendError> {
        let current = self.current_state();
        match current {
            SessionState::Scanning { .. } => {
                if let Ok(guard) = self.pause_tx.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(true);
                    }
                }
                self.transition_to(SessionState::Paused)
            }
            _ => Err(BackendError::OrchestratorError(format!(
                "非法状态，无法暂停: {:?}",
                current
            ))),
        }
    }

    /// 恢复当前扫描会话
    ///
    /// 仅当状态为 Paused 时有效。
    /// 通过 watch channel 发送 false，Scanner 继续执行。
    /// 恢复后状态回到 Scanning，并保留原 scan_id。
    pub fn resume_session(&self) -> Result<(), BackendError> {
        let current = self.current_state();
        match current {
            SessionState::Paused => {
                if let Ok(guard) = self.pause_tx.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(false);
                    }
                }
                let scan_id = self
                    .current_scan_id
                    .lock()
                    .ok()
                    .and_then(|g| g.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                self.transition_to(SessionState::Scanning { scan_id })
            }
            _ => Err(BackendError::OrchestratorError(format!(
                "非法状态，无法恢复: {:?}",
                current
            ))),
        }
    }

    /// 提交用户决策
    ///
    /// 接受状态：Confirming 或 Scanned（自动转为 Confirming）。
    /// 提交后状态强制转为 Executing，这是 RULE-01 的执行层保障：
    /// 只有在用户明确提交决策清单后，才会进入执行阶段，防止误删。
    pub fn submit_decisions(&self, decisions: Vec<Decision>) -> Result<(), BackendError> {
        let current = self.current_state();
        match current {
            SessionState::Confirming => {}
            SessionState::Scanned { .. } => {
                self.transition_to(SessionState::Confirming)?;
            }
            _ => {
                return Err(BackendError::OrchestratorError(format!(
                    "非法状态，无法提交决策: {:?}",
                    current
                )));
            }
        }

        if let Ok(mut guard) = self.decisions.lock() {
            *guard = decisions;
        }

        // Confirming → Executing：此后不允许再修改决策
        self.transition_to(SessionState::Executing)
    }

    /// 执行用户决策计划
    ///
    /// 完整流程：
    /// 1. 校验状态为 Executing
    /// 2. 从 TempStore 分页读取全部扫描结果
    /// 3. 按决策中的 action 分发到对应 Executor（Delete / Pack / Preserve）
    /// 4. 调用 PackExecutor::finalize() 生成 French-exit.zip（RULE-06）
    /// 5. 汇总 ExecutionResult 生成 ExecutionReport
    /// 6. 调用 Reporter 生成 HTML 庆祝页并打开浏览器
    /// 7. 调用 TempStore::self_destruct() 清理临时数据（RULE-04）
    /// 8. 状态转为 Completed，返回 ExecutionReport
    ///
    /// 注意：HTML 报告保存位置遵循 RULE-09（有打包放 zip 同目录，无打包放桌面）。
    pub fn execute_plan(&self) -> Result<ExecutionReport, BackendError> {
        let current = self.current_state();
        if !matches!(current, SessionState::Executing) {
            return Err(BackendError::OrchestratorError(format!(
                "非法状态，无法执行计划: {:?}",
                current
            )));
        }

        // ------------------------------------------------------------------
        // 阶段 1：加载全部扫描结果
        // ------------------------------------------------------------------
        let mut all_items = Vec::new();
        let mut offset = 0usize;
        const PAGE_SIZE: usize = 1000;
        loop {
            let page = self
                .temp_store
                .load_scan_results(offset, PAGE_SIZE)
                .map_err(|e| {
                    BackendError::OrchestratorError(format!("读取扫描结果失败: {}", e))
                })?;
            if page.is_empty() {
                break;
            }
            offset += page.len();
            all_items.extend(page);
        }

        let item_map: HashMap<String, crate::types::TraceItem> =
            all_items.into_iter().map(|item| (item.id.clone(), item)).collect();

        let decisions = self
            .decisions
            .lock()
            .map_err(|e| BackendError::OrchestratorError(format!("决策锁被污染: {:?}", e)))?
            .clone();

        // ------------------------------------------------------------------
        // 阶段 2：按决策分发到各 Executor
        // ------------------------------------------------------------------
        let mut results: Vec<ExecutionResult> = Vec::new();
        let mut deleted_count = 0usize;
        let mut deleted_bytes = 0u64;
        let mut packed_count = 0usize;
        let mut packed_bytes = 0u64;
        let mut preserved_count = 0usize;

        for decision in decisions {
            let item = match item_map.get(&decision.item_id) {
                Some(i) => i,
                None => {
                    results.push(ExecutionResult {
                        item_id: decision.item_id.clone(),
                        action: decision.action,
                        status: ExecutionStatus::Skipped(format!(
                            "未在扫描结果中找到: {}",
                            decision.item_id
                        )),
                        detail: None,
                    });
                    continue;
                }
            };

            let exec_result = match decision.action {
                Action::Delete => self.delete_executor.execute(item),
                Action::Pack => self.pack_executor.execute(item),
                Action::Preserve => self.preserve_executor.execute(item),
                Action::DeleteOrPack => {
                    // 前端应当已将 DeleteOrPack 解析为具体动作（Delete 或 Pack）。
                    // 若仍存在未解析的 DeleteOrPack，默认执行删除（安全优先）。
                    log::warn!(
                        "决策中仍存在未解析的 DeleteOrPack，item_id={}，默认按删除处理",
                        decision.item_id
                    );
                    self.delete_executor.execute(item)
                }
            };

            let result = match exec_result {
                Ok(r) => r,
                Err(e) => ExecutionResult {
                    item_id: decision.item_id.clone(),
                    action: decision.action,
                    status: ExecutionStatus::Failed(e.to_string()),
                    detail: None,
                },
            };

            // 统计成功执行项
            if matches!(result.status, ExecutionStatus::Success) {
                match result.action {
                    Action::Delete => {
                        deleted_count += 1;
                        deleted_bytes += item.size_bytes.unwrap_or(0);
                    }
                    Action::Pack => {
                        packed_count += 1;
                        packed_bytes += item.size_bytes.unwrap_or(0);
                    }
                    Action::Preserve => {
                        preserved_count += 1;
                    }
                    Action::DeleteOrPack => {}
                }
            }

            results.push(result);
        }

        // ------------------------------------------------------------------
        // 阶段 3：打包收尾（RULE-06：文件名固定为 French-exit.zip）
        // ------------------------------------------------------------------
        let pack_file_path = match self.pack_executor.finalize() {
            Ok(path) => Some(path),
            Err(e) => {
                log::error!("打包 finalize 失败: {}", e);
                None
            }
        };

        // 处理被用户跳过的加密文件：将对应 ExecutionResult 从 Success 改为 Skipped
        let skipped_ids = self.pack_executor.take_skipped_items();
        for skipped_id in &skipped_ids {
            if let Some(result) = results.iter_mut().find(|r| r.item_id == *skipped_id) {
                if matches!(result.status, crate::types::ExecutionStatus::Success) {
                    result.status = crate::types::ExecutionStatus::Skipped(
                        "用户取消加密文件打包".to_string(),
                    );
                    result.detail = Some("检测到加密文件，用户选择不打包".to_string());
                    if let Some(item) = item_map.get(skipped_id) {
                        packed_count = packed_count.saturating_sub(1);
                        packed_bytes = packed_bytes.saturating_sub(item.size_bytes.unwrap_or(0));
                    }
                }
            }
        }

        let report = ExecutionReport {
            deleted_count,
            deleted_bytes,
            packed_count,
            packed_bytes,
            preserved_count,
            pack_file_path,
            items: results,
        };

        // ------------------------------------------------------------------
        // 阶段 4：生成 HTML 庆祝页并打开浏览器
        // ------------------------------------------------------------------
        // RULE-09：有打包则放 zip 同目录，无打包则放桌面
        let output_dir = if let Some(ref path) = report.pack_file_path {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::temp_dir())
        } else {
            std::env::var("USERPROFILE")
                .map(|p| PathBuf::from(p).join("Desktop"))
                .unwrap_or_else(|_| std::env::temp_dir())
        };

        let html_path = Reporter::write_report(&report, &output_dir)
            .map_err(|e| BackendError::OrchestratorError(format!("生成报告失败: {}", e)))?;

        Reporter::open_in_browser(&html_path)
            .map_err(|e| BackendError::OrchestratorError(format!("打开浏览器失败: {}", e)))?;

        // ------------------------------------------------------------------
        // 阶段 5：临时数据自毁（RULE-04）
        // ------------------------------------------------------------------
        // HTML 报告已保存在 TempStore 外部，不会被 self_destruct 清理
        self.temp_store
            .self_destruct()
            .map_err(|e| BackendError::OrchestratorError(format!("临时存储自毁失败: {}", e)))?;

        // ------------------------------------------------------------------
        // 阶段 6：状态收尾
        // ------------------------------------------------------------------
        self.transition_to(SessionState::Completed {
            report: report.clone(),
        })?;

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_state_transitions() {
        // Idle → Scanning
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Idle,
            &SessionState::Scanning { scan_id: "x".to_string() }
        ));
        // Scanning → Paused
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Scanning { scan_id: "x".to_string() },
            &SessionState::Paused
        ));
        // Paused → Scanning
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Paused,
            &SessionState::Scanning { scan_id: "x".to_string() }
        ));
        // Scanning → Scanned
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Scanning { scan_id: "x".to_string() },
            &SessionState::Scanned { item_count: 0 }
        ));
        // Scanned → Confirming
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Scanned { item_count: 0 },
            &SessionState::Confirming
        ));
        // Confirming → Executing
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Confirming,
            &SessionState::Executing
        ));
        // Executing → Completed
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Executing,
            &SessionState::Completed { report: ExecutionReport { deleted_count: 0, deleted_bytes: 0, packed_count: 0, packed_bytes: 0, preserved_count: 0, pack_file_path: None, items: vec![] } }
        ));
        // Executing → Failed
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Executing,
            &SessionState::Failed { reason: "test".to_string() }
        ));
        // Any → Idle
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Failed { reason: "test".to_string() },
            &SessionState::Idle
        ));
    }

    #[test]
    fn test_invalid_state_transitions() {
        // Idle → Scanned (跳过 Scanning)
        assert!(!Orchestrator::is_valid_transition(
            &SessionState::Idle,
            &SessionState::Scanned { item_count: 0 }
        ));
        // Idle → Executing
        assert!(!Orchestrator::is_valid_transition(
            &SessionState::Idle,
            &SessionState::Executing
        ));
        // Scanned → Executing (跳过 Confirming)
        assert!(!Orchestrator::is_valid_transition(
            &SessionState::Scanned { item_count: 0 },
            &SessionState::Executing
        ));
        // Confirming → Scanned (反向)
        assert!(!Orchestrator::is_valid_transition(
            &SessionState::Confirming,
            &SessionState::Scanned { item_count: 0 }
        ));
        // Completed → Idle (Any → Idle 是允许的，所以这是合法的)
        assert!(Orchestrator::is_valid_transition(
            &SessionState::Completed { report: ExecutionReport { deleted_count: 0, deleted_bytes: 0, packed_count: 0, packed_bytes: 0, preserved_count: 0, pack_file_path: None, items: vec![] } },
            &SessionState::Idle
        ));
    }

    #[test]
    fn test_orchestrator_initial_state() {
        let orch = create_test_orchestrator();
        assert!(matches!(orch.current_state(), SessionState::Idle));
    }

    #[test]
    fn test_transition_to_invalid() {
        let orch = create_test_orchestrator();
        // Idle → Executing 是非法转换
        let result = orch.transition_to(SessionState::Executing);
        assert!(result.is_err());
        assert!(matches!(orch.current_state(), SessionState::Idle));
    }

    #[test]
    fn test_pause_session_invalid_state() {
        let orch = create_test_orchestrator();
        // Idle 状态下无法暂停
        let result = orch.pause_session();
        assert!(result.is_err());
        assert!(matches!(orch.current_state(), SessionState::Idle));
    }

    #[test]
    fn test_resume_session_invalid_state() {
        let orch = create_test_orchestrator();
        // Idle 状态下无法恢复
        let result = orch.resume_session();
        assert!(result.is_err());
        assert!(matches!(orch.current_state(), SessionState::Idle));
    }

    #[test]
    fn test_submit_decisions_from_scanned() {
        let orch = create_test_orchestrator();
        // 合法路径：Idle → Scanning → Scanned
        orch.transition_to(SessionState::Scanning {
            scan_id: "test-1".to_string(),
        })
        .unwrap();
        orch.transition_to(SessionState::Scanned { item_count: 3 })
            .unwrap();

        // 从 Scanned 提交决策应自动经过 Confirming → Executing
        let decisions = vec![
            Decision {
                item_id: "a".to_string(),
                action: Action::Delete,
            },
        ];
        let result = orch.submit_decisions(decisions);
        assert!(result.is_ok());
        assert!(matches!(orch.current_state(), SessionState::Executing));
    }

    #[test]
    fn test_submit_decisions_invalid_state() {
        let orch = create_test_orchestrator();
        // Idle 状态下无法提交决策
        let decisions = vec![Decision {
            item_id: "a".to_string(),
            action: Action::Delete,
        }];
        let result = orch.submit_decisions(decisions);
        assert!(result.is_err());
        assert!(matches!(orch.current_state(), SessionState::Idle));
    }

    #[test]
    fn test_execute_plan_invalid_state() {
        let orch = create_test_orchestrator();
        // Idle 状态下无法执行计划
        let result = orch.execute_plan();
        assert!(result.is_err());
        assert!(matches!(orch.current_state(), SessionState::Idle));
    }

    /// 构造一个用于测试的 Orchestrator，使用真实依赖但空 ScannerRegistry
    fn create_test_orchestrator() -> Orchestrator {
        use crate::executor::secure_erase::DoDEraser;
        use crate::scanner::registry::ScannerRegistry;
        use std::path::PathBuf;

        let scanner_registry = ScannerRegistry::new();
        let test_dir = std::env::temp_dir()
            .join("french-exit")
            .join(format!("test-{}", uuid::Uuid::new_v4()));
        let temp_store = Arc::new(TempStore::with_root(test_dir).unwrap());
        let resource_controller = Arc::new(ResourceController::new());
        let eraser = Arc::new(DoDEraser::default());
        let delete_executor = crate::executor::delete::DeleteExecutor::new(eraser);
        let pack_executor = crate::executor::pack::PackExecutor::new(PathBuf::from("."), None);
        let preserve_executor = crate::executor::preserve::PreserveExecutor::new();

        Orchestrator::new(
            scanner_registry,
            temp_store,
            resource_controller,
            delete_executor,
            pack_executor,
            preserve_executor,
        )
    }
}
