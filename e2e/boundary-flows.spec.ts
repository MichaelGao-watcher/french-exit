import { test, expect, setupStandardMock, createMockTraceItems } from "./fixtures";

/**
 * 边界流程与系统行为 E2E
 *
 * 覆盖：深色模式、重置流程、空结果、扫描失败、取消扫描
 */

test.describe("边界流程", () => {
  test("深色模式切换", async ({ page }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    // 默认检查 html 是否有 dark class
    const html = page.locator("html");

    // 模拟系统切换到 dark
    await page.emulateMedia({ colorScheme: "dark" });
    await expect(html).toHaveClass(/dark/);

    // 模拟系统切换到 light
    await page.emulateMedia({ colorScheme: "light" });
    await expect(html).not.toHaveClass(/dark/);
  });

  test("重置流程：ReportPage 点击重新开始清空状态", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    // 走完整流程到 ReportPage
    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();
    await page.click('button:has-text("下一步：确认执行")');
    await expect(page.locator('h2:has-text("最终确认")')).toBeVisible();
    await page.click('button:has-text("确认执行")');
    await page.click('button:has-text("确定继续")');
    await expect(page.locator('text=清理完成')).toBeVisible();

    // 点击重新开始
    await page.click('button:has-text("开始新的清理")');
    await expect(page.locator('text=French Exit').first()).toBeVisible();

    // 从欢迎页重新进入输入页
    await page.click('button:has-text("开始使用")');

    // 日期输入应被清空
    const dateInput = page.locator('#start-date');
    await expect(dateInput).toHaveValue("");

    // 开始扫描按钮应被禁用
    await expect(page.locator('button:has-text("开始扫描")')).toBeDisabled();
  });

  test("空扫描结果：ResultsPage 显示空状态", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page, []);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    // 等待 ResultsPage
    await expect(page.locator('h2:has-text("发现")')).toContainText("发现 0 条痕迹");

    // 空状态提示
    await expect(page.locator('text=该分类下暂无痕迹')).toBeVisible();
  });

  test("扫描失败：显示错误并保持可取消", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');

    await expect(page.locator('text=正在扫描…')).toBeVisible();

    // 模拟扫描失败
    await emitEvent("scan_progress", {
      type: "ScanFailed",
      reason: "磁盘读取权限不足",
    });

    // 错误提示应显示
    await expect(page.locator('text=磁盘读取权限不足')).toBeVisible();

    // 应停留在 ScanPage（或允许用户取消返回首页）
    await expect(page.locator('button:has-text("取消")')).toBeVisible();
  });

  test("取消扫描：返回 InputPage", async ({ page }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await expect(page.locator('text=正在扫描…')).toBeVisible();

    // 点击取消
    await page.click('button:has-text("取消")');

    // 返回 InputPage
    await expect(page.locator('#start-date')).toBeVisible();
    await expect(page.locator('button:has-text("开始扫描")')).toBeDisabled();
  });
});
