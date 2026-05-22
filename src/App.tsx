/**
 * 应用根组件
 *
 * 职责：
 * 1. 提供全局状态 Provider
 * 2. 监听系统深色/浅色模式
 * 3. 根据当前 page 状态渲染对应页面
 * 4. 初始化时加载资源配置
 */
import { useEffect } from "react";
import { AppProvider, useAppState } from "./store/AppContext";
import { getResourceConfig } from "./api/commands";
import { WelcomePage } from "./pages/WelcomePage";
import { InputPage } from "./pages/InputPage";
import { ScanPage } from "./pages/ScanPage";
import { ResultsPage } from "./pages/ResultsPage";
import { ConfirmPage } from "./pages/ConfirmPage";
import { ExecutingPage } from "./pages/ExecutingPage";
import { ReportPage } from "./pages/ReportPage";
import type { ExecutionReport } from "./types";

const MOCK_REPORT: ExecutionReport = {
  deleted_count: 12,
  deleted_bytes: 1024 * 1024 * 128,
  packed_count: 5,
  packed_bytes: 1024 * 1024 * 64,
  preserved_count: 3,
  pack_file_path: "C:\\Users\\Admin\\Desktop\\French-exit.zip",
  items: [],
};

const PAGES: { key: string; label: string }[] = [
  { key: "welcome", label: "欢迎" },
  { key: "input", label: "输入" },
  { key: "scanning", label: "扫描" },
  { key: "results", label: "结果" },
  { key: "confirm", label: "确认" },
  { key: "executing", label: "执行" },
  { key: "report", label: "报告" },
];

function AppContent() {
  const { state, dispatch } = useAppState();

  /**
   * 默认深色模式，以黑色为底色
   * 仍监听系统主题变化，但默认保持 dark（用户可后续扩展手动切换）
   */
  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  // 初始化：加载资源配置
  useEffect(() => {
    getResourceConfig()
      .then((config) => dispatch({ type: "SET_RESOURCE_CONFIG", payload: config }))
      .catch(() => {/* 忽略 */});
  }, [dispatch]);

  return (
    <div className="min-h-screen bg-background text-foreground transition-colors duration-300">
      {/* 开发者导航：仅在非 Tauri 环境（纯前端预览）显示 */}
      {!window.__TAURI_INTERNALS__ && (
        <div className="border-b border-border/50 bg-card/50 backdrop-blur-sm">
          <div className="container mx-auto px-4 py-2 flex flex-wrap items-center gap-2 text-xs">
            <span className="text-muted-foreground font-medium">调试导航:</span>
            {PAGES.map((p) => (
              <button
                key={p.key}
                onClick={() => {
                  if (p.key === "report" && !state.report) {
                    dispatch({ type: "SET_REPORT", payload: MOCK_REPORT });
                  }
                  dispatch({ type: "SET_PAGE", payload: p.key as any });
                }}
                className={`px-2.5 py-1 rounded-md transition ${
                  state.page === p.key
                    ? "bg-blue-600 text-white"
                    : "bg-muted text-muted-foreground hover:bg-muted/80"
                }`}
              >
                {p.label}
              </button>
            ))}
            <button
              onClick={() => dispatch({ type: "RESET" })}
              className="px-2.5 py-1 rounded-md bg-red-600/20 text-red-400 hover:bg-red-600/30 transition ml-auto"
            >
              重置
            </button>
          </div>
        </div>
      )}
      {/* 大 Logo：欢迎页居中，离开时向上移动并消失 */}
      <div
        className={`
          fixed z-50 font-semibold tracking-tight text-foreground select-none
          transition-all duration-500 ease-out pointer-events-none
          ${state.page === "welcome"
            ? "top-[32%] left-1/2 -translate-x-1/2 text-5xl opacity-100"
            : "top-0 left-1/2 -translate-x-1/2 text-3xl opacity-0"
          }
        `}
      >
        French Exit
      </div>

      {/* 小 Logo：常驻左上角，欢迎页时隐藏 */}
      <button
        onClick={() => {
          if (state.page !== "welcome") {
            dispatch({ type: "SET_PAGE", payload: "welcome" });
          }
        }}
        className={`
          fixed z-50 font-semibold tracking-tight text-foreground select-none
          transition-opacity duration-500 ease-out
          ${state.page === "welcome"
            ? "opacity-0 pointer-events-none"
            : "opacity-100"
          }
          top-4 left-4 text-xl cursor-pointer hover:opacity-80
        `}
      >
        French Exit
      </button>

      <main className={`container mx-auto px-4 ${state.page === "welcome" ? "py-8" : "pt-16 pb-8"}`}>
        {state.page === "welcome" && <WelcomePage />}
        {state.page === "input" && <InputPage />}
        {state.page === "scanning" && <ScanPage />}
        {state.page === "results" && <ResultsPage />}
        {state.page === "confirm" && <ConfirmPage />}
        {state.page === "executing" && <ExecutingPage />}
        {state.page === "report" && <ReportPage />}
      </main>
    </div>
  );
}

function App() {
  return (
    <AppProvider>
      <AppContent />
    </AppProvider>
  );
}

export default App;
