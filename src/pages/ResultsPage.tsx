/**
 * 结果确认页面
 *
 * 职责：
 * 1. 展示扫描发现的痕迹列表
 * 2. 按分类 Tab 过滤 + 搜索过滤
 * 3. 支持单条勾选 / 全选本页 / 取消全选
 * 4. 批量标记为删除 / 保留 / 打包
 * 5. 分页加载（每次 50 条）
 * 6. 应用默认勾选规则（RULE-02 / RULE-03）
 * 7. 支持文件预览弹窗
 */
import { useEffect, useMemo, useRef, useState } from "react";
import { motion } from "framer-motion";
import { useAppState } from "../store/AppContext";
import { getScanResults, getAllScanSummaries, openPath } from "../api/commands";
import type { TraceCategory, TraceItem, Decision } from "../types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { formatBytes, formatDate } from "../utils/format";

const CATEGORIES: { key: TraceCategory | "all"; label: string }[] = [
  { key: "all", label: "全部" },
  { key: "Chat", label: "聊天" },
  { key: "Browser", label: "浏览器" },
  { key: "System", label: "系统" },
  { key: "Registry", label: "注册表" },
  { key: "FileSystem", label: "文件" },
  { key: "DevTools", label: "开发工具" },
  { key: "EnvVar", label: "环境变量" },
];

const FILE_TYPE_GROUPS: { key: string; label: string }[] = [
  { key: "all", label: "全部" },
  { key: "photo", label: "照片/图片" },
  { key: "video", label: "视频" },
  { key: "audio", label: "音频" },
  { key: "work_doc", label: "工作文档" },
  { key: "code", label: "代码" },
  { key: "archive", label: "压缩包" },
  { key: "design", label: "设计文件" },
  { key: "executable", label: "程序" },
  { key: "temp", label: "临时文件" },
  { key: "other", label: "其他" },
];

const SOURCE_LABELS: Record<string, string> = {
  personal_desktop: "桌面文件",
  personal_downloads: "下载文件",
  personal_documents: "文档目录",
  other: "其他位置",
};

function getDefaultAction(item: TraceItem): "Delete" | "Preserve" | "Pack" | null {
  if (item.suggested_action === "DeleteOrPack" || item.suggested_action === "Delete") {
    return "Delete";
  }
  if (item.suggested_action === "Preserve") {
    return "Preserve";
  }
  if (item.suggested_action === "Pack") {
    return "Pack";
  }
  return null;
}

function canPreview(item: TraceItem): boolean {
  if (!item.path) return false;
  const ext = item.path.split(".").pop()?.toLowerCase();
  const textExts = [
    "txt", "md", "json", "csv", "log", "xml", "yaml", "yml",
    "js", "ts", "tsx", "rs", "html", "css",
  ];
  const imageExts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];
  return textExts.includes(ext || "") || imageExts.includes(ext || "");
}

export function ResultsPage() {
  const { state, dispatch } = useAppState();
  const [activeCategory, setActiveCategory] = useState<TraceCategory | "all">("all");
  const [page, setPage] = useState(1);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState("");
  const [activeFileType, setActiveFileType] = useState<string>("all");
  const [previewItem, setPreviewItem] = useState<TraceItem | null>(null);
  const [previewContent, setPreviewContent] = useState<string | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const PAGE_SIZE = 50;

  // 数据加载
  useEffect(() => {
    loadPage(1);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const loadPage = async (p: number) => {
    try {
      const result = await getScanResults(p, PAGE_SIZE);
      if (p === 1) {
        dispatch({
          type: "SET_SCAN_RESULTS",
          payload: { items: result.items, total: result.total },
        });
      } else {
        dispatch({
          type: "APPEND_SCAN_RESULTS",
          payload: { items: result.items, total: result.total },
        });
      }
      setPage(p);
    } catch (e: any) {
      dispatch({ type: "SET_ERROR", payload: e.message || "加载结果失败" });
    }
  };

  // 默认勾选逻辑已移除：所有勾选必须由用户显式操作，防止误删
  // RULE-02 / RULE-03 的默认行为改为仅在用户手动勾选时应用建议动作

  const filteredItems = useMemo(() => {
    let items = activeCategory === "all"
      ? state.scanResults
      : state.scanResults.filter((item) => item.category === activeCategory);

    // 文件类型筛选（仅对 FileSystem 类别生效）
    if (activeCategory === "FileSystem" && activeFileType !== "all") {
      items = items.filter((item) => item.file_type === activeFileType);
    }

    // 搜索过滤
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      items = items.filter(
        (item) =>
          item.name.toLowerCase().includes(q) ||
          (item.path && item.path.toLowerCase().includes(q))
      );
    }

    return items;
  }, [state.scanResults, activeCategory, activeFileType, searchQuery]);

  const selectedCount = selectedIds.size;
  const totalCount = state.scanTotal;

  const toggleItem = (id: string) => {
    const item = state.scanResults.find((i) => i.id === id);
    if (!item) return;

    const next = new Set(selectedIds);
    const newDecisions = new Map(state.decisions);

    if (next.has(id)) {
      next.delete(id);
      newDecisions.delete(id);
    } else {
      next.add(id);
      const action = getDefaultAction(item);
      if (action) {
        newDecisions.set(id, { item_id: id, action });
      }
    }

    setSelectedIds(next);
    dispatch({ type: "SET_DECISIONS", payload: newDecisions });
  };

  const selectAllPage = () => {
    const ids = new Set(filteredItems.map((i) => i.id));
    setSelectedIds(ids);

    const newDecisions = new Map(state.decisions);
    filteredItems.forEach((item) => {
      if (item.category === "EnvVar") return;
      const action = getDefaultAction(item);
      if (action) {
        newDecisions.set(item.id, { item_id: item.id, action });
      }
    });
    dispatch({ type: "SET_DECISIONS", payload: newDecisions });
  };

  const selectAllAll = async () => {
    try {
      const summaries = await getAllScanSummaries();
      const ids = new Set<string>();
      const newDecisions = new Map(state.decisions);
      const newItems: TraceItem[] = [];

      summaries.forEach((summary) => {
        ids.add(summary.id);
        if (summary.category === "EnvVar") return;
        const action = getDefaultAction(summary);
        if (action) {
          newDecisions.set(summary.id, { item_id: summary.id, action });
        }
        // 将 summary 转为轻量 TraceItem 追加到 scanResults，供 ConfirmPage 展示
        newItems.push({
          id: summary.id,
          name: summary.name,
          path: null,
          category: summary.category,
          scanner_id: "",
          size_bytes: null,
          modified_at: null,
          inferred: false,
          risk_note: null,
          suggested_action: summary.suggested_action,
          source: "other",
          file_type: "other",
        });
      });

      // 合并到 scanResults（去重：已加载的完整数据优先保留）
      const existingIds = new Set(state.scanResults.map((i) => i.id));
      const mergedItems = [
        ...state.scanResults,
        ...newItems.filter((i) => !existingIds.has(i.id)),
      ];
      dispatch({
        type: "SET_SCAN_RESULTS",
        payload: { items: mergedItems, total: state.scanTotal },
      });

      setSelectedIds(ids);
      dispatch({ type: "SET_DECISIONS", payload: newDecisions });
    } catch (e: any) {
      dispatch({ type: "SET_ERROR", payload: e.message || "全选全部失败" });
    }
  };

  const deselectAll = () => {
    setSelectedIds(new Set());
    // 清除全部决策，不只当前页/搜索过滤后的数据
    dispatch({ type: "SET_DECISIONS", payload: new Map() });
  };

  const markSelected = (action: "Delete" | "Preserve" | "Pack") => {
    const newDecisions = new Map(state.decisions);
    selectedIds.forEach((id) => {
      newDecisions.set(id, { item_id: id, action });
    });
    dispatch({ type: "SET_DECISIONS", payload: newDecisions });
  };

  const handleLoadMore = () => {
    loadPage(page + 1);
  };

  const hasMore = state.scanResults.length < state.scanTotal;

  // 预览逻辑
  const handlePreview = async (item: TraceItem) => {
    if (!item.path) return;
    setPreviewItem(item);
    setPreviewLoading(true);
    setPreviewContent(null);

    const ext = item.path.split(".").pop()?.toLowerCase();
    const imageExts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];

    if (imageExts.includes(ext || "")) {
      setPreviewContent("IMAGE:" + item.path);
    } else {
      try {
        const { readTextFile } = await import("@tauri-apps/api/fs");
        const content = await readTextFile(item.path);
        setPreviewContent(content.slice(0, 4096));
      } catch {
        setPreviewContent("ERROR: 无法读取文件内容");
      }
    }
    setPreviewLoading(false);
  };

  const closePreview = () => {
    setPreviewItem(null);
    setPreviewContent(null);
    setPreviewLoading(false);
  };

  // ESC 关闭预览
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && previewItem) {
        closePreview();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [previewItem]);

  const container = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
        delayChildren: 0.3,
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

  return (
    <motion.div
      className="flex flex-col min-h-[70vh] max-w-3xl mx-auto"
      variants={container}
      initial="hidden"
      animate="show"
    >
      {/* 标题栏 */}
      <motion.div variants={item} className="mb-6">
        <h2 className="text-2xl font-semibold">发现 {totalCount} 条痕迹</h2>
        <p className="text-muted-foreground mt-1">已选择 {selectedCount} 条</p>
      </motion.div>

      {/* 分类 Tab */}
      <div className="flex flex-wrap gap-2 mb-4">
        {CATEGORIES.map((cat) => (
          <button
            key={cat.key}
            onClick={() => { setActiveCategory(cat.key); setActiveFileType("all"); }}
            className={`px-4 py-1.5 rounded-full text-sm font-medium transition ${
              activeCategory === cat.key
                ? "bg-blue-600 text-white"
                : "bg-muted text-muted-foreground hover:bg-muted/80"
            }`}
          >
            {cat.label}
          </button>
        ))}
      </div>

      {/* 二级 Tab：文件类型筛选（仅 FileSystem 类别） */}
      {activeCategory === "FileSystem" && (
        <div className="flex flex-wrap gap-1.5 mb-4">
          {FILE_TYPE_GROUPS.map((group) => (
            <button
              key={group.key}
              onClick={() => setActiveFileType(activeFileType === group.key ? "all" : group.key)}
              className={`px-3 py-1 rounded-lg text-xs font-medium transition ${
                activeFileType === group.key
                  ? "bg-primary/20 text-primary"
                  : "bg-muted/50 text-muted-foreground hover:bg-muted"
              }`}
            >
              {group.label}
            </button>
          ))}
        </div>
      )}

      {/* 搜索栏 */}
      <div className="relative mb-4">
        <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
          </svg>
        </div>
        <input
          type="text"
          placeholder="搜索文件名或路径..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full rounded-xl border border-border px-10 py-2.5 bg-white/50 dark:bg-black/30 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition"
        />
        {searchQuery && (
          <button
            onClick={() => setSearchQuery("")}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        )}
      </div>

      {/* 列表区域 */}
      <div className="flex-1 space-y-2 mb-6">
        {filteredItems.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground">
            {searchQuery.trim() ? "无匹配结果" : "该分类下暂无痕迹"}
          </div>
        ) : activeCategory === "FileSystem" ? (
          (() => {
            const groups = new Map<string, typeof filteredItems>();
            filteredItems.forEach((item) => {
              const src = item.source || "other";
              if (!groups.has(src)) groups.set(src, []);
              groups.get(src)!.push(item);
            });
            return Array.from(groups.entries()).map(([source, items]) => (
              <div key={source} className="mb-4">
                <div className="text-xs font-medium text-muted-foreground mb-2 px-1">
                  {SOURCE_LABELS[source] || source} ({items.length})
                </div>
                <div className="space-y-2">
                  {items.map((item) => (
                    <div
                      key={item.id}
                      className="flex items-center gap-3 bg-card/80 backdrop-blur-xl rounded-xl p-4 border border-border shadow-sm hover:shadow-md transition"
                    >
                      <input
                        type="checkbox"
                        checked={selectedIds.has(item.id)}
                        onChange={() => toggleItem(item.id)}
                        className="w-4 h-4 rounded border-border text-blue-600 focus:ring-blue-600 shrink-0"
                      />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm truncate" title={item.name}>{item.name}</span>
                          {item.inferred && (
                            <span className="text-[10px] px-1.5 py-0.5 bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 rounded">
                              推断
                            </span>
                          )}
                        </div>
                        {item.path ? (
                          <button
                            onClick={() => openPath(item.path!)}
                            className="text-xs text-muted-foreground truncate mt-0.5 text-left hover:text-blue-400 hover:underline transition block w-full"
                            title="点击打开所在文件夹"
                          >
                            {item.path}
                          </button>
                        ) : (
                          <div className="text-xs text-muted-foreground truncate mt-0.5">-</div>
                        )}
                        <div className="text-xs text-muted-foreground mt-0.5 flex gap-3">
                          <span>{formatBytes(item.size_bytes)}</span>
                          <span className="text-foreground/70">修改于 {formatDate(item.modified_at)}</span>
                        </div>
                      </div>
                      <div className="shrink-0 flex items-center gap-2">
                        {item.path && (
                          <button
                            onClick={() => openPath(item.path!)}
                            className="px-2 py-1 text-xs rounded-lg bg-muted hover:bg-muted/80 transition"
                            title="打开所在文件夹"
                          >
                            打开
                          </button>
                        )}
                        {canPreview(item) && (
                          <button
                            onClick={() => handlePreview(item)}
                            className="px-2 py-1 text-xs rounded-lg bg-muted hover:bg-muted/80 transition"
                          >
                            预览
                          </button>
                        )}
                        {item.risk_note && (
                          <span title={item.risk_note} className="text-yellow-500 text-lg">
                            ⚠️
                          </span>
                        )}
                        {state.decisions.has(item.id) && (
                          <span
                            className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                              state.decisions.get(item.id)?.action === "Delete"
                                ? "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200"
                                : state.decisions.get(item.id)?.action === "Preserve"
                                ? "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300"
                                : "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200"
                            }`}
                          >
                            {state.decisions.get(item.id)?.action === "Delete"
                              ? "删除"
                              : state.decisions.get(item.id)?.action === "Preserve"
                              ? "保留"
                              : "打包"}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ));
          })()
        ) : (
          filteredItems.map((item) => (
            <div
              key={item.id}
              className="flex items-center gap-3 bg-card/80 backdrop-blur-xl rounded-xl p-4 border border-border shadow-sm hover:shadow-md transition"
            >
              <input
                type="checkbox"
                checked={selectedIds.has(item.id)}
                onChange={() => toggleItem(item.id)}
                className="w-4 h-4 rounded border-border text-blue-600 focus:ring-blue-600 shrink-0"
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-sm truncate" title={item.name}>{item.name}</span>
                  {item.inferred && (
                    <span className="text-[10px] px-1.5 py-0.5 bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 rounded">
                      推断
                    </span>
                  )}
                </div>
                {item.path ? (
                  <button
                    onClick={() => openPath(item.path!)}
                    className="text-xs text-muted-foreground truncate mt-0.5 text-left hover:text-blue-400 hover:underline transition block w-full"
                    title="点击打开所在文件夹"
                  >
                    {item.path}
                  </button>
                ) : (
                  <div className="text-xs text-muted-foreground truncate mt-0.5">-</div>
                )}
                <div className="text-xs text-muted-foreground mt-0.5 flex gap-3">
                  <span>{formatBytes(item.size_bytes)}</span>
                  <span className="text-foreground/70">修改于 {formatDate(item.modified_at)}</span>
                </div>
              </div>
              <div className="shrink-0 flex items-center gap-2">
                {item.path && (
                  <button
                    onClick={() => openPath(item.path!)}
                    className="px-2 py-1 text-xs rounded-lg bg-muted hover:bg-muted/80 transition"
                    title="打开所在文件夹"
                  >
                    打开
                  </button>
                )}
                {canPreview(item) && (
                  <button
                    onClick={() => handlePreview(item)}
                    className="px-2 py-1 text-xs rounded-lg bg-muted hover:bg-muted/80 transition"
                  >
                    预览
                  </button>
                )}
                {item.risk_note && (
                  <span title={item.risk_note} className="text-yellow-500 text-lg">
                    ⚠️
                  </span>
                )}
                {state.decisions.has(item.id) && (
                  <span
                    className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                      state.decisions.get(item.id)?.action === "Delete"
                        ? "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200"
                        : state.decisions.get(item.id)?.action === "Preserve"
                        ? "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300"
                        : "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200"
                    }`}
                  >
                    {state.decisions.get(item.id)?.action === "Delete"
                      ? "删除"
                      : state.decisions.get(item.id)?.action === "Preserve"
                      ? "保留"
                      : "打包"}
                  </span>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* 分页控制 */}
      {state.scanResults.length > 0 && (
        <div className="flex flex-col items-center gap-2 mb-6">
          <p className="text-xs text-muted-foreground">
            已加载 {state.scanResults.length} / 共 {totalCount} 条
          </p>
          {hasMore && (
            <button
              onClick={handleLoadMore}
              className="px-6 py-2 bg-secondary text-secondary-foreground rounded-xl font-medium hover:bg-secondary/80 active:scale-95 transition"
            >
              加载更多
            </button>
          )}
        </div>
      )}

      {/* 批量操作栏 + 下一步按钮 */}
      <div className="sticky bottom-0 bg-background/90 backdrop-blur-xl border-t border-border -mx-4 px-4 py-4 flex flex-col gap-4">
        {filteredItems.length > 0 && (
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex gap-2">
              <button
                onClick={selectAllPage}
                className="px-3 py-1.5 text-xs bg-muted text-muted-foreground rounded-lg hover:bg-muted/80 transition"
              >
                全选本页
              </button>
              <button
                onClick={selectAllAll}
                className="px-3 py-1.5 text-xs bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
              >
                全选全部 ({totalCount})
              </button>
              <button
                onClick={deselectAll}
                className="px-3 py-1.5 text-xs bg-muted text-muted-foreground rounded-lg hover:bg-muted/80 transition"
              >
                取消全选
              </button>
            </div>
            <div className="w-px h-6 bg-border hidden sm:block" />
            <div className="flex gap-2">
              <button
                onClick={() => markSelected("Delete")}
                disabled={selectedCount === 0}
                className="px-3 py-1.5 text-xs bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-40 disabled:cursor-not-allowed transition"
              >
                标记为删除
              </button>
              <button
                onClick={() => markSelected("Preserve")}
                disabled={selectedCount === 0}
                className="px-3 py-1.5 text-xs bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/80 disabled:opacity-40 disabled:cursor-not-allowed transition"
              >
                标记为保留
              </button>
              <button
                onClick={() => markSelected("Pack")}
                disabled={selectedCount === 0}
                className="px-3 py-1.5 text-xs bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition"
              >
                标记为打包
              </button>
            </div>
          </div>
        )}

        <div className="flex justify-end">
          <button
            onClick={() => dispatch({ type: "SET_PAGE", payload: "confirm" })}
            disabled={state.decisions.size === 0}
            className="px-8 py-3 bg-blue-600 text-white rounded-xl font-medium hover:bg-blue-700 active:scale-95 transition disabled:opacity-50 disabled:cursor-not-allowed"
          >
            下一步：确认执行
          </button>
        </div>
      </div>

      {/* 预览弹窗 */}
      {previewItem && (
        <div
          className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
          onClick={closePreview}
        >
          <div
            className="bg-card/95 backdrop-blur-xl rounded-2xl max-w-2xl w-full max-h-[80vh] flex flex-col shadow-2xl border border-border"
            onClick={(e) => e.stopPropagation()}
          >
            {/* 弹窗头部 */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-border">
              <h3 className="font-semibold text-sm truncate pr-4">
                {previewItem.name}
              </h3>
              <button
                onClick={closePreview}
                className="p-1 rounded-lg hover:bg-muted transition shrink-0"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M18 6 6 18" />
                  <path d="m6 6 12 12" />
                </svg>
              </button>
            </div>

            {/* 弹窗内容 */}
            <div className="flex-1 overflow-hidden p-6 flex items-center justify-center">
              {previewLoading ? (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <svg
                    className="animate-spin h-5 w-5"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  <span className="text-sm">加载中...</span>
                </div>
              ) : previewContent?.startsWith("ERROR:") ? (
                <p className="text-sm">
                  {previewContent.replace("ERROR: ", "")}
                </p>
              ) : previewContent?.startsWith("IMAGE:") ? (
                <img
                  src={convertFileSrc(previewItem.path!)}
                  alt={previewItem.name}
                  className="max-w-full max-h-[60vh] object-contain rounded-lg"
                />
              ) : (
                <pre className="whitespace-pre-wrap break-all text-sm max-h-[60vh] overflow-y-auto p-4 bg-muted/50 rounded-lg w-full">
                  {previewContent}
                </pre>
              )}
            </div>

            {/* 弹窗底部 */}
            <div className="px-6 py-3 border-t border-border flex items-center justify-between gap-4">
              <span className="text-xs text-muted-foreground truncate flex-1">
                {previewItem.path}
              </span>
              <span className="text-xs text-muted-foreground shrink-0">
                {formatBytes(previewItem.size_bytes)}
              </span>
            </div>
          </div>
        </div>
      )}

      {state.error && (
        <p className="mt-4 text-sm text-center">{state.error}</p>
      )}
    </motion.div>
  );
}
