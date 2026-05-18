/**
 * 扫描进度页面
 *
 * 职责：
 * 1. 展示扫描进度条和状态文案
 * 2. 提供暂停 / 恢复 / 取消操作
 * 3. 轮询 session state，扫描完成后自动跳转到 results
 */
import { useEffect, useRef } from "react";
import { useAppState } from "../store/AppContext";
import { getSessionState, pauseScan, resumeScan, listenScanProgress } from "../api/commands";

export function ScanPage() {
  const { state, dispatch } = useAppState();
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  // 监听 Tauri Event 实时进度推送（替代轮询的主方案）
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listenScanProgress((event) => {
      switch (event.type) {
        case "ScanStarted":
          dispatch({
            type: "SET_PROGRESS",
            payload: { message: "扫描开始…", percent: 5 },
          });
          break;
        case "ScanProgress":
          dispatch({
            type: "SET_PROGRESS",
            payload: {
              message: event.message || "正在扫描…",
              percent:
                event.total > 0
                  ? Math.round((event.current / event.total) * 100)
                  : state.progressPercent,
            },
          });
          break;
        case "ScanCompleted":
          dispatch({ type: "SET_SCANNING", payload: false });
          dispatch({ type: "SET_PAGE", payload: "results" });
          if (intervalRef.current) clearInterval(intervalRef.current);
          break;
        case "ScanFailed":
          dispatch({ type: "SET_ERROR", payload: event.reason || "扫描失败" });
          dispatch({ type: "SET_SCANNING", payload: false });
          if (intervalRef.current) clearInterval(intervalRef.current);
          break;
        case "ScanPaused":
          dispatch({ type: "SET_PAUSED", payload: true });
          break;
        case "ScanResumed":
          dispatch({ type: "SET_PAUSED", payload: false });
          break;
      }
    }).then((fn) => {
      unlisten = fn;
      unlistenRef.current = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [dispatch, state.progressPercent]);

  // 轮询 session state，作为 fallback 兜底
  useEffect(() => {
    intervalRef.current = setInterval(async () => {
      try {
        const sessionState = await getSessionState();
        // sessionState 是 Rust 枚举的序列化形式：{"Scanned":{...}} 或 "Idle" 等
        if (typeof sessionState === "object" && sessionState !== null) {
          const stateType = Object.keys(sessionState)[0];
          if (stateType === "Scanned") {
            dispatch({ type: "SET_SCANNING", payload: false });
            dispatch({ type: "SET_PAGE", payload: "results" });
            if (intervalRef.current) clearInterval(intervalRef.current);
          } else if (stateType === "Failed") {
            const reason =
              (sessionState as Record<string, any>).Failed?.reason || "扫描失败";
            dispatch({ type: "SET_ERROR", payload: reason });
            dispatch({ type: "SET_SCANNING", payload: false });
            if (intervalRef.current) clearInterval(intervalRef.current);
          } else if (stateType === "Paused") {
            dispatch({ type: "SET_PAUSED", payload: true });
          } else if (stateType === "Scanning") {
            dispatch({ type: "SET_PAUSED", payload: false });
          }
        } else if (sessionState === "Idle") {
          // 扫描被重置或尚未开始
          dispatch({ type: "SET_SCANNING", payload: false });
          if (intervalRef.current) clearInterval(intervalRef.current);
        }
      } catch {
        // 轮询错误忽略，继续轮询
      }
    }, 1000);

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [dispatch]);

  const handlePause = async () => {
    try {
      await pauseScan();
      dispatch({ type: "SET_PAUSED", payload: true });
    } catch (e: any) {
      dispatch({ type: "SET_ERROR", payload: e.message || "暂停失败" });
    }
  };

  const handleResume = async () => {
    try {
      await resumeScan();
      dispatch({ type: "SET_PAUSED", payload: false });
    } catch (e: any) {
      dispatch({ type: "SET_ERROR", payload: e.message || "恢复失败" });
    }
  };

  const handleCancel = () => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    dispatch({ type: "RESET" });
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-[70vh] max-w-lg mx-auto">
      <div className="w-full bg-card/80 backdrop-blur-xl rounded-2xl p-8 shadow-lg border border-border">
        <h2 className="text-2xl font-semibold mb-2 text-center">
          {state.isPaused ? "扫描已暂停" : "正在扫描…"}
        </h2>
        <p className="text-muted-foreground text-center mb-6">
          {state.isPaused
            ? "点击恢复继续扫描"
            : "正在检查您的个人痕迹，请稍候"}
        </p>

        {/* 进度条 */}
        <div className="w-full bg-muted rounded-full h-3 mb-6 overflow-hidden">
          <div
            className="bg-blue-600 h-full rounded-full transition-all duration-500"
            style={{ width: `${state.progressPercent}%` }}
          />
        </div>

        <p className="text-sm text-muted-foreground text-center mb-8">
          {state.progressMessage || "初始化扫描器…"}
        </p>

        {/* 操作按钮 */}
        <div className="flex gap-3 justify-center">
          {state.isPaused ? (
            <button
              onClick={handleResume}
              className="px-6 py-2.5 bg-blue-600 text-white rounded-xl font-medium hover:bg-blue-700 active:scale-95 transition"
            >
              恢复扫描
            </button>
          ) : (
            <button
              onClick={handlePause}
              className="px-6 py-2.5 bg-secondary text-secondary-foreground rounded-xl font-medium hover:bg-secondary/80 active:scale-95 transition"
            >
              暂停
            </button>
          )}
          <button
            onClick={handleCancel}
            className="px-6 py-2.5 border border-border text-foreground rounded-xl font-medium hover:bg-muted active:scale-95 transition"
          >
            取消
          </button>
        </div>

        {state.error && (
          <p className="mt-4 text-sm text-red-500 text-center">{state.error}</p>
        )}
      </div>
    </div>
  );
}
