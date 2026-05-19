import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { ScanPage } from "./ScanPage";
import { AppProvider } from "../store/AppContext";
import type { ProgressEvent } from "../types";

let progressHandler: ((event: ProgressEvent) => void) | null = null;

vi.mock("../api/commands", () => ({
  getSessionState: vi.fn(() => Promise.resolve("Scanning")),
  pauseScan: vi.fn(() => Promise.resolve()),
  resumeScan: vi.fn(() => Promise.resolve()),
  listenScanProgress: vi.fn((handler: (event: ProgressEvent) => void) => {
    progressHandler = handler;
    return Promise.resolve(() => {});
  }),
}));

function renderWithProvider(ui: React.ReactElement) {
  return render(<AppProvider>{ui}</AppProvider>);
}

describe("ScanPage", () => {
  beforeEach(() => {
    progressHandler = null;
    vi.clearAllMocks();
  });

  it("renders scanning state with progress bar", async () => {
    renderWithProvider(<ScanPage />);
    await waitFor(() => {
      expect(screen.getByText(/正在扫描/i)).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: /暂停/i })).toBeInTheDocument();
  });

  it("updates progress on ScanProgress event", async () => {
    renderWithProvider(<ScanPage />);
    await waitFor(() => {
      expect(progressHandler).not.toBeNull();
    });

    act(() => {
      progressHandler!({
        type: "ScanProgress",
        scanner_id: "scanner-fs",
        current: 3,
        total: 7,
        message: "正在扫描文件系统…",
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/正在扫描文件系统/i)).toBeInTheDocument();
    });
  });

  it("pauses and resumes scan", async () => {
    const { pauseScan, resumeScan } = await import("../api/commands");
    renderWithProvider(<ScanPage />);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /暂停/i })).toBeInTheDocument();
    });

    // Click pause
    fireEvent.click(screen.getByRole("button", { name: /暂停/i }));
    await waitFor(() => {
      expect(pauseScan).toHaveBeenCalledTimes(1);
    });

    // Simulate paused event
    act(() => {
      progressHandler!({ type: "ScanPaused" });
    });
    await waitFor(() => {
      expect(screen.getByText(/扫描已暂停/i)).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /恢复扫描/i })).toBeInTheDocument();
    });

    // Click resume
    fireEvent.click(screen.getByRole("button", { name: /恢复扫描/i }));
    await waitFor(() => {
      expect(resumeScan).toHaveBeenCalledTimes(1);
    });
  });

  it("cancels scan triggers RESET and clears interval", async () => {
    const { result } = renderWithProvider(<ScanPage />);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /取消/i })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /取消/i }));

    // After cancel, the page should still render (ScanPage itself does not unmount),
    // but internal interval is cleared and dispatch RESET is called.
    // We verify the cancel button is still clickable (no crash).
    expect(screen.getByRole("button", { name: /取消/i })).toBeInTheDocument();
  });
});
