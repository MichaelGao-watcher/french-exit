import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright E2E 配置
 *
 * 策略：browser-only 模式，通过 Vite dev server 运行前端，
 * 用 e2e/tauri-mock.js 替代真实 Tauri IPC。
 * 覆盖完整用户流程：Input → Scan → Results → Confirm → Executing → Report
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",

  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    stderr: "pipe",
  },
});
