import { test as base, expect, type Page } from "@playwright/test";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import type { TraceItem, ExecutionReport, ResourceConfig } from "../src/types";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/**
 * 触发 Tauri Event（扫描进度等）
 */
async function emitTauriEvent(page: Page, event: string, payload: unknown) {
  await page.evaluate(
    ({ event, payload }) => {
      window.__emitTauriEvent__(event, payload);
    },
    { event, payload }
  );
}

/**
 * 常用 mock 数据工厂
 */
export function createMockTraceItems(): TraceItem[] {
  return [
    {
      id: "item-chat-1",
      name: "微信聊天记录",
      path: "C:\\Users\\User\\Documents\\WeChat Files\\wxid_xxx",
      category: "Chat",
      size_bytes: 1024 * 1024 * 100,
      modified_at: "2026-05-01T10:00:00Z",
      suggested_action: "DeleteOrPack",
      inferred: false,
      risk_note: null,
    },
    {
      id: "item-browser-1",
      name: "Chrome 历史记录",
      path: null,
      category: "Browser",
      size_bytes: null,
      modified_at: null,
      suggested_action: "Delete",
      inferred: false,
      risk_note: null,
    },
    {
      id: "item-env-1",
      name: "API_TOKEN",
      path: null,
      category: "EnvVar",
      size_bytes: null,
      modified_at: null,
      suggested_action: "Delete",
      inferred: false,
      risk_note: "可能包含共享 TOKEN",
    },
    {
      id: "item-fs-1",
      name: "工作文件.txt",
      path: "C:\\Users\\User\\Desktop\\工作文件.txt",
      category: "FileSystem",
      size_bytes: 2048,
      modified_at: "2026-05-10T08:00:00Z",
      suggested_action: "Pack",
      inferred: false,
      risk_note: null,
    },
  ];
}

export function createMockReport(): ExecutionReport {
  return {
    deleted_count: 2,
    packed_count: 1,
    preserved_count: 1,
    deleted_bytes: 1024 * 1024 * 100 + 2048,
    packed_bytes: 0,
    preserved_bytes: 0,
    pack_file_path: "C:\\Users\\User\\Desktop\\French-exit.zip",
    report_html_path: "C:\\Users\\User\\Desktop\\French-exit-report.html",
  };
}

/**
 * 在 page 中设置标准 mock handler。
 * 直接在浏览器上下文中定义 handler，避免 toString() 丢失闭包的问题。
 */
export async function setupStandardMock(
  page: Page,
  traceItems: TraceItem[] = createMockTraceItems()
) {
  await page.evaluate((items) => {
    let scanCompleted = false;

    window.__setTauriMockHandler__((cmd: string, payload: Record<string, unknown>) => {
      switch (cmd) {
        case "get_resource_config":
          return { unlimited: false, cpu_percent: 30 } satisfies ResourceConfig;

        case "start_scan": {
          scanCompleted = false;
          setTimeout(() => {
            scanCompleted = true;
          }, 500);
          return "test-scan-001";
        }

        case "pause_scan":
        case "resume_scan":
          return undefined;

        case "get_session_state": {
          if (scanCompleted) return { Scanned: {} };
          return "Scanning";
        }

        case "get_scan_results": {
          const pageNum = (payload.page as number) || 1;
          const pageSize = (payload.pageSize as number) || 50;
          const start = (pageNum - 1) * pageSize;
          const end = start + pageSize;
          return { items: items.slice(start, end), total: items.length };
        }

        case "submit_decisions":
          return undefined;

        case "start_execution": {
          // 模拟执行耗时，让 ExecutingPage 有展示时间
          return new Promise((resolve) => {
            setTimeout(() => {
              resolve({
                deleted_count: 2,
                packed_count: 1,
                preserved_count: 1,
                deleted_bytes: 1024 * 1024 * 100 + 2048,
                packed_bytes: 0,
                preserved_bytes: 0,
                pack_file_path: "C:\\Users\\User\\Desktop\\French-exit.zip",
                report_html_path: "C:\\Users\\User\\Desktop\\French-exit-report.html",
              } satisfies ExecutionReport);
            }, 800);
          });
        }

        default:
          throw new Error(`Unhandled mock command: ${cmd}`);
      }
    });
  }, traceItems);
}

/**
 * 自定义 Playwright fixture
 * - 自动注入 tauri-mock.js
 * - 提供 emitEvent 辅助方法
 */
export const test = base.extend<{
  emitEvent: (event: string, payload: unknown) => Promise<void>;
}>({
  page: async ({ page }, use) => {
    const mockScript = fs.readFileSync(
      path.join(__dirname, "tauri-mock.js"),
      "utf-8"
    );
    await page.addInitScript(mockScript);
    await use(page);
  },

  emitEvent: async ({ page }, use) => {
    await use(async (event, payload) => emitTauriEvent(page, event, payload));
  },
});

/**
 * 通过自定义 DatePicker 设置日期（年/月/日三级下拉面板）
 */
export async function fillDatePicker(page: Page, dateStr: string) {
  const [year, month, day] = dateStr.split("-");

  // 年份
  await page.getByRole("button", { name: "年", exact: true }).click();
  await page.getByRole("button", { name: `${year}年`, exact: true }).click();

  // 月份
  await page.getByRole("button", { name: "月", exact: true }).click();
  await page.getByRole("button", { name: `${month}月`, exact: true }).click();

  // 日期
  await page.getByRole("button", { name: "日", exact: true }).click();
  await page.getByRole("button", { name: `${day}日`, exact: true }).click();
}

export { expect };
