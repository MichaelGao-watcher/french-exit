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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_error_display_variants() {
        assert_eq!(
            format!("{}", BackendError::ScanError("foo".to_string())),
            "扫描错误: foo"
        );
        assert_eq!(
            format!("{}", BackendError::ExecutionError("bar".to_string())),
            "执行错误: bar"
        );
        assert_eq!(
            format!("{}", BackendError::EraseError("baz".to_string())),
            "擦除错误: baz"
        );
        assert_eq!(
            format!("{}", BackendError::ResourceError("qux".to_string())),
            "资源控制错误: qux"
        );
        assert_eq!(
            format!("{}", BackendError::OrchestratorError("quux".to_string())),
            "流程调度错误: quux"
        );
        assert_eq!(
            format!("{}", BackendError::StoreError("corge".to_string())),
            "存储错误: corge"
        );
    }

    #[test]
    fn test_backend_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let backend_err: BackendError = io_err.into();
        assert!(matches!(backend_err, BackendError::IoError(_)));
        assert_eq!(format!("{}", backend_err), "IO错误: file not found");
    }

    #[test]
    fn test_backend_to_frontend_error_mapping() {
        let fe: FrontendError = BackendError::ScanError("scan failed".to_string()).into();
        assert_eq!(fe.code, "SCAN_ERROR");
        assert_eq!(fe.message, "scan failed");

        let fe: FrontendError = BackendError::ExecutionError("exec failed".to_string()).into();
        assert_eq!(fe.code, "EXECUTION_ERROR");
        assert_eq!(fe.message, "exec failed");

        let fe: FrontendError = BackendError::EraseError("erase failed".to_string()).into();
        assert_eq!(fe.code, "ERASE_ERROR");

        let fe: FrontendError = BackendError::ResourceError("resource failed".to_string()).into();
        assert_eq!(fe.code, "RESOURCE_ERROR");

        let fe: FrontendError = BackendError::OrchestratorError("orch failed".to_string()).into();
        assert_eq!(fe.code, "ORCHESTRATOR_ERROR");

        let fe: FrontendError = BackendError::StoreError("store failed".to_string()).into();
        assert_eq!(fe.code, "STORE_ERROR");

        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io failed");
        let fe: FrontendError = BackendError::IoError(io_err).into();
        assert_eq!(fe.code, "IO_ERROR");
        assert_eq!(fe.message, "io failed");
    }

    #[test]
    fn test_frontend_error_serialize() {
        let err = FrontendError {
            code: "TEST".to_string(),
            message: "hello".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":\"TEST\""));
        assert!(json.contains("\"message\":\"hello\""));
    }
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
