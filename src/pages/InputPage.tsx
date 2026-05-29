/**
 * 输入页（InputPage）
 *
 * 用户进入应用后的首个页面，负责收集入职日期并启动扫描。
 * 极简苹果风：纯黑白、细体字体、大留白。
 */
import { useState, useCallback } from "react";
import { motion } from "framer-motion";
import { useAppState } from "../store/AppContext";
import { startScan } from "../api/commands";
import { DatePicker } from "../components/DatePicker";

/** 将用户选择的局部日期补全为 YYYY-MM-DD，供后端使用 */
function normalizeDate(dateStr: string): string {
  if (!dateStr) return "";
  if (/^\d{4}$/.test(dateStr)) return `${dateStr}-01-01`;
  if (/^\d{4}-\d{2}$/.test(dateStr)) return `${dateStr}-01`;
  return dateStr;
}

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.12,
      delayChildren: 0.5,
    },
  },
};

const item = {
  hidden: { opacity: 0, y: 12 },
  show: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.7,
      ease: [0.4, 0, 0.2, 1],
    },
  },
};

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

  return (
    <motion.div
      className="flex flex-col items-center justify-center min-h-[80vh]"
      variants={container}
      initial="hidden"
      animate="show"
    >
      <div className="w-full max-w-lg">
        {/* 输入卡片 */}
        <motion.div
          className="rounded-2xl border border-white/10 bg-transparent p-8"
          variants={item}
        >
          {/* 日期输入 */}
          <div className="mb-6">
            <DatePicker
              label="您开始使用这台电脑的时间"
              value={state.startDate}
              onChange={handleDateChange}
              placeholder="请选择时间"
            />
            <p className="mt-1.5 text-xs text-muted-foreground font-light">
              系统将扫描该时间之后产生的个人文件与痕迹
            </p>
          </div>

          {/* 开始按钮 */}
          <motion.button
            onClick={handleStart}
            disabled={!canStart}
            className={`
              w-full rounded-full px-8 py-3 font-light
              transition-all duration-300 ease-out
              ${
                canStart
                  ? "text-black bg-white hover:bg-white/90 active:shadow-[inset_0_2px_4px_rgba(0,0,0,0.3)]"
                  : "bg-white/20 text-white/40 cursor-not-allowed"
              }
            `}
            whileHover={canStart ? { scale: 1.01 } : {}}
            whileTap={canStart ? { scale: 0.99 } : {}}
          >
            {isLoading ? (
              <span className="flex items-center justify-center gap-2">
                <span className="w-4 h-4 border-2 border-black/30 border-t-black rounded-full animate-spin" />
                正在启动…
              </span>
            ) : (
              "开始扫描"
            )}
          </motion.button>

          {/* 错误提示 */}
          {state.error && (
            <motion.p
              className="mt-4 text-sm text-center text-white/70"
              initial={{ opacity: 0, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
            >
              {state.error}
            </motion.p>
          )}
        </motion.div>

        {/* 底部说明 */}
        <motion.p
          className="mt-6 text-center text-xs text-muted-foreground font-light"
          variants={item}
        >
          所有操作在本地完成，不会上传任何数据
        </motion.p>
      </div>
    </motion.div>
  );
}
