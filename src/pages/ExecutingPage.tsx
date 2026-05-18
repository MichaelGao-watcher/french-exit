import { useEffect, useRef } from "react";
import { useAppState } from "../store/AppContext";
import { startExecution } from "../api/commands";

export function ExecutingPage() {
  const { state, dispatch } = useAppState();
  const hasStarted = useRef(false);

  useEffect(() => {
    if (hasStarted.current) return;
    hasStarted.current = true;

    startExecution()
      .then((report) => {
        dispatch({ type: "SET_REPORT", payload: report });
        dispatch({ type: "SET_PAGE", payload: "report" });
      })
      .catch((e: any) => {
        dispatch({ type: "SET_ERROR", payload: e.message || "执行失败" });
        dispatch({ type: "SET_PAGE", payload: "confirm" });
      });
  }, [dispatch]);

  return (
    <div className="flex flex-col items-center justify-center min-h-[70vh] max-w-lg mx-auto">
      <div className="w-full bg-card/80 backdrop-blur-xl rounded-2xl p-8 shadow-lg border border-border text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" />
        <h2 className="text-xl font-semibold mb-2">正在执行清理...</h2>
        <p className="text-muted-foreground text-sm">
          正在安全删除文件、打包数据，请稍候
        </p>
        <p className="text-xs text-muted-foreground mt-4">
          此过程可能需要几分钟，取决于文件数量
        </p>

        {state.error && (
          <p className="mt-4 text-sm text-red-500">{state.error}</p>
        )}
      </div>
    </div>
  );
}
