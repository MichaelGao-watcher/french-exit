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
import { motion, AnimatePresence } from "framer-motion";
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

const pageVariants = {
  initial: {
    opacity: 0,
    y: 8,
  },
  animate: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.4,
      ease: [0.4, 0, 0.2, 1],
    },
  },
  exit: {
    opacity: 0,
    y: -8,
    transition: {
      duration: 0.2,
      ease: [0.4, 0, 0.2, 1],
    },
  },
};

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

  const renderPage = () => {
    switch (state.page) {
      case "welcome":
        return <WelcomePage />;
      case "input":
        return <InputPage />;
      case "scanning":
        return <ScanPage />;
      case "results":
        return <ResultsPage />;
      case "confirm":
        return <ConfirmPage />;
      case "executing":
        return <ExecutingPage />;
      case "report":
        return <ReportPage />;
      default:
        return <WelcomePage />;
    }
  };

  return (
    <div className="min-h-screen bg-background text-foreground">
      {/* 开发者导航：仅在非 Tauri 环境（纯前端预览）显示 */}
      {!window.__TAURI_INTERNALS__ && (
        <div className="border-b border-white/10 bg-black/50 backdrop-blur-sm">
          <div className="container mx-auto px-4 py-2 flex flex-wrap items-center gap-2 text-xs">
            <span className="text-muted-foreground font-light">调试导航:</span>
            {PAGES.map((p) => (
              <button
                key={p.key}
                onClick={() => {
                  if (p.key === "report" && !state.report) {
                    dispatch({ type: "SET_REPORT", payload: MOCK_REPORT });
                  }
                  dispatch({ type: "SET_PAGE", payload: p.key as any });
                }}
                className={`px-2.5 py-1 rounded-md transition-all duration-200 ${
                  state.page === p.key
                    ? "bg-white text-black"
                    : "text-muted-foreground hover:bg-white/10"
                }`}
              >
                {p.label}
              </button>
            ))}
            <button
              onClick={() => dispatch({ type: "RESET" })}
              className="px-2.5 py-1 rounded-md text-white/40 hover:text-white/60 hover:bg-white/5 transition-all duration-200 ml-auto"
            >
              重置
            </button>
          </div>
        </div>
      )}

      {/* 大 Logo：欢迎页居中，离开时向上移动并消失 */}
      <motion.div
        className="fixed z-50 font-extralight tracking-tight text-foreground select-none pointer-events-none"
        initial={{ opacity: 0, y: 12, left: "50%", top: "32%", x: "-50%", fontSize: "3rem" }}
        animate={{
          left: "50%",
          top: state.page === "welcome" ? "32%" : "0%",
          x: "-50%",
          y: state.page === "welcome" ? 0 : -12,
          fontSize: state.page === "welcome" ? "3rem" : "1.5rem",
          opacity: state.page === "welcome" ? 1 : 0,
        }}
        transition={{ duration: 1, ease: [0.4, 0, 0.2, 1] }}
      >
        French Exit
      </motion.div>

      {/* 小 Logo：常驻左上角，欢迎页时隐藏 */}
      <motion.button
        onClick={() => {
          if (state.page !== "welcome") {
            dispatch({ type: "SET_PAGE", payload: "welcome" });
          }
        }}
        className="fixed z-50 font-extralight tracking-tight text-foreground select-none top-4 left-4 text-xl cursor-pointer"
        animate={{
          opacity: state.page === "welcome" ? 0 : 1,
          pointerEvents: state.page === "welcome" ? "none" : "auto",
        }}
        transition={{ duration: 0.5, ease: [0.4, 0, 0.2, 1] }}
        whileHover={{ opacity: 0.8 }}
      >
        French Exit
      </motion.button>

      <main className={`container mx-auto px-4 ${state.page === "welcome" ? "py-8" : "pt-16 pb-8"}`}>
        <AnimatePresence mode="wait">
          <motion.div
            key={state.page}
            variants={pageVariants}
            initial="initial"
            animate="animate"
            exit="exit"
          >
            {renderPage()}
          </motion.div>
        </AnimatePresence>
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
