/**
 * 输入页（InputPage）
 *
 * 用户进入应用后的首个页面，负责收集入职日期并启动扫描。
 * 设计遵循 Apple Design：大圆角、留白、毛玻璃质感、清晰的视觉层级。
 */
import { useState, useCallback } from "react";
import { useAppState } from "../store/AppContext";
import { startScan, setResourceConfig } from "../api/commands";
import { DatePicker } from "../components/DatePicker";

/** 将用户选择的局部日期补全为 YYYY-MM-DD，供后端使用 */
function normalizeDate(dateStr: string): string {
  if (!dateStr) return "";
  if (/^\d{4}$/.test(dateStr)) return `${dateStr}-01-01`;
  if (/^\d{4}-\d{2}$/.test(dateStr)) return `${dateStr}-01`;
  return dateStr;
}

export function InputPage() {
  const { state, dispatch } = useAppState();
  const [isLoading, setIsLoading] = useState(false);

  /** 今天的日期字符串（YYYY-MM-DD），用于代码级校验 */
  const todayStr = new Date().toISOString().split("T")[0];

  /** 日期输入变化 */
  const handleDateChange = useCallback(
    (date: string) => {
      dispatch({ type: "SET_START_DATE", payload: date });
      if (state.error) {
        dispatch({ type: "SET_ERROR", payload: null });
      }
    },
    [dispatch, state.error],
  );

  /** 校验日期是否合法（非空） */
  const validateDate = (dateStr: string): string | null => {
    if (!dateStr) {
      return "请选择时间";
    }
    // UI 层面已完全阻止未来日期选择，此处保留兜底
    const fullDate = normalizeDate(dateStr);
    const inputDate = new Date(fullDate + "T00:00:00");
    const today = new Date(todayStr + "T00:00:00");
    if (inputDate.getTime() > today.getTime()) {
      return "时间不能是未来日期";
    }
    return null;
  };

  /** 点击开始扫描 */
  const handleStart = useCallback(async () => {
    const error = validateDate(state.startDate);
    if (error) {
      dispatch({ type: "SET_ERROR", payload: error });
      return;
    }

    setIsLoading(true);
    dispatch({ type: "SET_ERROR", payload: null });

    const normalized = normalizeDate(state.startDate);

    try {
      const scanId = await startScan(normalized);
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
  }, [state.startDate, dispatch, todayStr]);

  const canStart = Boolean(state.startDate) && !isLoading;

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
    <div className="flex flex-col items-center justify-center min-h-[80vh]">
      <div className="w-full max-w-lg">
        {/* 标题区域 */}
        <div className="text-center mb-10">
          <h1 className="text-4xl font-semibold tracking-tight text-foreground mb-3">
            French Exit
          </h1>
          <p className="text-lg text-muted-foreground">
            在撤离公用电脑前，安全处理您留下的痕迹
          </p>
        </div>

        {/* 输入卡片 */}
        <div className="rounded-2xl border border-border bg-card/60 backdrop-blur-md p-8 shadow-sm">
          {/* 日期输入 */}
          <div className="mb-6">
            <DatePicker
              label="你开始使用这台电脑的时间"
              value={state.startDate}
              onChange={handleDateChange}
              placeholder="请选择时间"
            />
            <p className="mt-1.5 text-xs text-muted-foreground">
              系统将扫描该时间之后产生的个人文件与痕迹
            </p>
          </div>

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
                  : "CPU 限制在 30% 以下，办公不卡顿"}
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
