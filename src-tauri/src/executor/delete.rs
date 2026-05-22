use std::sync::Arc;

use crate::error::BackendError;
use crate::executor::secure_erase::SecureEraser;
use crate::executor::Executor;
use crate::types::{Action, ExecutionResult, ExecutionStatus, TraceCategory, TraceItem};

/// 删除执行器
///
/// 对标记为 `Action::Delete` 的 `TraceItem` 调用安全擦除。
/// 文件和目录由 `SecureEraser` 处理；注册表和环境变量暂时跳过，留待后续迭代。
pub struct DeleteExecutor {
    /// DoD 安全擦除器（当前策略已降级为普通删除，保留字段以便未来恢复）
    #[allow(dead_code)]
    secure_eraser: Arc<dyn SecureEraser>,
    /// 测试模式：不实际删除文件，仅记录日志
    dry_run: bool,
}

impl DeleteExecutor {
    /// 创建新的删除执行器
    ///
    /// # 参数
    /// - `eraser`: 安全擦除器实例，使用 `Arc` 包装以便多线程共享
    pub fn new(eraser: Arc<dyn SecureEraser>) -> Self {
        Self {
            secure_eraser: eraser,
            dry_run: std::env::var("FRENCH_EXIT_DRY_RUN").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false),
        }
    }

    /// 创建指定模式的删除执行器（用于测试显式控制）
    pub fn with_mode(eraser: Arc<dyn SecureEraser>, dry_run: bool) -> Self {
        Self { secure_eraser: eraser, dry_run }
    }
}

impl Executor for DeleteExecutor {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, BackendError> {
        match item.category {
            // 以下类别通常带有可操作的路径（文件或目录）
            TraceCategory::FileSystem
            | TraceCategory::Chat
            | TraceCategory::Browser
            | TraceCategory::System
            | TraceCategory::DevTools => {
                if let Some(ref path) = item.path {
                    if path.exists() {
                        if self.dry_run {
                            // 测试模式：仅记录，不实际删除
                            log::info!("[DRY RUN] 将删除: {}", path.display());
                            return Ok(ExecutionResult {
                                item_id: item.id.clone(),
                                action: Action::Delete,
                                status: ExecutionStatus::Success,
                                detail: Some(format!("[测试模式] 模拟删除: {}", path.display())),
                            });
                        }
                        // 普通删除（非安全擦除），可通过数据恢复软件恢复
                        if path.is_file() {
                            std::fs::remove_file(path)
                                .map_err(|e| BackendError::IoError(e))?;
                        } else if path.is_dir() {
                            std::fs::remove_dir_all(path)
                                .map_err(|e| BackendError::IoError(e))?;
                        }
                        Ok(ExecutionResult {
                            item_id: item.id.clone(),
                            action: Action::Delete,
                            status: ExecutionStatus::Success,
                            detail: Some(format!(
                                "已删除: {}",
                                path.display()
                            )),
                        })
                    } else {
                        // 路径不存在，无需操作，标记为跳过
                        Ok(ExecutionResult {
                            item_id: item.id.clone(),
                            action: Action::Delete,
                            status: ExecutionStatus::Skipped(format!(
                                "路径不存在: {}",
                                path.display()
                            )),
                            detail: None,
                        })
                    }
                } else {
                    // 无可操作路径，标记为跳过
                    Ok(ExecutionResult {
                        item_id: item.id.clone(),
                        action: Action::Delete,
                        status: ExecutionStatus::Skipped("该条目无可操作路径".to_string()),
                        detail: None,
                    })
                }
            }

            // 注册表删除需要管理员权限和 Windows API，当前版本暂不实现
            TraceCategory::Registry => Ok(ExecutionResult {
                item_id: item.id.clone(),
                action: Action::Delete,
                status: ExecutionStatus::Skipped(
                    "注册表删除需要管理员权限，暂由系统工具处理".to_string(),
                ),
                detail: Some(format!("注册表项: {:?}", item.path)),
            }),

            // 环境变量删除需要修改系统配置，当前版本跳过，由用户手动处理或后续迭代支持
            TraceCategory::EnvVar => Ok(ExecutionResult {
                item_id: item.id.clone(),
                action: Action::Delete,
                status: ExecutionStatus::Skipped("环境变量修改需手动处理".to_string()),
                detail: Some(item.name.clone()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::secure_erase::DoDEraser;

    #[test]
    fn test_delete_executor_registry_skipped() {
        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);

        let item = TraceItem {
            id: "reg-test".to_string(),
            category: TraceCategory::Registry,
            scanner_id: "test".to_string(),
            name: "注册表项".to_string(),
            path: None,
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Delete),
        };

        let result = executor.execute(&item).unwrap();
        assert!(matches!(result.status, ExecutionStatus::Skipped(_)));
        assert_eq!(result.item_id, "reg-test");
        assert_eq!(result.action, Action::Delete);
    }

    #[test]
    fn test_delete_executor_envvar_skipped() {
        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);

        let item = TraceItem {
            id: "env-test".to_string(),
            category: TraceCategory::EnvVar,
            scanner_id: "test".to_string(),
            name: "环境变量".to_string(),
            path: None,
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Delete),
        };

        let result = executor.execute(&item).unwrap();
        assert!(matches!(result.status, ExecutionStatus::Skipped(_)));
    }

    #[test]
    fn test_delete_executor_no_path_skipped() {
        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);

        let item = TraceItem {
            id: "fs-test".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "文件".to_string(),
            path: None,
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Delete),
        };

        let result = executor.execute(&item).unwrap();
        assert!(matches!(result.status, ExecutionStatus::Skipped(_)));
    }

    // 以下测试覆盖实际文件/目录删除路径（FileSystem / Chat / Browser / System / DevTools）

    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_test_path(name: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "french-exit-test-{}-{}-{}",
            name,
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        dir
    }

    #[test]
    fn test_delete_executor_file_removed() {
        let path = temp_test_path("file");
        std::fs::write(&path, "test content").unwrap();
        assert!(path.exists());

        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);
        let item = TraceItem {
            id: "file-test".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "测试文件".to_string(),
            path: Some(path.clone()),
            size_bytes: Some(12),
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Delete),
        };

        let result = executor.execute(&item).unwrap();
        assert!(matches!(result.status, ExecutionStatus::Success));
        assert_eq!(result.item_id, "file-test");
        assert_eq!(result.action, Action::Delete);
        assert!(!path.exists());
    }

    #[test]
    fn test_delete_executor_directory_removed() {
        let path = temp_test_path("dir");
        std::fs::create_dir_all(&path).unwrap();
        let child = path.join("child.txt");
        std::fs::write(&child, "nested").unwrap();
        assert!(path.exists());
        assert!(child.exists());

        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);
        let item = TraceItem {
            id: "dir-test".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "测试目录".to_string(),
            path: Some(path.clone()),
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Delete),
        };

        let result = executor.execute(&item).unwrap();
        assert!(matches!(result.status, ExecutionStatus::Success));
        assert!(!path.exists());
        assert!(!child.exists());
    }

    #[test]
    fn test_delete_executor_missing_path_skipped() {
        let path = temp_test_path("missing");
        assert!(!path.exists());

        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);
        let item = TraceItem {
            id: "missing-test".to_string(),
            category: TraceCategory::FileSystem,
            scanner_id: "test".to_string(),
            name: "不存在的文件".to_string(),
            path: Some(path),
            size_bytes: None,
            modified_at: None,
            inferred: false,
            risk_note: None,
            suggested_action: Some(Action::Delete),
        };

        let result = executor.execute(&item).unwrap();
        assert!(matches!(result.status, ExecutionStatus::Skipped(_)));
    }

    #[test]
    fn test_delete_executor_chat_browser_system_devtools_removed() {
        let eraser = Arc::new(DoDEraser::default());
        let executor = DeleteExecutor::new(eraser);

        for category in [
            TraceCategory::Chat,
            TraceCategory::Browser,
            TraceCategory::System,
            TraceCategory::DevTools,
        ] {
            let path = temp_test_path(&format!("{:?}", category).to_lowercase());
            std::fs::write(&path, "x").unwrap();

            let item = TraceItem {
                id: format!("{:?}-test", category),
                category,
                scanner_id: "test".to_string(),
                name: format!("{:?}", category),
                path: Some(path.clone()),
                size_bytes: None,
                modified_at: None,
                inferred: false,
                risk_note: None,
                suggested_action: Some(Action::Delete),
            };

            let result = executor.execute(&item).unwrap();
            assert!(
                matches!(result.status, ExecutionStatus::Success),
                "{:?} should delete file",
                category
            );
            assert!(!path.exists(), "{:?} file should be removed", category);
        }
    }
}
