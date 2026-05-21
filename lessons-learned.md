# French Exit — 项目经验总结

> 模板化知识沉淀，供未来项目复用。结构分为「技术点」和「流程点」两类。
>
> **不是每个问题都值得记在这里**——详见下方「何时记录」。

---

## 何时记录

### 触发标准（满足任一即记录）

| 标准 | 说明 | 示例 |
|------|------|------|
| **跨项目可复用** | 换个项目做类似功能时，这条经验仍然有用 | "Tauri v2 中 `@tauri-apps/api/fs` 已移除，用 vite alias 指向 mock" |
| **花了 >30 分钟才解决** | 时间成本高，未来不值得再付一次 | "`tauri::AppHandle` + `async fn` + MinGW = `STATUS_ENTRYPOINT_NOT_FOUND`" |
| **反直觉** | 表面看起来该 A，实际必须 B，违反第一直觉 | "`cargo test --bin` 能过、`cargo test --lib` 崩溃 → 问题在仅被 lib 测试链接的代码中" |
| **未来可能重复踩** | 架构/环境/工具链的固有陷阱，新成员大概率遇到 | "中文路径 + MinGW = 链接器失败" |

### 不记录的内容（排除标准）

- ❌ 已记入 `troubleshooting.md` 的具体错误修复步骤 → 那里是"急救手册"，这里是"模式总结"
- ❌ 一次性环境配置错误（如输错密码、网络临时中断）
- ❌ 过于基础的知识（如 "List 的 `add()` 是 O(1)")
- ❌ 仅适用于本项目特定业务逻辑的 hack

### 与 troubleshooting.md 的分界

| | `troubleshooting.md` | `lessons-learned.md` |
|---|---|---|
| **问法** | "这个报错怎么修？" | "这类问题为什么会发生 / 怎么预防？" |
| **粒度** | 具体错误关键词 + 具体解决步骤 | 抽象模式 + 根本原因 + 预防策略 |
| **时效** | 只要错误还在发生，就保持有效 | 即使工具版本升级，底层模式可能仍有效 |
| **示例** | "`act is not a function` → 从 `@testing-library/react` 导入" | "Tauri 前端测试必须 mock 所有 `@tauri-apps/api/*` 模块，否则 vitest 会尝试解析不存在的模块" |

### 记录时机

**用户说「存储」时**，与 `status.md`、`session-log.md` 同步评估更新：

```
用户说「存储」→ AI 回顾本轮内容 → 更新 status.md → 追加 session-log.md → 【评估本轮是否有值得记入 lessons-learned.md 的经验】
```

评估方式：AI 回顾本轮会话内容，检查是否有符合「触发标准」的经验。有则提炼写入本文件；没有则跳过。

### 谁来记录

- **AI 助手**：每次会话结束后执行上述评估流程，自主判断并写入
- **人类把控者**：如发现 AI 漏记了明显有价值的经验，随时补录

---

## 技术经验

### Rust / Windows 系统编程

| # | 经验 | 来源 |
|---|------|------|
| 1 | `windows-rs` 0.61 的错误处理统一用 `.map_err(|e| ...)`，其中 `e` 是 `windows::core::Error` | `resource/controller.rs` |
| 2 | `GetDiskFreeSpaceExW` 传 `&HSTRING` 作为路径参数，`Option<&mut u64>` 接收可用字节 | `executor/pack.rs` |
| 3 | CPU% 精确计算只需 `GetProcessTimes` + wall clock elapsed，不需要 `GetSystemTimes` | `resource/controller.rs` |
| 4 | `FILETIME` 转 u64：`((high as u64) << 32) | (low as u64)`，单位是 100ns | `resource/controller.rs` |
| 5 | `Arc<dyn Fn(...) + Send + Sync>` 是 Rust 中给同步结构体注入回调的标准方式 | `executor/pack.rs` |

### Tauri / 前端测试

| # | 经验 | 来源 |
|---|------|------|
| 6 | Tauri 前端用 vitest + jsdom 测试时，必须在 `setup.ts` 中 `vi.mock()` 所有 `@tauri-apps/api/*` 模块 | `src/test/setup.ts` |
| 7 | 若 `@tauri-apps/api/xxx` 模块不存在（如 v2 移除了 `fs`），用 **vite alias** 指向本地 mock，而非试图安装 | `vite.config.ts` |
| 8 | Controlled checkbox 的测试用 `@testing-library/user-event` 的 `user.click()`，不要用 `fireEvent.click()` | `ResultsPage.test.tsx` |
| 9 | `tokio::sync::mpsc::Sender::try_send()` 适合非阻塞的进度回调，避免 Scanner 被 channel 阻塞 | `orchestrator/mod.rs` |

### React 状态管理

| # | 经验 | 来源 |
|---|------|------|
| 10 | **绝对不要**在 `setState` 的 updater 函数内部调用 `dispatch()` 或其他 setState，会触发 React "渲染时更新" 警告 | `ResultsPage.tsx` |
| 11 | `useEffect` 依赖 `state.xxx.size === 0` 作为触发条件时，容易形成死循环（用户操作 → size 变 0 → effect 重设 → 又变回来） | `ResultsPage.tsx` |
| 12 | `useRef` 作为"只执行一次"的标志，比依赖数组更可靠，尤其涉及批量初始化逻辑时 | `ResultsPage.tsx` |

---

## 流程经验

### 问题发现机制
- **测试驱动暴露 Bug**：ResultsPage 的默认勾选死循环是在写单元测试时发现的，手工测试几乎不可能复现（需要恰好取消所有勾选）
- **结论**：前端状态管理类的 bug，单元测试是最有效的发现手段，远超手工测试

### 文档维护
- `prompt-next-session.md` 的问题：每次都要重写环境初始化、模块速查表等**不变内容**
- **改进**：`status.md`（活文档，只记录变化）+ `AGENTS.md`（固定规则）
- **收益**：新会话读 2 份文件即可开工，维护成本降低 80%

### 沟通 / 需求澄清
- **横跨工具层和应用层的词汇必须确认语境**。用户问"一个项目多个终端能否实现同步处理进度"——"终端"可以指 French Exit 的并行 executor、Kimi CLI 的多窗口、或 ai-project-skeleton 的多会话。我默认跳到了代码层面分析并行化，结果完全偏题。
  - **正确做法**：遇到"终端""同步""项目"这类横跨多层含义的词，先给两个选项让用户确认，不要默认展开分析
- **工具硬性限制不要绕圈分析可行性**。Kimi CLI 多窗口无 IPC、无共享内存、无实时同步——这不是"有难度"，是"设计上就不支持"。回答应直接给结论 + 风险 + 替代方案，省掉技术可行性分析

### SOP 模板同步
- **从 SOP 模板采纳更新时，必须逐字核对关键字段，不要凭记忆改写**。本轮将 `vibe-coding-project-sop/AGENTS.md` 中的触发词「存档」错误抄写为「存储」，原因是未逐字比对就按直觉填写。SOP 模板中的占位符（如 `[项目名]`）在实际项目中需替换，但硬规则（触发词、流程步骤）应原样保留。
  - **正确做法**：Side-by-side 对比源文件和目标文件的关键段落，尤其是表格、触发词、命令等不可改动的内容

### 编译/环境
- 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/french-exit`）后编译
- `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test --no-run` 同理
- **`0xc0000139` 不一定是 UCRT/MinGW 兼容性 issue**。先跑一个**最简单 lib 测试**（空 crate + `cargo test --lib`），如果能过，就说明工具链没问题，问题在项目的特定代码中
- **`cargo test --bin` 能过、`cargo test --lib` 崩溃** → 问题出在**仅被 lib 测试链接的代码**中（bin 测试做了死代码消除，没链接到问题代码）。这是极强的定位信号
- **定位代码的最快方法**：清空 `lib.rs` 只保留一个空测试，逐步 `pub mod` 添加模块，直到崩溃复现。比分析 PE 导入表快 10 倍
- **`tauri::AppHandle` 出现在 `async fn` 签名中 + MinGW = `STATUS_ENTRYPOINT_NOT_FOUND`**。原因未知（PE 导入表生成 bug？），但 workaround 明确：把这些函数拆到子模块，用 `#[cfg(not(test))]` 条件编译，测试模式下不链接
- **`#[cfg(not(test))]` 隔离问题代码**是零副作用的修复手法：release 构建完全不受影响，测试逻辑移至独立模块继续跑

---

## 可复用模板

以下为通用结构，新项目可复制后填充：

```markdown
# [项目名] — 经验总结

## 技术经验
| # | 经验 | 来源模块 |

## 流程经验
### 问题发现
### 文档维护
### 环境陷阱

## 待验证假设（本轮未证实，下轮验证）
- [ ] xxx
```

---

## 待验证假设（本轮未证实，下轮验证）

- [ ] 无

---

*新增假设时直接追加到上方列表。验证后勾选并迁移到对应经验区或删除。*

### 测试：mock useAppState 替代 TestProvider

当组件直接从 Context state 读取（不调用 API）时，mock `useAppState` 比构建 TestProvider 更可控。尤其当组件内部 dispatch 后会触发 reducer 状态变化（如 RESET 回到初始状态），使用 mock 可以避免组件在测试中途意外卸载或切换视图。

```tsx
vi.mock("../store/AppContext", async () => {
  const actual = await vi.importActual<typeof import("../store/AppContext")>(
    "../store/AppContext"
  );
  return { ...actual, useAppState: vi.fn() };
});
```

### 测试：工厂函数中计算属性要放在 `...overrides` 之后

```tsx
// ❌ 错误：overrides.id 会覆盖模板字符串结果
function makeItem(overrides) {
  return {
    id: `item-${overrides.id || "1"}`,
    ...overrides,  // ← 这里会把 id 又覆盖回 "1"
  };
}

// ✅ 正确：计算属性放在最后
function makeItem(overrides) {
  const id = `item-${overrides.id || "1"}`;
  return {
    ...overrides,
    id,
  };
}
```

### E2E：Playwright 模拟系统主题色

```ts
await page.emulateMedia({ colorScheme: "dark" });
await expect(page.locator("html")).toHaveClass(/dark/);
```

### Tauri 零依赖分发：WebView2Loader.dll 提取与打包

```bash
# 从 NuGet 提取 WebView2Loader.dll（微软官方允许随应用分发）
curl -L -o webview2.nupkg "https://www.nuget.org/api/v2/package/Microsoft.Web.WebView2/"
unzip -o webview2.nupkg "build/native/x64/WebView2Loader.dll" -d ./extracted/
# 复制到 src-tauri/，并在 tauri.conf.json 中配置 bundle.resources
```

配合 EdgeCore 回退检测（`WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`），可在不安装 WebView2 Runtime 的系统上直接运行 Tauri 应用。

### 自定义日期选择器：不引入第三方库

本项目自建 `DatePicker` 组件（年/月/日三精度，丝滑下拉面板，Apple Design），比引入 `react-datepicker` 或 `date-fns` 更轻量，且完全符合设计系统。关键实现：
- `useRef` + `mousedown` 监听实现点击外部关闭
- CSS `@keyframes dropdownIn` 实现淡入+位移动画
- 年月日联动限制（如今年只显示到当前月）

### Tauri dev 模式与后台任务的兼容性

- `cargo tauri dev` 必须在**交互式 Windows 桌面会话**中运行，无法通过远程/后台任务启动（WebView2 需要 GUI 上下文）
- **替代方案**：`npm run dev` 启动 Vite 服务器 → 浏览器访问 `http://localhost:1420` → 可实时预览前端 UI（HMR 热更新），但 IPC 调用会失败
- **完整功能验证**：仍需本地运行 `cargo tauri dev` 或双击 release `.exe`

### 全选大批量数据的前后端协作模式

当扫描结果达一万条以上时，前端逐页加载再全选不现实：
- 不要一次性加载所有完整 `TraceItem` 到前端（内存 + DOM 渲染压力大）
- 正确做法：后端提供**轻量摘要接口**（只返回 id + category + suggested_action），前端用它批量生成 decisions
- 用户实际浏览仍按分页，但"全选全部"走轻量接口，两者解耦

### 前端代码审计的四维度分类法

对已有前端代码进行系统审计时，按以下四个维度分类，可避免遗漏且便于排优先级：

| 维度 | 关注内容 | 示例 |
|------|---------|------|
| **Bug** | 逻辑错误导致功能异常 | reducer 中 SET_PAGE 自动清空 error，导致错误信息被吞 |
| **性能** | 重复计算、不必要的渲染 | ConfirmPage 每次渲染三次全量 filter |
| **DRY** | 重复代码、可提取组件 | formatBytes 在三处重复定义、三个清单卡片结构完全重复 |
| **UX** | 交互设计缺陷 | 下一步按钮无前置校验、文件名 truncate 后无法查看全称 |

审计完成后按 **Bug > 性能 > DRY > UX** 优先级修复，先保正确性再保体验。

### 纯前端预览模式下的 IPC mock 策略

`npm run dev`（Vite 纯前端）没有 Rust 后端，Tauri IPC 调用会失败。若需在浏览器中完整预览所有页面流程：

1. **开发调试导航面板**：在 App.tsx 中通过检测 `window.__TAURI_INTERNALS__` 是否缺失，显示页面跳转按钮， bypass 正常流程
2. **模拟异步进度**：ExecutingPage 中检测非 Tauri 环境，用 `setInterval` 模拟进度递增，完成后自动跳转并注入 mock report
3. **自动注入 mock 数据**：调试面板点击"报告"时，若 `state.report` 为空，自动 dispatch SET_REPORT 注入 mock 数据

核心原则：**mock 只注入数据，不改页面渲染逻辑**——真实 Tauri 环境中的行为与 mock 环境完全一致。

### GitHub 用户名变更的批量处理脚本

用户名变更后，每台设备的每个本地仓库都需要更新 remote URL（Git 不跨设备同步）。自动化脚本要点：

1. **dry-run 先预览**：只读列出会被修改的仓库，用户确认后再执行
2. **全局 Git 配置同步**：同时更新 `user.name` 和 `user.email`（noreply 格式）
3. **生成可复用脚本**：填好新旧用户名后保存为 `.sh`，复制到另一台设备直接运行

脚本模板核心（sed 替换旧用户名为新用户名）：
```bash
new_url=$(echo "$old_url" | sed "s/$OLD_USER/$NEW_USER/")
git remote set-url origin "$new_url"
```

---

*最后更新：2026-05-21*
