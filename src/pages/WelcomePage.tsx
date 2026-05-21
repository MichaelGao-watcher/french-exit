/**
 * 欢迎页（WelcomePage）
 *
 * 应用入口页面，展示副标题和 CTA 按钮。
 * 主标题 "French Exit" 由 App.tsx 的常驻 Logo 提供，带动画过渡。
 */
import { useAppState } from "../store/AppContext";

export function WelcomePage() {
  const { dispatch } = useAppState();

  return (
    <div className="flex flex-col items-center min-h-[80vh] pt-[38vh]">
      <p className="text-lg text-muted-foreground max-w-md mx-auto leading-relaxed text-center mb-12">
        在撤离公用电脑前，安全处理您留下的痕迹
      </p>

      <button
        onClick={() => dispatch({ type: "SET_PAGE", payload: "input" })}
        className="rounded-xl px-10 py-3 font-medium text-white bg-blue-600 hover:bg-blue-700 active:scale-95 shadow-md hover:shadow-lg transition-all duration-200"
      >
        开始使用
      </button>

      <p className="mt-8 text-center text-xs text-muted-foreground">
        所有操作在本地完成，不会上传任何数据
      </p>
    </div>
  );
}
