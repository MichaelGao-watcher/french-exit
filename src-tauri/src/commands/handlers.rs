use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::commands::AppState;
use crate::error::FrontendError;
use crate::orchestrator::{Orchestrator, SessionState};
use crate::types::{Decision, ExecutionReport, PaginatedResult, ResourceConfig, TraceItem};

/// CMD-02: 启动扫描
///
/// 参数：start_date (YYYY-MM-DD 格式字符串)
/// 返回：scan_id 或 FrontendError
#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    start_date: String,
) -> Result<String, FrontendError> {
    // 1. 解析日期
    let date = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| FrontendError {
            code: "INVALID_DATE".to_string(),
            message: "日期格式必须是 YYYY-MM-DD".to_string(),
        })?;

    // 2. 构造 ScanContext
    let user_home = std::env::var("USERPROFILE")
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|_| std::env::temp_dir());

    let ctx = crate::types::ScanContext {
        start_date: date,
        user_home,
        temp_dir: std::env::temp_dir(),
    };

    // 3. 创建进度通道并注入 orchestrator
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<crate::types::ProgressEvent>(128);
    state.orchestrator.set_progress_tx(progress_tx);

    // 4. 调用 orchestrator 启动扫描
    let scan_id = state.orchestrator.start_scan(ctx)?;

    // 5. 推送扫描开始事件
    let _ = app.emit(
        "scan_progress",
        crate::types::ProgressEvent::ScanStarted { total_scanners: 7 },
    );

    // 6. 启动后台进度转发 task：将 scanner 细粒度进度实时推送到前端
    let app_progress = app.clone();
    tokio::spawn(async move {
        while let Some(event) = progress_rx.recv().await {
            let _ = app_progress.emit("scan_progress", event);
        }
    });

    // 7. 启动后台监控 task，轮询 orchestrator 状态并推送事件
    let app_clone = app.clone();
    let orch = Arc::clone(&state.orchestrator);
    tokio::spawn(async move {
        let mut was_paused = false;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            match orch.current_state() {
                SessionState::Scanned { item_count } => {
                    let _ = app_clone.emit(
                        "scan_progress",
                        crate::types::ProgressEvent::ScanCompleted { item_count },
                    );
                    break;
                }
                SessionState::Failed { reason } => {
                    let _ = app_clone.emit(
                        "scan_progress",
                        crate::types::ProgressEvent::ScanFailed { reason },
                    );
                    break;
                }
                SessionState::Paused => {
                    if !was_paused {
                        let _ = app_clone.emit(
                            "scan_progress",
                            crate::types::ProgressEvent::ScanPaused,
                        );
                        was_paused = true;
                    }
                }
                SessionState::Scanning { .. } => {
                    if was_paused {
                        let _ = app_clone.emit(
                            "scan_progress",
                            crate::types::ProgressEvent::ScanResumed,
                        );
                        was_paused = false;
                    }
                }
                _ => break,
            }
        }
    });

    Ok(scan_id)
}

/// CMD-03: 暂停扫描
#[tauri::command]
pub async fn pause_scan(state: State<'_, AppState>) -> Result<(), FrontendError> {
    state.orchestrator.pause_session().map_err(FrontendError::from)
}

/// CMD-03: 恢复扫描
#[tauri::command]
pub async fn resume_scan(state: State<'_, AppState>) -> Result<(), FrontendError> {
    state.orchestrator.resume_session().map_err(FrontendError::from)
}

/// CMD-04: 获取扫描结果（分页）
///
/// 参数：page (从1开始), page_size (10~500)
/// 返回：PaginatedResult<TraceItem>
#[tauri::command]
pub async fn get_scan_results(
    state: State<'_, AppState>,
    page: u32,
    page_size: u32,
) -> Result<PaginatedResult<TraceItem>, FrontendError> {
    // 参数校验
    if page < 1 {
        return Err(FrontendError {
            code: "INVALID_PARAM".to_string(),
            message: "page 必须从 1 开始".to_string(),
        });
    }
    if page_size < 10 || page_size > 500 {
        return Err(FrontendError {
            code: "INVALID_PARAM".to_string(),
            message: "page_size 必须在 10~500 之间".to_string(),
        });
    }

    let offset = ((page - 1) as usize) * (page_size as usize);
    let limit = page_size as usize;

    // 加载当前页数据
    let items = state
        .temp_store
        .load_scan_results(offset, limit)
        .map_err(FrontendError::from)?;

    // 统计总数：从当前页末尾继续向后读取，直到空
    let mut total = offset + items.len();
    if items.len() == limit {
        let mut scan_offset = offset + limit;
        loop {
            let chunk = state
                .temp_store
                .load_scan_results(scan_offset, 1000)
                .map_err(FrontendError::from)?;
            if chunk.is_empty() {
                break;
            }
            total += chunk.len();
            scan_offset += chunk.len();
        }
    }

    Ok(PaginatedResult {
        items,
        total,
        page: page as usize,
        page_size: limit,
    })
}

/// CMD-06: 提交用户决策
#[tauri::command]
pub async fn submit_decisions(
    state: State<'_, AppState>,
    decisions: Vec<Decision>,
) -> Result<(), FrontendError> {
    // 校验：无重复 item_id
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for d in &decisions {
        if !seen.insert(&d.item_id) {
            return Err(FrontendError {
                code: "INVALID_PARAM".to_string(),
                message: format!("重复的 item_id: {}", d.item_id),
            });
        }
    }

    state
        .orchestrator
        .submit_decisions(decisions)
        .map_err(FrontendError::from)
}

/// CMD-07: 开始执行（执行用户决策）
#[tauri::command]
pub async fn start_execution(
    state: State<'_, AppState>,
) -> Result<ExecutionReport, FrontendError> {
    state.orchestrator.execute_plan().map_err(FrontendError::from)
}

/// CMD-08: 获取资源配置
#[tauri::command]
pub async fn get_resource_config(
    state: State<'_, AppState>,
) -> Result<ResourceConfig, FrontendError> {
    Ok(state.resource_controller.get_config())
}

/// CMD-08: 设置资源配置
#[tauri::command]
pub async fn set_resource_config(
    state: State<'_, AppState>,
    config: ResourceConfig,
) -> Result<(), FrontendError> {
    // 校验
    if config.cpu_limit_percent < 1 || config.cpu_limit_percent > 100 {
        return Err(FrontendError {
            code: "INVALID_PARAM".to_string(),
            message: "cpu_limit_percent 必须在 1~100 之间".to_string(),
        });
    }

    state
        .resource_controller
        .apply_limits(config)
        .map_err(FrontendError::from)
}

/// 获取当前会话状态
#[tauri::command]
pub async fn get_session_state(
    state: State<'_, AppState>,
) -> Result<SessionState, FrontendError> {
    Ok(state.orchestrator.current_state())
}

/// CMD-09: 获取所有扫描结果轻量摘要（用于全选全部）
#[tauri::command]
pub async fn get_all_scan_summaries(
    state: State<'_, AppState>,
) -> Result<Vec<crate::types::ScanResultSummary>, FrontendError> {
    let mut all = Vec::new();
    let mut offset = 0usize;
    const PAGE_SIZE: usize = 1000;
    loop {
        let page = state
            .temp_store
            .load_scan_results(offset, PAGE_SIZE)
            .map_err(FrontendError::from)?;
        if page.is_empty() {
            break;
        }
        for item in page {
            all.push(crate::types::ScanResultSummary {
                id: item.id,
                name: item.name.clone(),
                category: item.category,
                suggested_action: item.suggested_action,
            });
        }
        offset += PAGE_SIZE;
    }
    Ok(all)
}

/// CMD-10: 打开文件所在文件夹
#[tauri::command]
pub fn open_path(path: String) -> Result<(), FrontendError> {
    let parent = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    std::process::Command::new("explorer")
        .arg(&parent)
        .spawn()
        .map_err(|e| FrontendError {
            code: "OPEN_ERROR".to_string(),
            message: format!("打开路径失败: {}", e),
        })?;

    Ok(())
}
