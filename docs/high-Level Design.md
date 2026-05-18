# French Exit — 概要设计文档（High-Level Design）

> 本文档基于 `docs/proposal.md` 需求提案，定义系统架构、模块划分、接口契约及测试策略。  
> 技术栈：Tauri（Rust + WebView2），绿色免安装，完全离线。  
> 状态：阶段二 · 已确认决策已全部纳入。

---

## 1. 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (WebView2)                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐   │
│  │  输入页  │→│ 扫描页  │→│ 确认页  │→│ 执行/报告页  │   │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘   │
│       ↑            ↑            ↑             ↑            │
│       └────────────┴────────────┴─────────────┘            │
│                    State Store (in-memory)                  │
└─────────────────────────────────────────────────────────────┘
                              │ Tauri Commands (IPC)
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      Backend (Rust)                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Commands   │  │ Orchestrator │  │   Resource Controller│ │
│  │  (IPC入口)   │  │  (流程调度)   │  │   (CPU/内存限制)     │ │
│  └──────┬──────┘  └──────┬──────┘  └─────────────────────┘ │
│         │                │                                   │
│         └────────────────┘                                   │
│                          │                                   │
│         ┌────────────────┼────────────────┐                 │
│         ↓                ↓                ↓                 │
│  ┌────────────┐   ┌────────────┐   ┌────────────┐         │
│  │  Scanner   │   │  Executor  │   │  Reporter  │         │
│  │  (扫描器)   │   │  (执行器)   │   │  (报告器)   │         │
│  └────────────┘   └────────────┘   └────────────┘         │
│         ↑                ↑                                   │
│         └────────────────┘                                   │
│              Config & Temp Store                             │
└─────────────────────────────────────────────────────────────┘
```

**分层说明**：

| 层级 | 职责 | 技术 |
|------|------|------|
| **Frontend** | UI 渲染、用户交互、状态管理 | TypeScript + React（或 Vue） |
| **Commands** | 暴露安全的 Rust 函数给前端调用 | Tauri `#[tauri::command]` |
| **Orchestrator** | 状态机驱动整个流程：扫描 → 确认 → 执行 → 报告 | Rust 状态机 + 通道（Channel） |
| **Scanner** | 发现痕迹，返回结构化数据 | Rust trait + 多实现 |
| **Executor** | 执行用户决策（删/留/打包） | Rust trait + 多实现 |
| **Reporter** | 生成操作报告 + HTML 庆祝页 | Rust HTML 模板引擎 |
| **ResourceCtl** | 限制 CPU/内存占用 | OS API（Windows Job Object） |

---

## 2. 模块划分

### 2.1 模块列表（每个模块只干一件事）

| 模块 ID | 模块名 | 一句话职责 |
|---------|--------|-----------|
| `M01` | **frontend** | 渲染所有页面，收集用户输入，展示扫描结果 |
| `M02` | **commands** | 接收前端 IPC 调用，参数校验，转发给后端服务 |
| `M03` | **orchestrator** | 维护全局状态机，调度扫描/执行/报告流程，处理暂停/恢复/取消 |
| `M04` | **scanner-registry** | 管理所有扫描器实例，按类别分组调度，聚合扫描结果 |
| `M05` | **scanner-fs** | 扫描文件系统中的个人痕迹（Desktop、Downloads、微信记录等） |
| `M06` | **scanner-browser** | 扫描浏览器数据（历史、Cookie、密码、缓存） |
| `M07` | **scanner-chat** | 扫描聊天软件本地数据库（微信、QQ、钉钉、飞书、企业微信） |
| `M08` | **scanner-registry-sys** | 扫描 Windows 注册表中疑似个人信息的条目 |
| `M09` | **scanner-system** | 扫描系统日志、最近文档、Temp、搜索索引、缩略图缓存等 |
| `M10` | **scanner-devtools** | 扫描开发工具链痕迹（Git 配置、SSH、IDE 配置、GitHub CLI 等） |
| `M11` | **scanner-env** | 扫描用户级环境变量中疑似个人相关的 PATH/TOKEN 等 |
| `M12` | **executor-delete** | 对标记为"删除"的条目执行安全擦除 |
| `M13` | **executor-pack** | 对标记为"打包"的条目压缩为 `French-exit.zip` |
| `M14` | **executor-preserve** | 对标记为"保留"的条目执行无操作（仅记录） |
| `M15` | **secure-erase** | 底层安全擦除：多次覆写文件内容，确保不可恢复 |
| `M16` | **reporter** | 汇总操作结果，生成文本报告 + HTML 庆祝页 |
| `M17` | **resource-controller** | 监控并限制进程 CPU ≤30%（默认可解除），内存可控 |
| `M18` | **temp-store** | 临时数据管理：扫描中间结果、预览缓存、自毁时清理 |

---

### 2.2 每个模块的详细职责 + 对外接口

#### `M01` — frontend（前端）

**职责**：
- 渲染 5 个核心页面：入职日期输入、扫描进度、结果清单、最终确认、执行报告
- 管理前端全局状态（React Context / Zustand / Pinia）
- 调用 Tauri Commands 与后端通信
- 响应后端推送的进度事件（SSE/Channel）

**对外接口**（TypeScript，调用后端）：

```typescript
// 启动扫描
function startScan(startDate: string, categories: TraceCategory[]): Promise<void>;

// 暂停/恢复扫描
function pauseScan(): Promise<void>;
function resumeScan(): Promise<void>;

// 获取扫描结果（分页/分类）
function getScanResults(
  category?: TraceCategory,
  page: number,
  pageSize: number
): Promise<PaginatedResult<TraceItem>>;

// 预览某个条目（文本/图片/不支持的提示）
function previewItem(itemId: string): Promise<PreviewResult>;

// 提交用户决策清单
function submitDecisions(decisions: Decision[]): Promise<void>;

// 开始执行（删除/打包）
function startExecution(): Promise<void>;

// 监听进度事件（由后端主动推送）
function onProgress(callback: (event: ProgressEvent) => void): Unsubscribe;
```

---

#### `M02` — commands（IPC 命令层）

**职责**：
- 作为 Frontend 与 Backend 的唯一通道
- 所有入参做合法性校验（日期格式、路径合法性等）
- 异常转换为前端友好的错误码

**对外接口**（Rust，`#[tauri::command]`）：

```rust
#[tauri::command]
async fn start_scan(
    start_date: String,           // ISO 8601: YYYY-MM-DD
    categories: Vec<String>,      // 选中的痕迹类别
    state: tauri::State<'_, AppState>,
) -> Result<ScanId, FrontendError>;

#[tauri::command]
async fn pause_scan(
    scan_id: ScanId,
    state: tauri::State<'_, AppState>,
) -> Result<(), FrontendError>;

#[tauri::command]
async fn resume_scan(
    scan_id: ScanId,
    state: tauri::State<'_, AppState>,
) -> Result<(), FrontendError>;

#[tauri::command]
async fn get_scan_results(
    scan_id: ScanId,
    category: Option<TraceCategory>,
    page: u32,
    page_size: u32,
    state: tauri::State<'_, AppState>,
) -> Result<PaginatedResult<TraceItem>, FrontendError>;

#[tauri::command]
async fn preview_item(
    item_id: TraceItemId,
    state: tauri::State<'_, AppState>,
) -> Result<PreviewResult, FrontendError>;

#[tauri::command]
async fn submit_decisions(
    scan_id: ScanId,
    decisions: Vec<Decision>,      // [{item_id, action: Delete/Preserve/Pack}]
    state: tauri::State<'_, AppState>,
) -> Result<ExecutionPlan, FrontendError>;

#[tauri::command]
async fn start_execution(
    plan_id: ExecutionPlanId,
    output_dir: Option<PathBuf>,   // 打包输出目录，None 则用桌面
    state: tauri::State<'_, AppState>,
) -> Result<(), FrontendError>;

#[tauri::command]
async fn get_resource_config(
    state: tauri::State<'_, AppState>,
) -> Result<ResourceConfig, FrontendError>;

#[tauri::command]
async fn set_resource_config(
    config: ResourceConfig,        // {cpu_limit_percent: u8, unlimited: bool}
    state: tauri::State<'_, AppState>,
) -> Result<(), FrontendError>;
```

---

#### `M03` — orchestrator（流程调度器）

**职责**：
- 维护一个有限状态机（FSM）：`Idle → Scanning → Paused → Scanned → Confirming → Executing → Completed`
- 接收用户指令，驱动 Scanner / Executor / Reporter 按序执行
- 管理扫描任务的暂停/恢复/取消信号（通过 `tokio::sync::watch` 或 `broadcast`）
- 扫描阶段分批聚合结果，避免内存爆炸

**对外接口**：

```rust
pub struct Orchestrator;

impl Orchestrator {
    /// 创建新的扫描会话
    pub async fn start_scan(
        &self,
        ctx: ScanContext,
        progress_tx: mpsc::Sender<ProgressEvent>,
        pause_rx: watch::Receiver<bool>,
    ) -> Result<ScanSession, OrchestratorError>;

    /// 暂停当前会话
    pub async fn pause_session(&self, session_id: SessionId) -> Result<(), OrchestratorError>;

    /// 恢复当前会话
    pub async fn resume_session(&self, session_id: SessionId) -> Result<(), OrchestratorError>;

    /// 提交用户决策，生成执行计划
    pub async fn plan_execution(
        &self,
        session_id: SessionId,
        decisions: Vec<Decision>,
    ) -> Result<ExecutionPlan, OrchestratorError>;

    /// 执行计划
    pub async fn execute_plan(
        &self,
        plan: ExecutionPlan,
        output_dir: PathBuf,
        progress_tx: mpsc::Sender<ProgressEvent>,
    ) -> Result<ExecutionReport, OrchestratorError>;

    /// 获取当前会话状态
    pub fn session_state(&self, session_id: SessionId) -> Option<SessionState>;
}
```

---

#### `M04` — scanner-registry（扫描器注册中心）

**职责**：
- 持有所有 Scanner 实例列表
- 根据用户勾选的类别，过滤出需要运行的扫描器
- 逐个或并行调度扫描器，聚合结果
- 统一处理扫描器抛出的错误（某个扫描器失败不中断整体流程）

**对外接口**：

```rust
/// 扫描器统一 trait
pub trait Scanner: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> TraceCategory;
    fn display_name(&self) -> &'static str;

    /// 核心扫描方法
    fn scan(
        &self,
        ctx: &ScanContext,
        pause_rx: &watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, ScanError>;
}

pub struct ScannerRegistry {
    scanners: Vec<Box<dyn Scanner>>,
}

impl ScannerRegistry {
    pub fn new() -> Self;
    
    /// 注册一个扫描器
    pub fn register(&mut self, scanner: Box<dyn Scanner>);

    /// 按类别过滤并执行扫描
    pub async fn run_selected(
        &self,
        categories: &[TraceCategory],
        ctx: &ScanContext,
        pause_rx: watch::Receiver<bool>,
        progress_tx: mpsc::Sender<ProgressEvent>,
    ) -> Vec<ScanResultBundle>;
}
```

---

#### `M05` — scanner-fs（文件系统扫描器）

**职责**：
- 扫描 `Desktop`、`Documents`、`Downloads` 中入职日期后的新增/修改文件
- 特殊处理：微信聊天记录目录直接标记为"建议处理"，不按类型拆分
- 返回文件元数据：路径、大小、修改时间、MIME 类型推断

**不扫描**：Windows 系统目录、Program Files、French Exit 自身目录

**对外接口**：实现 `Scanner` trait，无额外公共接口。

```rust
pub struct FileSystemScanner {
    // 配置项：额外包含/排除路径
    include_paths: Vec<PathBuf>,
    exclude_paths: Vec<PathBuf>,
}

impl Scanner for FileSystemScanner { /* ... */ }
```

---

#### `M06` — scanner-browser（浏览器扫描器）

**职责**：
- 检测系统中安装的浏览器（Chrome、Edge、Firefox 等）
- 扫描各浏览器的用户数据目录：历史记录、Cookie、保存的密码、缓存
- 识别浏览器账号登录状态，为"账号退出清单"提供数据来源

**对外接口**：实现 `Scanner` trait。

```rust
pub struct BrowserScanner;
impl Scanner for BrowserScanner { /* ... */ }
```

---

#### `M07` — scanner-chat（聊天软件扫描器）

**职责**：
- 检测并扫描：微信、QQ、钉钉、飞书、企业微信的本地数据目录
- 提取数据库文件、图片/视频缓存、文件传输助手接收的文件
- 微信记录：整目录标记为 `suggested_action: Action::DeleteOrPack`，前端直接展示为"建议处理"

**对外接口**：实现 `Scanner` trait。

```rust
pub struct ChatScanner;
impl Scanner for ChatScanner { /* ... */ }
```

---

#### `M08` — scanner-registry-sys（注册表扫描器）

**职责**：
- 扫描 `HKEY_CURRENT_USER` 下入职日期后修改的键值
- 按时间 + 键名 + 值内容做启发式推断，筛选疑似含个人信息的条目
- 每个结果标注 `inferred: true` 和警告文案

**对外接口**：实现 `Scanner` trait。

```rust
pub struct RegistryScanner;
impl Scanner for RegistryScanner { /* ... */ }
```

---

#### `M09` — scanner-system（系统痕迹扫描器）

**职责**：
- 扫描：最近打开文档列表、Temp 文件夹、事件查看器日志、搜索索引、缩略图缓存、休眠文件、系统还原点
- 过滤出入职日期之后产生/修改的条目

**对外接口**：实现 `Scanner` trait。

```rust
pub struct SystemTraceScanner;
impl Scanner for SystemTraceScanner { /* ... */ }
```

---

#### `M10` — scanner-devtools（开发工具扫描器）

**职责**：
- 扫描：Git 全局配置（`~/.gitconfig`）、SSH 密钥（`~/.ssh/`）、IDE 配置（VS Code、JetBrains）、GitHub CLI 配置
- 区分"安全清除"（仅配置文件）和"有风险"（环境变量）

**对外接口**：实现 `Scanner` trait。

```rust
pub struct DevToolsScanner;
impl Scanner for DevToolsScanner { /* ... */ }
```

---

#### `M11` — scanner-env（环境变量扫描器）

**职责**：
- 扫描用户级环境变量，识别与已知工具相关的条目（`GH_TOKEN`、`GITHUB_TOKEN`、工具 PATH 等）
- 明确标注风险："⚠️ 可能与其他工具共用"
- **默认不自动勾选**，需用户手动确认

**对外接口**：实现 `Scanner` trait。

```rust
pub struct EnvVarScanner;
impl Scanner for EnvVarScanner { /* ... */ }
```

---

#### `M12` — executor-delete（删除执行器）

**职责**：
- 对标记为 `Action::Delete` 的条目，调用 `secure-erase` 模块进行彻底删除
- 文件：直接安全擦除
- 注册表项：调用 Windows API 删除
- 记录操作结果（成功/失败/路径）

**对外接口**：

```rust
pub struct DeleteExecutor {
    secure_eraser: Arc<dyn SecureEraser>,
}

impl Executor for DeleteExecutor {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, ExecutionError>;
}
```

---

#### `M13` — executor-pack（打包执行器）

**职责**：
- 收集所有标记为 `Action::Pack` 的条目
- 打包为 `French-exit.zip`，保留原始目录结构（相对路径）
- 遇到加密/不可读文件时，先弹窗提醒（通过 Orchestrator 回调 Frontend），用户确认后再写入
- 输出到用户指定的目录

**对外接口**：

```rust
pub struct PackExecutor {
    output_path: PathBuf,
    on_encrypted: Box<dyn Fn(&Path) -> bool + Send + Sync>, // 回调：遇到加密文件时询问用户
}

impl Executor for PackExecutor {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, ExecutionError>;
    
    /// 所有条目处理完成后，finalize 生成 zip 文件
    fn finalize(&self) -> Result<PathBuf, ExecutionError>;
}
```

---

#### `M14` — executor-preserve（保留执行器）

**职责**：
- 对标记为 `Action::Preserve` 的条目执行无操作
- 仅记录"用户选择保留"，用于最终报告

**对外接口**：

```rust
pub struct PreserveExecutor;

impl Executor for PreserveExecutor {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, ExecutionError>;
}
```

---

#### `M15` — secure-erase（安全擦除）

**职责**：
- 对文件进行多次覆写（DoD 5220.22-M 标准：3 次覆写，或用户可配置）
- 覆写后重命名并删除
- 对目录：递归安全擦除所有子文件，然后删除空目录

**对外接口**：

```rust
pub trait SecureEraser: Send + Sync {
    /// 安全擦除单个文件
    fn erase_file(&self, path: &Path) -> Result<(), EraseError>;
    
    /// 安全擦除整个目录
    fn erase_directory(&self, path: &Path) -> Result<(), EraseError>;
}

pub struct DoDEraser;
impl SecureEraser for DoDEraser { /* ... */ }
```

---

#### `M16` — reporter（报告生成器）

**职责**：
- 汇总 ExecutionReport：删除数量/大小、打包数量/大小、保留数量
- 生成文本摘要（用于弹窗和日志）
- 生成 HTML 庆祝页：主文案 + 统计卡片 + 清单明细
- HTML 保存位置：有打包则放 `French-exit.zip` 同目录，无打包则放桌面
- 清理完成后自动调用系统浏览器打开 HTML

**对外接口**：

```rust
pub struct Reporter;

impl Reporter {
    /// 生成操作摘要（文本）
    pub fn generate_summary(&self, report: &ExecutionReport) -> String;

    /// 生成 HTML 庆祝页，返回写入的路径
    pub fn generate_celebration_html(
        &self,
        report: &ExecutionReport,
        output_dir: &Path,
    ) -> Result<PathBuf, ReporterError>;

    /// 调用系统默认浏览器打开文件
    pub fn open_in_browser(&self, path: &Path) -> Result<(), ReporterError>;
}
```

---

#### `M17` — resource-controller（资源控制器）

**职责**：
- 默认限制当前进程及其子进程的 CPU 使用率 ≤30%
- 使用 Windows Job Object API 实现限制
- 提供"解除限制"模式：取消 Job Object 限制，全速运行
- 监控内存占用，防止扫描大量文件时 OOM

**对外接口**：

```rust
pub struct ResourceController;

impl ResourceController {
    pub fn new() -> Self;
    
    /// 应用限制（默认启动时调用）
    pub fn apply_limits(&self, config: ResourceConfig) -> Result<(), ResourceError>;
    
    /// 解除所有限制
    pub fn remove_limits(&self) -> Result<(), ResourceError>;
    
    /// 获取当前资源使用情况
    pub fn current_usage(&self) -> ResourceUsage;
}

pub struct ResourceConfig {
    pub cpu_limit_percent: u8,  // 1-100
    pub unlimited: bool,        // true 时忽略 cpu_limit_percent
}
```

---

#### `M18` — temp-store（临时数据管理）

**职责**：
- 管理 `%TEMP%/french-exit/` 目录下的所有临时文件
- 扫描中间结果（大型结果集落盘，避免内存爆炸）
- 预览缓存（临时解压的聊天记录图片等）
- **自毁机制**：程序退出时自动清理本目录下所有内容
- HTML 庆祝页**不放在此处**，避免被自毁误删

**对外接口**：

```rust
pub struct TempStore {
    root: PathBuf,
}

impl TempStore {
    pub fn new() -> Result<Self, std::io::Error>;
    
    /// 分配一个临时文件/目录
    pub fn allocate(&self, prefix: &str) -> Result<PathBuf, std::io::Error>;
    
    /// 扫描结果分批持久化（JSON Lines 格式）
    pub fn save_scan_batch(&self, batch: &[TraceItem]) -> Result<(), std::io::Error>;
    
    /// 按批次读取扫描结果（支持分页）
    pub fn load_scan_results(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<TraceItem>, std::io::Error>;
    
    /// 自毁：删除所有临时文件
    pub fn self_destruct(&self) -> Result<(), std::io::Error>;
}

/// 在程序退出时自动调用
impl Drop for TempStore {
    fn drop(&mut self) {
        let _ = self.self_destruct();
    }
}
```

---

## 3. 模块间调用关系矩阵

> 行 = 调用方，列 = 被调用方，✅ = 直接调用，📡 = 通过事件/通道间接通信

| 调用方 \ 被调用方 | M01 frontend | M02 commands | M03 orchestrator | M04 scanner-registry | M05-11 scanners | M12 delete | M13 pack | M14 preserve | M15 secure-erase | M16 reporter | M17 resource-ctl | M18 temp-store |
|-------------------|:------------:|:------------:|:----------------:|:--------------------:|:---------------:|:----------:|:--------:|:------------:|:----------------:|:------------:|:----------------:|:--------------:|
| **M01** frontend  | — | ✅ IPC调用 | 📡 接收事件 | — | — | — | — | — | — | — | — | — |
| **M02** commands  | ✅ 返回结果 | — | ✅ 同步/异步调用 | — | — | — | — | — | — | — | — | — |
| **M03** orchestrator | 📡 推送事件 | — | — | ✅ 调度扫描 | ✅ 逐个调用 | ✅ 调度 | ✅ 调度 | ✅ 调度 | — | ✅ 生成报告 | ✅ 应用限制 | ✅ 读写临时数据 |
| **M04** scanner-registry | — | — | 📡 回传进度 | — | ✅ 并行/串行调用 | — | — | — | — | — | — | — |
| **M05-11** scanners | — | — | 📡 回传进度 | — | — | — | — | — | — | — | — | ✅ 大结果落盘 |
| **M12** delete-executor | — | — | 📡 回传结果 | — | — | — | — | — | ✅ 调用擦除 | — | — | — |
| **M13** pack-executor | — | — | 📡 回传结果+回调询问 | — | — | — | — | — | — | — | — | — |
| **M14** preserve-executor | — | — | 📡 回传结果 | — | — | — | — | — | — | — | — | — |
| **M16** reporter | — | — | — | — | — | — | — | — | — | — | — | — |
| **M17** resource-ctl | — | — | — | — | — | — | — | — | — | — | — | — |
| **M18** temp-store | — | — | — | — | — | — | — | — | — | — | — | — |

**关键调用链**：

```
1. 扫描链：
   M01 → M02 → M03 → M04 → (M05|M06|M07|M08|M09|M10|M11)
                          ↓
                        M18 (大结果落盘)

2. 确认链：
   M01 ← M02 ← M03 ← M18 (分页读取结果)
   M01 → M02 → M03 (提交 decisions)

3. 执行链：
   M03 → M12/M13/M14 (按决策类型分发)
   M12 → M15 (安全擦除)
   M13 → 磁盘 (生成 zip)

4. 报告链：
   M03 → M16 (生成摘要 + HTML)
   M16 → 系统 API (打开浏览器)

5. 自毁链：
   M03 → M18 (self_destruct)
```

---

## 4. 核心数据结构

```rust
/// 痕迹类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceCategory {
    Chat,           // 聊天记录
    Browser,        // 浏览器
    System,         // 系统痕迹
    Registry,       // 注册表
    FileSystem,     // 个人文件
    DevTools,       // 开发工具
    EnvVar,         // 环境变量
}

/// 扫描上下文
#[derive(Debug, Clone)]
pub struct ScanContext {
    pub start_date: NaiveDate,       // 入职日期
    pub user_home: PathBuf,          // C:\Users\<username>
    pub temp_dir: PathBuf,           // %TEMP%
}

/// 单条痕迹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceItem {
    pub id: TraceItemId,             // UUID
    pub category: TraceCategory,
    pub scanner_id: String,          // 来源扫描器
    pub name: String,                // 展示名称
    pub path: Option<PathBuf>,       // 文件/注册表路径
    pub size_bytes: Option<u64>,
    pub modified_at: Option<DateTime<Local>>,
    pub inferred: bool,              // 是否为程序推断
    pub risk_note: Option<String>,   // 风险提示文案
    pub suggested_action: Option<Action>, // 扫描器建议的处理方式
}

/// 用户决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub item_id: TraceItemId,
    pub action: Action,
}

/// 处理方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Delete,     // 彻底删除
    Preserve,   // 保留
    Pack,       // 打包带走
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub item_id: TraceItemId,
    pub action: Action,
    pub status: ExecutionStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failed(String),      // 错误原因
    Skipped(String),     // 跳过原因（如用户取消加密文件）
}

/// 执行报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub deleted_count: usize,
    pub deleted_bytes: u64,
    pub packed_count: usize,
    pub packed_bytes: u64,
    pub preserved_count: usize,
    pub pack_file_path: Option<PathBuf>,
    pub items: Vec<ExecutionResult>,
}

/// 进度事件（推送到前端）
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ProgressEvent {
    ScanStarted { total_scanners: usize },
    ScanProgress { scanner_id: String, current: usize, total: usize, message: String },
    ScanCompleted { item_count: usize },
    ScanPaused,
    ScanResumed,
    ExecutionStarted { total_items: usize },
    ExecutionProgress { current: usize, total: usize, message: String },
    ExecutionCompleted { report: ExecutionReport },
}
```

---

## 5. 数据持久化方案

### 5.1 不持久化（用完即走）

| 数据 | 存储位置 | 说明 |
|------|---------|------|
| **用户配置** | 内存（HashMap） | 不写入注册表、不写入磁盘配置文件 |
| **入职日期** | 内存 + 单次会话 | 程序关闭即遗忘 |
| **扫描参数** | 内存 | 同上 |

### 5.2 临时持久化（自毁时清理）

| 数据 | 存储位置 | 格式 | 说明 |
|------|---------|------|------|
| **扫描结果（大结果集）** | `%TEMP%/french-exit/results/` | JSON Lines | 扫描器分批写入，前端分页读取 |
| **预览缓存** | `%TEMP%/french-exit/preview/` | 原始文件 | 聊天记录图片临时解压 |
| **执行中间日志** | `%TEMP%/french-exit/logs/` | 文本 | 每一步操作的详细记录 |

**自毁触发时机**：
1. 正常流程：报告生成并打开浏览器后，调用 `M18::self_destruct()`
2. 异常退出：`TempStore` 实现 `Drop` trait，确保进程退出时自动清理
3. **例外**：HTML 庆祝页**不保存在临时目录**，避免被误删

### 5.3 永久输出（保留给用户）

| 数据 | 存储位置 | 说明 |
|------|---------|------|
| **`French-exit.zip`** | 用户指定目录 | 打包带走的个人文件 |
| **`French-exit-report.html`** | 与 zip 同目录；无打包则放桌面 | 庆祝页，永久保留 |

---

## 6. 测试策略

### 6.1 测试分层

```
┌────────────────────────────────────────┐
│  E2E 测试（Tauri + 前端完整流程）        │  ← 模拟完整用户操作
│  工具：WebDriver / Playwright + Tauri    │
├────────────────────────────────────────┤
│  集成测试（Backend 端到端）              │  ← Orchestrator 全链路
│  工具：Rust #[test] + tempdir            │
├────────────────────────────────────────┤
│  单元测试（单个模块）                    │  ← 每个 Scanner / Executor
│  工具：Rust #[test] + mockall            │
└────────────────────────────────────────┘
```

### 6.2 各模块测试方案

| 模块 | 测试类型 | 测试方法 | Mock 策略 |
|------|---------|---------|----------|
| `M03` orchestrator | 集成测试 | 构造假 Scanner/Executor，验证状态机流转 | mock scanner-registry、executor |
| `M04` scanner-registry | 单元测试 | 注入多个假 Scanner，验证调度顺序和聚合 | mock Scanner trait |
| `M05` scanner-fs | 单元测试 | 在临时目录构造已知文件树，验证扫描结果 | 虚拟文件系统（tempdir） |
| `M06` scanner-browser | 单元测试 | 构造假浏览器配置目录，验证检测逻辑 | 虚拟目录 + 假数据库文件 |
| `M08` scanner-registry-sys | 单元测试 | 仅测试推断算法（不真读注册表） | 构造假注册表键值数据 |
| `M12` executor-delete | 单元测试 | 在 tempdir 创建文件，执行后尝试恢复验证不可恢复 | 虚拟文件 |
| `M13` executor-pack | 单元测试 | 打包后解压验证内容完整性 | 虚拟文件 |
| `M15` secure-erase | 单元测试 | 覆写后读取文件验证内容非原始数据 | 虚拟文件 |
| `M16` reporter | 单元测试 | 注入假 ExecutionReport，验证 HTML 输出包含预期文案 | mock 数据 |
| `M17` resource-ctl | 单元测试 | 验证 API 调用参数正确（不真限制 CPU） | mock Windows API |
| `M18` temp-store | 单元测试 | 创建/读取/自毁流程验证 | tempdir |

### 6.3 关键测试用例

1. **扫描暂停/恢复**：在 scanner 运行中途发送 pause 信号，验证 scanner 正确暂停，resume 后从断点继续
2. **微信记录建议处理**：scanner-chat 返回的微信相关 TraceItem 必须 `suggested_action = Some(Action::DeleteOrPack)`
3. **环境变量不自动勾选**：scanner-env 返回的 TraceItem 在 Frontend 默认状态为未选中
4. **加密文件弹窗**：executor-pack 遇到加密文件时，必须触发 `on_encrypted` 回调，用户取消后标记为 `Skipped`
5. **自毁完整性**：程序退出后，`%TEMP%/french-exit/` 目录必须为空或不存在
6. **HTML 保留**：自毁后，HTML 文件必须仍然存在于目标目录

---

## 7. 已确认决策清单

> 以下决策全部来自 `docs/proposal.md` 阶段一确认，设计已完全遵循。

| # | 决策项 | 设计方案体现 |
|---|--------|-------------|
| 1 | 技术栈：Tauri（Rust + WebView2） | 架构总览、M01-M02 IPC 设计 |
| 2 | 绿色免安装、完全离线 | 无安装逻辑、无网络请求模块 |
| 3 | UI 风格：Apple Design，跟随系统 | Frontend 实现层面（非本文档重点） |
| 4 | 扫描范围：20+ 类痕迹 | M05-M11 七个扫描器覆盖全部类别 |
| 5 | 微信记录直接建议处理 | `TraceItem::suggested_action` + 前端默认选中 |
| 6 | 需询问的文件仅限 Desktop + Downloads | `M05` 仅扫描这两个目录（微信除外） |
| 7 | 打包格式：`French-exit.zip` | `M13` 输出固定文件名 |
| 8 | 加密文件弹窗提醒 | `M13::on_encrypted` 回调机制 |
| 9 | 账号退出：列出清单 + 一键跳转 | `M16` 在报告中生成"待退出账号"列表和链接 |
| 10 | 默认彻底删除，执行前弹窗 + 最终清单 | `M03` 状态机在 `Confirming` 阶段阻塞，等用户最终确认后才进入 `Executing` |
| 11 | 资源占用：默认 CPU ≤30%，可解除 | `M17` 默认限制，M01/M02 提供解除接口 |
| 12 | 扫描随时可暂停 | `M03` pause/resume + `watch::Receiver<bool>` |
| 13 | 庆祝页文案 + 自动弹浏览器 | `M16` 生成 HTML + 调用系统浏览器 |
| 14 | HTML 保存位置：zip 同目录 / 桌面 | `M16::generate_celebration_html` 的 `output_dir` 参数逻辑 |
| 15 | 自毁机制：保留 HTML，其余清理 | `M18` Drop trait + 自毁范围排除 HTML |
| 16 | GitHub CLI 配置文件安全清，环境变量不自动勾选 | `M10` 区分配置/环境变量，`M11` 环境变量默认 `inferred + risk_note` |
| 17 | 零容忍误删，最终确认清单 | `M03` `Confirming` 阶段必须用户显式提交 decisions 才能继续 |

---

## 8. 风险与限制

| 风险点 | 影响 | 缓解措施 |
|--------|------|---------|
| 某些聊天软件数据库被占用（进程运行中） | 扫描/删除失败 | Scanner 检测到文件被占用时标记 `risk_note = "软件正在运行，请先退出"`，不强行操作 |
| 注册表/系统日志扫描需要管理员权限 | 部分条目无法读取 | Scanner 捕获权限错误，标记为"需要管理员权限运行"，不中断流程 |
| 安全擦除大文件（如微信聊天记录 10GB+）耗时极长 | 用户以为卡死 | `M15` 提供进度回调，`M03` 实时推送到前端显示"正在安全擦除 xxx / 共 yyy" |
| 打包输出目录磁盘空间不足 | zip 生成失败 | `M13` 预先计算所需空间，不足时提前报错，让用户更换目录 |
| 扫描结果过多（百万级文件）导致前端卡顿 | UI 无响应 | `M18` 分批落盘，`M02::get_scan_results` 分页返回，前端虚拟滚动 |

---

## 9. 下一步（阶段三）

1. **详细设计（Low-Level Design）**：每个 Scanner/Executor 的内部算法、错误处理、边界条件
2. **技术选型确认**：Frontend 框架（React vs Vue）、Rust 异步运行时细节、HTML 模板引擎
3. **原型搭建**：先实现 `M05 scanner-fs` + `M12 executor-delete` + `M16 reporter` 的最小可用链路
