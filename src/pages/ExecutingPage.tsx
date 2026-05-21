import { useEffect, useRef } from "react";
import { useAppState } from "../store/AppContext";
import { startExecution, listenScanProgress } from "../api/commands";
import type { ExecutionReport } from "../types";

const MOCK_REPORT: ExecutionReport = {
  deleted_count: 12,
  deleted_bytes: 1024 * 1024 * 128,
  packed_count: 5,
  packed_bytes: 1024 * 1024 * 64,
  preserved_count: 3,
  pack_file_path: "C:\\Users\\Admin\\Desktop\\French-exit.zip",
  items: [],
};

export function ExecutingPage() {
  const { state, dispatch } = useAppState();
  const hasStarted = useRef(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  // 显示进度：保证只增不减，防止后端进度回跳导致进度条横跳
  const displayPercentRef = useRef(0);

  // 启动执行 + 监听进度
  useEffect(() => {
    if (hasStarted.current) return;
    hasStarted.current = true;

    // 重置进度显示
    displayPercentRef.current = 0;
    dispatch({ type: "SET_PROGRESS", payload: { message: "准备执行…", percent: 0 } });

    // 纯前端预览模式：模拟执行进度
    if (!window.__TAURI_INTERNALS__) {
      let current = 0;
      const interval = setInterval(() => {
        current += Math.floor(Math.random() * 15) + 5;
        if (current > 100) current = 100;
        dispatch({
          type: "SET_PROGRESS",
          payload: { message: `正在清理… ${current}%`, percent: current },
        });
        if (current >= 100) {
          clearInterval(interval);
          dispatch({ type: "SET_REPORT", payload: MOCK_REPORT });
          dispatch({ type: "SET_PAGE", payload: "report" });
        }
      }, 300);
      return () => clearInterval(interval);
    }

    // 真实 Tauri 模式
    listenScanProgress((event) => {
      switch (event.type) {
        case "ExecutionStarted":
          dispatch({
            type: "SET_PROGRESS",
            payload: { message: "开始执行清理…", percent: 5 },
          });
          break;
        case "ExecutionProgress":
          dispatch({
            type: "SET_PROGRESS",
            payload: {
              message: event.message || "正在处理…",
              percent:
                event.total > 0
                  ? Math.round((event.current / event.total) * 100)
                  : displayPercentRef.current,
            },
          });
          break;
        case "ExecutionCompleted":
          dispatch({ type: "SET_REPORT", payload: event.report });
          dispatch({ type: "SET_PAGE", payload: "report" });
          break;
      }
    }).then((fn) => {
      unlistenRef.current = fn;
    });

    startExecution().catch((e: any) => {
      dispatch({ type: "SET_ERROR", payload: e.message || "执行失败" });
      dispatch({ type: "SET_PAGE", payload: "confirm" });
    });
  }, [dispatch]);

  // 清理监听器
  useEffect(() => {
    return () => {
      if (unlistenRef.current) unlistenRef.current();
    };
  }, []);

  return (
    <div className="flex flex-col items-center justify-center min-h-[70vh] max-w-lg mx-auto">
      <div className="w-full bg-card/80 backdrop-blur-xl rounded-2xl p-8 shadow-lg border border-border text-center">
        <h2 className="text-xl font-semibold mb-6">正在执行清理...</h2>

        {/* 进度条 */}
        <div className="w-full bg-muted h-0.5 mb-6 overflow-hidden">
          <div
            className="bg-blue-600 h-full transition-all duration-300 ease-out"
            style={{ width: `${displayPercentRef.current}%` }}
          />
        </div>

        <p className="text-sm text-muted-foreground mb-2">
          {state.progressMessage || "初始化执行器…"}
        </p>
        <p className="text-xs text-muted-foreground">
          此过程可能需要几分钟，取决于文件数量
        </p>

        {state.error && (
          <p className="mt-4 text-sm">{state.error}</p>
        )}
      </div>
    </div>
  );
}
