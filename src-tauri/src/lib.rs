pub mod commands;
pub mod error;
pub mod executor;
pub mod orchestrator;
pub mod reporter;
pub mod resource;
pub mod scanner;
pub mod store;
pub mod types;
pub mod webview2_fallback;

#[cfg(not(test))]
use commands::AppState;
#[cfg(not(test))]
use std::sync::Arc;

#[cfg(not(test))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 在 Tauri 初始化前检测 EdgeCore 作为 WebView2 回退
    webview2_fallback::try_setup_webview2_fallback();
    // ------------------------------------------------------------------
    // 1. 初始化 ScannerRegistry，注册所有扫描器
    // ------------------------------------------------------------------
    let scanner_registry = {
        let mut reg = scanner::registry::ScannerRegistry::new();
        reg.register(Box::new(scanner::fs::FileSystemScanner::new()));
        reg.register(Box::new(scanner::env::EnvVarScanner));
        reg.register(Box::new(scanner::browser::BrowserScanner::new()));
        reg.register(Box::new(scanner::chat::ChatScanner));
        reg.register(Box::new(scanner::registry_sys::RegistryScanner));
        reg.register(Box::new(scanner::system::SystemScanner));
        reg.register(Box::new(scanner::devtools::DevToolsScanner));
        reg
    };

    // ------------------------------------------------------------------
    // 2. 初始化 TempStore（临时数据管理）
    // ------------------------------------------------------------------
    let temp_store = match store::temp_store::TempStore::new() {
        Ok(ts) => Arc::new(ts),
        Err(e) => {
            eprintln!("初始化 TempStore 失败: {}", e);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 3. 初始化 ResourceController（资源限制）
    // ------------------------------------------------------------------
    let resource_controller = Arc::new(resource::controller::ResourceController::new());
    // 应用默认 CPU 限制（RULE-05：默认启用 ≤30%）
    if let Err(e) = resource_controller.apply_limits(resource::controller::ResourceController::default_config()) {
        eprintln!("应用默认资源限制失败: {}", e);
    }

    // ------------------------------------------------------------------
    // 4. 初始化各执行器
    // ------------------------------------------------------------------
    let eraser = Arc::new(executor::secure_erase::DoDEraser::default());
    let delete_executor = executor::delete::DeleteExecutor::new(eraser);

    let pack_output_dir = std::env::var("USERPROFILE")
        .map(|p| std::path::PathBuf::from(p).join("Desktop"))
        .unwrap_or_else(|_| std::env::temp_dir());
    let pack_executor = executor::pack::PackExecutor::new(pack_output_dir, None);

    let preserve_executor = executor::preserve::PreserveExecutor::new();

    // ------------------------------------------------------------------
    // 5. 初始化 Orchestrator（流程调度中心）
    // ------------------------------------------------------------------
    let orchestrator = Arc::new(orchestrator::Orchestrator::new(
        scanner_registry,
        Arc::clone(&temp_store),
        Arc::clone(&resource_controller),
        delete_executor,
        pack_executor,
        preserve_executor,
    ));

    // ------------------------------------------------------------------
    // 6. 组装 AppState，注入 Tauri State
    // ------------------------------------------------------------------
    let app_state = AppState {
        orchestrator,
        temp_store: Arc::clone(&temp_store),
        resource_controller: Arc::clone(&resource_controller),
    };

    // ------------------------------------------------------------------
    // 7. 构建并运行 Tauri 应用
    // ------------------------------------------------------------------
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::pause_scan,
            commands::resume_scan,
            commands::get_scan_results,
            commands::submit_decisions,
            commands::start_execution,
            commands::get_resource_config,
            commands::set_resource_config,
            commands::get_session_state,
            commands::get_all_scan_summaries,
            commands::open_path,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
