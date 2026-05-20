/// WebView2 回退检测
///
/// 某些 Windows 环境（尤其是重装系统或企业定制镜像）缺少 WebView2 Runtime 的注册表注册，
/// 但系统已安装 Edge/EdgeCore。本模块在程序启动前自动检测 EdgeCore，并设置环境变量
/// 让 Tauri/wry 使用它作为 WebView2 引擎，从而避免强制用户安装 WebView2 Runtime。
#[cfg(windows)]
pub fn try_setup_webview2_fallback() {
    use std::env;

    // 如果用户或系统已经指定了 WebView2 路径，不覆盖
    if env::var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER").is_ok() {
        return;
    }

    // 常见的 EdgeCore 安装路径
    let edge_core_bases = [
        r"C:\Program Files (x86)\Microsoft\EdgeCore",
        r"C:\Program Files\Microsoft\EdgeCore",
    ];

    for base in &edge_core_bases {
        let base_path = std::path::Path::new(base);
        if !base_path.exists() {
            continue;
        }

        // 遍历子目录（版本号目录，如 118.0.2088.76），找含 msedgewebview2.exe 的
        let mut candidates = Vec::new();
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("msedgewebview2.exe").exists() {
                    candidates.push(path);
                }
            }
        }

        if candidates.is_empty() {
            continue;
        }

        // 按目录名排序，取最后一个（通常是最新版）
        candidates.sort();
        if let Some(latest) = candidates.last() {
            let _ = env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", latest);
            log::info!("WebView2 fallback: 使用 EdgeCore {}", latest.display());
            return;
        }
    }

    log::warn!("WebView2 fallback: 未找到 EdgeCore，程序可能无法启动");
}

#[cfg(not(windows))]
pub fn try_setup_webview2_fallback() {
    // 非 Windows 平台无需处理
}
