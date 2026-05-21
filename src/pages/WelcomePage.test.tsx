import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WelcomePage } from "./WelcomePage";
import { AppProvider, useAppState } from "../store/AppContext";

function CurrentPageIndicator() {
  const { state } = useAppState();
  return <div data-testid="current-page">{state.page}</div>;
}

function renderWithProvider(ui: React.ReactElement) {
  return render(<AppProvider>{ui}</AppProvider>);
}

describe("WelcomePage", () => {
  it("renders subtitle", () => {
    renderWithProvider(<WelcomePage />);
    expect(
      screen.getByText(/在撤离公用电脑前，安全处理您留下的痕迹/i),
    ).toBeInTheDocument();
  });

  it("renders start button", () => {
    renderWithProvider(<WelcomePage />);
    expect(
      screen.getByRole("button", { name: /开始使用/i }),
    ).toBeInTheDocument();
  });

  it("navigates to input page when start button is clicked", async () => {
    renderWithProvider(
      <>
        <WelcomePage />
        <CurrentPageIndicator />
      </>,
    );

    const user = userEvent.setup();
    const startBtn = screen.getByRole("button", { name: /开始使用/i });
    await user.click(startBtn);

    expect(screen.getByTestId("current-page")).toHaveTextContent("input");
  });
});
