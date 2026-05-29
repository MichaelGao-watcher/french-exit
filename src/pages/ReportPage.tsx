import { motion } from "framer-motion";
import { useAppState } from "../store/AppContext";
import { formatBytes } from "../utils/format";

export function ReportPage() {
  const { state } = useAppState();
  const report = state.report;

  const container = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.15,
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
        duration: 0.8,
        ease: [0.4, 0, 0.2, 1],
      },
    },
  };

  if (!report) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <p className="text-muted-foreground">暂无报告数据</p>
      </div>
    );
  }

  return (
    <motion.div
      className="max-w-lg mx-auto flex flex-col items-center justify-center min-h-[70vh] text-center px-4"
      variants={container}
      initial="hidden"
      animate="show"
    >
      {/* 主文案 - 占主要空间 */}
      <motion.div variants={item} className="flex-1 flex flex-col items-center justify-center">
        <h1 className="text-4xl font-semibold tracking-tight mb-4">
          清理完成
        </h1>
        <p className="text-lg text-muted-foreground">
          您已完成 French Exit，现在去享受生活吧
        </p>
      </motion.div>

      {/* 底部明细 - 最下方小字 */}
      <motion.div variants={item} className="mt-auto pt-16 pb-8 text-xs text-muted-foreground space-y-1">
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
      </motion.div>
    </motion.div>
  );
}


