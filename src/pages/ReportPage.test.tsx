import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ReportPage } from "./ReportPage";
import { initialState } from "../store/AppContext";
import type { ExecutionReport, AppState } from "../types";

vi.mock("../store/AppContext", async () => {
  const actual = await vi.importActual<typeof import("../store/AppContext")>(
    "../store/AppContext"
  );
  return {
    ...actual,
    useAppState: vi.fn(),
  };
});

import { useAppState } from "../store/AppContext";
const mockUseAppState = vi.mocked(useAppState);

function makeReport(overrides: Partial<ExecutionReport> = {}): ExecutionReport {
  return {
    deleted_count: 0,
    deleted_bytes: 0,
    packed_count: 0,
    packed_bytes: 0,
    preserved_count: 0,
    pack_file_path: null,
    items: [],
    ...overrides,
  };
}

function setupMockState(override: Partial<AppState> = {}) {
  const mockDispatch = vi.fn();
  const state = { ...initialState, ...override } as AppState;
  mockUseAppState.mockReturnValue({ state, dispatch: mockDispatch });
  return { state, mockDispatch };
}

describe("ReportPage", () => {
  beforeEach(() => {
    mockUseAppState.mockClear();
  });

  it("shows empty state when no report", () => {
    setupMockState({ page: "report", report: null });
    render(<ReportPage />);
    expect(screen.getByText(/暂无报告数据/i)).toBeInTheDocument();
  });

  it("renders celebration and summary", () => {
    const report = makeReport({
      deleted_count: 10,
      deleted_bytes: 1024 * 1024 * 50,
      packed_count: 3,
      packed_bytes: 1024 * 1024 * 20,
      preserved_count: 2,
      pack_file_path: "C:\\Users\\test\\French-exit.zip",
    });

    setupMockState({ page: "report", report });
    render(<ReportPage />);

    expect(screen.getByText(/清理完成/i)).toBeInTheDocument();
    expect(screen.getByText(/您已完成 French Exit/i)).toBeInTheDocument();
    expect(screen.getByText(/已删除/i)).toBeInTheDocument();
    expect(screen.getByText(/已打包/i)).toBeInTheDocument();
    expect(screen.getByText(/已保留/i)).toBeInTheDocument();
  });

  it("displays pack file path when available", () => {
    const report = makeReport({
      packed_count: 1,
      pack_file_path: "C:\\Users\\test\\Desktop\\French-exit.zip",
    });

    setupMockState({ page: "report", report });
    render(<ReportPage />);

    expect(screen.getByText(/French-exit.zip/i)).toBeInTheDocument();
  });

  it("hides pack file path when not available", () => {
    const report = makeReport({
      packed_count: 0,
      pack_file_path: null,
    });

    setupMockState({ page: "report", report });
    render(<ReportPage />);

    expect(screen.queryByText(/打包文件/i)).not.toBeInTheDocument();
  });


});
