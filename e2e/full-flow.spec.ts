import { test, expect, setupStandardMock, createMockTraceItems } from "./fixtures";

/**
 * French Exit 完整用户流程 E2E 测试
 *
 * 流程：InputPage → ScanPage → ResultsPage → ConfirmPage → ExecutingPage → ReportPage
 * 所有 Tauri IPC 通过 e2e/tauri-mock.js 在浏览器端 mock。
 */

test.describe("完整流程骨架", () => {
  test("输入日期 → 开始扫描 → 结果页默认勾选 → 确认执行 → 生成报告", async ({
    page,
    emitEvent,
  }) => {
    const traceItems = createMockTraceItems();

    // 1. 打开应用首页（必须先 goto，addInitScript 才会在目标页面生效）
    await page.goto("/");

    // 2. 设置标准 mock
    await setupStandardMock(page, traceItems);

    await expect(page.locator("h1")).toContainText("French Exit");
    await expect(page.locator('button:has-text("开始扫描")')).toBeVisible();

    // 3. InputPage：选择入职日期并点击开始扫描
    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');

    // 4. ScanPage：等待扫描完成（轮询 + 事件双保险）
    await expect(page.locator('text=正在扫描…')).toBeVisible();

    // 模拟实时进度推送
    await emitEvent("scan_progress", {
      type: "ScanProgress",
      message: "正在扫描浏览器历史…",
      current: 2,
      total: 4,
    });

    await emitEvent("scan_progress", {
      type: "ScanCompleted",
    });

    // 5. ResultsPage：等待结果加载
    await expect(page.locator('h2:has-text("发现")')).toContainText("发现 4 条痕迹");

    // 检查默认勾选（RULE-03 微信默认选中；RULE-02 环境变量默认不选）
    // 微信记录（DeleteOrPack）→ Delete，默认选中
    // Browser（Delete）→ Delete，默认选中
    // EnvVar（Delete）→ 默认不选 (RULE-02)
    // FileSystem（Pack）→ Pack，默认选中
    const chatCheckbox = page.locator('input[type="checkbox"]').nth(0);
    const browserCheckbox = page.locator('input[type="checkbox"]').nth(1);
    const envCheckbox = page.locator('input[type="checkbox"]').nth(2);
    const fsCheckbox = page.locator('input[type="checkbox"]').nth(3);

    await expect(page.locator('text=微信聊天记录')).toBeVisible();

    await expect(chatCheckbox).toBeChecked();
    await expect(browserCheckbox).toBeChecked();
    await expect(envCheckbox).not.toBeChecked();
    await expect(fsCheckbox).toBeChecked();

    // 6. 交互：取消勾选 Browser，额外勾选 EnvVar（默认 action=Delete）
    await browserCheckbox.click();
    await expect(browserCheckbox).not.toBeChecked();

    await envCheckbox.click();
    await expect(envCheckbox).toBeChecked();

    // 7. 点击下一步进入 ConfirmPage
    await page.click('button:has-text("下一步：确认执行")');
    await expect(page.locator('h2:has-text("最终确认")')).toBeVisible();

    // 确认列表包含删除、打包（EnvVar 的默认 action 也是 Delete）
    await expect(page.locator('text=待删除')).toBeVisible();
    await expect(page.locator('text=待打包')).toBeVisible();

    // 8. 点击确认执行，弹出二次确认弹窗（RULE-01）
    await page.click('button:has-text("确认执行")');
    await expect(page.locator('text=操作不可逆')).toBeVisible();

    // 弹窗中点击"确定继续"
    await page.click('button:has-text("确定继续")');

    // 9. ExecutingPage：等待执行完成
    await expect(page.locator('text=正在执行清理...')).toBeVisible();

    // 10. ReportPage：验证报告数据
    await expect(page.locator('text=清理完成')).toBeVisible();
    await expect(page.getByText('已删除', { exact: true }).first()).toBeVisible();
    await expect(page.getByText('已打包', { exact: true }).first()).toBeVisible();
    await expect(page.getByText('已保留', { exact: true }).first()).toBeVisible();

    // 检查报告统计数字
    await expect(page.locator('.text-green-600')).toContainText("2");
    await expect(page.locator('.text-blue-600')).toContainText("1");
    await expect(page.locator('.text-amber-600')).toContainText("1");

    // 11. 点击"开始新的清理"返回首页
    await page.click('button:has-text("开始新的清理")');
    await expect(page.locator("h1")).toContainText("French Exit");
  });

  test("InputPage 日期校验：未选择日期时禁止启动", async ({ page }) => {
    await page.goto("/");
    await setupStandardMock(page);

    const startBtn = page.locator('button:has-text("开始扫描")');
    await expect(startBtn).toBeDisabled();

    // 尝试点击（应该无反应）
    await expect(page.locator("h1")).toContainText("French Exit");
  });

  test("ScanPage 暂停与恢复", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    await page.fill('#start-date', '2026-01-01');
    await page.click('button:has-text("开始扫描")');

    await expect(page.locator('text=正在扫描…')).toBeVisible();

    // 点击暂停
    await page.click('button:has-text("暂停")');
    await expect(page.locator('text=扫描已暂停')).toBeVisible();

    // 点击恢复
    await page.click('button:has-text("恢复扫描")');
    await expect(page.locator('text=正在扫描…')).toBeVisible();

    // 完成扫描跳转
    await emitEvent("scan_progress", { type: "ScanCompleted" });
    await expect(page.locator('h2:has-text("发现")')).toBeVisible();
  });
});
