/**
 * 输入页（InputPage）
 *
 * 用户进入应用后的首个页面，负责收集入职日期并启动扫描。
 * 设计遵循 Apple Design：大圆角、留白、毛玻璃质感、清晰的视觉层级。
 */
import { useState, useCallback } from "react";
import { useAppState } from "../store/AppContext";
import { startScan } from "../api/commands";

export function InputPage() {
  const { state, dispatch } = useAppState();
  const [isLoading, setIsLoading] = useState(false);

  /** 今天的日期，用于限制日期选择器的最大值 */
  const today = new Date().toISOString().split("T")[0];

  /** 日期输入变化 */
  const handleDateChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      dispatch({ type: "SET_START_DATE", payload: e.target.value });
      if (state.error) {
        dispatch({ type: "SET_ERROR", payload: null });
      }
    },
    [dispatch, state.error],
  );

  /** 点击开始扫描 */
  const handleStart = useCallback(async () => {
    if (!state.startDate) {
      dispatch({ type: "SET_ERROR", payload: "请选择入职日期" });
      return;
    }

    setIsLoading(true);
    dispatch({ type: "SET_ERROR", payload: null });

    try {
      const scanId = await startScan(state.startDate);
      dispatch({ type: "SET_SCAN_ID", payload: scanId });
      dispatch({ type: "SET_SCANNING", payload: true });
      dispatch({ type: "SET_PAGE", payload: "scanning" });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "启动扫描失败，请稍后重试";
      dispatch({ type: "SET_ERROR", payload: message });
    } finally {
      setIsLoading(false);
    }
  }, [state.startDate, dispatch]);

  const canStart = Boolean(state.startDate) && !isLoading;

  return (
    <div className="flex flex-col items-center justify-center min-h-[80vh]">
      <div className="w-full max-w-lg">
        {/* 标题区域 */}
        <div className="text-center mb-10">
          <h1 className="text-4xl font-semibold tracking-tight text-foreground mb-3">
            French Exit
          </h1>
          <p className="text-lg text-muted-foreground">
            离职前，优雅地清理个人痕迹
          </p>
        </div>

        {/* 输入卡片 */}
        <div className="rounded-2xl border border-border bg-card/60 backdrop-blur-md p-8 shadow-sm">
          {/* 日期输入 */}
          <div className="mb-6">
            <label
              htmlFor="start-date"
              className="block text-sm font-medium text-foreground mb-2"
            >
              你的入职日期
            </label>
            <input
              id="start-date"
              type="date"
              value={state.startDate}
              onChange={handleDateChange}
              max={today}
              className="w-full rounded-xl border border-input bg-white/50 dark:bg-black/30 
                         px-4 py-3 text-foreground outline-none
                         focus:ring-2 focus:ring-ring focus:border-transparent
                         transition-colors duration-200"
            />
            <p className="mt-1.5 text-xs text-muted-foreground">
              系统将扫描该日期之后产生的个人文件与痕迹
            </p>
          </div>

          {/* CPU 限制说明 */}
          <div className="mb-8 flex items-start gap-2">
            <svg
              className="w-4 h-4 text-muted-foreground mt-0.5 shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p className="text-xs text-muted-foreground leading-relaxed">
              程序默认限制 CPU 使用率 ≤30%，保证办公不卡顿
            </p>
          </div>

          {/* 开始按钮 */}
          <button
            onClick={handleStart}
            disabled={!canStart}
            className={`
              w-full rounded-xl px-8 py-3 font-medium text-white
              transition-all duration-200
              ${
                canStart
                  ? "bg-blue-600 hover:bg-blue-700 active:scale-95 shadow-md hover:shadow-lg"
                  : "bg-blue-600/50 opacity-50 cursor-not-allowed"
              }
            `}
          >
            {isLoading ? "正在启动…" : "开始扫描"}
          </button>

          {/* 错误提示 */}
          {state.error && (
            <p className="mt-4 text-sm text-red-500 text-center animate-pulse">
              {state.error}
            </p>
          )}
        </div>

        {/* 底部说明 */}
        <p className="mt-6 text-center text-xs text-muted-foreground">
          所有操作在本地完成，不会上传任何数据
        </p>
      </div>
    </div>
  );
}
