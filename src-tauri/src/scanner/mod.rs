use crate::types::{ScanContext, TraceCategory, TraceItem};
use thiserror::Error;
use tokio::sync::watch;

/// 扫描错误类型
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("无效路径: {0}")]
    InvalidPath(String),

    #[error("权限拒绝: {0}")]
    PermissionDenied(String),

    #[error("不支持: {0}")]
    Unsupported(String),

    #[error("扫描被中断")]
    Interrupted,

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 扫描进度
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub scanner_id: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
    /// 全局加权进度百分比（0-100），由 ScannerRegistry 计算。
    /// 若存在则前端应优先使用此值，而非自行计算局部进度。
    pub global_percent: Option<u8>,
}

/// 所有扫描器必须实现的 trait
///
/// 扫描器负责发现特定类别的痕迹数据，返回结构化的 `TraceItem` 列表。
/// 每个扫描器在独立线程中运行（由 ScannerRegistry 调度），因此要求 `Send + Sync`。
pub trait Scanner: Send + Sync {
    /// 扫描器唯一标识（如 "scanner-fs"）
    fn id(&self) -> &'static str;

    /// 扫描器所属痕迹类别
    fn category(&self) -> TraceCategory;

    /// 人类可读名称（如 "文件系统扫描器"）
    fn display_name(&self) -> &'static str;

    /// 执行扫描
    ///
    /// # 参数
    /// - `ctx`: 扫描上下文（入职日期、用户目录等）
    /// - `pause_rx`: 暂停信号通道，`true` 表示暂停
    /// - `progress`: 进度回调，扫描器应定期调用以报告进度
    ///
    /// # 返回值
    /// 发现的痕迹列表。如无痕迹，返回空 Vec。
    fn scan(
        &self,
        ctx: &ScanContext,
        pause_rx: &watch::Receiver<bool>,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError>;
}

pub mod registry;
pub mod env;
pub mod fs;
pub mod browser;
pub mod chat;
pub mod registry_sys;
pub mod system;
pub mod devtools;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_error_display_variants() {
        assert_eq!(
            format!("{}", ScanError::InvalidPath("/bad".to_string())),
            "无效路径: /bad"
        );
        assert_eq!(
            format!("{}", ScanError::PermissionDenied("denied".to_string())),
            "权限拒绝: denied"
        );
        assert_eq!(
            format!("{}", ScanError::Unsupported("unsupported".to_string())),
            "不支持: unsupported"
        );
        assert_eq!(
            format!("{}", ScanError::Internal("internal".to_string())),
            "内部错误: internal"
        );
        assert_eq!(format!("{}", ScanError::Interrupted), "扫描被中断");
    }

    #[test]
    fn test_scan_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let scan_err: ScanError = io_err.into();
        assert!(matches!(scan_err, ScanError::IoError(_)));
        assert_eq!(format!("{}", scan_err), "IO错误: missing");
    }
}
