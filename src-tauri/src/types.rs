use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

pub type TraceItemId = String;
pub type ScanId = String;
pub type ExecutionPlanId = String;
pub type SessionId = String;

/// 痕迹类别
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TraceCategory {
    Chat,
    Browser,
    System,
    Registry,
    FileSystem,
    DevTools,
    EnvVar,
}

/// 扫描上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanContext {
    pub start_date: NaiveDate,
    pub user_home: PathBuf,
    pub temp_dir: PathBuf,
}

/// 单条痕迹
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TraceItem {
    pub id: TraceItemId,
    pub category: TraceCategory,
    pub scanner_id: String,
    pub name: String,
    pub path: Option<PathBuf>,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<DateTime<Local>>,
    pub inferred: bool,
    pub risk_note: Option<String>,
    pub suggested_action: Option<Action>,
    /// 来源分类：personal_desktop / personal_downloads / personal_documents / other
    pub source: String,
    /// 文件类型分类：photo / video / audio / personal_doc / work_doc / code / archive / design / executable / temp / other
    pub file_type: String,
}

/// 用户决策
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Decision {
    pub item_id: TraceItemId,
    pub action: Action,
}

/// 处理方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum Action {
    Delete,
    Preserve,
    Pack,
    /// 微信等聊天软件记录：建议删除或打包
    DeleteOrPack,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutionResult {
    pub item_id: TraceItemId,
    pub action: Action,
    pub status: ExecutionStatus,
    pub detail: Option<String>,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum ExecutionStatus {
    Success,
    Failed(String),
    Skipped(String),
}

/// 执行报告
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutionReport {
    pub deleted_count: usize,
    pub deleted_bytes: u64,
    pub packed_count: usize,
    pub packed_bytes: u64,
    pub preserved_count: usize,
    pub pack_file_path: Option<PathBuf>,
    pub items: Vec<ExecutionResult>,
}

/// 进度事件（推送到前端）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ProgressEvent {
    ScanStarted { total_scanners: usize },
    ScanProgress {
        scanner_id: String,
        current: usize,
        total: usize,
        message: String,
        /// 全局加权进度百分比（0-100），由 ScannerRegistry 计算
        global_percent: Option<u8>,
    },
    ScanCompleted { item_count: usize },
    ScanFailed { reason: String },
    ScanPaused,
    ScanResumed,
    ExecutionStarted { total_items: usize },
    ExecutionProgress {
        current: usize,
        total: usize,
        message: String,
    },
    ExecutionCompleted { report: ExecutionReport },
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PaginatedResult<T>
where
    T: TS,
{
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// 资源配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResourceConfig {
    pub cpu_limit_percent: u8,
    pub unlimited: bool,
}

/// 扫描结果轻量摘要（用于全选全部，避免加载完整 TraceItem）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScanResultSummary {
    pub id: TraceItemId,
    pub name: String,
    pub category: TraceCategory,
    pub suggested_action: Option<Action>,
}

/// 预览结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum PreviewResult {
    Text(String),
    Image(PathBuf),
    Unsupported(String),
}
