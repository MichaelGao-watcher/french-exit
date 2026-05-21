# French Exit — Vibe Coding Prompt

> **项目定位**：French Exit 是一款面向非技术背景职场人的 Windows 离职清理工具。绿色免安装单文件，双击即运行，用完即走。技术栈：Tauri（Rust backend + WebView2 frontend），完全离线。UI 风格：Apple Design，简洁圆角毛玻璃，跟随系统深色/浅色模式。
>
> **文档依赖**：本 prompt 基于 `docs/proposal.md`、`docs/high-Level Design.md`、`docs/brief.md`、`docs/tasks/*.md` 生成。如输出与 `docs/brief.md` 矛盾，Orchestrator 直接引用 `docs/brief.md` 纠偏。

---

## 0. 现有代码评估

| 评估项 | 结论 |
|--------|------|
| 现有代码状态 | **零代码起步**。项目中只有 `docs/` 目录下的需求/设计文档，无 `src-tauri/`、`src/` 或任何实际代码。 |
| 保留项 | `docs/` 下所有文档作为唯一信源保留，不可覆盖。 |
| 重构项 | 无（没有旧代码）。 |
| 新建项 | **全部新建**：Tauri 项目骨架、Rust 后端 18 个模块、React + TypeScript 前端 5 个页面、测试体系。 |
| 技术栈锁定 | Tauri v2（或最新稳定版）+ React 18 + TypeScript + Tailwind CSS + Rust stable。不可替换为 Electron / 纯网页 / Python。 |

---

## 1. 项目架构

### 1.1 模块列表（18 个模块 + INF 基础设施）

| 模块 ID | 模块名 | 一句话职责 | 开发阶段 |
|---------|--------|-----------|---------|
| `INF` | 公共基础设施 | 类型系统、错误类型、IPC 同步、工程配置 | Phase 1 |
| `M01` | frontend | 渲染 5 个页面，管理状态，调用 Tauri Commands | Phase 4 |
| `M02` | commands | Tauri IPC 入口，参数校验，错误转换 | Phase 4 |
| `M03` | orchestrator | 状态机驱动全流程（Idle→Scanning→Scanned→Confirming→Executing→Completed） | Phase 4 |
| `M04` | scanner-registry | 管理所有 Scanner，按类别调度，聚合结果 | Phase 1 |
| `M05` | scanner-fs | 扫 Desktop、Downloads、微信记录目录 | Phase 2 |
| `M06` | scanner-browser | 扫 Chrome/Edge/Firefox 历史/Cookie/密码/缓存 | Phase 2 |
| `M07` | scanner-chat | 扫微信/QQ/钉钉/飞书/企业微信 | Phase 2 |
| `M08` | scanner-registry-sys | 扫注册表，启发式推断个人信息 | Phase 2 |
| `M09` | scanner-system | 扫系统日志/最近文档/Temp/搜索索引 | Phase 2 |
| `M10` | scanner-devtools | 扫 Git/SSH/IDE/GitHub CLI 配置 | Phase 2 |
| `M11` | scanner-env | 扫用户级环境变量，默认不勾选 | Phase 2 |
| `M12` | executor-delete | 执行删除，调用安全擦除 | Phase 3 |
| `M13` | executor-pack | 打包为 French-exit.zip | Phase 3 |
| `M14` | executor-preserve | 执行保留（无操作，仅记录） | Phase 3 |
| `M15` | secure-erase | DoD 标准安全擦除（3 次覆写） | Phase 1 |
| `M16` | reporter | 生成 HTML 庆祝页 + 调用浏览器打开 | Phase 3 |
| `M17` | resource-ctl | 限制 CPU ≤30%，可手动解除 | Phase 1 |
| `M18` | temp-store | 临时数据管理 + 自毁（JSON Lines 分批落盘） | Phase 1 |

### 1.2 开发阶段与依赖顺序

```
Phase 1（基础设施 + 最底层）
  INF → M18 → M17 → M15 → M04

Phase 2（扫描器集群，可高度并行）
  M05, M06, M07, M08, M09, M10, M11（全部依赖 M04）

Phase 3（执行器 + 报告器）
  M12（依赖 M15）, M13, M14, M16

Phase 4（调度 + IPC + 前端）
  M03（依赖 M04, M12~M14, M16, M17, M18）
  M02（依赖 M03, M17）
  M01（依赖 M02）
```

### 1.3 接口规范（核心契约）

**Scanner trait**（所有扫描器必须实现）：

```rust
pub trait Scanner: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> TraceCategory;
    fn display_name(&self) -> &'static str;
    fn scan(
        &self,
        ctx: &ScanContext,
        pause_rx: &watch::Receiver<bool>,
        progress: &dyn Fn(ScanProgress),
    ) -> Result<Vec<TraceItem>, ScanError>;
}
```

**Executor trait**（所有执行器必须实现）：

```rust
pub trait Executor: Send + Sync {
    fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, ExecutionError>;
}
```

**SecureEraser trait**：

```rust
pub trait SecureEraser: Send + Sync {
    fn erase_file(&self, path: &Path) -> Result<(), EraseError>;
    fn erase_directory(&self, path: &Path) -> Result<(), EraseError>;
}
```

**Orchestrator 状态机**：

```
Idle → Scanning → Scanned → Confirming → Executing → Completed
       ↕ Paused
```

状态转换规则：
- `Idle → Scanning`：用户调用 `start_scan`
- `Scanning ↔ Paused`：`pause_scan` / `resume_scan`
- `Scanning → Scanned`：所有 Scanner 完成
- `Scanned → Confirming`：前端加载结果完成（隐式）
- `Confirming → Executing`：用户提交 `submit_decisions`
- `Executing → Completed`：所有 Executor 完成 + Reporter 生成 HTML
- 任何状态 → `Idle`（取消）

非法转换必须返回 `OrchestratorError::InvalidStateTransition`。

---

## 2. 主Agent（Orchestrator）职责

### 2.1 调度逻辑

1. **阅读输入**：每次会话开始时，主Agent必须先读取 `prompt.md` 和 `docs/brief.md`，确认当前上下文。
2. **任务派发**：根据当前项目状态，从 `docs/tasks/task-*.md` 中识别未完成任务，拆分为 subAgent 可执行的独立任务。
3. **依赖检查**：派发前确认前置依赖模块是否已完成编译和测试。
4. **并行化**：同一 Phase 内无依赖冲突的模块可并行派发多个 subAgent。
5. **结果合并**：subAgent 完成后，主Agent负责集成、解决冲突、运行全量测试。

### 2.2 验收标准（模块级）

每个模块交付时必须满足：
- [ ] `cargo check` 无错误（Rust 侧）
- [ ] `npm run build` 无错误（前端侧）
- [ ] 该模块所有 P1 单元测试通过（`cargo test`）
- [ ] 新代码符合 Rust Clippy 规则 + 前端 ESLint/Prettier
- [ ] 不引入未声明的外部依赖（ Cargo.toml / package.json 变更需说明）
- [ ] 硬规则（RULE-01 ~ RULE-10）在本模块中有体现或测试覆盖

### 2.3 SubAgent 派发模板

主Agent向 subAgent 派发任务时，必须包含以下信息：

```markdown
## 任务背景
- 模块：{Mxx / INF}
- 目标：{实现具体功能}
- 前置依赖：{已完成的模块列表}

## 设计参考
- `docs/high-Level Design.md` 第 X 节（模块详细接口）
- `docs/tasks/task-{模块}.md` 子任务 {XX-01} ~ {XX-NN}

## 已知约束
- {从 docs/brief.md 提取的相关决策}

## 输入
- {相关 trait 定义 / 数据结构设计}

## 输出
- {具体文件路径和接口签名}

## 测试要求
- {必须通过的测试用例}
```

---

## 3. SubAgent 职责

### 3.1 开发流程

1. **先读后写**：实施前必须阅读 `docs/high-Level Design.md` 中对应模块的接口定义和 `docs/tasks/task-*.md` 中的子任务清单。
2. **先接口后实现**：优先定义/确认 trait 和数据结构，再写业务逻辑。
3. **增量提交**：每完成一个子任务（如 SR-01），运行 `cargo check` 或 `npm run build` 验证编译通过。
4. **测试先行**：P1 测试用例必须在业务代码完成后立即编写并验证通过。
5. **文档同步**：如果实现与设计文档中的接口签名不一致，必须记录差异并在返回时报告给主Agent。

### 3.2 代码规范（Rust）

- 错误处理：禁止 `unwrap()` / `expect()` 出现在生产代码路径。使用 `?` 或显式 `match`。
- 异步：使用 `tokio` 作为异步运行时。
- 日志：使用 `tracing` crate，级别默认 `INFO`，不持久化到磁盘。
-  unsafe：尽量减少 unsafe 代码；必须使用时报备主Agent并加详细注释。
- Windows API：所有 Win32 API 调用加 `cfg(windows)` 保护，失败时优雅降级（不 panic）。
- 安全：禁止用 `std::fs::remove_file` 代替 `secure-erase`（RULE-17）。

### 3.3 代码规范（前端）

- 框架：React 18 + TypeScript + Tailwind CSS
- 状态管理：Zustand 或 React Context
- UI 组件：shadcn/ui 或等效方案，必须符合 Apple Design（圆角、毛玻璃、系统主题跟随）
- IPC：所有文件系统/系统操作必须通过 Tauri Commands，禁止前端直接操作文件系统
- 性能：结果列表使用虚拟滚动（`react-window` 或 `@tanstack/react-virtual`），分页 pageSize = 50

### 3.4 禁止事项（零容忍）

| 禁令 | 原因 |
|------|------|
| 禁止在扫描阶段删除任何文件 | RULE-01：所有删除必须在用户最终确认后执行 |
| 禁止默认勾选环境变量条目 | RULE-02：避免误删共享 TOKEN |
| 禁止将 HTML 庆祝页放入 TempStore 目录 | RULE-04：自毁会清理 TempStore，HTML 必须保留 |
| 禁止默认全速扫描（CPU 无限制） | RULE-05：默认必须 ≤30% |
| 禁止在前端直接操作文件系统 | 安全沙箱规则 |
| 禁止用普通删除代替安全擦除 | RULE-17：普通删除可恢复 |
| 禁止修改 `docs/brief.md` 中的已确认决策 | 这些是用户已确认的铁律 |

---

## 4. 测试规范

### 4.1 测试分层

```
┌────────────────────────────────────────┐
│  E2E 测试（可选，V1.0 不强制）           │
├────────────────────────────────────────┤
│  集成测试（Orchestrator 全链路）         │
│  工具：Rust #[test] + tempdir            │
├────────────────────────────────────────┤
│  单元测试（单个模块）                    │
│  工具：Rust #[test] + mockall            │
│  前端：vitest + @testing-library/react   │
└────────────────────────────────────────┘
```

### 4.2 覆盖率要求

| 层级 | 要求 | 说明 |
|------|------|------|
| 单元测试 | **必须覆盖所有 P1 子任务** | 每个子任务至少 1 个对应测试 |
| 错误路径 | **必须覆盖所有 `Err` 分支** | 权限不足、文件锁定、路径不存在等 |
| 硬规则 | **必须单独测试** | RULE-01 ~ RULE-10 每个规则至少 1 个测试用例 |
| 集成测试 | Phase 1/2/3/4 各至少 1 个端到端测试 | 验证模块间接口契约 |

### 4.3 关键硬规则测试用例（必须实现）

| 规则 | 测试用例 | 验证方式 |
|------|---------|---------|
| RULE-01 | 未提交 decisions 时调用 execute → 报错 | `assert!(matches!(err, InvalidStateTransition))` |
| RULE-02 | scanner-env 返回的 TraceItem 默认不勾选 | 验证 `scanner_id == "scanner-env"` 时前端/数据标记为未选中 |
| RULE-03 | 微信 TraceItem `suggested_action = DeleteOrPack` | `assert_eq!(item.suggested_action, Some(Action::DeleteOrPack))` |
| RULE-04 | 自毁后 TempStore 为空，HTML 仍存在 | 断言目录不存在 + HTML 文件存在 |
| RULE-05 | ResourceConfig 默认 `unlimited = false, cpu = 30` | `assert!(!config.unlimited)` |
| RULE-06 | 打包文件名固定为 `French-exit.zip` | 断言输出路径文件名 |
| RULE-07 | 扫描结果不按工作/私人区分 | 验证 TraceItem 无 `work_or_personal` 字段 |
| RULE-08 | Desktop + Downloads 外不逐条展示 | 验证 scanner-fs 不递归 Documents（P1） |
| RULE-09 | HTML 保存位置正确 | 有 pack → zip 同目录；无 pack → 桌面 |
| RULE-10 | 注册表推断结果 `inferred = true` + 风险文案 | `assert!(item.inferred)` + 文案断言 |

### 4.4 TestRunner API 约定

Rust 侧统一使用标准测试框架：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        // Act
        // Assert
    }

    #[test]
    fn test_error_path() {
        // 验证错误分支
    }
}
```

前端侧统一使用 vitest：

```typescript
import { describe, it, expect } from 'vitest';

describe('ComponentName', () => {
  it('should render correctly', () => {
    // Arrange, Act, Assert
  });
});
```

---

## 5. 依赖关系与数据契约

### 5.1 模块调用矩阵

| 调用方 \ 被调用方 | M01 | M02 | M03 | M04 | M05-11 | M12 | M13 | M14 | M15 | M16 | M17 | M18 |
|------------------|:---:|:---:|:---:|:---:|:------:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **M01** frontend | — | ✅ IPC | 📡 事件 | — | — | — | — | — | — | — | — | — |
| **M02** commands | ✅ 返回 | — | ✅ 调用 | — | — | — | — | — | — | — | — | — |
| **M03** orchestrator | 📡 事件 | — | — | ✅ 调用 | ✅ 调用 | ✅ 调度 | ✅ 调度 | ✅ 调度 | — | ✅ 调用 | ✅ 调用 | ✅ 读写 |
| **M04** registry | — | — | 📡 事件 | — | ✅ 调用 | — | — | — | — | — | — | ✅ 落盘 |
| **M05-11** scanners | — | — | 📡 事件 | — | — | — | — | — | — | — | — | ✅ 落盘 |
| **M12** delete | — | — | 📡 事件 | — | — | — | — | — | ✅ 调用 | — | — | — |
| **M13** pack | — | — | 📡 事件+回调 | — | — | — | — | — | — | — | — | — |
| **M14** preserve | — | — | 📡 事件 | — | — | — | — | — | — | — | — | — |

✅ = 直接调用，📡 = 通过事件/通道间接通信

### 5.2 核心数据结构（Rust）

```rust
// 所有类型必须实现 Serialize + Deserialize + Clone + Debug

pub enum TraceCategory {
    Chat, Browser, System, Registry, FileSystem, DevTools, EnvVar,
}

pub struct ScanContext {
    pub start_date: NaiveDate,   // 入职日期
    pub user_home: PathBuf,      // C:\Users\<username>
    pub temp_dir: PathBuf,       // %TEMP%
}

pub struct TraceItem {
    pub id: TraceItemId,              // UUID
    pub category: TraceCategory,
    pub scanner_id: String,
    pub name: String,
    pub path: Option<PathBuf>,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<DateTime<Local>>,
    pub inferred: bool,               // RULE-10
    pub risk_note: Option<String>,
    pub suggested_action: Option<Action>,
}

pub struct Decision {
    pub item_id: TraceItemId,
    pub action: Action,               // Delete / Preserve / Pack
}

pub enum Action { Delete, Preserve, Pack }

pub struct ExecutionResult {
    pub item_id: TraceItemId,
    pub action: Action,
    pub status: ExecutionStatus,
    pub detail: Option<String>,
}

pub enum ExecutionStatus {
    Success,
    Failed(String),
    Skipped(String),
}

pub struct ExecutionReport {
    pub deleted_count: usize,
    pub deleted_bytes: u64,
    pub packed_count: usize,
    pub packed_bytes: u64,
    pub preserved_count: usize,
    pub pack_file_path: Option<PathBuf>,
    pub items: Vec<ExecutionResult>,
}

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

### 5.3 数据持久化规则

| 数据 | 存储位置 | 格式 | 生命周期 |
|------|---------|------|---------|
| 用户配置 | 内存（HashMap） | — | 程序关闭即遗忘 |
| 扫描结果（大结果集） | `%TEMP%/french-exit/results/` | JSON Lines | 自毁时清理 |
| 预览缓存 | `%TEMP%/french-exit/preview/` | 原始文件 | 自毁时清理 |
| 执行中间日志 | `%TEMP%/french-exit/logs/` | 文本 | 自毁时清理 |
| French-exit.zip | 用户指定目录 | ZIP | **永久保留** |
| French-exit-report.html | zip 同目录 / 桌面 | HTML | **永久保留** |

---

## 6. 已知确认事项（来源：docs/brief.md）

以下决策已获用户确认，**不可变更、不可重新询问用户**。如实现与以下条目矛盾，直接按文档修正。

| # | 决策 | 结论 |
|---|------|------|
| 1 | 产品形态 | 绿色免安装小软件，双击运行，无需联网 |
| 2 | 界面风格 | 苹果风格，跟随 Windows 深色/浅色模式 |
| 3 | 删除策略 | **绝不自动删除**。所有删除必须用户最终点确认 |
| 4 | 微信记录 | 直接标记为"建议处理"，不逐条询问 |
| 5 | 文件询问范围 | **仅限桌面和下载文件夹**，其他目录不逐条展示 |
| 6 | 工作/私人区分 | **不区分**。按文件类型列出（文档/图片/视频），用户自行判断 |
| 7 | 环境变量 | **只列出、标注风险，默认不勾选**。需用户手动确认才清除 |
| 8 | 注册表/系统日志 | 程序自动推断疑似条目，**每条标注"由程序推断，请仔细确认"** |
| 9 | 打包文件名 | 固定叫 `French-exit.zip` |
| 10 | 加密文件 | 打包时遇到加密/打不开的文件，**弹窗提醒用户再确认** |
| 11 | 账号退出 | 工具**不能直接退出账号**，而是列出清单 + 一键跳转退出页面 |
| 12 | 资源占用 | 默认限制 CPU 不超过 30%，可手动解除限制 |
| 13 | 扫描暂停 | **随时可以暂停/继续** |
| 14 | 庆祝页 | 清理完成后自动打开浏览器，显示"您已完成 French Exit，现在去享受生活吧" |
| 15 | 庆祝页保存位置 | 有打包 → 放 zip 同目录；没打包 → 放桌面 |
| 16 | 自毁机制 | 程序关闭后自动清理自己的临时文件，**但庆祝页 HTML 保留** |
| 17 | 安全删除 | 采用彻底删除（多次覆盖，无法恢复），不是普通删除 |

---

## 7. 目标目录结构

```
french-exit/
├── docs/                          # 需求/设计文档（已有，保留）
│   ├── proposal.md
│   ├── high-Level Design.md
│   ├── brief.md
│   └── tasks/
│       ├── task-infra.md
│       ├── task-M01.md
│       ├── task-M02.md
│       ├── task-M03.md
│       ├── task-M04.md
│       ├── task-M05.md
│       ├── task-M06.md
│       ├── task-M07.md
│       ├── task-M08.md
│       ├── task-M09.md
│       ├── task-M10.md
│       ├── task-M11.md
│       ├── task-M12.md
│       ├── task-M13.md
│       ├── task-M14.md
│       ├── task-M15.md
│       ├── task-M16.md
│       ├── task-M17.md
│       ├── task-M18.md
│       └── task-progress.md
├── prompt.md                      # 本文件
├── README.md                      # 本地开发指南
├── AGENTS.md                      # Agent 启动指令
├── .gitignore
├── package.json                   # 前端依赖
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── src/                           # 前端源码
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/
│   │   └── commands.ts            # IPC 封装层
│   ├── components/                # UI 组件
│   ├── pages/                     # 5 个核心页面
│   │   ├── InputPage.tsx
│   │   ├── ScanningPage.tsx
│   │   ├── ResultsPage.tsx
│   │   ├── ConfirmPage.tsx
│   │   └── ReportPage.tsx
│   ├── store/                     # 状态管理（Zustand）
│   ├── types.ts                   # 前端类型（与 Rust 同步）
│   ├── styles/                    # 全局样式 + CSS 变量
│   └── tests/                     # 前端测试
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs                # 程序入口
│       ├── lib.rs                 # 库入口（测试用）
│       ├── error.rs               # 全局错误类型（INF-05）
│       ├── models.rs              # 全局数据结构（INF-04）
│       ├── types.rs               # IPC 共享类型（INF-07）
│       ├── commands/
│       │   └── mod.rs             # M02
│       ├── orchestrator/
│       │   └── mod.rs             # M03
│       ├── scanner/
│       │   ├── mod.rs             # M04 trait + registry
│       │   ├── fs.rs              # M05
│       │   ├── browser.rs         # M06
│       │   ├── chat.rs            # M07
│       │   ├── registry_sys.rs    # M08
│       │   ├── system.rs          # M09
│       │   ├── devtools.rs        # M10
│       │   └── env.rs             # M11
│       ├── executor/
│       │   ├── mod.rs             # Executor trait
│       │   ├── delete.rs          # M12
│       │   ├── pack.rs            # M13
│       │   └── preserve.rs        # M14
│       ├── secure_erase/
│       │   └── mod.rs             # M15
│       ├── reporter/
│       │   └── mod.rs             # M16
│       ├── resource/
│       │   └── mod.rs             # M17
│       └── store/
│           └── mod.rs             # M18
└── target/                        # Rust 构建输出（.gitignore）
```

---

## 8. 验收标准（DoD Checklist）

### 8.1 每个模块的 Definition of Done

- [ ] 代码实现完成，通过 `cargo check` / `npm run build`
- [ ] 该模块所有 P1 子任务对应代码已提交
- [ ] 单元测试覆盖所有 P1 子任务和错误路径
- [ ] `cargo test` / `npm test` 全部通过
- [ ] Clippy / ESLint 无错误（warning 需注释说明）
- [ ] 代码符合本文档第 3 节的规范
- [ ] 新增/修改的外部依赖已记录（更新 README.md）
- [ ] 接口实现与 `docs/high-Level Design.md` 一致，差异已记录

### 8.2 每个 Phase 的 Definition of Done

**Phase 1 完成标志**：
- [ ] `cargo tauri dev` 正常启动空白窗口
- [ ] `cargo test` 中 Phase 1 相关测试全部通过
- [ ] Scanner trait 稳定，后续扫描器可直接 impl
- [ ] TempStore 可落盘/读取/自毁（M18）
- [ ] DoDEraser 可安全擦除文件和目录（M15）
- [ ] ResourceController 默认限制 CPU ≤30%（M17）

**Phase 2 完成标志**：
- [ ] 7 个扫描器全部注册到 ScannerRegistry
- [ ] 每个扫描器的单元测试通过
- [ ] 扫描结果可正确写入 TempStore（JSON Lines）
- [ ] 微信条目 `suggested_action = DeleteOrPack`（RULE-03）
- [ ] 环境变量条目默认不勾选标识到位（RULE-02）
- [ ] 注册表推断结果 `inferred = true` + 风险文案（RULE-10）

**Phase 3 完成标志**：
- [ ] `French-exit.zip` 可被标准解压工具打开且内容完整
- [ ] 安全擦除后文件不可恢复（普通恢复工具无法还原）
- [ ] HTML 文件包含主文案和统计数字
- [ ] HTML 保存位置符合 RULE-09（zip 同目录 / 桌面）

**Phase 4 完成标志**：
- [ ] 用户可输入日期 → 开始扫描 → 查看结果 → 勾选决策 → 确认执行 → 看到报告 → 浏览器打开庆祝页
- [ ] 所有 RULE-01 ~ RULE-10 硬规则在 UI 层有体现
- [ ] 扫描可随时暂停/恢复（RULE-13）
- [ ] CPU 默认限制 30%（RULE-05）
- [ ] 自毁后 TempStore 为空，HTML 保留（RULE-04）

### 8.3 V1.0 最终 Definition of Done

- [ ] 全部 18 个模块 + INF 基础设施开发完成
- [ ] 全部 197 个 P1 子任务实现并测试通过
- [ ] 全部 10 条硬规则（RULE-01 ~ RULE-10）有对应的测试用例并通过
- [ ] `cargo tauri build` 能编译出 `.exe`
- [ ] 前端 5 个页面可完整交互，UI 风格符合 Apple Design
- [ ] 端到端流程：输入日期 → 扫描 → 确认 → 执行 → 浏览器打开 HTML 庆祝页
- [ ] `docs/brief.md` 全部 17 项决策在代码中有体现

---

## 9. 决策摘要引用与纠偏机制

### 9.1 纠偏模板

如主Agent或 subAgent 的输出与已确认决策矛盾，任何人可直接使用以下模板纠正：

> "根据 `docs/brief.md` 第 {条目} 条，当前实现与已确认决策不一致。已确认结论是：{结论}。请修正。"

示例：

> "根据 `docs/brief.md` 第 7 条，环境变量相关条目默认应该**不勾选**，而不是自动选中。请修正。"

### 9.2 常见跑偏检查点

| 检查点 | 跑偏信号 | 纠偏依据 |
|--------|---------|---------|
| 删除策略 | AI 建议"静默清理"或"自动删除" | brief.md 第 3 条 |
| 微信处理 | AI 让逐条勾选微信记录 | brief.md 第 4 条 |
| 文件范围 | AI 扫描全盘让您勾选 | brief.md 第 5 条 |
| 环境变量 | AI 默认勾选 TOKEN/PATH 清理 | brief.md 第 7 条 |
| 注册表 | AI 直接清理注册表而不标注推断 | brief.md 第 8 条 |
| 打包文件名 | AI 让自定义文件名 | brief.md 第 9 条 |
| 账号退出 | AI 承诺自动退出微信/浏览器账号 | brief.md 第 11 条 |
| 资源占用 | AI 默认全速扫描导致电脑卡顿 | brief.md 第 12 条 |
| 自毁机制 | AI 清理了 HTML 庆祝页 | brief.md 第 16 条 |
| 安全删除 | AI 用普通删除（`std::fs::remove_file`） | brief.md 第 17 条 |

---

## 10. 快速参考

### 10.1 硬规则速查（RULE-01 ~ RULE-10）

| 规则 ID | 规则内容 | 违反后果 |
|---------|---------|---------|
| RULE-01 | 所有 `Action::Delete` 必须在用户提交最终决策清单后执行 | 零容忍误删 |
| RULE-02 | 来源于 `scanner-env` 的 `TraceItem`，默认决策状态为**未选中** | 避免误删共享 TOKEN |
| RULE-03 | 微信相关 `TraceItem` 的 `suggested_action = DeleteOrPack`，前端默认选中 | 用户已确认微信直接建议处理 |
| RULE-04 | 程序退出时必须调用 `TempStore::self_destruct()`，但 HTML 报告路径**必须排除** | 庆祝页是用户唯一保留物 |
| RULE-05 | 默认启用 CPU ≤30% 限制（`ResourceConfig::unlimited = false`） | 保证用户办公不卡顿 |
| RULE-06 | 打包输出文件名固定为 `French-exit.zip` | 用户已确认 |
| RULE-07 | 扫描结果不按"工作/私人"区分，按类型列出 | 用户确认无法区分 |
| RULE-08 | 需要询问的文件范围**仅限 Desktop 和 Downloads** | 其他目录不逐条展示 |
| RULE-09 | HTML 庆祝页保存位置：有打包则放 zip 同目录，无打包则放桌面 | 用户已确认 |
| RULE-10 | 所有注册表/系统日志的推断结果必须标注 `inferred: true` 和风险提示 | 用户看不懂，需要程序兜底 |

### 10.2 关键数据流（再次强调）

```
扫描阶段：
  Frontend → Commands → Orchestrator → ScannerRegistry → [M05-M11 并行扫描]
                                                           ↓
                                                        TempStore (分批落盘)

确认阶段：
  Frontend ← Commands ← Orchestrator ← TempStore (分页读取)
  Frontend → Commands → Orchestrator (提交 Decisions)

执行阶段：
  Orchestrator → [M12 Delete → M15 SecureErase]
               → [M13 Pack → French-exit.zip]
               → [M14 Preserve (记录)]
               → M16 Reporter (HTML + 打开浏览器)
               → M18 TempStore::self_destruct()
```

---

## 11. 开始工作

如果您是**主Agent**：
1. 读取 `docs/tasks/task-progress.md` 确认当前进度。
2. 根据 Phase 1 → 2 → 3 → 4 的顺序，识别当前可执行的模块。
3. 按第 2.3 节模板向 subAgent 派发任务。
4. 集成结果，更新 `docs/tasks/task-progress.md` 中的进度表。

如果您是**subAgent**：
1. 确认您收到的任务包含：模块 ID、前置依赖状态、设计文档引用、已知约束。
2. 阅读 `docs/high-Level Design.md` 对应模块的接口定义。
3. 阅读 `docs/tasks/task-{模块}.md` 的子任务清单。
4. 按 INF → 具体模块的顺序编写代码，先 trait/类型后实现。
5. 编写并通过所有 P1 测试用例。
6. 返回时汇报：完成了哪些子任务、测试通过情况、与设计文档的差异（如有）。

---

*本 prompt 由阶段四生成，覆盖 docs/proposal.md、docs/high-Level Design.md、docs/brief.md、docs/tasks/*.md 全部内容。新会话读取本文件即可开始执行，无需重新阅读全部设计文档。*
