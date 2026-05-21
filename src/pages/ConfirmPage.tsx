import { useAppState } from "../store/AppContext";
import { submitDecisions } from "../api/commands";
import { useState, useMemo } from "react";
import { formatBytes } from "../utils/format";
import type { TraceItem } from "../types";

export function ConfirmPage() {
  const { state, dispatch } = useAppState();
  const [showDialog, setShowDialog] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // 按 action 分组（useMemo 避免每次渲染重复全量遍历）
  const { deleteItems, packItems, preserveItems } = useMemo(() => {
    const deleteItems: TraceItem[] = [];
    const packItems: TraceItem[] = [];
    const preserveItems: TraceItem[] = [];

    // 建立已加载完整数据的查找表
    const itemMap = new Map(state.scanResults.map((i) => [i.id, i]));

    // 遍历所有决策，确保数量不丢失
    for (const [id, decision] of state.decisions) {
      const item = itemMap.get(id) || {
        id,
        name: id,
        path: null,
        category: "FileSystem" as TraceCategory,
        scanner_id: "",
        size_bytes: null,
        modified_at: null,
        inferred: false,
        risk_note: null,
        suggested_action: decision.action,
      };

      if (decision.action === "Delete") deleteItems.push(item);
      else if (decision.action === "Pack") packItems.push(item);
      else if (decision.action === "Preserve") preserveItems.push(item);
    }

    return { deleteItems, packItems, preserveItems };
  }, [state.scanResults, state.decisions]);

  const handleBack = () => {
    dispatch({ type: "SET_PAGE", payload: "results" });
  };

  const handleConfirm = () => {
    if (deleteItems.length === 0 && packItems.length === 0) {
      // 没有需要操作的内容，直接提示
      dispatch({ type: "SET_ERROR", payload: "没有标记任何需要删除或打包的内容" });
      return;
    }
    setShowDialog(true);
  };

  const handleExecute = async () => {
    setIsSubmitting(true);
    setShowDialog(false);
    try {
      const decisions = Array.from(state.decisions.values());
      await submitDecisions(decisions);
      dispatch({ type: "SET_PAGE", payload: "executing" });
    } catch (e: any) {
      dispatch({ type: "SET_ERROR", payload: e.message || "提交决策失败" });
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-3xl mx-auto py-6">
      {/* 顶部标题 */}
      <div className="mb-6">
        <h2 className="text-2xl font-semibold mb-1">最终确认</h2>
        <p className="text-muted-foreground text-sm">
          请最后检查您的选择，确认后操作将不可逆
        </p>
      </div>

      <DecisionGroup items={deleteItems} icon="🗑️" title="待删除" variant="red" />
      <DecisionGroup items={packItems} icon="📦" title="待打包" variant="blue" />
      <DecisionGroup items={preserveItems} icon="✓" title="待保留" variant="gray" />

      {/* 底部操作 */}
      <div className="flex gap-3 sticky bottom-4 bg-background/80 backdrop-blur-xl p-4 rounded-2xl border border-border">
        <button
          onClick={handleBack}
          className="px-5 py-2.5 border border-border rounded-xl font-medium hover:bg-muted active:scale-95 transition"
        >
          返回修改
        </button>
        <button
          onClick={handleConfirm}
          disabled={isSubmitting}
          className="flex-1 px-5 py-2.5 bg-red-600 text-white rounded-xl font-medium hover:bg-red-700 active:scale-95 transition disabled:opacity-50"
        >
          {isSubmitting ? "提交中..." : "确认执行"}
        </button>
      </div>

      {/* 二次确认弹窗（RULE-01） */}
      {showDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
          <div className="bg-card rounded-2xl p-6 max-w-md w-full shadow-2xl border border-border">
            <h3 className="text-lg font-semibold mb-3">⚠️ 操作不可逆</h3>
            <p className="text-sm text-muted-foreground mb-2">
              以下文件将被<strong className="text-red-600">彻底删除</strong>
              ，无法恢复：
            </p>
            <p className="text-sm font-medium mb-4">
              {deleteItems.length} 条痕迹（共{" "}
              {formatBytes(
                deleteItems.reduce((sum, i) => sum + (i.size_bytes || 0), 0)
              )}）
            </p>
            {packItems.length > 0 && (
              <p className="text-sm text-muted-foreground mb-4">
                {packItems.length} 个文件将打包至 French-exit.zip
              </p>
            )}
            <div className="flex gap-3">
              <button
                onClick={() => setShowDialog(false)}
                className="flex-1 px-4 py-2.5 border border-border rounded-xl font-medium hover:bg-muted transition"
              >
                再想想
              </button>
              <button
                onClick={handleExecute}
                className="flex-1 px-4 py-2.5 bg-red-600 text-white rounded-xl font-medium hover:bg-red-700 active:scale-95 transition"
              >
                确定继续
              </button>
            </div>
          </div>
        </div>
      )}

      {state.error && (
        <p className="mt-4 text-sm text-center">{state.error}</p>
      )}
    </div>
  );
}

interface DecisionGroupProps {
  items: TraceItem[];
  icon: string;
  title: string;
  variant: "red" | "blue" | "gray";
}

function DecisionGroup({ items, icon, title, variant }: DecisionGroupProps) {
  if (items.length === 0) return null;

  const styles = {
    red: {
      border: "border-red-200 dark:border-red-900/50",
      headerBg: "bg-red-50 dark:bg-red-900/20",
      headerBorder: "border-red-100 dark:border-red-900/30",
      title: "text-red-700 dark:text-red-400",
      size: "text-red-600 dark:text-red-400",
    },
    blue: {
      border: "border-blue-200 dark:border-blue-900/50",
      headerBg: "bg-blue-50 dark:bg-blue-900/20",
      headerBorder: "border-blue-100 dark:border-blue-900/30",
      title: "text-blue-700 dark:text-blue-400",
      size: "text-blue-600 dark:text-blue-400",
    },
    gray: {
      border: "border-border",
      headerBg: "bg-muted/50",
      headerBorder: "border-border/50",
      title: "text-muted-foreground",
      size: "text-muted-foreground",
    },
  };

  const s = styles[variant];

  return (
    <div className={`mb-6 bg-card/80 backdrop-blur-xl rounded-2xl border ${s.border} overflow-hidden`}>
      <div className={`px-5 py-3 ${s.headerBg} border-b ${s.headerBorder}`}>
        <h3 className={`font-medium ${s.title} flex items-center gap-2`}>
          <span>{icon}</span> {title}（{items.length} 条）
        </h3>
      </div>
      <ul className="divide-y divide-border/50 max-h-60 overflow-y-auto">
        {items.map((item) => (
          <li key={item.id} className="px-5 py-2.5 flex items-center justify-between">
            <div className="min-w-0">
              <p className="text-sm font-medium truncate">{item.name}</p>
              <p className="text-xs text-muted-foreground truncate">{item.path || "-"}</p>
            </div>
            <span className={`text-xs ${s.size} ml-4 shrink-0`}>
              {item.size_bytes ? formatBytes(item.size_bytes) : ""}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
