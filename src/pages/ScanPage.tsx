/**
 * 扫描进度页面
 *
 * 职责：
 * 1. 展示扫描进度条和状态文案
 * 2. 提供暂停 / 恢复 / 取消操作
 * 3. 轮询 session state，扫描完成后自动跳转到 results
 */
import { useEffect, useRef, useCallback } from "react";
import { useAppState } from "../store/AppContext";
import { getSessionState, pauseScan, resumeScan, listenScanProgress, setResourceConfig } from "../api/commands";

export function ScanPage() {
  const { state, dispatch } = useAppState();
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const progressPercentRef = useRef(state.progressPercent);
  progressPercentRef.current = state.progressPercent;

  // 显示进度：保证只增不减，防止后端进度回跳导致进度条横跳
  const displayPercentRef = useRef(0);

  // 进入扫描页时强制重置进度，避免第二次扫描残留上次进度
  useEffect(() => {
    dispatch({ type: "SET_PROGRESS", payload: { message: "准备扫描…", percent: 0 } });
  }, [dispatch]);

  // 组件挂载时重置显示进度
  useEffect(() => {
    displayPercentRef.current = 0;
  }, []);

  // 每次渲染时，仅当新进度大于当前显示进度才更新
  if (state.progressPercent > displayPercentRef.current) {
    displayPercentRef.current = state.progressPercent;
  }

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
                  : displayPercentRef.current,
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
  }, [dispatch]);

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

  const handleToggleResource = useCallback(async () => {
    const next = {
      ...state.resourceConfig,
      unlimited: !state.resourceConfig.unlimited,
    };
    try {
      await setResourceConfig(next);
      dispatch({ type: "SET_RESOURCE_CONFIG", payload: next });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "设置资源限制失败，请稍后重试";
      dispatch({ type: "SET_ERROR", payload: message });
    }
  }, [state.resourceConfig, dispatch]);

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
        <div className="w-full bg-muted h-0.5 mb-6 overflow-hidden">
          <div
            className="bg-blue-600 h-full transition-all duration-300 ease-out"
            style={{ width: `${displayPercentRef.current}%` }}
          />
        </div>

        <p className="text-sm text-muted-foreground text-center mb-8">
          {state.progressMessage || "初始化扫描器…"}
        </p>

        {/* CPU 限速开关 */}
        <div className="flex items-center justify-between mb-5 rounded-xl border border-border bg-card/40 p-4">
          <div className="flex-1 pr-4">
            <p className="text-sm font-medium text-foreground">
              {state.resourceConfig.unlimited
                ? "不限速全量运行"
                : "智能限速模式"}
            </p>
            <p className="text-xs text-muted-foreground mt-0.5">
              {state.resourceConfig.unlimited
                ? "使用全部 CPU 性能，可能略微影响其他程序"
                : "CPU 限制在 30% 以下，不影响您的正常使用"}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={!state.resourceConfig.unlimited}
            onClick={handleToggleResource}
            className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 ${
              !state.resourceConfig.unlimited
                ? "bg-blue-600"
                : "bg-muted-foreground/30"
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200 ${
                !state.resourceConfig.unlimited
                  ? "translate-x-6"
                  : "translate-x-1"
              }`}
            />
          </button>
        </div>

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
          <p className="mt-4 text-sm text-center">{state.error}</p>
        )}
      </div>
    </div>
  );
}
