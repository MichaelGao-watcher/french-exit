import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface DatePickerProps {
  value: string; // YYYY, YYYY-MM, or YYYY-MM-DD
  onChange: (date: string) => void;
  label?: string;
  placeholder?: string;
}

function getTodayParts() {
  const d = new Date();
  return {
    year: d.getFullYear(),
    month: d.getMonth() + 1,
    day: d.getDate(),
  };
}

export function DatePicker({
  value,
  onChange,
  label,
  placeholder = "请选择时间",
}: DatePickerProps) {
  const [openPanel, setOpenPanel] = useState<"year" | "month" | "day" | null>(
    null,
  );
  const containerRef = useRef<HTMLDivElement>(null);

  const today = getTodayParts();

  const parseDate = (d: string) => {
    if (!d) return { year: "", month: "", day: "" };
    const parts = d.split("-");
    return {
      year: parts[0] || "",
      month: parts[1] || "",
      day: parts[2] || "",
    };
  };

  const { year, month, day } = parseDate(value);
  const yNum = year ? Number(year) : 0;
  const mNum = month ? Number(month) : 0;

  // 年份列表：今年 到 1970（未来年份完全不显示）
  const years = Array.from(
    { length: today.year - 1969 },
    (_, i) => String(today.year - i),
  );

  // 月份：如果选的是今年，最多只到当前月
  const maxMonth = yNum === today.year ? today.month : 12;
  const months = Array.from({ length: maxMonth }, (_, i) =>
    String(i + 1).padStart(2, "0"),
  );

  // 日期：如果选的是今年当月，最多只到今天
  const maxDay = (() => {
    if (!year || !month) return 31;
    const monthTotal = new Date(yNum, mNum, 0).getDate();
    if (yNum === today.year && mNum === today.month) {
      return Math.min(monthTotal, today.day);
    }
    return monthTotal;
  })();
  const days = Array.from({ length: maxDay }, (_, i) =>
    String(i + 1).padStart(2, "0"),
  );

  const updateDate = (
    newYear: string,
    newMonth: string,
    newDay: string,
  ) => {
    if (!newYear) {
      onChange("");
      return;
    }
    if (!newMonth) {
      onChange(newYear);
      return;
    }
    const ny = Number(newYear);
    const nm = Number(newMonth);
    const monthTotal = new Date(ny, nm, 0).getDate();
    const limitDay =
      ny === today.year && nm === today.month
        ? Math.min(monthTotal, today.day)
        : monthTotal;
    const safeDay =
      Number(newDay) > limitDay
        ? String(limitDay).padStart(2, "0")
        : newDay;
    if (!safeDay) {
      onChange(`${newYear}-${newMonth}`);
      return;
    }
    onChange(`${newYear}-${newMonth}-${safeDay}`);
  };

  // 点击外部关闭
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setOpenPanel(null);
      }
    };
    if (openPanel) document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [openPanel]);

  const displayValue = (() => {
    if (!year) return placeholder;
    if (!month) return `${year}年`;
    if (!day) return `${year}年 ${month}月`;
    return `${year}年 ${month}月 ${day}日`;
  })();

  const renderPanel = (
    type: "year" | "month" | "day",
    options: string[],
    currentValue: string,
    onSelect: (v: string) => void,
  ) => {
    return (
      <AnimatePresence>
        {openPanel === type && (
          <motion.div
            className="
              absolute z-50 mt-1.5 left-0 right-0
              rounded-xl border border-white/10
              bg-black/95 backdrop-blur-xl overflow-hidden
            "
            initial={{ opacity: 0, y: -4, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98 }}
            transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}
          >
            <div className="max-h-52 overflow-y-auto py-1.5 no-scrollbar">
              {options.map((opt) => (
                <button
                  key={opt}
                  type="button"
                  onClick={() => {
                    onSelect(opt);
                    setOpenPanel(null);
                  }}
                  className={`
                    w-full px-3 py-2 text-left text-sm transition-colors duration-200
                    ${
                      opt === currentValue
                        ? "bg-white text-black font-medium"
                        : "text-foreground hover:bg-white/10"
                    }
                  `}
                >
                  {opt}
                  {type === "year" && "年"}
                  {type === "month" && "月"}
                  {type === "day" && "日"}
                </button>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    );
  };

  return (
    <div className="relative" ref={containerRef}>
      {label && (
        <label className="block text-sm font-light text-foreground mb-2">
          {label}
        </label>
      )}

      {/* 触发按钮行 */}
      <div className="flex gap-2">
        {/* 年份 */}
        <div className="flex-1 relative min-w-0">
          <button
            type="button"
            onClick={() =>
              setOpenPanel(openPanel === "year" ? null : "year")
            }
            className={`
              w-full rounded-xl border border-white/20
              bg-transparent
              px-3 py-3 text-left text-sm outline-none
              focus:border-white/40 focus:ring-1 focus:ring-white/20
              transition-all duration-300
              ${year ? "text-foreground" : "text-muted-foreground"}
            `}
          >
            <span className="flex items-center justify-between">
              <span className="font-light">{year ? `${year}年` : "年"}</span>
              <motion.svg
                className="w-3.5 h-3.5 text-muted-foreground"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.5}
                animate={{ rotate: openPanel === "year" ? 180 : 0 }}
                transition={{ duration: 0.3, ease: [0.4, 0, 0.2, 1] }}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M19 9l-7 7-7-7"
                />
              </motion.svg>
            </span>
          </button>
          {renderPanel("year", years, year, (v) =>
            updateDate(v, month, day),
          )}
        </div>

        {/* 月份 */}
        <div className="flex-1 relative min-w-0">
          <button
            type="button"
            onClick={() =>
              year && setOpenPanel(openPanel === "month" ? null : "month")
            }
            disabled={!year}
            className={`
              w-full rounded-xl border border-white/20
              bg-transparent
              px-3 py-3 text-left text-sm outline-none
              focus:border-white/40 focus:ring-1 focus:ring-white/20
              transition-all duration-300
              ${
                !year
                  ? "opacity-40 cursor-not-allowed text-muted-foreground"
                  : month
                    ? "text-foreground"
                    : "text-muted-foreground"
              }
            `}
          >
            <span className="flex items-center justify-between">
              <span className="font-light">{month ? `${month}月` : "月"}</span>
              <motion.svg
                className="w-3.5 h-3.5 text-muted-foreground"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.5}
                animate={{ rotate: openPanel === "month" ? 180 : 0 }}
                transition={{ duration: 0.3, ease: [0.4, 0, 0.2, 1] }}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M19 9l-7 7-7-7"
                />
              </motion.svg>
            </span>
          </button>
          {renderPanel("month", months, month, (v) =>
            updateDate(year, v, day),
          )}
        </div>

        {/* 日期 */}
        <div className="flex-1 relative min-w-0">
          <button
            type="button"
            onClick={() =>
              year &&
              month &&
              setOpenPanel(openPanel === "day" ? null : "day")
            }
            disabled={!year || !month}
            className={`
              w-full rounded-xl border border-white/20
              bg-transparent
              px-3 py-3 text-left text-sm outline-none
              focus:border-white/40 focus:ring-1 focus:ring-white/20
              transition-all duration-300
              ${
                !year || !month
                  ? "opacity-40 cursor-not-allowed text-muted-foreground"
                  : day
                    ? "text-foreground"
                    : "text-muted-foreground"
              }
            `}
          >
            <span className="flex items-center justify-between">
              <span className="font-light">{day ? `${day}日` : "日"}</span>
              <motion.svg
                className="w-3.5 h-3.5 text-muted-foreground"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.5}
                animate={{ rotate: openPanel === "day" ? 180 : 0 }}
                transition={{ duration: 0.3, ease: [0.4, 0, 0.2, 1] }}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M19 9l-7 7-7-7"
                />
              </motion.svg>
            </span>
          </button>
          {renderPanel("day", days, day, (v) =>
            updateDate(year, month, v),
          )}
        </div>
      </div>
    </div>
  );
}
