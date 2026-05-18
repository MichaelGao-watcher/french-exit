# 任务拆分：INF — 公共基础设施与项目初始化

> 本文件列出所有模块共享的基础设施建设任务。这些任务必须在具体模块开发之前完成，为整个项目提供统一的类型系统、错误处理和工程脚手架。

---

## 前置条件

- [ ] 无（本文件本身就是最前置任务）

## 推荐开发顺序

1. INF-01 ~ INF-03（项目骨架）
2. INF-04 ~ INF-07（类型与错误）
3. INF-08 ~ INF-10（工程化配置）

---

## 子任务清单

### P1 — 必须做

- [ ] **INF-01** 初始化 Tauri 项目骨架（`cargo create-tauri-app`，选择 React + TypeScript）
  - 验证 `cargo tauri dev` 能正常启动空白窗口
  - 验证 `cargo tauri build` 能编译出 `.exe`

- [ ] **INF-02** 配置 Rust 工程结构（workspace 或单 crate 分层目录 `src/`）
  - 目录：`src/commands/`, `src/orchestrator/`, `src/scanner/`, `src/executor/`, `src/reporter/`, `src/resource/`, `src/store/`
  - 每个目录下建 `mod.rs`

- [ ] **INF-03** 配置前端工程（React 18 + TypeScript + Tailwind CSS + shadcn/ui 或等效方案）
  - 配置 PostCSS / Autoprefixer
  - 配置 Apple Design 基础颜色 token（浅色/深色模式 CSS 变量）

- [ ] **INF-04** 定义全局共享数据结构（`src/models.rs` 或 `src/types.rs`）
  - `TraceCategory`, `TraceItem`, `Decision`, `Action`, `ExecutionResult`, `ExecutionStatus`, `ExecutionReport`, `ProgressEvent`, `ScanContext`
  - 所有类型实现 `Serialize` + `Deserialize` + `Clone` + `Debug`

- [ ] **INF-05** 定义全局错误类型（`src/error.rs`）
  - `FrontendError`：前端友好的错误码 + 文案
  - `BackendError`：内部错误，含 `ScanError`, `ExecutionError`, `EraseError`, `ResourceError`, `OrchestratorError`
  - 实现 `From<BackendError> for FrontendError` 转换

- [ ] **INF-06** 配置 Rust 日志与追踪（`tracing` crate，仅内存/stdout，不持久化到磁盘）
  - 配置 `tracing-subscriber` 的 `fmt` layer
  - 日志级别默认 `INFO`，开发时可调 `DEBUG`

- [ ] **INF-07** 定义前端 ↔ 后端 IPC 共享类型文件（`src-tauri/src/types.rs` 映射到前端 `src/types.ts`）
  - 使用 `ts-rs` crate 自动生成 TypeScript 类型（或手写保持同步）
  - 测试点：验证 Rust 枚举映射到 TS union type 无歧义

- [ ] **INF-08** 配置 Git 忽略规则（`.gitignore`）
  - Rust：`target/`, `Cargo.lock`（如作为 lib）
  - Node：`node_modules/`, `dist/`, `.turbo/`
  - Tauri：`src-tauri/target/`, `src-tauri/gen/`
  - 排除敏感文件：`.env`

- [ ] **INF-09** 编写 `README.md` 本地开发指南（如何启动 dev / 如何运行测试 / 如何打包）

- [ ] **INF-10** 配置 Rust Clippy 规则与前端 ESLint/Prettier（编码风格统一）

---

## 测试点

| 测试项 | 方法 |
|--------|------|
| 项目能编译通过 | `cargo check` + `npm run build` |
| Tauri dev 模式正常启动 | `cargo tauri dev` 不报错 |
| 类型序列化一致性 | Rust `serde_json::to_string` ↔ TS `JSON.parse` 互操作 |
| 错误转换全覆盖 | 每个 `BackendError` 变体都能转成 `FrontendError` |

---

## 依赖关系

```
无前置依赖
所有其他模块（M01~M18）都依赖 INF
```

---

## 预计工时

- 1 个 subAgent / 1 次会话可完成 INF-01 ~ INF-05
- 另 1 个 subAgent 可完成 INF-06 ~ INF-10
