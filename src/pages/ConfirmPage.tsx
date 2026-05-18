import { useAppState } from "../store/AppContext";
import { submitDecisions } from "../api/commands";
import { useState } from "react";

export function ConfirmPage() {
  const { state, dispatch } = useAppState();
  const [showDialog, setShowDialog] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // 按 action 分组
  const deleteItems = state.scanResults.filter(
    (item) => state.decisions.get(item.id)?.action === "Delete"
  );
  const packItems = state.scanResults.filter(
    (item) => state.decisions.get(item.id)?.action === "Pack"
  );
  const preserveItems = state.scanResults.filter(
    (item) => state.decisions.get(item.id)?.action === "Preserve"
  );

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

      {/* 删除清单 */}
      {deleteItems.length > 0 && (
        <div className="mb-6 bg-card/80 backdrop-blur-xl rounded-2xl border border-red-200 dark:border-red-900/50 overflow-hidden">
          <div className="px-5 py-3 bg-red-50 dark:bg-red-900/20 border-b border-red-100 dark:border-red-900/30">
            <h3 className="font-medium text-red-700 dark:text-red-400 flex items-center gap-2">
              <span>🗑️</span> 待删除（{deleteItems.length} 条）
            </h3>
          </div>
          <ul className="divide-y divide-border/50 max-h-60 overflow-y-auto">
            {deleteItems.map((item) => (
              <li
                key={item.id}
                className="px-5 py-2.5 flex items-center justify-between"
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium truncate">{item.name}</p>
                  <p className="text-xs text-muted-foreground truncate">
                    {item.path || "-"}
                  </p>
                </div>
                <span className="text-xs text-red-600 dark:text-red-400 ml-4 shrink-0">
                  {item.size_bytes ? formatBytes(item.size_bytes) : ""}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 打包清单 */}
      {packItems.length > 0 && (
        <div className="mb-6 bg-card/80 backdrop-blur-xl rounded-2xl border border-blue-200 dark:border-blue-900/50 overflow-hidden">
          <div className="px-5 py-3 bg-blue-50 dark:bg-blue-900/20 border-b border-blue-100 dark:border-blue-900/30">
            <h3 className="font-medium text-blue-700 dark:text-blue-400 flex items-center gap-2">
              <span>📦</span> 待打包（{packItems.length} 条）
            </h3>
          </div>
          <ul className="divide-y divide-border/50 max-h-60 overflow-y-auto">
            {packItems.map((item) => (
              <li
                key={item.id}
                className="px-5 py-2.5 flex items-center justify-between"
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium truncate">{item.name}</p>
                  <p className="text-xs text-muted-foreground truncate">
                    {item.path || "-"}
                  </p>
                </div>
                <span className="text-xs text-blue-600 dark:text-blue-400 ml-4 shrink-0">
                  {item.size_bytes ? formatBytes(item.size_bytes) : ""}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 保留清单 */}
      {preserveItems.length > 0 && (
        <div className="mb-6 bg-card/80 backdrop-blur-xl rounded-2xl border border-border overflow-hidden">
          <div className="px-5 py-3 bg-muted/50 border-b border-border/50">
            <h3 className="font-medium text-muted-foreground flex items-center gap-2">
              <span>✓</span> 待保留（{preserveItems.length} 条）
            </h3>
          </div>
          <ul className="divide-y divide-border/50 max-h-60 overflow-y-auto">
            {preserveItems.map((item) => (
              <li
                key={item.id}
                className="px-5 py-2.5 flex items-center justify-between"
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium truncate">{item.name}</p>
                  <p className="text-xs text-muted-foreground truncate">
                    {item.path || "-"}
                  </p>
                </div>
                <span className="text-xs text-muted-foreground ml-4 shrink-0">
                  {item.size_bytes ? formatBytes(item.size_bytes) : ""}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

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
        <p className="mt-4 text-sm text-red-500 text-center">{state.error}</p>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
}
