//! M16 — reporter（报告生成器）
//!
//! 职责：汇总操作结果，生成文本摘要 + HTML 庆祝页。
//! HTML 保存位置由调用方决定（有打包放 zip 同目录，无打包放桌面 — RULE-09）。
//! 自动调用系统浏览器打开 HTML 报告。

use crate::error::BackendError;
use crate::types::{Action, ExecutionReport, ExecutionResult, ExecutionStatus};
use std::fs;
use std::path::{Path, PathBuf};

/// 报告生成器，无状态，纯函数集合
pub struct Reporter;

impl Reporter {
    /// 生成文本摘要
    ///
    /// 格式示例："已删除 1,247 条痕迹（共 3.2 GB），打包 56 个文件，保留 12 条"
    /// 如果有 pack_file_path，追加 "打包文件: {path}"
    pub fn generate_summary(report: &ExecutionReport) -> String {
        let mut summary = format!(
            "已删除 {} 条痕迹（共 {}），打包 {} 个文件，保留 {} 条",
            format_number(report.deleted_count),
            format_bytes(report.deleted_bytes),
            format_number(report.packed_count),
            format_number(report.preserved_count),
        );

        if let Some(path) = &report.pack_file_path {
            summary.push_str(&format!(
                "，打包文件: {}",
                path.display()
            ));
        }

        summary
    }

    /// 生成完整的 HTML 庆祝页字符串（单文件，内嵌 CSS + 内嵌 JS）
    ///
    /// 设计要点：
    /// - Apple Design 风格（圆角、毛玻璃、系统颜色模式跟随）
    /// - 主文案固定："你已完成 French Exit，现在去享受生活吧"
    /// - 操作明细按 status 分组，每组最多 50 条，超出用 `<details>` 折叠
    pub fn generate_html(report: &ExecutionReport) -> String {
        let summary_text = html_escape(&Self::generate_summary(report));

        // 按执行状态分组
        let success_items: Vec<&ExecutionResult> = report
            .items
            .iter()
            .filter(|i| matches!(i.status, ExecutionStatus::Success))
            .collect();
        let failed_items: Vec<&ExecutionResult> = report
            .items
            .iter()
            .filter(|i| matches!(i.status, ExecutionStatus::Failed(_)))
            .collect();
        let skipped_items: Vec<&ExecutionResult> = report
            .items
            .iter()
            .filter(|i| matches!(i.status, ExecutionStatus::Skipped(_)))
            .collect();

        let detail_html = build_detail_html(&success_items, "成功")
            + &build_detail_html(&failed_items, "失败")
            + &build_detail_html(&skipped_items, "跳过");

        format!(
            r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>French Exit — 清理完成</title>
  <style>
    :root {{
      --bg: #f5f5f7;
      --card-bg: rgba(255,255,255,0.8);
      --text: #1d1d1f;
      --text-secondary: #86868b;
      --accent: #0071e3;
      --success: #34c759;
      --warning: #ff9500;
      --error: #ff3b30;
    }}
    @media (prefers-color-scheme: dark) {{
      :root {{
        --bg: #000000;
        --card-bg: rgba(28,28,30,0.8);
        --text: #f5f5f7;
        --text-secondary: #86868b;
        --accent: #0a84ff;
        --success: #30d158;
        --warning: #ff9f0a;
        --error: #ff453a;
      }}
    }}
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 60px 20px;
    }}
    .container {{ max-width: 720px; width: 100%; }}
    h1 {{
      font-size: 48px;
      font-weight: 700;
      text-align: center;
      margin-bottom: 12px;
      letter-spacing: -0.5px;
    }}
    .subtitle {{
      text-align: center;
      font-size: 20px;
      color: var(--text-secondary);
      margin-bottom: 48px;
    }}
    .stats {{
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 16px;
      margin-bottom: 48px;
    }}
    .stat-card {{
      background: var(--card-bg);
      backdrop-filter: blur(20px);
      border-radius: 20px;
      padding: 24px;
      text-align: center;
      box-shadow: 0 4px 24px rgba(0,0,0,0.08);
    }}
    .stat-number {{ font-size: 36px; font-weight: 700; margin-bottom: 4px; }}
    .stat-label {{ font-size: 14px; color: var(--text-secondary); }}
    .stat-deleted {{ color: var(--success); }}
    .stat-packed {{ color: var(--accent); }}
    .stat-preserved {{ color: var(--warning); }}
    .summary {{
      background: var(--card-bg);
      backdrop-filter: blur(20px);
      border-radius: 20px;
      padding: 24px;
      margin-bottom: 32px;
      font-size: 16px;
      line-height: 1.6;
    }}
    .detail-section {{ margin-bottom: 24px; }}
    .detail-section h3 {{
      font-size: 18px;
      margin-bottom: 12px;
      padding-bottom: 8px;
      border-bottom: 1px solid rgba(128,128,128,0.2);
    }}
    .detail-section h4 {{
      font-size: 15px;
      margin: 16px 0 8px;
      color: var(--text-secondary);
    }}
    .detail-list {{ list-style: none; }}
    .detail-list li {{
      padding: 8px 0;
      font-size: 14px;
      color: var(--text-secondary);
      border-bottom: 1px solid rgba(128,128,128,0.1);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .detail-list li:last-child {{ border-bottom: none; }}
    .detail-list li .item-id {{ color: var(--text); font-weight: 500; }}
    .detail-list li .item-action {{
      font-size: 12px;
      padding: 2px 8px;
      border-radius: 4px;
      background: rgba(128,128,128,0.15);
      margin-left: 8px;
    }}
    details {{
      margin-top: 8px;
      padding: 8px 12px;
      background: var(--card-bg);
      border-radius: 12px;
    }}
    summary {{
      cursor: pointer;
      color: var(--accent);
      font-size: 14px;
      user-select: none;
    }}
    .footer {{
      text-align: center;
      color: var(--text-secondary);
      font-size: 13px;
      margin-top: 48px;
    }}
  </style>
</head>
<body>
  <div class="container">
    <h1>🎉 清理完成</h1>
    <p class="subtitle">你已完成 French Exit，现在去享受生活吧</p>

    <div class="stats">
      <div class="stat-card">
        <div class="stat-number stat-deleted">{}</div>
        <div class="stat-label">已删除</div>
      </div>
      <div class="stat-card">
        <div class="stat-number stat-packed">{}</div>
        <div class="stat-label">已打包</div>
      </div>
      <div class="stat-card">
        <div class="stat-number stat-preserved">{}</div>
        <div class="stat-label">已保留</div>
      </div>
    </div>

    <div class="summary">
      <p>{}</p>
    </div>

    <div class="detail-section">
      <h3>操作明细</h3>
      {}
    </div>

    <div class="footer">
      <p>French Exit — 离职清理工具</p>
      <p>所有操作已记录，HTML 报告为本程序唯一保留文件</p>
    </div>
  </div>
</body>
</html>"##,
            format_number(report.deleted_count),
            format_number(report.packed_count),
            format_number(report.preserved_count),
            summary_text,
            detail_html
        )
    }

    /// 将 HTML 报告写入指定目录
    ///
    /// 文件名固定为 `French-exit-report.html`
    pub fn write_report(
        report: &ExecutionReport,
        output_dir: &Path,
    ) -> Result<PathBuf, BackendError> {
        let html = Self::generate_html(report);
        let file_path = output_dir.join("French-exit-report.html");
        fs::write(&file_path, html).map_err(|e| {
            BackendError::ExecutionError(format!("写入 HTML 报告失败: {}", e))
        })?;
        Ok(file_path)
    }

    /// 调用系统浏览器打开 HTML 文件（仅限 Windows）
    ///
    /// 使用 `cmd /c start "" "path"` 确保含空格路径也能正确打开
    pub fn open_in_browser(path: &Path) -> Result<(), BackendError> {
        let path_str = path.to_string_lossy();
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path_str])
            .spawn()
            .map(|_| ())
            .map_err(|e| {
                BackendError::ExecutionError(format!("打开浏览器失败: {}", e))
            })?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 辅助函数
// -----------------------------------------------------------------------------

/// 将数字格式化为千分位分隔字符串
///
/// 示例：1247 → "1,247"
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// 将字节数转换为人类可读字符串
///
/// 规则：
/// - < 1KB → "X 字节"
/// - < 1MB → "X.X KB"
/// - < 1GB → "X.X MB"
/// - >= 1GB → "X.X GB"
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes < KB {
        format!("{} 字节", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    }
}

/// 对字符串进行简单的 HTML 转义，防止特殊字符破坏页面结构
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 构建某一状态分组的操作明细 HTML
///
/// 每组最多显示 50 条，超出部分用 `<details>` 标签折叠
fn build_detail_html(items: &[&ExecutionResult], title: &str) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut html = format!(
        r#"<h4>{} ({})</h4>
<ul class="detail-list">
"#,
        title,
        items.len()
    );

    for item in items.iter().take(50) {
        html.push_str(&format_detail_item(item));
    }
    html.push_str("</ul>\n");

    if items.len() > 50 {
        html.push_str(&format!(
            r#"<details>
  <summary>还有 {} 条...</summary>
  <ul class="detail-list">
"#,
            items.len() - 50
        ));
        for item in items.iter().skip(50) {
            html.push_str(&format_detail_item(item));
        }
        html.push_str("  </ul>\n</details>\n");
    }

    html
}

/// 格式化单条执行结果为 HTML 列表项
///
/// 显示内容：item_id + action（左侧），status + detail（右侧）
fn format_detail_item(item: &ExecutionResult) -> String {
    let action_text = match item.action {
        Action::Delete => "删除",
        Action::Preserve => "保留",
        Action::Pack => "打包",
        Action::DeleteOrPack => "删除或打包",
    };

    let status_text = match &item.status {
        ExecutionStatus::Success => "成功".to_string(),
        ExecutionStatus::Failed(msg) => format!("失败: {}", html_escape(msg)),
        ExecutionStatus::Skipped(msg) => format!("跳过: {}", html_escape(msg)),
    };

    let detail_html = item
        .detail
        .as_ref()
        .map(|d| format!(" ({})", html_escape(d)))
        .unwrap_or_default();

    format!(
        r#"<li>
  <span><span class="item-id">{}</span><span class="item-action">{}</span></span>
  <span>{}{}</span>
</li>
"#,
        html_escape(&item.item_id),
        action_text,
        status_text,
        detail_html
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 字节");
        assert_eq!(format_bytes(512), "512 字节");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_generate_summary_basic() {
        let report = ExecutionReport {
            deleted_count: 100,
            deleted_bytes: 1024 * 1024,
            packed_count: 5,
            packed_bytes: 1024,
            preserved_count: 2,
            pack_file_path: None,
            items: vec![],
        };
        let summary = Reporter::generate_summary(&report);
        assert!(summary.contains("100"));
        assert!(summary.contains("1.0 MB"));
        assert!(summary.contains("5"));
        assert!(summary.contains("2"));
    }

    #[test]
    fn test_generate_html_contains_stats() {
        let report = ExecutionReport {
            deleted_count: 10,
            deleted_bytes: 0,
            packed_count: 2,
            packed_bytes: 0,
            preserved_count: 1,
            pack_file_path: None,
            items: vec![],
        };
        let html = Reporter::generate_html(&report);
        assert!(html.contains("清理完成"));
        assert!(html.contains("French Exit"));
        assert!(html.contains("10"));
        assert!(html.contains("2"));
        assert!(html.contains("1"));
    }
}
