use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use tokio::sync::watch;

/// 环境变量扫描器（M11）
///
/// 扫描当前进程环境变量，识别两类敏感信息：
/// 1. TOKEN/Key 类变量 —— 建议删除，但默认不勾选（RULE-02）
/// 2. PATH 中的工具路径 —— 建议保留，仅提醒用户
///
/// 注意：默认不勾选由 Frontend 根据 scanner_id 或 category 统一处理，
/// Scanner 本身不额外标识。
pub struct EnvVarScanner;

/// TOKEN 类环境变量关键字列表（大小写不敏感匹配）
const TOKEN_KEYWORDS: &[&str] = &[
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "AZURE_TOKEN",
    "NPM_TOKEN",
    "DOCKER_TOKEN",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "SLACK_TOKEN",
    "DISCORD_TOKEN",
    "STRIPE_KEY",
    "SENDGRID_KEY",
];

/// 已知开发工具名称列表（用于 PATH 路径识别，大小写不敏感）
const KNOWN_TOOLS: &[&str] = &[
    "github",
    "nodejs",
    "node",
    "python",
    "docker",
    "code",
    "git",
    "cargo",
    "rust",
    "java",
    "maven",
    "gradle",
    "go",
    "kubectl",
    "terraform",
];

/// TOKEN 类变量风险文案
const TOKEN_RISK_NOTE: &str = "⚠️ 该环境变量可能包含账号凭证（Token/Key），删除后相关工具将无法认证。如与他人共用此电脑，建议清除。";

/// PATH 工具路径风险文案
const PATH_RISK_NOTE: &str = "⚠️ PATH 中包含该工具路径，删除后命令行将找不到该工具。如非个人安装，建议保留。";

impl Scanner for EnvVarScanner {
    fn id(&self) -> &'static str {
        "scanner-env"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::EnvVar
    }

    fn display_name(&self) -> &'static str {
        "环境变量"
    }

    fn scan(
        &self,
        _ctx: &ScanContext,
        _pause_rx: &watch::Receiver<bool>,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let vars: Vec<(String, String)> = std::env::vars().collect();
        let total = vars.len();
        let mut items = Vec::new();

        for (idx, (key, value)) in vars.iter().enumerate() {
            // 报告进度
            progress(ScanProgress {
                scanner_id: self.id().to_string(),
                current: idx + 1,
                total,
                message: format!("正在检查环境变量: {}", key),
                global_percent: None,
            });

            let key_lower = key.to_lowercase();

            // a) TOKEN 识别
            if is_token_key(&key_lower) || looks_like_token_value(value) {
                items.push(TraceItem {
                    id: format!("env-{}", sanitize_id(&key_lower)),
                    category: TraceCategory::EnvVar,
                    scanner_id: self.id().to_string(),
                    name: format!("环境变量: {}", key),
                    path: None,
                    size_bytes: Some(value.len() as u64),
                    modified_at: None,
                    inferred: false,
                    risk_note: Some(TOKEN_RISK_NOTE.to_string()),
                    suggested_action: Some(Action::Delete),
                });
                continue;
            }

            // b) PATH 拆分识别
            if key_lower == "path" {
                for segment in value.split(';') {
                    let segment_trimmed = segment.trim();
                    if segment_trimmed.is_empty() {
                        continue;
                    }
                    if is_tool_path(segment_trimmed) {
                        // 用路径段本身生成唯一 id（替换特殊字符）
                        let segment_id = sanitize_id(segment_trimmed);
                        items.push(TraceItem {
                            id: format!("env-path-{}", segment_id),
                            category: TraceCategory::EnvVar,
                            scanner_id: self.id().to_string(),
                            name: format!("PATH 工具路径: {}", segment_trimmed),
                            path: Some(std::path::PathBuf::from(segment_trimmed)),
                            size_bytes: Some(segment_trimmed.len() as u64),
                            modified_at: None,
                            inferred: false,
                            risk_note: Some(PATH_RISK_NOTE.to_string()),
                            suggested_action: Some(Action::Preserve),
                        });
                    }
                }
                continue;
            }

            // c) 其他环境变量：跳过，避免结果过多
        }

        Ok(items)
    }
}

/// 检查环境变量名是否匹配已知的 TOKEN 关键字（大小写不敏感）
fn is_token_key(key: &str) -> bool {
    TOKEN_KEYWORDS.iter().any(|&kw| kw.to_lowercase() == key)
}

/// 检查值内容是否看起来像 TOKEN
///
/// 规则：
/// - 长度 >= 20
/// - 且主要由 Base64 字符集 [A-Za-z0-9+/=] 组成（允许少量其他字符，如前缀）
///   或全部为十六进制字符
fn looks_like_token_value(value: &str) -> bool {
    if value.len() < 20 {
        return false;
    }

    // 计算 Base64 字符和十六进制字符数量
    let mut base64_like_count = 0;
    let mut hex_count = 0;
    let mut total_count = 0;

    for ch in value.chars() {
        total_count += 1;
        if ch.is_ascii_alphanumeric() || ch == '+' || ch == '/' || ch == '=' {
            base64_like_count += 1;
        }
        if ch.is_ascii_hexdigit() {
            hex_count += 1;
        }
    }

    // 如果 Base64-like 字符占比 >= 85%，认为像 TOKEN
    let base64_ratio = base64_like_count as f64 / total_count as f64;
    // 如果全部为十六进制字符
    let is_all_hex = hex_count == total_count;

    base64_ratio >= 0.85 || is_all_hex
}

/// 检查 PATH 路径段是否包含已知工具名称（大小写不敏感）
fn is_tool_path(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    KNOWN_TOOLS.iter().any(|&tool| path_lower.contains(tool))
}

/// 将字符串转为合法的 id（小写，非字母数字替换为下划线，连续下划线合并）
fn sanitize_id(key: &str) -> String {
    let mut result = String::with_capacity(key.len());
    let mut prev_underscore = false;

    for ch in key.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch);
            prev_underscore = false;
        } else {
            // 连续非字母数字只保留一个下划线
            if !prev_underscore {
                result.push('_');
                prev_underscore = true;
            }
        }
    }

    // 去除首尾下划线
    result.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_scanner_trait_compiles() {
        let scanner = EnvVarScanner;
        assert_eq!(scanner.id(), "scanner-env");
        assert_eq!(scanner.category(), TraceCategory::EnvVar);
        assert_eq!(scanner.display_name(), "环境变量");
    }

    #[test]
    fn test_token_key_recognition() {
        assert!(is_token_key("github_token"));
        assert!(is_token_key("openai_api_key"));
        assert!(is_token_key("aws_access_key_id"));
        assert!(!is_token_key("path"));
        assert!(!is_token_key("userprofile"));
        assert!(!is_token_key("home"));
    }

    #[test]
    fn test_looks_like_token_value() {
        // Base64-like
        assert!(looks_like_token_value("abcdef1234567890abcd1234567890abcd1234=="));
        // Hex
        assert!(looks_like_token_value("deadbeef1234567890abcdef12345678"));
        // Too short
        assert!(!looks_like_token_value("short"));
        // Normal text
        assert!(!looks_like_token_value("this is just a normal sentence with words"));
    }

    #[test]
    fn test_is_tool_path() {
        assert!(is_tool_path("C:\\Program Files\\Git\\bin"));
        assert!(is_tool_path("C:\\Users\\Dev\\nodejs"));
        assert!(!is_tool_path("C:\\Windows\\System32"));
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("FOO-BAR"), "foo_bar");
        assert_eq!(sanitize_id("hello.world"), "hello_world");
        assert_eq!(sanitize_id("__leading__"), "leading");
        assert_eq!(sanitize_id("a--b"), "a_b");
    }
}
