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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ExecutionStatus, TraceCategory};

    #[test]
    fn test_preserve_executor_new() {
        let executor = PreserveExecutor::new();
        // PreserveExecutor 是零大小类型，只需确认能构造
        let _ = executor;
    }

    #[test]
    fn test_preserve_executor_execute() {
        let executor = PreserveExecutor::new();
        let item = TraceItem {
            id: "item-1".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "scanner-fs".to_string(),
            name: "工作文件.txt".to_string(),
            path: None,
            size_bytes: Some(1024),
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Preserve),
        };

        let result = executor.execute(&item).unwrap();
        assert_eq!(result.item_id, "item-1");
        assert_eq!(result.action, Action::Preserve);
        assert!(matches!(result.status, ExecutionStatus::Success));
        assert_eq!(result.detail, Some("用户选择保留".to_string()));
    }
}
