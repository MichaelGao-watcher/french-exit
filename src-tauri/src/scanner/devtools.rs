use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use tokio::sync::watch;

/// 开发工具扫描器（M10）
///
/// 检测常见开发工具的配置文件和凭证，包括：
/// - Git 全局配置
/// - SSH 密钥（私钥建议打包带走，公钥建议删除）
/// - VS Code 用户设置
/// - GitHub CLI 配置
///
/// 私钥类标记为"有风险"，建议 Pack；配置文件标记为"安全清除"，建议 Delete。
pub struct DevToolsScanner;

impl Scanner for DevToolsScanner {
    fn id(&self) -> &'static str {
        "scanner-devtools"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::DevTools
    }

    fn display_name(&self) -> &'static str {
        "开发工具配置"
    }

    fn scan(
        &self,
        _ctx: &ScanContext,
        _pause_rx: &watch::Receiver<bool>,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut items = Vec::new();
        let total = 4;

        // a) Git 配置
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let gitconfig = PathBuf::from(userprofile).join(".gitconfig");
            if let Some(mut item) = check_file(&gitconfig) {
                item.id = "git-config".to_string();
                item.name = "Git 全局配置".to_string();
                item.risk_note =
                    Some("Git 配置可能包含用户名、邮箱、凭证助手设置".to_string());
                item.suggested_action = Some(Action::Delete);
                items.push(item);
            }
        }

        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: 1,
            total,
            message: "Git 配置检测完成".to_string(),
            global_percent: None,
        });

        // b) SSH 密钥
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let ssh_dir = PathBuf::from(userprofile).join(".ssh");
            if ssh_dir.exists() && ssh_dir.is_dir() {
                items.extend(scan_ssh_dir(&ssh_dir));
            }
        }

        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: 2,
            total,
            message: "SSH 密钥检测完成".to_string(),
            global_percent: None,
        });

        // c) VS Code 配置
        if let Ok(appdata) = std::env::var("APPDATA") {
            let vscode_settings = PathBuf::from(appdata)
                .join("Code")
                .join("User")
                .join("settings.json");
            if let Some(mut item) = check_file(&vscode_settings) {
                item.id = "vscode-settings".to_string();
                item.name = "VS Code 用户设置".to_string();
                item.risk_note =
                    Some("VS Code 设置可能包含同步账号、扩展配置等个人信息".to_string());
                item.suggested_action = Some(Action::Delete);
                items.push(item);
            }
        }

        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: 3,
            total,
            message: "VS Code 配置检测完成".to_string(),
            global_percent: None,
        });

        // d) GitHub CLI 配置
        if let Ok(appdata) = std::env::var("APPDATA") {
            let gh_cli_dir = PathBuf::from(appdata).join("GitHub CLI");
            if gh_cli_dir.exists() && gh_cli_dir.is_dir() {
                let modified_at = std::fs::metadata(&gh_cli_dir)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let dt: DateTime<Local> = t.into();
                        dt
                    });

                items.push(TraceItem {
                    id: "gh-cli-config".to_string(),
                    category: TraceCategory::DevTools,
                    scanner_id: self.id().to_string(),
                    name: "GitHub CLI 配置".to_string(),
                    path: Some(gh_cli_dir),
                    size_bytes: None,
                    modified_at,
                    inferred: false,
                    risk_note: Some("GitHub CLI 配置包含认证凭证和仓库别名".to_string()),
                    suggested_action: Some(Action::Delete),
                    source: "other".to_string(),
                    file_type: "other".to_string(),
                });
            }
        }

        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: 4,
            total,
            message: "GitHub CLI 配置检测完成".to_string(),
            global_percent: None,
        });

        Ok(items)
    }
}

/// 检查文件是否存在并返回基础 TraceItem
///
/// 返回的 TraceItem 包含文件路径、大小、修改时间等基础信息，
/// 调用方需补充 `id`、`name`、`risk_note`、`suggested_action`。
fn check_file(path: &Path) -> Option<TraceItem> {
    let metadata = std::fs::metadata(path).ok()?;
    let size = metadata.len();
    let modified_at = metadata.modified().ok().map(|t| {
        let dt: DateTime<Local> = t.into();
        dt
    });

    let file_name = path.file_name()?.to_string_lossy().to_string();

    Some(TraceItem {
        id: file_name.clone(),
        category: TraceCategory::DevTools,
        scanner_id: "scanner-devtools".to_string(),
        name: file_name,
        path: Some(path.to_path_buf()),
        size_bytes: Some(size),
        modified_at,
        inferred: false,
        risk_note: None,
        suggested_action: None,
        source: "other".to_string(),
        file_type: "other".to_string(),
    })
}

/// 扫描 SSH 目录，识别私钥和公钥文件
///
/// 跳过目录、known_hosts、config 等非密钥文件。
/// 私钥建议 Pack（带走），公钥建议 Delete。
fn scan_ssh_dir(ssh_dir: &Path) -> Vec<TraceItem> {
    let mut items = Vec::new();

    let Ok(entries) = std::fs::read_dir(ssh_dir) else {
        return items;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // 跳过子目录
        if path.is_dir() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(mut item) = check_file(&path) {
            if is_private_key(&file_name) {
                item.id = format!("ssh-{}", file_name);
                item.name = format!("SSH 私钥: {}", file_name);
                item.risk_note = Some(
                    "⚠️ SSH 私钥用于服务器认证，删除后无法登录对应服务器。如为个人密钥，建议带走或删除。"
                        .to_string(),
                );
                item.suggested_action = Some(Action::Pack);
                items.push(item);
            } else if is_public_key(&file_name) {
                item.id = format!("ssh-{}", file_name);
                item.name = format!("SSH 公钥: {}", file_name);
                item.risk_note = Some("SSH 公钥可以安全删除".to_string());
                item.suggested_action = Some(Action::Delete);
                items.push(item);
            }
            // known_hosts、config 等其他文件跳过，不生成条目
        }
    }

    items
}

/// 判断文件名是否为 SSH 私钥
///
/// 支持的私钥类型：id_rsa、id_ed25519、id_ecdsa、id_dsa
fn is_private_key(name: &str) -> bool {
    matches!(name, "id_rsa" | "id_ed25519" | "id_ecdsa" | "id_dsa")
}

/// 判断文件名是否为 SSH 公钥
///
/// 公钥文件名以 `.pub` 结尾。
fn is_public_key(name: &str) -> bool {
    name.ends_with(".pub")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_scanner_trait_compiles() {
        let scanner = DevToolsScanner;
        assert_eq!(scanner.id(), "scanner-devtools");
        assert_eq!(scanner.category(), TraceCategory::DevTools);
        assert_eq!(scanner.display_name(), "开发工具配置");
    }

    #[test]
    fn test_ssh_private_key_recognition() {
        assert!(is_private_key("id_rsa"));
        assert!(is_private_key("id_ed25519"));
        assert!(is_private_key("id_ecdsa"));
        assert!(is_private_key("id_dsa"));
        assert!(!is_private_key("id_rsa.pub"));
        assert!(!is_private_key("known_hosts"));
        assert!(!is_private_key("config"));
    }

    #[test]
    fn test_ssh_public_key_recognition() {
        assert!(is_public_key("id_rsa.pub"));
        assert!(is_public_key("id_ed25519.pub"));
        assert!(!is_public_key("id_rsa"));
        assert!(!is_public_key("config"));
    }

    #[test]
    fn test_check_file_on_missing_path() {
        let path = Path::new("C:\\nonexistent\\path\\file.txt");
        assert!(check_file(path).is_none());
    }

    #[test]
    fn test_scan_ssh_dir_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let items = scan_ssh_dir(temp_dir.path());
        assert!(items.is_empty());
    }

    #[test]
    fn test_scan_ssh_dir_recognizes_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::File::create(temp_dir.path().join("id_rsa")).unwrap();
        std::fs::File::create(temp_dir.path().join("id_rsa.pub")).unwrap();
        std::fs::File::create(temp_dir.path().join("known_hosts")).unwrap();

        let items = scan_ssh_dir(temp_dir.path());
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.name.contains("私钥")));
        assert!(items.iter().any(|i| i.name.contains("公钥")));
    }
}
