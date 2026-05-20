import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConfirmPage } from "./ConfirmPage";
import { initialState } from "../store/AppContext";
import type { TraceItem, Decision, AppState } from "../types";

vi.mock("../api/commands", () => ({
  submitDecisions: vi.fn(() => Promise.resolve()),
}));

vi.mock("../store/AppContext", async () => {
  const actual = await vi.importActual<typeof import("../store/AppContext")>(
    "../store/AppContext"
  );
  return {
    ...actual,
    useAppState: vi.fn(),
  };
});

import { submitDecisions } from "../api/commands";
import { useAppState } from "../store/AppContext";

const mockSubmitDecisions = vi.mocked(submitDecisions);
const mockUseAppState = vi.mocked(useAppState);

function makeItem(overrides: Partial<TraceItem> = {}): TraceItem {
  const id = `item-${overrides.id || "1"}`;
  return {
    category: "FileSystem",
    scanner_id: "test",
    name: "test.txt",
    path: "/home/test.txt",
    size_bytes: 1024,
    modified_at: "2024-01-01T00:00:00Z",
    inferred: false,
    risk_note: null,
    suggested_action: "Delete",
    ...overrides,
    id,
  };
}

function setupMockState(override: Partial<AppState> = {}) {
  const mockDispatch = vi.fn();
  const state = { ...initialState, ...override } as AppState;
  mockUseAppState.mockReturnValue({ state, dispatch: mockDispatch });
  return { state, mockDispatch };
}

describe("ConfirmPage", () => {
  beforeEach(() => {
    mockSubmitDecisions.mockClear();
    mockUseAppState.mockClear();
  });

  it("renders grouped items (delete / pack / preserve)", () => {
    const items = [
      makeItem({ id: "1", name: "delete.txt", suggested_action: "Delete" }),
      makeItem({ id: "2", name: "pack.zip", suggested_action: "Pack" }),
      makeItem({ id: "3", name: "keep.doc", suggested_action: "Preserve" }),
    ];
    const decisions = new Map<string, Decision>([
      ["item-1", { item_id: "item-1", action: "Delete" }],
      ["item-2", { item_id: "item-2", action: "Pack" }],
      ["item-3", { item_id: "item-3", action: "Preserve" }],
    ]);

    setupMockState({ page: "confirm", scanResults: items, decisions });
    render(<ConfirmPage />);

    expect(screen.getByText(/待删除/i)).toBeInTheDocument();
    expect(screen.getByText(/待打包/i)).toBeInTheDocument();
    expect(screen.getByText(/待保留/i)).toBeInTheDocument();
    expect(screen.getByText("delete.txt")).toBeInTheDocument();
    expect(screen.getByText("pack.zip")).toBeInTheDocument();
    expect(screen.getByText("keep.doc")).toBeInTheDocument();
  });

  it("shows error when no delete or pack items", async () => {
    const items = [
      makeItem({ id: "1", name: "keep.doc", suggested_action: "Preserve" }),
    ];
    const decisions = new Map<string, Decision>([
      ["item-1", { item_id: "item-1", action: "Preserve" }],
    ]);

    const { mockDispatch } = setupMockState({
      page: "confirm",
      scanResults: items,
      decisions,
    });
    render(<ConfirmPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /确认执行/i }));

    expect(mockDispatch).toHaveBeenCalledWith({
      type: "SET_ERROR",
      payload: "没有标记任何需要删除或打包的内容",
    });
  });

  it("opens confirmation dialog and submits decisions", async () => {
    const items = [
      makeItem({ id: "1", name: "delete.txt", size_bytes: 2048 }),
    ];
    const decisions = new Map<string, Decision>([
      ["item-1", { item_id: "item-1", action: "Delete" }],
    ]);

    setupMockState({ page: "confirm", scanResults: items, decisions });
    render(<ConfirmPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /确认执行/i }));

    await waitFor(() => {
      expect(screen.getByText(/操作不可逆/i)).toBeInTheDocument();
    });

    expect(screen.getByText(/1 条痕迹/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /确定继续/i }));

    await waitFor(() => {
      expect(mockSubmitDecisions).toHaveBeenCalledTimes(1);
    });
    expect(mockSubmitDecisions).toHaveBeenCalledWith([
      { item_id: "item-1", action: "Delete" },
    ]);
  });

  it("can cancel confirmation dialog", async () => {
    const items = [makeItem({ id: "1", name: "delete.txt" })];
    const decisions = new Map<string, Decision>([
      ["item-1", { item_id: "item-1", action: "Delete" }],
    ]);

    setupMockState({ page: "confirm", scanResults: items, decisions });
    render(<ConfirmPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /确认执行/i }));

    await waitFor(() => {
      expect(screen.getByText(/操作不可逆/i)).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /再想想/i }));

    await waitFor(() => {
      expect(screen.queryByText(/操作不可逆/i)).not.toBeInTheDocument();
    });

    expect(mockSubmitDecisions).not.toHaveBeenCalled();
  });

  it("navigates back to results on back button click", async () => {
    const items = [makeItem({ id: "1", name: "delete.txt" })];
    const decisions = new Map<string, Decision>([
      ["item-1", { item_id: "item-1", action: "Delete" }],
    ]);
    const { mockDispatch } = setupMockState({
      page: "confirm",
      scanResults: items,
      decisions,
    });
    render(<ConfirmPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /返回修改/i }));

    expect(mockDispatch).toHaveBeenCalledWith({
      type: "SET_PAGE",
      payload: "results",
    });
  });
});
