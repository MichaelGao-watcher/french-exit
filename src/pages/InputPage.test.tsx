import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InputPage } from "./InputPage";
import { AppProvider } from "../store/AppContext";

vi.mock("../api/commands", () => ({
  startScan: vi.fn(),
}));

import { startScan } from "../api/commands";
const mockStartScan = vi.mocked(startScan);

function renderWithProvider(ui: React.ReactElement) {
  return render(<AppProvider>{ui}</AppProvider>);
}

describe("InputPage", () => {
  beforeEach(() => {
    mockStartScan.mockClear();
  });

  it("renders title and date input", () => {
    renderWithProvider(<InputPage />);
    expect(screen.getByText(/French Exit/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/入职日期/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /开始扫描/i })).toBeInTheDocument();
  });

  it("disables start button when no date is selected", () => {
    renderWithProvider(<InputPage />);
    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    expect(startBtn).toBeDisabled();
  });

  it("selecting a date enables the start button", async () => {
    renderWithProvider(<InputPage />);
    const dateInput = screen.getByLabelText(/入职日期/i) as HTMLInputElement;
    const startBtn = screen.getByRole("button", { name: /开始扫描/i });

    expect(startBtn).toBeDisabled();

    const user = userEvent.setup();
    await user.type(dateInput, "2024-01-15");

    await waitFor(() => {
      expect(startBtn).not.toBeDisabled();
    });
  });

  it("calls startScan and navigates to scanning on success", async () => {
    mockStartScan.mockResolvedValue("scan-123");
    renderWithProvider(<InputPage />);

    const user = userEvent.setup();
    const dateInput = screen.getByLabelText(/入职日期/i);
    await user.type(dateInput, "2024-01-15");

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    await user.click(startBtn);

    await waitFor(() => {
      expect(mockStartScan).toHaveBeenCalledWith("2024-01-15");
    });
  });

  it("shows error message when startScan fails", async () => {
    mockStartScan.mockRejectedValue(new Error("扫描服务未启动"));
    renderWithProvider(<InputPage />);

    const user = userEvent.setup();
    const dateInput = screen.getByLabelText(/入职日期/i);
    await user.type(dateInput, "2024-01-15");

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    await user.click(startBtn);

    await waitFor(() => {
      expect(screen.getByText(/扫描服务未启动/i)).toBeInTheDocument();
    });
  });
});
