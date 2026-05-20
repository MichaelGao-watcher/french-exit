# French Exit — 决策日志

> 记录本项目的关键设计决策。当有人质疑"为什么这样选"时，来这里找答案，不要重复争论。
>
> 格式：日期 → 问题 → 决策 → 理由 → 后果（可逆性）。

---

## ADR-001：为什么用 Tauri（Rust + WebView2）而非 Electron？

| 字段 | 内容 |
|------|------|
| **日期** | 2026-05-18（项目启动前） |
| **问题** | 桌面应用框架选什么？ |
| **决策** | 使用 Tauri（Rust backend + WebView2 frontend） |
| **理由** | 1. 单文件绿色免安装（Electron 打包 >100MB，Tauri <5MB）<br>2. 完全离线，无需 Node.js 运行时<br>3. Rust 后端可直接调用 Windows API（注册表、Job Object、安全擦除）<br>4. 目标用户是非技术白领，"双击即运行"是硬需求 |
| **后果** | 前端无法直接访问文件系统，必须通过 Tauri Commands IPC 调用后端<br>开发复杂度高于纯 Web，但交付形态符合需求 |
| **可逆性** | 不可逆。已写 3000+ 行 Rust 后端代码，迁移成本极高 |

---

## ADR-002：为什么前端用 React（而非 Vue/Svelte）？

| 字段 | 内容 |
|------|------|
| **日期** | 2026-05-18 |
| **问题** | 前端框架选什么？ |
| **候选方案** | A. React（生态成熟，测试基础设施完善）<br>B. Vue（学习曲线低，但 Tauri 官方示例以 React 为主）<br>C. Svelte（编译时优化，但社区规模较小） |
| **决策** | React + TypeScript + TailwindCSS |
| **理由** | 1. 阶段二已确认技术栈<br>2. @testing-library/react 生态成熟，测试基础设施完善<br>3. Tauri 官方示例以 React 为主，社区支持更好 |
| **后果** | 需要处理 React 的闭包陷阱和 useEffect 依赖问题 |
| **可逆性** | 低。6 个页面全部用 React 实现，重写成本高 |

---

## ADR-003：为什么 CPU% 用 `GetProcessTimes` 而非 `sysinfo` crate？

| 字段 | 内容 |
|------|------|
| **日期** | 2026-05-19 |
| **问题** | 如何精确计算进程 CPU 使用率？ |
| **候选方案** | A. `sysinfo` crate（跨平台，但需额外依赖）<br>B. `GetProcessTimes` + wall clock（Windows only，零额外依赖） |
| **决策** | 方案 B |
| **理由** | 1. 本项目是 Windows-only（已大量依赖 `windows` crate）<br>2. 避免引入新依赖，减少编译时间和二进制体积<br>3. `GetProcessTimes` 精度足够（100ns 单位） |
| **后果** | 首次调用返回 0.0（无历史采样），第二次调用才有精确值<br>公式：`cpu% = (proc_delta / elapsed) * 100 / num_cpus` |
| **可逆性** | 高。如需跨平台，可替换为 `sysinfo`，接口隔离在 `resource/controller.rs` 内 |

---

## ADR-004：为什么 Scanner 进度用 `mpsc::channel` 而非 `tokio::sync::watch`？

| 字段 | 内容 |
|------|------|
| **日期** | 2026-05-19 |
| **问题** | Scanner 细粒度进度如何推送到前端？ |
| **候选方案** | A. `watch::channel<bool>`（已有，但只传布尔暂停信号）<br>B. `mpsc::channel<ProgressEvent>`（可传结构化进度数据）<br>C. 全局状态 + 轮询（简单但实时性差） |
| **决策** | 方案 B，`tokio::sync::mpsc::channel(128)` |
| **理由** | 1. `ProgressEvent` 是结构化枚举（含 scanner_id / current / total / message），mpsc 天然支持<br>2. `try_send` 不会阻塞 Scanner，channel 满时自动丢弃旧进度（可接受）<br>3. 与已有 `watch::channel` 职责分离：watch 管暂停，mpsc 管进度 |
| **后果** | 需要 Orchestrator 暴露 `set_progress_tx()` 方法，由 Commands 层注入 channel sender |
| **可逆性** | 中。可改用 broadcast channel 支持多订阅者，但当前单前端订阅者足够 |

---

## ADR-005：为什么加密文件回调用同步 `Fn` 而非 `async`？

| 字段 | 内容 |
|------|------|
| **日期** | 2026-05-19 |
| **问题** | PackExecutor 遇到加密文件时，如何让用户确认？ |
| **候选方案** | A. `async Fn(&Path) -> bool`（可 await 前端弹窗）<br>B. 同步 `Fn(&Path) -> bool`（调用方阻塞等待结果） |
| **决策** | 方案 B，`Arc<dyn Fn(&Path) -> bool + Send + Sync>` |
| **理由** | 1. `PackExecutor::finalize()` 是同步方法，签名不可轻易改为 async（会波及 orchestrator 和 commands）<br>2. Tauri 的 dialog API 实际上可在 Rust 端同步调用（阻塞式 ask）<br>3. 保持 executor trait 简洁：`fn execute(&self, item: &TraceItem) -> Result<...>` |
| **后果** | 回调在调用线程同步执行，若回调内部 await 会导致编译错误。当前默认传 `None`（不弹窗直接打包） |
| **可逆性** | 中。如需真正的异步回调，需重构 `Executor` trait 为 async，影响所有 executor |

---

## ADR-006：为什么用 `status.md` + `session-log.md` 替代 `prompt-next-session.md`？

| 字段 | 内容 |
|------|------|
| **日期** | 2026-05-19 |
| **问题** | 会话接力时如何传递上下文？ |
| **候选方案** | A. 每次重写 `prompt-next-session.md`（完整但维护重）<br>B. `status.md`（活状态）+ `session-log.md`（过程日志）+ `AGENTS.md`（固定规则）<br>C. 每次会话开始时让 AI 读全部源码重新推理（无文档依赖，但 context 消耗大） |
| **决策** | 方案 B |
| **理由** | 1. `prompt-next-session.md` 每次都要重复写环境初始化、模块速查表等不变内容<br>2. `status.md` 只记录变化（进度、待办），维护成本低<br>3. `session-log.md` 作为外部记忆，解决 context 压缩丢失问题 |
| **后果** | 需要 AGENTS.md 中明确文档体系职责边界和接力流程 |
| **可逆性** | 高。`prompt-next-session.md` 仍可保留作为阶段总结，但不再每次重写 |

---

*新增决策时复制上方模板，填写后追加到文件末尾。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否做出了新的关键设计/技术决策需要记入本文件。有则按 ADR 模板追加；没有则跳过。
