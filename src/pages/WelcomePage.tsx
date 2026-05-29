/**
 * 欢迎页（WelcomePage）
 *
 * 应用入口页面，展示副标题和 CTA 按钮。
 * 主标题 "French Exit" 由 App.tsx 的常驻 Logo 提供，带动画过渡。
 */
import { motion } from "framer-motion";
import { useAppState } from "../store/AppContext";

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.15,
      delayChildren: 0.8,
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

export function WelcomePage() {
  const { dispatch } = useAppState();

  return (
    <motion.div
      className="flex flex-col items-center min-h-[80vh] pt-[38vh]"
      variants={container}
      initial="hidden"
      animate="show"
    >
      <motion.p
        className="text-lg text-muted-foreground max-w-md mx-auto leading-relaxed text-center mb-12"
        variants={item}
      >
        在撤离公用电脑前，安全处理您留下的痕迹
      </motion.p>

      <motion.button
        onClick={() => dispatch({ type: "SET_PAGE", payload: "input" })}
        className="rounded-full px-10 py-3 font-light text-foreground border border-white/20
          hover:bg-white hover:text-black
          active:scale-[0.98] active:shadow-[inset_0_2px_4px_rgba(0,0,0,0.3)]
          transition-all duration-300 ease-out"
        variants={item}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
      >
        开始使用
      </motion.button>

      <motion.p
        className="mt-8 text-center text-xs text-muted-foreground font-light"
        variants={item}
      >
        所有操作在本地完成，不会上传任何数据
      </motion.p>
    </motion.div>
  );
}
