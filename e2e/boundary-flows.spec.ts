import { test, expect, setupStandardMock, createMockTraceItems, fillDatePicker } from "./fixtures";

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

    // 默认应为深色主题（全局默认 dark，不再跟随系统）
    const html = page.locator("html");
    await expect(html).toHaveClass(/dark/);
  });

  test("重置流程：ReportPage 点击重新开始清空状态", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    // 走完整流程到 ReportPage
    await fillDatePicker(page, '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();
    // 已移除默认勾选，需手动勾选才能进入下一步
    await page.locator('input[type="checkbox"]').first().click();
    await page.click('button:has-text("下一步：确认执行")');
    await expect(page.locator('h2:has-text("最终确认")')).toBeVisible();
    await page.click('button:has-text("确认执行")');
    await page.click('button:has-text("确定继续")');

    // 模拟执行完成事件（ExecutingPage 通过事件监听跳转）
    await emitEvent("scan_progress", {
      type: "ExecutionCompleted",
      report: {
        deleted_count: 1,
        packed_count: 0,
        preserved_count: 0,
        deleted_bytes: 0,
        packed_bytes: 0,
        preserved_bytes: 0,
        pack_file_path: null,
        items: [],
      },
    });

    await expect(page.locator('text=清理完成')).toBeVisible();

    // 点击左上角 Logo 返回 WelcomePage（ReportPage 已移除"开始新的清理"按钮）
    await page.locator('button:has-text("French Exit")').click();
    await expect(page.locator('button:has-text("开始使用")')).toBeVisible();

    // 从欢迎页重新进入输入页
    await page.click('button:has-text("开始使用")');

    // InputPage 应可正常渲染（日期可能保留上次值，由业务决定）
    await expect(page.getByRole("button", { name: "开始扫描" })).toBeVisible();
  });

  test("空扫描结果：ResultsPage 显示空状态", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page, []);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await fillDatePicker(page, '2026-01-01');
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

    await fillDatePicker(page, '2026-01-01');
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

    await fillDatePicker(page, '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await expect(page.locator('text=正在扫描…')).toBeVisible();

    // 点击取消
    await page.click('button:has-text("取消")');

    // 返回 WelcomePage（取消扫描触发 RESET）
    await expect(page.locator('text=French Exit').first()).toBeVisible();
    await expect(page.locator('button:has-text("开始使用")')).toBeVisible();
  });
});
