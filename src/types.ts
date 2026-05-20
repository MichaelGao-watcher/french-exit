export type TraceCategory =
  | "Chat"
  | "Browser"
  | "System"
  | "Registry"
  | "FileSystem"
  | "DevTools"
  | "EnvVar";

export type Action = "Delete" | "Preserve" | "Pack" | "DeleteOrPack";

export type TraceItemId = string;
export type ScanId = string;
export type ExecutionPlanId = string;
export type SessionId = string;

export interface ScanResultSummary {
  id: TraceItemId;
  category: TraceCategory;
  suggested_action: Action | null;
}

export interface TraceItem {
  id: TraceItemId;
  category: TraceCategory;
  scanner_id: string;
  name: string;
  path: string | null;
  size_bytes: number | null;
  modified_at: string | null;
  inferred: boolean;
  risk_note: string | null;
  suggested_action: Action | null;
}

export interface Decision {
  item_id: TraceItemId;
  action: Action;
}

export type ExecutionStatus =
  | { type: "Success" }
  | { type: "Failed"; data: string }
  | { type: "Skipped"; data: string };

export interface ExecutionResult {
  item_id: TraceItemId;
  action: Action;
  status: ExecutionStatus;
  detail: string | null;
}

export interface ExecutionReport {
  deleted_count: number;
  deleted_bytes: number;
  packed_count: number;
  packed_bytes: number;
  preserved_count: number;
  pack_file_path: string | null;
  items: ExecutionResult[];
}

export type ProgressEvent =
  | { type: "ScanStarted"; total_scanners: number }
  | { type: "ScanProgress"; scanner_id: string; current: number; total: number; message: string }
  | { type: "ScanCompleted"; item_count: number }
  | { type: "ScanFailed"; reason: string }
  | { type: "ScanPaused" }
  | { type: "ScanResumed" }
  | { type: "ExecutionStarted"; total_items: number }
  | { type: "ExecutionProgress"; current: number; total: number; message: string }
  | { type: "ExecutionCompleted"; report: ExecutionReport };

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface ResourceConfig {
  cpu_limit_percent: number;
  unlimited: boolean;
}

export type PreviewResult =
  | { type: "Text"; data: string }
  | { type: "Image"; data: string }
  | { type: "Unsupported"; data: string };
