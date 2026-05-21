import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InputPage } from "./InputPage";
import { AppProvider, TestAppProvider } from "../store/AppContext";

vi.mock("../api/commands", () => ({
  startScan: vi.fn(),
  setResourceConfig: vi.fn(),
}));

import { startScan, setResourceConfig } from "../api/commands";
const mockStartScan = vi.mocked(startScan);
const mockSetResourceConfig = vi.mocked(setResourceConfig);

function renderWithProvider(ui: React.ReactElement) {
  return render(<AppProvider>{ui}</AppProvider>);
}

function renderWithState(
  ui: React.ReactElement,
  state: Record<string, unknown>,
) {
  return render(<TestAppProvider initialState={state}>{ui}</TestAppProvider>);
}

describe("InputPage", () => {
  beforeEach(() => {
    mockStartScan.mockClear();
    mockSetResourceConfig.mockClear();
  });

  it("renders title and date picker", () => {
    renderWithProvider(<InputPage />);
    expect(screen.getByText(/French Exit/i)).toBeInTheDocument();
    expect(
      screen.getByText(/在撤离公用电脑前，安全处理您留下的痕迹/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/您开始使用这台电脑的时间/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /开始扫描/i }),
    ).toBeInTheDocument();
  });

  it("disables start button when no date is selected", () => {
    renderWithProvider(<InputPage />);
    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    expect(startBtn).toBeDisabled();
  });

  it("allows scanning with only year selected", async () => {
    renderWithProvider(<InputPage />);
    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    expect(startBtn).toBeDisabled();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(screen.getByRole("button", { name: /2022年/i }));

    await waitFor(() => {
      expect(startBtn).not.toBeDisabled();
    });
  });

  it("allows scanning with year and month selected", async () => {
    renderWithProvider(<InputPage />);
    const startBtn = screen.getByRole("button", { name: /开始扫描/i });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(screen.getByRole("button", { name: /2022年/i }));
    await user.click(screen.getByRole("button", { name: /月/i }));
    await user.click(screen.getByRole("button", { name: /06月/i }));

    await waitFor(() => {
      expect(startBtn).not.toBeDisabled();
    });
  });

  it("allows scanning with full date selected", async () => {
    renderWithProvider(<InputPage />);
    const startBtn = screen.getByRole("button", { name: /开始扫描/i });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(screen.getByRole("button", { name: /2022年/i }));
    await user.click(screen.getByRole("button", { name: /月/i }));
    await user.click(screen.getByRole("button", { name: /06月/i }));
    await user.click(screen.getByRole("button", { name: /日/i }));
    await user.click(screen.getByRole("button", { name: /15日/i }));

    await waitFor(() => {
      expect(startBtn).not.toBeDisabled();
    });
  });

  it("normalizes partial dates before calling startScan", async () => {
    mockStartScan.mockResolvedValue("scan-123");
    renderWithProvider(<InputPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(screen.getByRole("button", { name: /2022年/i }));
    await user.click(screen.getByRole("button", { name: /月/i }));
    await user.click(screen.getByRole("button", { name: /06月/i }));

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    await user.click(startBtn);

    await waitFor(() => {
      expect(mockStartScan).toHaveBeenCalledWith("2022-06-01");
    });
  });

  it("calls startScan with full date when all fields selected", async () => {
    mockStartScan.mockResolvedValue("scan-123");
    renderWithProvider(<InputPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(screen.getByRole("button", { name: /2022年/i }));
    await user.click(screen.getByRole("button", { name: /月/i }));
    await user.click(screen.getByRole("button", { name: /06月/i }));
    await user.click(screen.getByRole("button", { name: /日/i }));
    await user.click(screen.getByRole("button", { name: /15日/i }));

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    await user.click(startBtn);

    await waitFor(() => {
      expect(mockStartScan).toHaveBeenCalledWith("2022-06-15");
    });
  });

  it("shows error message when startScan fails", async () => {
    mockStartScan.mockRejectedValue(new Error("扫描服务未启动"));
    renderWithProvider(<InputPage />);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(screen.getByRole("button", { name: /2022年/i }));

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    await user.click(startBtn);

    await waitFor(() => {
      expect(screen.getByText(/扫描服务未启动/i)).toBeInTheDocument();
    });
  });

  it("does not offer future years in the date picker", async () => {
    renderWithProvider(<InputPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /年/i }));
    const currentYear = String(new Date().getFullYear());
    expect(
      screen.getByRole("button", { name: new RegExp(`^${currentYear}年$`) }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", {
        name: new RegExp(`^${Number(currentYear) + 1}年$`),
      }),
    ).not.toBeInTheDocument();
  });

  it("limits months to current month when current year is selected", async () => {
    renderWithProvider(<InputPage />);
    const user = userEvent.setup();
    const today = new Date();
    const currentYear = String(today.getFullYear());
    const currentMonth = today.getMonth() + 1;

    // 选今年
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(
      screen.getByRole("button", { name: new RegExp(`^${currentYear}年$`) }),
    );

    // 打开月份面板
    await user.click(screen.getByRole("button", { name: /月/i }));

    // 当前月应该存在
    expect(
      screen.getByRole("button", {
        name: new RegExp(`^${String(currentMonth).padStart(2, "0")}月$`),
      }),
    ).toBeInTheDocument();

    // 下个月不应该存在
    expect(
      screen.queryByRole("button", {
        name: new RegExp(
          `^${String(currentMonth + 1).padStart(2, "0")}月$`,
        ),
      }),
    ).not.toBeInTheDocument();
  });

  it("limits days to today when current year and month are selected", async () => {
    renderWithProvider(<InputPage />);
    const user = userEvent.setup();
    const today = new Date();
    const currentYear = String(today.getFullYear());
    const currentMonth = String(today.getMonth() + 1).padStart(2, "0");
    const currentDay = today.getDate();

    // 选今年当月
    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(
      screen.getByRole("button", { name: new RegExp(`^${currentYear}年$`) }),
    );
    await user.click(screen.getByRole("button", { name: /月/i }));
    await user.click(
      screen.getByRole("button", { name: new RegExp(`^${currentMonth}月$`) }),
    );

    // 打开日期面板
    await user.click(screen.getByRole("button", { name: /日/i }));

    // 今天应该存在
    expect(
      screen.getByRole("button", {
        name: new RegExp(`^${String(currentDay).padStart(2, "0")}日$`),
      }),
    ).toBeInTheDocument();

    // 明天不应该存在
    expect(
      screen.queryByRole("button", {
        name: new RegExp(
          `^${String(currentDay + 1).padStart(2, "0")}日$`,
        ),
      }),
    ).not.toBeInTheDocument();
  });

  it("blocks future dates when user bypasses UI and shows error", async () => {
    const nextYear = String(new Date().getFullYear() + 1);
    renderWithState(<InputPage />, { startDate: `${nextYear}-01-01` });
    const user = userEvent.setup();

    expect(
      screen.getByText(new RegExp(`${nextYear}年`)),
    ).toBeInTheDocument();

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    expect(startBtn).not.toBeDisabled();

    await user.click(startBtn);

    await waitFor(() => {
      expect(
        screen.getByText(/时间不能是未来日期/i),
      ).toBeInTheDocument();
    });
    expect(mockStartScan).not.toHaveBeenCalled();
  });

  it("allows today as the latest valid date", async () => {
    mockStartScan.mockResolvedValue("scan-123");
    renderWithProvider(<InputPage />);
    const user = userEvent.setup();

    const today = new Date();
    const y = String(today.getFullYear());
    const m = String(today.getMonth() + 1).padStart(2, "0");
    const d = String(today.getDate()).padStart(2, "0");

    await user.click(screen.getByRole("button", { name: /年/i }));
    await user.click(
      screen.getByRole("button", { name: new RegExp(`^${y}年$`) }),
    );
    await user.click(screen.getByRole("button", { name: /月/i }));
    await user.click(
      screen.getByRole("button", { name: new RegExp(`^${m}月$`) }),
    );
    await user.click(screen.getByRole("button", { name: /日/i }));
    await user.click(
      screen.getByRole("button", { name: new RegExp(`^${d}日$`) }),
    );

    const startBtn = screen.getByRole("button", { name: /开始扫描/i });
    await user.click(startBtn);

    await waitFor(() => {
      const errorEl = screen.queryByText(/时间不能是未来日期/i);
      expect(errorEl).not.toBeInTheDocument();
    });
    expect(mockStartScan).toHaveBeenCalledWith(`${y}-${m}-${d}`);
  });

});
