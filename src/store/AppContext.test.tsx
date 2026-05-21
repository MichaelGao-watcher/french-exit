import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { AppProvider, useAppState } from "./AppContext";
import type { TraceItem, Decision, ExecutionReport } from "../types";

function wrapper({ children }: { children: React.ReactNode }) {
  return <AppProvider>{children}</AppProvider>;
}

/** 构造一个模拟 TraceItem */
function makeItem(overrides: Partial<TraceItem> = {}): TraceItem {
  return {
    id: "item-1",
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
  };
}

describe("AppContext reducer", () => {
  it("initial state should have correct defaults", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    expect(result.current.state.page).toBe("input");
    expect(result.current.state.scanResults).toEqual([]);
    expect(result.current.state.scanTotal).toBe(0);
    expect(result.current.state.decisions.size).toBe(0);
    expect(result.current.state.isScanning).toBe(false);
    expect(result.current.state.isPaused).toBe(false);
    expect(result.current.state.error).toBeNull();
  });

  it("SET_PAGE changes page without clearing error", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    act(() => {
      result.current.dispatch({ type: "SET_ERROR", payload: "some error" });
    });
    act(() => {
      result.current.dispatch({ type: "SET_PAGE", payload: "results" });
    });
    expect(result.current.state.page).toBe("results");
    expect(result.current.state.error).toBe("some error");
  });

  it("SET_SCAN_RESULTS replaces results and total", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    const items = [makeItem({ id: "a" }), makeItem({ id: "b" })];
    act(() => {
      result.current.dispatch({
        type: "SET_SCAN_RESULTS",
        payload: { items, total: 10 },
      });
    });
    expect(result.current.state.scanResults).toHaveLength(2);
    expect(result.current.state.scanTotal).toBe(10);
  });

  it("APPEND_SCAN_RESULTS adds items to existing list", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    act(() => {
      result.current.dispatch({
        type: "SET_SCAN_RESULTS",
        payload: { items: [makeItem({ id: "a" })], total: 3 },
      });
    });
    act(() => {
      result.current.dispatch({
        type: "APPEND_SCAN_RESULTS",
        payload: { items: [makeItem({ id: "b" })], total: 3 },
      });
    });
    expect(result.current.state.scanResults).toHaveLength(2);
    expect(result.current.state.scanResults[0].id).toBe("a");
    expect(result.current.state.scanResults[1].id).toBe("b");
  });

  it("SET_DECISION adds a single decision", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    const decision: Decision = { item_id: "item-1", action: "Delete" };
    act(() => {
      result.current.dispatch({ type: "SET_DECISION", payload: decision });
    });
    expect(result.current.state.decisions.get("item-1")).toEqual(decision);
  });

  it("SET_DECISIONS replaces all decisions", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    const map = new Map<string, Decision>([
      ["item-1", { item_id: "item-1", action: "Delete" }],
      ["item-2", { item_id: "item-2", action: "Pack" }],
    ]);
    act(() => {
      result.current.dispatch({ type: "SET_DECISIONS", payload: map });
    });
    expect(result.current.state.decisions.size).toBe(2);
  });

  it("SET_PROGRESS updates message and percent", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    act(() => {
      result.current.dispatch({
        type: "SET_PROGRESS",
        payload: { message: "scanning…", percent: 42 },
      });
    });
    expect(result.current.state.progressMessage).toBe("scanning…");
    expect(result.current.state.progressPercent).toBe(42);
  });

  it("SET_SCANNING and SET_PAUSED toggle booleans", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    act(() => {
      result.current.dispatch({ type: "SET_SCANNING", payload: true });
    });
    expect(result.current.state.isScanning).toBe(true);

    act(() => {
      result.current.dispatch({ type: "SET_PAUSED", payload: true });
    });
    expect(result.current.state.isPaused).toBe(true);
  });

  it("SET_ERROR stores error message", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    act(() => {
      result.current.dispatch({ type: "SET_ERROR", payload: "oops" });
    });
    expect(result.current.state.error).toBe("oops");
  });

  it("RESET restores initial state", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    act(() => {
      result.current.dispatch({ type: "SET_PAGE", payload: "results" });
      result.current.dispatch({
        type: "SET_SCAN_RESULTS",
        payload: { items: [makeItem()], total: 1 },
      });
    });
    act(() => {
      result.current.dispatch({ type: "RESET" });
    });
    expect(result.current.state.page).toBe("input");
    expect(result.current.state.scanResults).toHaveLength(0);
    expect(result.current.state.decisions.size).toBe(0);
  });

  it("SET_REPORT stores execution report", () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    const report: ExecutionReport = {
      deleted_count: 1,
      deleted_bytes: 100,
      packed_count: 2,
      packed_bytes: 200,
      preserved_count: 0,
      pack_file_path: "/tmp/French-exit.zip",
      items: [],
    };
    act(() => {
      result.current.dispatch({ type: "SET_REPORT", payload: report });
    });
    expect(result.current.state.report).toEqual(report);
  });
});
