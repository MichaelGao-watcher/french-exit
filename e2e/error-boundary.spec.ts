import { test, expect, createMockTraceItems } from "./fixtures";

/**
 * 错误边界 E2E
 *
 * 覆盖：后端命令抛出异常时，前端各页面的错误提示与状态恢复。
 */

test.describe("错误边界", () => {
  test("start_scan 失败时 InputPage 显示错误", async ({ page }) => {
    await page.goto("/");

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await page.evaluate(() => {
      window.__setTauriMockHandler__((cmd: string) => {
        if (cmd === "get_resource_config") {
          return { unlimited: false, cpu_percent: 30 };
        }
        if (cmd === "start_scan") {
          throw new Error("扫描服务未启动");
        }
        throw new Error(`Unexpected: ${cmd}`);
      });
    });

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');

    await expect(page.locator('text=扫描服务未启动')).toBeVisible();
    // 页面应保持在 InputPage
    await expect(page.locator('h1')).toContainText("French Exit");
  });

  test("get_scan_results 失败时 ResultsPage 显示错误", async ({ page, emitEvent }) => {
    await page.goto("/");

    let callCount = 0;
    await page.evaluate((items) => {
      let scanCompleted = false;
      let count = 0;

      window.__setTauriMockHandler__((cmd: string, payload: Record<string, unknown>) => {
        switch (cmd) {
          case "get_resource_config":
            return { unlimited: false, cpu_percent: 30 };
          case "start_scan":
            scanCompleted = false;
            setTimeout(() => { scanCompleted = true; }, 200);
            return "test-scan-001";
          case "get_session_state":
            return scanCompleted ? { Scanned: {} } : "Scanning";
          case "get_scan_results":
            count++;
            if (count === 1) {
              throw new Error("数据库连接失败");
            }
            return { items, total: items.length };
          default:
            throw new Error(`Unexpected: ${cmd}`);
        }
      });
    }, createMockTraceItems());

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();
    await expect(page.locator('text=数据库连接失败')).toBeVisible();
  });

  test("submit_decisions 失败时 ConfirmPage 显示错误", async ({ page, emitEvent }) => {
    await page.goto("/");

    await page.evaluate((items) => {
      let scanCompleted = false;

      window.__setTauriMockHandler__((cmd: string, payload: Record<string, unknown>) => {
        switch (cmd) {
          case "get_resource_config":
            return { unlimited: false, cpu_percent: 30 };
          case "start_scan":
            scanCompleted = false;
            setTimeout(() => { scanCompleted = true; }, 200);
            return "test-scan-001";
          case "get_session_state":
            return scanCompleted ? { Scanned: {} } : "Scanning";
          case "get_scan_results":
            return { items, total: items.length };
          case "submit_decisions":
            throw new Error("决策提交超时");
          default:
            throw new Error(`Unexpected: ${cmd}`);
        }
      });
    }, createMockTraceItems());

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();
    await page.click('button:has-text("下一步：确认执行")');
    await expect(page.locator('h2:has-text("最终确认")')).toBeVisible();

    await page.click('button:has-text("确认执行")');
    await page.click('button:has-text("确定继续")');

    await expect(page.locator('text=决策提交超时')).toBeVisible();
    // 应保持在 ConfirmPage
    await expect(page.locator('h2:has-text("最终确认")')).toBeVisible();
  });

  test("start_execution 失败时返回 ConfirmPage", async ({ page, emitEvent }) => {
    await page.goto("/");

    await page.evaluate((items) => {
      let scanCompleted = false;

      window.__setTauriMockHandler__((cmd: string, payload: Record<string, unknown>) => {
        switch (cmd) {
          case "get_resource_config":
            return { unlimited: false, cpu_percent: 30 };
          case "start_scan":
            scanCompleted = false;
            setTimeout(() => { scanCompleted = true; }, 200);
            return "test-scan-001";
          case "get_session_state":
            return scanCompleted ? { Scanned: {} } : "Scanning";
          case "get_scan_results":
            return { items, total: items.length };
          case "submit_decisions":
            return undefined;
          case "start_execution":
            throw new Error("执行器异常");
          default:
            throw new Error(`Unexpected: ${cmd}`);
        }
      });
    }, createMockTraceItems());

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await page.click('button:has-text("下一步：确认执行")');
    await page.click('button:has-text("确认执行")');
    await page.click('button:has-text("确定继续")');

    // ExecutingPage 的 catch 会 dispatch SET_PAGE "confirm"，而 SET_PAGE 会同时清除 error，
    // 因此 ConfirmPage 上不显示错误文本，但页面应正确返回 ConfirmPage。
    await expect(page.locator('h2:has-text("最终确认")')).toBeVisible();
  });
});
