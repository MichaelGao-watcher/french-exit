use serde::Serialize;
use thiserror::Error;

/// 前端友好的错误类型
#[derive(Debug, Clone, Serialize)]
pub struct FrontendError {
    pub code: String,
    pub message: String,
}

/// 内部后端错误类型
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("扫描错误: {0}")]
    ScanError(String),

    #[error("执行错误: {0}")]
    ExecutionError(String),

    #[error("擦除错误: {0}")]
    EraseError(String),

    #[error("资源控制错误: {0}")]
    ResourceError(String),

    #[error("流程调度错误: {0}")]
    OrchestratorError(String),

    #[error("存储错误: {0}")]
    StoreError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<BackendError> for FrontendError {
    fn from(err: BackendError) -> Self {
        match err {
            BackendError::ScanError(msg) => FrontendError {
                code: "SCAN_ERROR".to_string(),
                message: msg,
            },
            BackendError::ExecutionError(msg) => FrontendError {
                code: "EXECUTION_ERROR".to_string(),
                message: msg,
            },
            BackendError::EraseError(msg) => FrontendError {
                code: "ERASE_ERROR".to_string(),
                message: msg,
            },
            BackendError::ResourceError(msg) => FrontendError {
                code: "RESOURCE_ERROR".to_string(),
                message: msg,
            },
            BackendError::OrchestratorError(msg) => FrontendError {
                code: "ORCHESTRATOR_ERROR".to_string(),
                message: msg,
            },
            BackendError::StoreError(msg) => FrontendError {
                code: "STORE_ERROR".to_string(),
                message: msg,
            },
            BackendError::IoError(e) => FrontendError {
                code: "IO_ERROR".to_string(),
                message: e.to_string(),
            },
        }
    }
}
