import { vi } from "vitest";
import "@testing-library/jest-dom/vitest";

// Mock Tauri API modules that are not available in jsdom
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `tauri://${path}`),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/api/fs", () => ({
  readTextFile: vi.fn(() => Promise.resolve("mock file content")),
}));
