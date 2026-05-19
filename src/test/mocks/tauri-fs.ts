/**
 * 通用 mock：@tauri-apps/api/fs
 *
 * 同时被 Vite alias（dev/build）和 vitest vi.mock 使用。
 * 不依赖 vitest API，确保在浏览器 E2E 环境中也能正常运行。
 */
export function readTextFile(): Promise<string> {
  return Promise.resolve("mock file content");
}
