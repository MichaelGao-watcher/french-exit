import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ResultsPage } from "./ResultsPage";
import { AppProvider } from "../store/AppContext";
import type { TraceItem } from "../types";

// Mock API commands module
vi.mock("../api/commands", () => ({
  getScanResults: vi.fn(),
}));

import { getScanResults } from "../api/commands";

const mockGetScanResults = vi.mocked(getScanResults);

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

function renderWithProvider(ui: React.ReactElement) {
  return render(<AppProvider>{ui}</AppProvider>);
}

describe("ResultsPage", () => {
  beforeEach(() => {
    mockGetScanResults.mockClear();
  });

  it("renders empty state when no results", async () => {
    mockGetScanResults.mockResolvedValue({
      items: [],
      total: 0,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText(/该分类下暂无痕迹/i)).toBeInTheDocument();
    });
  });

  it("loads scan results on mount", async () => {
    const items = [makeItem({ id: "1", name: "file1.txt" })];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 1,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("file1.txt")).toBeInTheDocument();
    });
    expect(mockGetScanResults).toHaveBeenCalledWith(1, 50);
  });

  it("filters by category tab", async () => {
    const items = [
      makeItem({ id: "1", name: "chat.db", category: "Chat" }),
      makeItem({ id: "2", name: "history.json", category: "Browser" }),
    ];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 2,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("chat.db")).toBeInTheDocument();
      expect(screen.getByText("history.json")).toBeInTheDocument();
    });

    const user = userEvent.setup();
    // Click "Browser" tab
    await user.click(screen.getByRole("button", { name: /浏览器/i }));
    await waitFor(() => {
      expect(screen.queryByText("chat.db")).not.toBeInTheDocument();
      expect(screen.getByText("history.json")).toBeInTheDocument();
    });
  });

  it("filters by search query", async () => {
    const items = [
      makeItem({ id: "1", name: "secret.doc" }),
      makeItem({ id: "2", name: "public.pdf" }),
    ];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 2,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("secret.doc")).toBeInTheDocument();
    });

    const user = userEvent.setup();
    const searchInput = screen.getByPlaceholderText(/搜索文件名或路径/i);
    await user.type(searchInput, "secret");

    await waitFor(() => {
      expect(screen.getByText("secret.doc")).toBeInTheDocument();
      expect(screen.queryByText("public.pdf")).not.toBeInTheDocument();
    });
  });

  it("does not auto-check EnvVar items (RULE-02)", async () => {
    const items = [
      makeItem({ id: "1", name: "TOKEN.env", category: "EnvVar", suggested_action: "Delete" }),
      makeItem({ id: "2", name: "file.txt", category: "FileSystem", suggested_action: "Delete" }),
    ];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 2,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("TOKEN.env")).toBeInTheDocument();
    });

    const checkboxes = screen.getAllByRole("checkbox");
    // First checkbox is EnvVar → unchecked
    expect(checkboxes[0]).not.toBeChecked();
    // Second checkbox is FileSystem → checked by default
    expect(checkboxes[1]).toBeChecked();
  });

  it("auto-checks items with suggested_action DeleteOrPack (RULE-03)", async () => {
    const items = [
      makeItem({ id: "1", name: "wechat.db", category: "Chat", suggested_action: "DeleteOrPack" }),
    ];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 1,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("wechat.db")).toBeInTheDocument();
    });

    const checkbox = screen.getByRole("checkbox");
    expect(checkbox).toBeChecked();
  });

  it("toggles item selection on checkbox click", async () => {
    const items = [
      makeItem({ id: "1", name: "file.txt", suggested_action: "Delete" }),
    ];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 1,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("file.txt")).toBeInTheDocument();
    });

    const checkbox = screen.getByRole("checkbox");
    expect(checkbox).toBeChecked();

    const user = userEvent.setup();
    // Uncheck
    await user.click(checkbox);
    await waitFor(() => {
      expect(checkbox).not.toBeChecked();
    });

    // Check again
    await user.click(checkbox);
    await waitFor(() => {
      expect(checkbox).toBeChecked();
    });
  });

  it("select all page and deselect all buttons work", async () => {
    const items = [
      makeItem({ id: "1", name: "a.txt" }),
      makeItem({ id: "2", name: "b.txt" }),
    ];
    mockGetScanResults.mockResolvedValue({
      items,
      total: 2,
      page: 1,
      page_size: 50,
    });
    renderWithProvider(<ResultsPage />);
    await waitFor(() => {
      expect(screen.getByText("a.txt")).toBeInTheDocument();
    });

    const user = userEvent.setup();
    const checkboxes = screen.getAllByRole("checkbox");
    // Uncheck one first
    await user.click(checkboxes[0]);
    await waitFor(() => expect(checkboxes[0]).not.toBeChecked());

    // Select all page
    await user.click(screen.getByRole("button", { name: /全选本页/i }));
    await waitFor(() => {
      expect(checkboxes[0]).toBeChecked();
      expect(checkboxes[1]).toBeChecked();
    });

    // Deselect all
    await user.click(screen.getByRole("button", { name: /取消全选/i }));
    await waitFor(() => {
      expect(checkboxes[0]).not.toBeChecked();
      expect(checkboxes[1]).not.toBeChecked();
    });
  });
});
