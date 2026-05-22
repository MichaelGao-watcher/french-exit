use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};

/// 聊天软件扫描器
///
/// 负责检测 QQ、钉钉、飞书、企业微信的本地数据目录。
/// 微信已在 M05（FileSystemScanner）中处理，此处不再重复。
/// 所有聊天软件目录均整目录标记为单条 TraceItem，建议删除或打包（RULE-03 精神）。
pub struct ChatScanner;

impl ChatScanner {
    pub fn new() -> Self {
        ChatScanner
    }

    /// 递归计算目录总大小（字节）
    fn dir_size(path: &Path) -> u64 {
        walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    }

    /// 获取目录修改时间
    fn dir_modified(path: &Path) -> Option<DateTime<Local>> {
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        Some(modified.into())
    }

    /// 检测 QQ 数据目录
    ///
    /// 路径：`%USERPROFILE%\Documents\Tencent Files\`
    /// 子目录通常为 QQ 号命名。
    fn detect_qq_paths(user_home: &Path) -> Vec<PathBuf> {
        let base = user_home.join("Documents").join("Tencent Files");
        if !base.exists() || !base.is_dir() {
            return Vec::new();
        }
        Self::collect_subdirs(&base)
    }

    /// 检测钉钉数据目录
    ///
    /// 路径：`%USERPROFILE%\AppData\Roaming\DingTalk\`
    fn detect_dingtalk_path(user_home: &Path) -> Option<PathBuf> {
        let path = user_home.join("AppData").join("Roaming").join("DingTalk");
        if path.exists() && path.is_dir() {
            Some(path)
        } else {
            None
        }
    }

    /// 检测飞书数据目录
    ///
    /// 路径：`%USERPROFILE%\AppData\Roaming\Lark\` 或 `%USERPROFILE%\AppData\Roaming\Feishu\`
    fn detect_lark_path(user_home: &Path) -> Option<PathBuf> {
        let lark = user_home.join("AppData").join("Roaming").join("Lark");
        if lark.exists() && lark.is_dir() {
            return Some(lark);
        }
        let feishu = user_home.join("AppData").join("Roaming").join("Feishu");
        if feishu.exists() && feishu.is_dir() {
            return Some(feishu);
        }
        None
    }

    /// 检测企业微信数据目录
    ///
    /// 路径：`%USERPROFILE%\Documents\WXWork\`
    /// 子目录通常为企业 ID 命名。
    fn detect_wxwork_paths(user_home: &Path) -> Vec<PathBuf> {
        let base = user_home.join("Documents").join("WXWork");
        if !base.exists() || !base.is_dir() {
            return Vec::new();
        }
        Self::collect_subdirs(&base)
    }

    /// 收集指定目录下的直接子目录
    fn collect_subdirs(path: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Scanner for ChatScanner {
    fn id(&self) -> &'static str {
        "scanner-chat"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::Chat
    }

    fn display_name(&self) -> &'static str {
        "聊天记录"
    }

    fn scan(
        &self,
        ctx: &ScanContext,
        _pause_rx: &tokio::sync::watch::Receiver<bool>,
        progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<TraceItem>, ScanError> {
        let mut items = Vec::new();
        let total_steps = 4;
        let mut current_step = 0;

        // 1. 扫描 QQ
        let qq_paths = Self::detect_qq_paths(&ctx.user_home);
        if !qq_paths.is_empty() {
            for path in &qq_paths {
                let qq_number = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let size = Self::dir_size(path);
                let modified = Self::dir_modified(path);

                items.push(TraceItem {
                    id: format!("qq-{}", qq_number),
                    category: TraceCategory::Chat,
                    scanner_id: self.id().to_string(),
                    name: format!("QQ 聊天记录: {}", qq_number),
                    path: Some(path.clone()),
                    size_bytes: Some(size),
                    modified_at: modified,
                    inferred: false,
                    risk_note: Some("QQ 聊天记录属于私人内容，建议处理".to_string()),
                    suggested_action: Some(Action::DeleteOrPack),
                });
            }
        }
        current_step += 1;
        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: current_step,
            total: total_steps,
            message: "QQ 检测完成".to_string(),
        });

        // 2. 扫描钉钉
        if let Some(path) = Self::detect_dingtalk_path(&ctx.user_home) {
            let size = Self::dir_size(&path);
            let modified = Self::dir_modified(&path);

            items.push(TraceItem {
                id: "dingtalk-data".to_string(),
                category: TraceCategory::Chat,
                scanner_id: self.id().to_string(),
                name: "钉钉本地数据".to_string(),
                path: Some(path),
                size_bytes: Some(size),
                modified_at: modified,
                inferred: false,
                risk_note: Some("钉钉本地数据包含聊天记录和缓存，建议处理".to_string()),
                suggested_action: Some(Action::DeleteOrPack),
            });
        }
        current_step += 1;
        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: current_step,
            total: total_steps,
            message: "钉钉检测完成".to_string(),
        });

        // 3. 扫描飞书
        if let Some(path) = Self::detect_lark_path(&ctx.user_home) {
            let size = Self::dir_size(&path);
            let modified = Self::dir_modified(&path);

            items.push(TraceItem {
                id: "lark-data".to_string(),
                category: TraceCategory::Chat,
                scanner_id: self.id().to_string(),
                name: "飞书本地数据".to_string(),
                path: Some(path),
                size_bytes: Some(size),
                modified_at: modified,
                inferred: false,
                risk_note: Some("飞书本地数据包含聊天记录和缓存，建议处理".to_string()),
                suggested_action: Some(Action::DeleteOrPack),
            });
        }
        current_step += 1;
        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: current_step,
            total: total_steps,
            message: "飞书检测完成".to_string(),
        });

        // 4. 扫描企业微信
        let wxwork_paths = Self::detect_wxwork_paths(&ctx.user_home);
        if !wxwork_paths.is_empty() {
            for path in &wxwork_paths {
                let corp_id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let size = Self::dir_size(path);
                let modified = Self::dir_modified(path);

                items.push(TraceItem {
                    id: format!("wxwork-{}", corp_id),
                    category: TraceCategory::Chat,
                    scanner_id: self.id().to_string(),
                    name: format!("企业微信本地数据: {}", corp_id),
                    path: Some(path.clone()),
                    size_bytes: Some(size),
                    modified_at: modified,
                    inferred: false,
                    risk_note: Some("企业微信本地数据包含工作聊天记录，建议处理".to_string()),
                    suggested_action: Some(Action::DeleteOrPack),
                });
            }
        }
        current_step += 1;
        progress(ScanProgress {
            scanner_id: self.id().to_string(),
            current: current_step,
            total: total_steps,
            message: "企业微信检测完成".to_string(),
        });

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_chat_scanner_trait_compiles() {
        let scanner = ChatScanner;
        assert_eq!(scanner.id(), "scanner-chat");
        assert_eq!(scanner.category(), TraceCategory::Chat);
        assert_eq!(scanner.display_name(), "聊天记录");
    }

    #[test]
    fn test_chat_detects_qq_path_does_not_panic() {
        let user_home = std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir());

        // 验证函数不 panic，返回值为 Vec（可能为空）
        let qq_paths = ChatScanner::detect_qq_paths(&user_home);
        assert!(qq_paths.is_empty() || qq_paths.iter().all(|p| p.is_absolute()));
    }

    #[test]
    fn test_chat_dir_size_on_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let size = ChatScanner::dir_size(temp_dir.path());
        assert_eq!(size, 0);
    }

    #[test]
    fn test_chat_collect_subdirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp_dir.path().join("sub1")).unwrap();
        std::fs::create_dir(temp_dir.path().join("sub2")).unwrap();
        std::fs::File::create(temp_dir.path().join("file.txt")).unwrap();

        let subdirs = ChatScanner::collect_subdirs(temp_dir.path());
        assert_eq!(subdirs.len(), 2);
    }

    // ── scan() 集成测试 ─────────────────────────────────────────

    fn make_ctx(user_home: &std::path::Path) -> ScanContext {
        ScanContext {
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            user_home: user_home.to_path_buf(),
            temp_dir: user_home.to_path_buf(),
        }
    }

    #[test]
    fn test_chat_scan_finds_all_apps() {
        let temp = tempfile::tempdir().unwrap();
        let user_home = temp.path();

        // QQ
        let qq_dir = user_home.join("Documents").join("Tencent Files").join("123456");
        std::fs::create_dir_all(&qq_dir).unwrap();
        std::fs::write(qq_dir.join("msg.db"), "qq data").unwrap();

        // DingTalk
        let dingtalk_dir = user_home.join("AppData").join("Roaming").join("DingTalk");
        std::fs::create_dir_all(&dingtalk_dir).unwrap();

        // Lark
        let lark_dir = user_home.join("AppData").join("Roaming").join("Lark");
        std::fs::create_dir_all(&lark_dir).unwrap();

        // WXWork
        let wxwork_dir = user_home.join("Documents").join("WXWork").join("corp123");
        std::fs::create_dir_all(&wxwork_dir).unwrap();

        let ctx = make_ctx(user_home);
        let scanner = ChatScanner::new();
        let (_tx, rx) = tokio::sync::watch::channel(false);

        let progress_count = Arc::new(AtomicUsize::new(0));
        let pc = progress_count.clone();
        let progress = move |_p: ScanProgress| {
            pc.fetch_add(1, Ordering::SeqCst);
        };
        let items = scanner.scan(&ctx, &rx, &progress).unwrap();

        assert_eq!(items.len(), 4, "应发现 QQ、钉钉、飞书、企业微信");
        let ids: std::collections::HashSet<_> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains("qq-123456"));
        assert!(ids.contains("dingtalk-data"));
        assert!(ids.contains("lark-data"));
        assert!(ids.contains("wxwork-corp123"));
        assert_eq!(progress_count.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_chat_scan_empty_home() {
        let temp = tempfile::tempdir().unwrap();
        let ctx = make_ctx(temp.path());
        let scanner = ChatScanner::new();
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let items = scanner.scan(&ctx, &rx, &|_p| {}).unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_chat_scan_qq_multiple_accounts() {
        let temp = tempfile::tempdir().unwrap();
        let qq_base = temp.path().join("Documents").join("Tencent Files");
        std::fs::create_dir_all(qq_base.join("111111")).unwrap();
        std::fs::create_dir_all(qq_base.join("222222")).unwrap();

        let ctx = make_ctx(temp.path());
        let scanner = ChatScanner::new();
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let items = scanner.scan(&ctx, &rx, &|_p| {}).unwrap();

        assert_eq!(items.len(), 2);
        let ids: Vec<_> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"qq-111111"));
        assert!(ids.contains(&"qq-222222"));
    }

    #[test]
    fn test_chat_scan_feishu_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let feishu_dir = temp.path().join("AppData").join("Roaming").join("Feishu");
        std::fs::create_dir_all(&feishu_dir).unwrap();

        let ctx = make_ctx(temp.path());
        let scanner = ChatScanner::new();
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let items = scanner.scan(&ctx, &rx, &|_p| {}).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "lark-data");
        assert!(items[0].name.contains("飞书"));
    }

    #[test]
    fn test_chat_scan_lark_preferred_over_feishu() {
        let temp = tempfile::tempdir().unwrap();
        let lark_dir = temp.path().join("AppData").join("Roaming").join("Lark");
        let feishu_dir = temp.path().join("AppData").join("Roaming").join("Feishu");
        std::fs::create_dir_all(&lark_dir).unwrap();
        std::fs::create_dir_all(&feishu_dir).unwrap();

        let ctx = make_ctx(temp.path());
        let scanner = ChatScanner::new();
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let items = scanner.scan(&ctx, &rx, &|_p| {}).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, Some(lark_dir));
    }
}
