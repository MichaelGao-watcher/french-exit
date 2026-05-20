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
import { InputPage } from "./pages/InputPage";
import { ScanPage } from "./pages/ScanPage";
import { ResultsPage } from "./pages/ResultsPage";
import { ConfirmPage } from "./pages/ConfirmPage";
import { ExecutingPage } from "./pages/ExecutingPage";
import { ReportPage } from "./pages/ReportPage";

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
      <main className="container mx-auto px-4 py-8">
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
