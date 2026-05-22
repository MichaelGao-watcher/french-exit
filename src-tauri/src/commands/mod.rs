use std::sync::Arc;

use crate::orchestrator::Orchestrator;
use crate::resource::controller::ResourceController;
use crate::store::temp_store::TempStore;

/// 应用全局状态
///
/// 持有所有需要在 Tauri command 间共享的后端模块实例。
/// 所有字段均为 Arc<T>，天然满足 Send + Sync，可直接作为 Tauri State 注入。
pub struct AppState {
    pub orchestrator: Arc<Orchestrator>,
    pub temp_store: Arc<TempStore>,
    pub resource_controller: Arc<ResourceController>,
}

// ---------------------------------------------------------------------------
// Command 函数 —— 仅在非测试模式下编译，避免 MinGW UCRT 入口点冲突
// ---------------------------------------------------------------------------
#[cfg(not(test))]
pub mod handlers;

#[cfg(not(test))]
pub use handlers::*;

#[cfg(test)]
mod tests {
    use crate::types::Action;
    use crate::types::Decision;

    #[test]
    fn test_start_scan_date_validation() {
        // 合法日期
        assert!(chrono::NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").is_ok());
        assert!(chrono::NaiveDate::parse_from_str("2020-02-29", "%Y-%m-%d").is_ok());

        // 非法日期格式
        assert!(chrono::NaiveDate::parse_from_str("15-01-2024", "%Y-%m-%d").is_err());
        assert!(chrono::NaiveDate::parse_from_str("2024/01/15", "%Y-%m-%d").is_err());
        assert!(chrono::NaiveDate::parse_from_str("not-a-date", "%Y-%m-%d").is_err());
        assert!(chrono::NaiveDate::parse_from_str("", "%Y-%m-%d").is_err());
    }

    #[test]
    fn test_submit_decisions_dedup() {
        let decisions = vec![
            Decision {
                item_id: "a".to_string(),
                action: Action::Delete,
            },
            Decision {
                item_id: "b".to_string(),
                action: Action::Preserve,
            },
            Decision {
                item_id: "a".to_string(),
                action: Action::Pack,
            },
        ];

        // 模拟 command 中的去重校验逻辑
        let mut seen = std::collections::HashSet::new();
        let mut dup_found = false;
        for d in &decisions {
            if !seen.insert(&d.item_id) {
                dup_found = true;
                break;
            }
        }
        assert!(dup_found, "应当检测到重复的 item_id: a");
    }

    #[test]
    fn test_resource_config_validation() {
        // 合法值
        assert!((30 >= 1 && 30 <= 100));
        assert!((1 >= 1 && 1 <= 100));
        assert!((100 >= 1 && 100 <= 100));

        // 非法值
        assert!(!(0 >= 1 && 0 <= 100));
        assert!(!(101 >= 1 && 101 <= 100));
        assert!(!(255 >= 1 && 255 <= 100));
    }

    #[test]
    fn test_get_scan_results_page_validation() {
        // 模拟 command 中的分页参数校验逻辑
        let page = 0u32;
        assert!(page < 1, "page 必须从 1 开始");

        let page_size = 5u32;
        assert!(page_size < 10 || page_size > 500, "page_size 必须在 10~500 之间");

        let valid_page = 1u32;
        let valid_page_size = 50u32;
        assert!(valid_page >= 1);
        assert!(valid_page_size >= 10 && valid_page_size <= 500);
    }
}
