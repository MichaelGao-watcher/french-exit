import { useAppState } from "../store/AppContext";

export function ReportPage() {
  const { state, dispatch } = useAppState();
  const report = state.report;

  if (!report) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <p className="text-muted-foreground">暂无报告数据</p>
      </div>
    );
  }

  const handleRestart = () => {
    dispatch({ type: "RESET" });
  };

  return (
    <div className="max-w-lg mx-auto py-10 text-center">
      <div className="text-5xl mb-4">🎉</div>
      <h2 className="text-2xl font-semibold mb-2">清理完成</h2>
      <p className="text-muted-foreground mb-8">
        你已完成 French Exit，现在去享受生活吧
      </p>

      {/* 统计卡片 */}
      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="bg-card/80 backdrop-blur-xl rounded-2xl p-4 border border-border">
          <div className="text-2xl font-bold text-green-600">
            {report.deleted_count}
          </div>
          <div className="text-xs text-muted-foreground">已删除</div>
        </div>
        <div className="bg-card/80 backdrop-blur-xl rounded-2xl p-4 border border-border">
          <div className="text-2xl font-bold text-blue-600">
            {report.packed_count}
          </div>
          <div className="text-xs text-muted-foreground">已打包</div>
        </div>
        <div className="bg-card/80 backdrop-blur-xl rounded-2xl p-4 border border-border">
          <div className="text-2xl font-bold text-amber-600">
            {report.preserved_count}
          </div>
          <div className="text-xs text-muted-foreground">已保留</div>
        </div>
      </div>

      {/* 摘要文案 */}
      <div className="bg-card/80 backdrop-blur-xl rounded-2xl p-5 border border-border mb-8 text-left text-sm leading-relaxed">
        <p className="mb-2">
          已删除 <strong>{report.deleted_count}</strong> 条痕迹
          {report.deleted_bytes > 0 && `（共 ${formatBytes(report.deleted_bytes)}）`}
          ，打包 <strong>{report.packed_count}</strong> 个文件
          {report.packed_bytes > 0 && `（共 ${formatBytes(report.packed_bytes)}）`}
          ，保留 <strong>{report.preserved_count}</strong> 条。
        </p>
        {report.pack_file_path && (
          <p className="text-muted-foreground">
            打包文件：{report.pack_file_path}
          </p>
        )}
      </div>

      <div className="text-sm text-muted-foreground mb-6">
        <p>HTML 庆祝页已保存，浏览器即将自动打开</p>
        <p className="text-xs mt-1">如未自动打开，请查看桌面或 zip 同目录</p>
      </div>

      <button
        onClick={handleRestart}
        className="px-8 py-3 bg-blue-600 text-white rounded-xl font-medium hover:bg-blue-700 active:scale-95 transition"
      >
        开始新的清理
      </button>
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
