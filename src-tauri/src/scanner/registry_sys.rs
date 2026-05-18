//! M08 — scanner-registry-sys（注册表扫描器）
//!
//! 扫描 HKEY_CURRENT_USER 下可能包含个人信息的键值。
//! 所有推断结果均标注 `inferred: true` 和 `risk_note`（RULE-10）。

use crate::scanner::{ScanError, ScanProgress, Scanner};
use crate::types::{Action, ScanContext, TraceCategory, TraceItem};
use tokio::sync::watch;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
};
use windows::core::{PCWSTR, PWSTR};

/// 注册表扫描器
pub struct RegistryScanner;

/// RULE-10 风险文案
const RISK_NOTE: &str = "⚠️ 该注册表项为程序启发式推断，可能误报。请仔细确认后再操作。";

impl Scanner for RegistryScanner {
    fn id(&self) -> &'static str {
        "scanner-registry-sys"
    }

    fn category(&self) -> TraceCategory {
        TraceCategory::Registry
    }

    fn display_name(&self) -> &'static str {
        "注册表痕迹"
    }

    fn scan(
        &self,
        _ctx: &ScanContext,
        _pause_rx: &watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, ScanError> {
        // 待扫描的注册表路径列表
        let subkeys = [
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs",
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths",
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery",
            r"Software\Microsoft\Windows\CurrentVersion\Applets\Regedit",
        ];

        let mut items = Vec::new();
        let total = subkeys.len();

        for (idx, subpath) in subkeys.iter().enumerate() {
            progress(ScanProgress {
                scanner_id: self.id().to_string(),
                current: idx + 1,
                total,
                message: format!("正在扫描注册表: {}", subpath),
            });

            // 单个键读取失败时跳过，不中断整体扫描
            if let Err(e) = scan_key(HKEY_CURRENT_USER, subpath, &mut items) {
                tracing::warn!("[scanner-registry-sys] 扫描 {} 失败: {}", subpath, e);
            }
        }

        Ok(items)
    }
}

/// 打开指定注册表键，枚举其所有值，对匹配启发式规则的值生成 TraceItem。
///
/// 错误处理：
/// - 键不存在 → 返回 Ok(())，静默跳过
/// - 权限不足 → 返回 Ok(())，静默跳过
/// - 枚举过程中出错 → 记录日志，返回已收集的结果
fn scan_key(hkey: HKEY, subpath: &str, items: &mut Vec<TraceItem>) -> Result<(), ScanError> {
    // 将子路径转为以 \0 结尾的 UTF-16 宽字符（Windows API 要求）
    let subkey_wide: Vec<u16> = subpath.encode_utf16().chain(std::iter::once(0)).collect();

    // 打开子键
    let mut hsubkey = HKEY::default();
    let open_result = unsafe {
        RegOpenKeyExW(
            hkey,
            PCWSTR(subkey_wide.as_ptr()),
            0,
            KEY_READ,
            &mut hsubkey,
        )
    };

    if open_result.is_err() {
        // 键不存在或无权限，优雅降级，不报错
        return Ok(());
    }

    // RAII 守卫：确保键句柄在函数退出时被关闭
    let _guard = RegistryKeyGuard(hsubkey);

    let mut index = 0u32;

    // 枚举该键下的所有值
    loop {
        // 为值名分配足够缓冲区（512 个宽字符应能覆盖绝大多数注册表值名）
        let mut name_buf = vec![0u16; 512];
        let mut name_len = name_buf.len() as u32;
        let mut data_type = 0u32;
        // 为值数据分配 64KB 缓冲区，足以覆盖绝大多数字符串值
        let mut data_buf = vec![0u8; 65536];
        let mut data_len = data_buf.len() as u32;

        let enum_result = unsafe {
            RegEnumValueW(
                hsubkey,
                index,
                PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
                std::ptr::null(),
                &mut data_type,
                data_buf.as_mut_ptr(),
                &mut data_len,
            )
        };

        // WIN32_ERROR 底层为 u32，0 表示 ERROR_SUCCESS
        match enum_result.0 {
            0 => {
                // ERROR_SUCCESS：成功读取一个值
                // name_len 为不包含终止符的字符数
                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);

                // 只处理字符串类型（REG_SZ = 1），其他类型（如二进制、DWORD）跳过
                if data_type == REG_SZ.0 {
                    // REG_SZ 数据在内存中以 UTF-16LE 编码存储
                    let wide_len = (data_len / 2) as usize;
                    let wide_data = unsafe {
                        std::slice::from_raw_parts(data_buf.as_ptr() as *const u16, wide_len)
                    };
                    let mut value = String::from_utf16_lossy(wide_data);
                    // 去除末尾可能的空字符（Windows 返回的数据通常包含终止符）
                    value = value.trim_end_matches('\0').to_string();

                    // 应用启发式推断规则
                    if looks_like_personal_info(&name, &value) {
                        let safe_name = sanitize_key_name(&name);
                        items.push(TraceItem {
                            id: format!("reg-{}", safe_name),
                            category: TraceCategory::Registry,
                            scanner_id: "scanner-registry-sys".to_string(),
                            name: format!("注册表: {}", name),
                            path: None, // 注册表项不是文件路径，设为 None
                            size_bytes: Some(value.len() as u64),
                            modified_at: None, // 注册表修改时间获取复杂，暂不设
                            inferred: true,    // RULE-10 硬规则
                            risk_note: Some(RISK_NOTE.to_string()), // RULE-10 硬规则
                            suggested_action: Some(Action::Delete),
                        });
                    }
                }

                index += 1;
            }
            259 => {
                // ERROR_NO_MORE_ITEMS = 0x103 = 259
                // 已枚举完所有值，退出循环
                break;
            }
            234 => {
                // ERROR_MORE_DATA = 0xEA = 234
                // 缓冲区不足以容纳该值，跳过，继续枚举下一个
                index += 1;
            }
            _ => {
                // 其他错误（如权限中途被收回），记录日志并停止枚举当前键
                tracing::warn!(
                    "[scanner-registry-sys] RegEnumValueW 错误，代码: {}",
                    enum_result.0
                );
                break;
            }
        }
    }

    Ok(())
}

/// 启发式推断：判断注册表键名和值内容是否疑似个人信息
///
/// 匹配规则：
/// 1. 键名包含敏感关键词（name / email / phone / address / company / title 等）
/// 2. 值内容看起来像邮箱（含 @ 和 .）
/// 3. 值内容看起来像手机号（11 位数字）
/// 4. 值内容看起来像身份证号（18 位数字或末尾 X）
fn looks_like_personal_info(name: &str, value: &str) -> bool {
    let name_lower = name.to_lowercase();

    // 1. 键名包含敏感关键词
    let sensitive_keywords = [
        "name", "email", "phone", "address", "company", "title",
        "username", "password", "token", "key", "account",
    ];
    if sensitive_keywords.iter().any(|kw| name_lower.contains(kw)) {
        return true;
    }

    // 2. 值内容看起来像邮箱
    if value.contains('@') && value.contains('.') {
        return true;
    }

    // 3. 值内容看起来像手机号：提取所有数字，恰好 11 位且总长不过大
    let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits_only.len() == 11 && value.len() <= 20 {
        return true;
    }

    // 4. 值内容看起来像身份证号
    if is_likely_id_card(value) {
        return true;
    }

    false
}

/// 检查字符串是否像中国大陆身份证号（18 位）
///
/// 规则：18 个字符，前 17 位为数字，最后一位为数字或 X/x。
fn is_likely_id_card(value: &str) -> bool {
    if value.len() != 18 {
        return false;
    }

    let mut digit_count = 0;
    let mut has_x = false;

    for (i, ch) in value.chars().enumerate() {
        if ch.is_ascii_digit() {
            digit_count += 1;
        } else if i == 17 && (ch == 'x' || ch == 'X') {
            has_x = true;
        } else {
            return false;
        }
    }

    digit_count == 18 || (digit_count == 17 && has_x)
}

/// 将注册表键名转为合法的 ID 字符串
///
/// 规则：
/// - 全部转为小写
/// - 字母、数字、连字符保留
/// - 其余字符替换为下划线
/// - 连续非合法字符只保留一个下划线
/// - 去除首尾下划线
fn sanitize_key_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut prev_underscore = false;

    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            result.push(ch);
            prev_underscore = false;
        } else {
            if !prev_underscore {
                result.push('_');
                prev_underscore = true;
            }
        }
    }

    result.trim_matches('_').to_string()
}

/// RAII 守卫：确保注册表键句柄在离开作用域时被关闭
struct RegistryKeyGuard(HKEY);

impl Drop for RegistryKeyGuard {
    fn drop(&mut self) {
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_scanner_trait_compiles() {
        let scanner = RegistryScanner;
        assert_eq!(scanner.id(), "scanner-registry-sys");
        assert_eq!(scanner.category(), TraceCategory::Registry);
        assert_eq!(scanner.display_name(), "注册表痕迹");
    }

    #[test]
    fn test_registry_inferred_flag() {
        // 验证启发式推断对可疑内容返回 true
        assert!(looks_like_personal_info("user_email", "test@example.com"));
        assert!(looks_like_personal_info("phone_number", "13800138000"));
        assert!(looks_like_personal_info("id_card", "11010119900101x"));
        assert!(looks_like_personal_info("company_name", "Acme Corp"));

        // 验证普通内容返回 false
        assert!(!looks_like_personal_info("window_width", "1024"));
        assert!(!looks_like_personal_info("theme", "dark"));
        assert!(!looks_like_personal_info("version", "1.2.3"));
    }

    #[test]
    fn test_is_likely_id_card() {
        assert!(is_likely_id_card("110101199001011234"));
        assert!(is_likely_id_card("11010119900101123X"));
        assert!(is_likely_id_card("11010119900101123x"));
        assert!(!is_likely_id_card("11010119900101123"));  // 17 位
        assert!(!is_likely_id_card("1101011990010112345")); // 19 位
        assert!(!is_likely_id_card("not-an-id-at-all!!"));
    }

    #[test]
    fn test_sanitize_key_name() {
        assert_eq!(sanitize_key_name("Hello World"), "hello_world");
        assert_eq!(sanitize_key_name("Foo-Bar_Baz!"), "foo-bar_baz");
        assert_eq!(sanitize_key_name("__leading__"), "leading");
        assert_eq!(sanitize_key_name(" trailing "), "trailing");
    }
}
