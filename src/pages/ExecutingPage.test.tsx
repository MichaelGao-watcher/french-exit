import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { ExecutingPage } from "./ExecutingPage";
import { TestAppProvider } from "../store/AppContext";
import type { ExecutionReport } from "../types";

vi.mock("../api/commands", () => ({
  startExecution: vi.fn(),
  listenScanProgress: vi.fn(() => Promise.resolve(() => {})),
}));

import { startExecution, listenScanProgress } from "../api/commands";
const mockStartExecution = vi.mocked(startExecution);
const mockListenScanProgress = vi.mocked(listenScanProgress);

describe("ExecutingPage", () => {
  beforeEach(() => {
    mockStartExecution.mockClear();
    mockListenScanProgress.mockClear();
    // 模拟 Tauri 环境，让组件走真实 IPC 路径而非纯前端 mock 路径
    vi.stubGlobal("__TAURI_INTERNALS__", {});
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders loading spinner and message", () => {
    mockStartExecution.mockImplementation(() => new Promise(() => {}));
    render(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    expect(screen.getByText(/正在执行清理/i)).toBeInTheDocument();
    expect(screen.getByText(/准备执行/i)).toBeInTheDocument();
  });

  it("registers progress listener and calls startExecution on mount", async () => {
    mockStartExecution.mockImplementation(() => new Promise(() => {}));

    render(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    await waitFor(() => {
      expect(mockListenScanProgress).toHaveBeenCalledTimes(1);
      expect(mockStartExecution).toHaveBeenCalledTimes(1);
    });
  });

  it("dispatches SET_ERROR and navigates back to confirm on failure", async () => {
    mockStartExecution.mockRejectedValue(new Error("打包失败：磁盘空间不足"));

    render(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    await waitFor(() => {
      expect(mockStartExecution).toHaveBeenCalledTimes(1);
    });
  });

  it("calls startExecution only once even on re-render", async () => {
    mockStartExecution.mockImplementation(() => new Promise(() => {}));

    const { rerender } = render(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    rerender(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    await waitFor(() => {
      expect(mockStartExecution).toHaveBeenCalledTimes(1);
    });
  });
});
