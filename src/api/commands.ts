/**
 * Tauri IPC 调用封装层
 *
 * 所有与 Rust 后端的通信都通过此文件中的函数进行，
 * 前端不直接调用 invoke，便于统一错误处理和类型约束。
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Decision,
  ExecutionReport,
  PaginatedResult,
  ResourceConfig,
  ScanResultSummary,
  TraceItem,
  ProgressEvent,
} from "../types";

/**
 * 启动扫描流程
 * @param startDate 入职日期，格式必须为 YYYY-MM-DD
 * @returns scanId 扫描会话唯一标识
 */
export async function startScan(startDate: string): Promise<string> {
  return invoke<string>("start_scan", { startDate });
}

/**
 * 暂停当前扫描
 */
export async function pauseScan(): Promise<void> {
  return invoke("pause_scan");
}

/**
 * 恢复当前扫描
 */
export async function resumeScan(): Promise<void> {
  return invoke("resume_scan");
}

/**
 * 分页获取扫描结果
 * @param page 页码，从 1 开始
 * @param pageSize 每页条数
 */
export async function getScanResults(
  page: number,
  pageSize: number,
): Promise<PaginatedResult<TraceItem>> {
  return invoke("get_scan_results", { page, pageSize });
}

/**
 * 提交用户对扫描结果的最终决策
 * @param decisions 决策列表
 */
export async function submitDecisions(decisions: Decision[]): Promise<void> {
  return invoke("submit_decisions", { decisions });
}

/**
 * 开始执行清理操作（删除/打包/保留）
 * @returns 执行报告
 */
export async function startExecution(): Promise<ExecutionReport> {
  return invoke("start_execution");
}

/**
 * 获取当前会话状态（用于页面刷新后恢复）
 * @returns 状态机字符串，如 "Idle" / "Scanning" / "Scanned" 等
 */
export async function getSessionState(): Promise<string> {
  return invoke("get_session_state");
}

/**
 * 获取所有扫描结果轻量摘要（用于全选全部）
 */
export async function getAllScanSummaries(): Promise<ScanResultSummary[]> {
  return invoke("get_all_scan_summaries");
}

/**
 * 获取当前资源限制配置
 */
export async function getResourceConfig(): Promise<ResourceConfig> {
  return invoke("get_resource_config");
}

/**
 * 设置资源限制配置
 * @param config 新的资源配置
 */
export async function setResourceConfig(config: ResourceConfig): Promise<void> {
  return invoke("set_resource_config", { config });
}

/**
 * 打开文件所在文件夹（使用 Windows 资源管理器）
 * @param path 文件完整路径
 */
export async function openPath(path: string): Promise<void> {
  return invoke("open_path", { path });
}

/**
 * 监听扫描进度事件（Tauri Event，替代轮询）
 * @param handler 进度事件处理函数
 * @returns 取消监听的函数
 */
export async function listenScanProgress(
  handler: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<ProgressEvent>("scan_progress", (event: Event<ProgressEvent>) => {
    handler(event.payload);
  });
}
