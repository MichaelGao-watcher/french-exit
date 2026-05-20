import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { ExecutingPage } from "./ExecutingPage";
import { TestAppProvider } from "../store/AppContext";
import type { ExecutionReport } from "../types";

vi.mock("../api/commands", () => ({
  startExecution: vi.fn(),
}));

import { startExecution } from "../api/commands";
const mockStartExecution = vi.mocked(startExecution);

describe("ExecutingPage", () => {
  beforeEach(() => {
    mockStartExecution.mockClear();
  });

  it("renders loading spinner and message", () => {
    mockStartExecution.mockImplementation(() => new Promise(() => {}));
    render(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    expect(screen.getByText(/正在执行清理/i)).toBeInTheDocument();
    expect(screen.getByText(/正在安全删除文件/i)).toBeInTheDocument();
  });

  it("dispatches SET_REPORT and navigates to report on success", async () => {
    const report: ExecutionReport = {
      deleted_count: 5,
      deleted_bytes: 1024,
      packed_count: 2,
      packed_bytes: 512,
      preserved_count: 1,
      pack_file_path: "/tmp/French-exit.zip",
      items: [],
    };
    mockStartExecution.mockResolvedValue(report);

    render(
      <TestAppProvider initialState={{ page: "executing" }}>
        <ExecutingPage />
      </TestAppProvider>
    );

    await waitFor(() => {
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
