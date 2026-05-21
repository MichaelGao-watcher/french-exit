/**
 * 全局状态管理
 *
 * 使用 React Context + useReducer 实现，不引入第三方状态管理库。
 * 状态设计围绕 orchestrator 的状态机展开，覆盖从输入到报告的完整生命周期。
 */
import React, { createContext, useContext, useReducer, ReactNode } from "react";
import type {
  TraceItem,
  Decision,
  ExecutionReport,
  ResourceConfig,
} from "../types";

/** 前端页面标识，与 orchestrator 状态机对应 */
export type AppPage =
  | "welcome"
  | "input"
  | "scanning"
  | "results"
  | "confirm"
  | "executing"
  | "report";

/** 全局状态结构 */
interface AppState {
  page: AppPage;
  startDate: string;
  scanId: string | null;
  scanResults: TraceItem[];
  scanTotal: number;
  decisions: Map<string, Decision>;
  progressMessage: string;
  progressPercent: number;
  report: ExecutionReport | null;
  resourceConfig: ResourceConfig;
  isScanning: boolean;
  isPaused: boolean;
  error: string | null;
}

/** 所有可派发 action 的联合类型 */
type AppAction =
  | { type: "SET_PAGE"; payload: AppPage }
  | { type: "SET_START_DATE"; payload: string }
  | { type: "SET_SCAN_ID"; payload: string }
  | { type: "SET_SCAN_RESULTS"; payload: { items: TraceItem[]; total: number } }
  | { type: "APPEND_SCAN_RESULTS"; payload: { items: TraceItem[]; total: number } }
  | { type: "SET_DECISION"; payload: Decision }
  | { type: "SET_DECISIONS"; payload: Map<string, Decision> }
  | { type: "SET_PROGRESS"; payload: { message: string; percent: number } }
  | { type: "SET_REPORT"; payload: ExecutionReport }
  | { type: "SET_RESOURCE_CONFIG"; payload: ResourceConfig }
  | { type: "SET_SCANNING"; payload: boolean }
  | { type: "SET_PAUSED"; payload: boolean }
  | { type: "SET_ERROR"; payload: string | null }
  | { type: "RESET" };

/** 初始状态 */
export const initialState: AppState = {
  page: "welcome",
  startDate: "",
  scanId: null,
  scanResults: [],
  scanTotal: 0,
  decisions: new Map(),
  progressMessage: "",
  progressPercent: 0,
  report: null,
  resourceConfig: { cpu_limit_percent: 30, unlimited: false },
  isScanning: false,
  isPaused: false,
  error: null,
};

/**
 * Reducer：纯函数，根据 action 类型更新状态
 */
export function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "SET_PAGE":
      return { ...state, page: action.payload };
    case "SET_START_DATE":
      return { ...state, startDate: action.payload };
    case "SET_SCAN_ID":
      return { ...state, scanId: action.payload };
    case "SET_SCAN_RESULTS":
      return {
        ...state,
        scanResults: action.payload.items,
        scanTotal: action.payload.total,
      };
    case "APPEND_SCAN_RESULTS":
      // 用于流式追加或分页加载，避免重复渲染整列表
      return {
        ...state,
        scanResults: [...state.scanResults, ...action.payload.items],
        scanTotal: action.payload.total,
      };
    case "SET_DECISION": {
      const next = new Map(state.decisions);
      next.set(action.payload.item_id, action.payload);
      return { ...state, decisions: next };
    }
    case "SET_DECISIONS":
      // 批量设置决策，例如应用默认勾选规则后
      return { ...state, decisions: action.payload };
    case "SET_PROGRESS":
      return {
        ...state,
        progressMessage: action.payload.message,
        progressPercent: action.payload.percent,
      };
    case "SET_REPORT":
      return { ...state, report: action.payload };
    case "SET_RESOURCE_CONFIG":
      return { ...state, resourceConfig: action.payload };
    case "SET_SCANNING":
      return { ...state, isScanning: action.payload };
    case "SET_PAUSED":
      return { ...state, isPaused: action.payload };
    case "SET_ERROR":
      return { ...state, error: action.payload };
    case "RESET":
      return initialState;
    default:
      return state;
  }
}

/** Context 类型 */
export const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
} | null>(null);

/**
 * 全局状态 Provider
 * 在 main.tsx 中包裹 <App /> 使用
 */
export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

/**
 * 测试专用 Provider，支持注入初始状态
 * 仅在测试文件中使用
 */
export function TestAppProvider({
  children,
  initialState: override = {},
}: {
  children: ReactNode;
  initialState?: Partial<AppState>;
}) {
  const merged: AppState = { ...initialState, ...override };
  const [state, dispatch] = useReducer(appReducer, merged);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

/**
 * 获取全局状态与 dispatch 的 Hook
 * 必须在 AppProvider 内部使用
 */
export function useAppState() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useAppState must be used within AppProvider");
  return ctx;
}
