pub mod secure_erase;
pub mod delete;
pub mod pack;
pub mod preserve;

use crate::types::{ExecutionResult, TraceItem};
use crate::error::BackendError;

/// 所有执行器必须实现的 trait
///
/// 执行器负责对单个 TraceItem 执行用户决策（删除/保留/打包）。
pub trait Executor: Send + Sync {
    /// 执行操作
    ///
    /// # 参数
    /// - `item`: 待执行的痕迹条目
    ///
    /// # 返回值
    /// 执行结果，包含成功/失败/跳过状态和详细信息。
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, BackendError>;
}
