/**
 * 格式化工具函数
 *
 * 前端多处共享，统一抽离避免重复定义。
 */

/**
 * 将字节数格式化为人类可读字符串
 * @param bytes 字节数，null/undefined 时返回 "-"
 */
export function formatBytes(bytes: number | null): string {
  if (bytes === null || bytes === undefined) return "-";
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * 将 ISO 日期字符串格式化为中文本地化显示
 * @param dateStr ISO 日期字符串，null 时返回 "-"
 */
export function formatDate(dateStr: string | null): string {
  if (!dateStr) return "-";
  try {
    const d = new Date(dateStr);
    return d.toLocaleString("zh-CN");
  } catch {
    return dateStr;
  }
}
