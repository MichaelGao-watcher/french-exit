# French Exit — 状态看板（新会话唯一入口）

> 先读此文件，再按需读 `AGENTS.md` / `docs/high-Level Design.md`。

## 必读顺序

1. `status.md` — 本文件（阶段 + 待办 + 环境 + 代码入口 + 规则）
2. `AGENTS.md` — 硬规则
3. `docs/high-Level Design.md` — 架构设计（如需改接口或数据流）

---

## 当前阶段

P2 ✅ | P3 vitest ✅ | P3 Playwright E2E ✅ | P1 UCRT ✅ | P4 后端 Rust 测试 ✅ | **UI/UX 迭代 + 分发就绪 ✅**

---

## 待办

### P1 — 环境（已修复 ✅）
- [x] **UCRT / entrypoint**：`cargo test --lib` 运行报 `0xc0000139`
  - 根因：`tauri::AppHandle` 类型出现在 `async fn` 签名中，与 MinGW UCRT 生成不兼容的 PE 导入表
  - 修复方案：将 `commands` 模块中的 Tauri command 函数（含 `AppHandle` + `async`）拆分到 `commands/handlers.rs`，在 `#[cfg(not(test))]` 下条件编译；`lib.rs` 的 `run()` 同样条件编译。测试模式下不链接这些函数，从而绕过 loader 入口点缺失问题
  - 副作用：零 —— release 构建和运行时行为完全不变
  - 额外修复：`scanner/registry_sys.rs` 测试中身份证号长度错误（15 位 → 18 位）

### P4 — 可选扩展
- [x] 前端 vitest 覆盖率提升（42 测）
- [x] E2E 扩展（16 测）
- [x] 后端 Rust `#[test]` 补充（88 → 103 测）

### P5 — UI/UX 迭代 & 分发就绪
- [x] 自定义 DatePicker（年/月/日精度，Apple Design，未来日期不可选）
- [x] WebView2 零依赖方案（NuGet 提取 WebView2Loader.dll + EdgeCore 回退检测）
- [x] 全局默认 dark 主题
- [x] ResultsPage 显示修改时间 + 打开所在文件夹
- [x] 工作区整理：release/ 目录（french-exit.exe + WebView2Loader.dll）

### P6 — 前端全面调整 & 骨架修复
- [x] AGENTS.md 骨架审计修复（新增 §0 文档体系说明、§3.6 约束补充）
- [x] Bug 修复：ExecutingPage 错误被吞（SET_PAGE 不清空 error）
- [x] 性能优化：ConfirmPage useMemo 缓存分组（O(3n)→O(n)）
- [x] DRY：formatBytes/formatDate 提取 utils、DecisionGroup 组件、selectAllAll 复用 getDefaultAction
- [x] UX：错误提示白色无闪烁、下一步按钮禁用校验、文件名 title 属性、DatePicker 隐藏滚动条
- [x] 布局调整：CPU toggle 移至 ScanPage、进度条极简风格（细线/无圆角/慢速）
- [x] ReportPage 重构：去掉 Emoji/卡片/按钮，主文案大标题居中，明细收缩底部小字
- [x] 纯前端预览：调试导航面板、ExecutingPage mock 进度、Report 自动注入 mock 数据
- [x] 文案统一：所有"你"→"您"、限速文案更新
- [x] GitHub 用户名批量更新（5 仓库 remote URL + 全局 Git user.name/email）

---

## 推荐策略

1. 用户如需继续调整 UI/UX，直接给出具体修改指令
2. 如需扩展功能（新增 scanner、导出格式等），进入功能需求讨论
3. 如需发布新版，执行 `cargo tauri build` 并更新 release/

---

## 环境

```bash
# PATH + 版本验证
export PATH="/c/tools/mingw64/bin:$PATH:/c/Users/Administrator/.cargo/bin"
rustc --version   # 1.95.0
cargo --version   # 1.95.0

# 编译（中文路径 MinGW 会失败，需复制到纯 ASCII 路径）
rm -rf /c/french-exit && cp -r "/e/工作文件/vs-code/french-exit" /c/french-exit
cd /c/french-exit/src-tauri && export CARGO_TARGET_DIR=/e/cargo-target
cargo check --lib       # ✅ 通过
cargo test --no-run     # ✅ 通过
cargo test --lib        # ✅ 103 测全绿（P1 已修复）

# Release 构建
cargo tauri build       # 产物在 E:/cargo-target/release/french-exit.exe
cp /e/cargo-target/release/french-exit.exe /e/cargo-target/release/WebView2Loader.dll "/e/工作文件/vs-code/french-exit/release/"  # 复制回工作区 release/

# 测试
npm run test:run          # vitest 42 测
npx playwright test e2e/  # E2E 16 测
```

---

## 关键代码入口

### Rust 后端
```
src-tauri/src/
├── commands/mod.rs          # Tauri IPC 入口（9 个 command）
├── orchestrator/mod.rs      # FSM 状态机 + 细粒度进度推送
├── executor/
│   ├── delete.rs            # 安全删除
│   ├── pack.rs              # zip 打包（磁盘检查 + 加密回调）
│   └── preserve.rs          # 保留记录
├── reporter/mod.rs          # HTML 庆祝页
├── scanner/                 # Scanner trait + 7 个具体 scanner
├── resource/controller.rs   # CPU 限制（精确计算）
├── store/temp_store.rs      # JSON Lines 临时存储
└── types.rs                 # 核心数据结构
```

### 前端
```
src/
├── pages/                   # Input / Scan / Results / Confirm / Executing / Report
├── api/commands.ts          # IPC 封装
├── store/AppContext.tsx     # 全局状态（有 reducer 单元测试）
└── test/                    # vitest 基础设施
```

### E2E
```
e2e/
├── fixtures.ts              # Playwright fixture + setupStandardMock
├── full-flow.spec.ts        # 完整流程 E2E（3 测）
├── results-interactions.spec.ts  # ResultsPage 交互 E2E（4 测）
├── error-boundary.spec.ts   # 错误边界 E2E（4 测）
├── boundary-flows.spec.ts   # 边界流程 E2E（5 测）
└── tauri-mock.js            # 浏览器端 IPC mock 运行时
```

---

## 核心规则（不可违反）

见 `AGENTS.md` 完整版。关键几条：
- **RULE-01**：所有 `Action::Delete` 必须在用户提交最终决策后执行
- **RULE-04**：程序退出必须调用 `TempStore::self_destruct()`，HTML 报告路径排除
- **RULE-05**：默认 CPU ≤30%
- **RULE-09**：HTML 保存位置：有打包放 zip 同目录，无打包放桌面

---

| 日期 | 更新 |
|------|------|
| 2026-05-19 | P2/P3 全部完成；Playwright E2E 11 测通过；P1 UCRT 已修复（`cargo test --lib` 88 测全绿）；status 合并为唯一入口 |
| 2026-05-20 | 从 vibe-coding-project-sop 同步采纳 SOP 更新：存档/恢复指令体系、各文档存档提示、lessons-learned「何时记录」规范；修正 AGENTS.md 触发词「存储」→「存档」 |
| 2026-05-20 | 前端 vitest 从 23 测提升到 42 测（新增 InputPage/ConfirmPage/ExecutingPage/ReportPage）；E2E 从 11 测提升到 16 测（新增 boundary-flows：深色模式、重置、空结果、扫描失败、取消扫描） |
| 2026-05-20 | P1 UCRT 实际修复（拆分 commands/handlers.rs + lib.rs `#[cfg(not(test))]`）；后端 Rust 测试从 88 测提升到 103 测（新增 error 4 + preserve 2 + scanner/mod 2 + orchestrator 7） |
| 2026-05-20 | UI/UX 迭代：自定义 DatePicker（年/月/日精度，未来日期不可选，丝滑下拉面板）；WebView2 零依赖方案（NuGet 提取 WebView2Loader.dll + EdgeCore 回退）；全局默认 dark 主题；ResultsPage 修改时间 + 打开路径；工作区 release/ 目录整理 |
| 2026-05-20 | 修复：纯黑色背景（CSS 变量纯黑/灰色调）；第二次扫描进度条残留（ScanPage 挂载重置 + 移除闭包依赖）；InputPage 新增 CPU 30%/全量 toggle；后端 lib.rs 启动时 apply_limits 真正生效 |
| 2026-05-20 | 新增：ResultsPage "全选全部"按钮（后端 get_all_scan_summaries 轻量接口）；路径文本可点击打开所在文件夹；Vite 开发服务器作为前端预览方案；release 重新构建并分发 |
| 2026-05-21 | 前端全面调整（8 项）：Bug/性能/DRY/UX/布局/文案/预览模式/骨架修复；GitHub 用户名批量更新（5 仓库 + 全局 Git 配置）；vitest 49 测全绿 |

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，更新本文件的以下章节：
- **当前阶段**：如有进展，更新百分比和下一步
- **进度总览**：更新各模块状态图标
- **待办**：勾选已完成项，新增下轮待办
- **更新记录**：追加本轮更新摘要

---

## 工作流提示

- **多窗口处理**：Kimi CLI 多窗口无实时同步。同一项目建议只开一个活跃会话写代码，其他窗口只读（`git status`、`cargo check` 等）。如必须多活跃会话，见 `session-log.md` 最新条目的"会话锁"建议。
