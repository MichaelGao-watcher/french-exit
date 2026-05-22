import { test, expect, setupStandardMock, createMockTraceItems, fillDatePicker } from "./fixtures";

/**
 * ResultsPage 交互细节 E2E
 *
 * 覆盖：分类 Tab 过滤、搜索过滤、预览弹窗、分页加载
 */

test.describe("ResultsPage 交互", () => {
  test("分类 Tab 过滤", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await fillDatePicker(page, '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();

    // 默认显示全部 4 条
    await expect(page.getByText('微信聊天记录', { exact: true })).toBeVisible();
    await expect(page.getByText('Chrome 历史记录', { exact: true })).toBeVisible();
    await expect(page.getByText('API_TOKEN', { exact: true })).toBeVisible();
    await expect(page.getByText('工作文件.txt', { exact: true }).first()).toBeVisible();

    // 切换到"聊天"分类
    await page.click('button:has-text("聊天")');
    await expect(page.getByText('微信聊天记录', { exact: true })).toBeVisible();
    await expect(page.getByText('Chrome 历史记录', { exact: true })).not.toBeVisible();
    await expect(page.getByText('API_TOKEN', { exact: true })).not.toBeVisible();
    await expect(page.getByText('工作文件.txt', { exact: true }).first()).not.toBeVisible();

    // 切换到"浏览器"分类
    await page.click('button:has-text("浏览器")');
    await expect(page.getByText('微信聊天记录', { exact: true })).not.toBeVisible();
    await expect(page.getByText('Chrome 历史记录', { exact: true })).toBeVisible();

    // 切换到"环境变量"分类
    await page.click('button:has-text("环境变量")');
    await expect(page.getByText('API_TOKEN', { exact: true })).toBeVisible();

    // 切换到"文件"分类
    await page.click('button:has-text("文件")');
    await expect(page.getByText('工作文件.txt', { exact: true }).first()).toBeVisible();
  });

  test("搜索过滤", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await fillDatePicker(page, '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();

    // 搜索"微信"
    await page.fill('input[placeholder="搜索文件名或路径..."]', '微信');
    await expect(page.getByText('微信聊天记录', { exact: true })).toBeVisible();
    await expect(page.getByText('Chrome 历史记录', { exact: true })).not.toBeVisible();
    await expect(page.getByText('工作文件.txt', { exact: true }).first()).not.toBeVisible();

    // 清空搜索（直接 fill 空字符串）
    await page.fill('input[placeholder="搜索文件名或路径..."]', '');
    await expect(page.getByText('微信聊天记录', { exact: true })).toBeVisible();
    await expect(page.getByText('Chrome 历史记录', { exact: true })).toBeVisible();

    // 搜索路径
    await page.fill('input[placeholder="搜索文件名或路径..."]', 'Desktop');
    await expect(page.getByText('工作文件.txt', { exact: true }).first()).toBeVisible();
    await expect(page.getByText('微信聊天记录', { exact: true })).not.toBeVisible();
  });

  test("预览弹窗打开与关闭", async ({ page, emitEvent }) => {
    await page.goto("/");
    await setupStandardMock(page);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await fillDatePicker(page, '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toBeVisible();

    // 找到"工作文件.txt"行并点击预览
    const workFileRow = page.locator("div", {
      hasText: "工作文件.txt",
    }).locator("..").first();
    await workFileRow.locator('button:has-text("预览")').click();

    // 弹窗出现
    await expect(page.locator('text=mock file content')).toBeVisible();
    await expect(page.getByText('工作文件.txt', { exact: true }).nth(1)).toBeVisible();

    // 点击关闭按钮（弹窗右上角的 X）
    await page.locator('div[class*="fixed inset-0"] button').first().click();
    await expect(page.locator('text=mock file content')).not.toBeVisible();

    // 再次打开，按 ESC 关闭
    await workFileRow.locator('button:has-text("预览")').click();
    await expect(page.locator('text=mock file content')).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator('text=mock file content')).not.toBeVisible();
  });

  test("分页加载", async ({ page, emitEvent }) => {
    // 构造 55 条 mock 数据以触发分页
    const manyItems = Array.from({ length: 55 }, (_, i) => ({
      id: `item-${i}`,
      name: `测试文件 ${i}.txt`,
      path: `C:\\Users\\User\\Desktop\\测试文件 ${i}.txt`,
      category: "FileSystem" as const,
      size_bytes: 1024,
      modified_at: "2026-05-10T08:00:00Z",
      suggested_action: "Delete" as const,
      inferred: false,
      risk_note: null,
    }));

    await page.goto("/");
    await setupStandardMock(page, manyItems);

    // 从欢迎页进入输入页
    await page.click('button:has-text("开始使用")');

    await fillDatePicker(page, '2026-01-01');
    await page.click('button:has-text("开始扫描")');
    await emitEvent("scan_progress", { type: "ScanCompleted" });

    await expect(page.locator('h2:has-text("发现")')).toContainText("发现 55 条痕迹");

    // 默认加载 50 条，应显示"加载更多"
    await expect(page.locator('button:has-text("加载更多")')).toBeVisible();
    await expect(page.getByText('测试文件 0.txt', { exact: true })).toBeVisible();

    // 点击加载更多
    await page.click('button:has-text("加载更多")');
    await expect(page.getByText('测试文件 54.txt', { exact: true })).toBeVisible();
    await expect(page.locator('button:has-text("加载更多")')).not.toBeVisible();
  });
});
