use crate::error::BackendError;
use crate::executor::Executor;
use crate::types::{Action, ExecutionResult, ExecutionStatus, TraceItem};

/// 保留执行器
///
/// 对标记为 `Action::Preserve` 的 `TraceItem` 不执行任何文件系统操作，
/// 仅返回成功的执行结果，用于最终报告统计。
pub struct PreserveExecutor;

impl PreserveExecutor {
    /// 创建新的保留执行器
    pub fn new() -> Self {
        Self
    }
}

impl Executor for PreserveExecutor {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, BackendError> {
        Ok(ExecutionResult {
            item_id: item.id.clone(),
            action: Action::Preserve,
            status: ExecutionStatus::Success,
            detail: Some("用户选择保留".to_string()),
        })
    }
}
