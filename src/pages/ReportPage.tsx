import { useAppState } from "../store/AppContext";
import { formatBytes } from "../utils/format";

export function ReportPage() {
  const { state } = useAppState();
  const report = state.report;

  if (!report) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <p className="text-muted-foreground">暂无报告数据</p>
      </div>
    );
  }

  return (
    <div className="max-w-lg mx-auto flex flex-col items-center justify-center min-h-[70vh] text-center px-4">
      {/* 主文案 - 占主要空间 */}
      <div className="flex-1 flex flex-col items-center justify-center">
        <h1 className="text-4xl font-semibold tracking-tight mb-4">
          清理完成
        </h1>
        <p className="text-lg text-muted-foreground">
          您已完成 French Exit，现在去享受生活吧
        </p>
      </div>

      {/* 底部明细 - 最下方小字 */}
      <div className="mt-auto pt-16 pb-8 text-xs text-muted-foreground space-y-1">
        <p>
          已删除 {report.deleted_count} 条
          {report.deleted_bytes > 0 && `（${formatBytes(report.deleted_bytes)}）`}
          {" · "}
          已打包 {report.packed_count} 个
          {report.packed_bytes > 0 && `（${formatBytes(report.packed_bytes)}）`}
          {" · "}
          已保留 {report.preserved_count} 条
        </p>
        {report.pack_file_path && (
          <p>打包文件：{report.pack_file_path}</p>
        )}
        <p>HTML 庆祝页已保存</p>
      </div>
    </div>
  );
}


