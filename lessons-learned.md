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
| # | 经验 | 来源模块 | | 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 Web Worker（如 Stockfish）时必须启 HTTP 服务器 [来源:blindfold-chess @2026-05-22] | EngineModule |
| | 手写 IIFE 模块时，用 `window.ModuleName = Module` 暴露 API，内部私有变量用下划线前缀，避免全局泄漏 [来源:blindfold-chess @2026-05-22] | 所有 js/*.js |
| | 浏览器集成测试用 TestRunner（自定义极简框架），保持与 Node 测试同一套断言 API，降低切换成本 [来源:blindfold-chess @2026-05-22] | docs/tests/ |
| | Canvas 图表渲染在浏览器中测试，Node 环境用 Mock 2D context 跳过绘制验证，各测其责 [来源:blindfold-chess @2026-05-22] | StatsModule |
| | PGN 解析器对空/无效输入返回 `[]`（空数组）而非 `null`，调用方需区分"无走法"和"解析失败" [来源:blindfold-chess @2026-05-22] | ReplayModule |
| | `cloneNode(true)` 替换含 SVG 的按钮会导致 SVG 渲染异常（显示不完整）；移除事件监听器应优先用 `removeEventListener` + 命名函数，避免替换 DOM 元素 [来源:blindfold-chess @2026-05-22] | SettingsModule |
| | 匿名事件监听器无法被后续代码移除；需要动态解除绑定的监听器必须用命名函数（暴露到 `window` 或模块内部变量） [来源:blindfold-chess @2026-05-22] | common.js / settings.js |
| | 屏幕切换导航不能只隐藏上一个屏幕，必须遍历 `.screen` 全部隐藏后再显示目标，否则多层屏幕重叠 [来源:blindfold-chess @2026-05-22] | 全局导航 |
| | SVG path 中密集参数（如 `a2 2 0 0 1-2.83 0`）在某些浏览器中可能解析异常，命令与参数间保留空格更稳妥 [来源:blindfold-chess @2026-05-22] | index.html SVG |
| | `document` 级事件监听器若引用了某个 DOM 元素，该元素被替换后监听器仍会按旧引用判断，导致逻辑错误（如点击新按钮被误判为"点击外部"） [来源:blindfold-chess @2026-05-22] | settings.js / common.js |
| | 项目文档结构会随时间进化，"存档"或"恢复"操作前应先 `ls`/`glob` 确认当前文件系统现状，避免按历史路径写入已不存在的文件 [来源:blindfold-chess @2026-05-22] | 文档维护 |
| | **UI 布局/样式不要猜测用户意图**：候选走法开关经历了 5 次位置/样式反复（设置面板 → header 图标 → 滑动开关 → 圆形按钮+标签 → 纯文字 → 下移），每次修改后用户都不满意；应在设计阶段出草图或描述供用户确认，再编码 [来源:blindfold-chess @2026-05-22] | BlindfoldModule UI |
| | **引擎候选走法的调用时机决定产品逻辑正确性**：用户走完后立即 `goMultiPv` 分析的是对手（黑方）局面，展示的是"对手会怎么走"；若要提示用户，必须在引擎执行完走法后、轮到白方时再调用 `goMultiPv` [来源:blindfold-chess @2026-05-22] | EngineModule / BlindfoldModule |
| | **引擎返回 UCI（e2e4），用户界面必须用 SAN（e4）**：`goMultiPv` 回调中的 `move` 是 UCI 坐标格式，展示前需通过 `_game.moves({verbose:true})` 映射为 SAN，否则用户无法阅读 [来源:blindfold-chess @2026-05-22] | BlindfoldModule |
| | **静态 HTML 结构与动态渲染模块的 DOM 冲突**：`index.html` 中预置了完整棋盘结构（含行/列标注），而 `BoardRenderer.create()` 会在容器内重新创建完整结构，导致两组行标注同时存在；应只保留空容器让渲染器全权负责 [来源:blindfold-chess @2026-05-22] | BoardRenderer / index.html |
| | **删除功能必须同步删除对应测试**：移除 `showHints` / `multiPvSetting` 后，`test-settings-node.js` 中相关测试会立即失败；功能清理和测试清理应视为同一任务 [来源:blindfold-chess @2026-05-22] | 测试维护 |
| | **焦点管理是盲棋产品的核心体验**：进入对局自动 `input.focus()`、引擎走完后恢复焦点、全局 Enter 键将焦点拉回输入框——三者缺一不可，否则用户被迫频繁使用鼠标 [来源:blindfold-chess @2026-05-22] | BlindfoldModule UX |
| | **i18n 分散架构必然导致翻译遗漏**：当项目同时存在"全局字典 + 模块私有字典 + 硬编码"三种翻译方式时，新增功能几乎必然漏掉其中一种或多种。唯一可持续的方案是"单一字典源" [来源:blindfold-chess @2026-05-22] | 全站 i18n |
| | **JS 中的硬编码人类可读字符串是翻译遗漏的重灾区**：HTML 中的 `data-i18n` 至少能被肉眼扫描到，但 JS 逻辑里直接写的 `"Time Up!"`、`"✓ 已复制"` 没有显式标记，切换语言时完全失效 [来源:blindfold-chess @2026-05-22] | common.js / coordinate.js / blindfold.js |
| | **复制粘贴是 i18n 错误的常见来源**：将中文值直接粘贴进英文字典（如 `boardToggle: "显示棋盘"`），或反之，属于低级但高频的疏忽 [来源:blindfold-chess @2026-05-22] | common.js |
| | **模块内部字典若从不主动更新 DOM，则纯属冗余**：welcome.js 有 `_i18n` 和 `_t()`，但从不调用，完全依赖 common.js 的 `updateTexts()`。这种"假私有字典"不仅没用，还会给维护者造成"这里已经翻译了"的错觉 [来源:blindfold-chess @2026-05-22] | welcome.js |
| | **settings.js 的独立字典与 common.js 的全局扫描存在竞争**：settings panel 的元素带 `data-i18n`，settings.js 自己 `_updateAllTexts()` 会覆盖，但 common.js 的 `updateTexts()` 也会扫到，如果 common.js 缺键，用户会看到 key 名闪一下才被正确文本覆盖 [来源:blindfold-chess @2026-05-22] | settings.js / common.js |
| | **已删除的 JS 文件若不从 index.html 移除引用，会导致 404**：game.js 删除后 index.html 仍 `<script src="js/game.js">`，浏览器控制台会报错。功能清理和引用清理必须是同一任务 [来源:blindfold-chess @2026-05-22] | 代码清理 |
| | **Node 测试不对 UI 文本做断言，无法捕获翻译错误**：`test-stats-node.js` 和 `test-replay-node.js` 只测 API 形状和数值，不检查按钮文字、提示语等人类可读内容。翻译质量必须靠人工检查或专门的 UI 测试覆盖 [来源:blindfold-chess @2026-05-22] | 测试策略 |
| | **删除生产代码的 fallback 函数前，必须先评估测试环境是否提供了该依赖**：`blindfold.js`/`coordinate.js` 的 `_t()` fallback 在测试中默默提供英文文本，删除后所有相关测试立即 `ReferenceError: t is not defined`。架构统一重构必须同时改代码+测试，只改一边会导致测试雪崩 [来源:blindfold-chess @2026-05-22] | 全站 i18n |
| | **`localStorage` mock 必须支持 `setItem` 持久化**：测试中 `global.localStorage = { getItem: () => null }` 会让 `t()` 永远读取默认语言，导致语言切换测试失效。可写的 localStorage mock 是 i18n 测试的前提 [来源:blindfold-chess @2026-05-22] | 测试基础设施 |
| | **全局 `updateTexts()` 与模块私有 `_updateXxx()` 可能存在 DOM 竞争**：`settings.js` 的 `_updateLangValue()` 显示"当前语言"，common.js 的 `updateTexts()` 显示"目标语言"，两者操作同一 DOM 元素。测试必须验证最终渲染结果，而非中间状态 [来源:blindfold-chess @2026-05-22] | settings.js / common.js |
| | **配置类设置项用「弹窗选择」优于「循环切换」**：循环切换隐藏了全部选项，用户不知道有哪些风格、当前在第几个；弹窗一次展示所有选项+预览，认知负荷更低，操作确定性更强 [来源:blindfold-chess @2026-05-22] | SettingsModule UI |
| | **`cloneNode(true)` 无法移除旧事件监听器，它只是复制了 DOM 结构**：`_rebind()` 用 clone+replace 来"换绑"事件，但如果匿名监听器无法被引用，clone 后的新元素上旧的监听器仍然通过作用域链引用着旧变量。真正安全的解绑是 `removeEventListener` + 保存引用 [来源:blindfold-chess @2026-05-22] | settings.js |
| | **UI 风格不一致的根因通常是「硬编码颜色」**：盲棋练习和坐标练习的棋盘颜色不一致，是因为两者各自硬编码了不同色值。引入统一的「棋盘风格配置源」后，所有棋盘自动同步，消除了不一致的根因 [来源:blindfold-chess @2026-05-22] | BoardRenderer / coordinate.js |
| | **功能入口迁移需要同步更新「正向路径」和「反向路径」**：将复盘从首页移到设置面板，不仅要添加新入口（设置面板点击），还要移除旧入口（首页卡片 + welcome.js 绑定），否则用户会在两个地方看到同一功能，或测试断言旧路径仍然有效 [来源:blindfold-chess @2026-05-22] | WelcomeModule / index.html |
| | **数据层的双语字段与代码层的硬编码分支是两个问题**：`games.js` 的 `titleZh/titleEn` 是数据内容，保留双语字段合理；但 `replay.js` 中的三元组 `lang === 'en' ? game.titleEn : game.titleZh` 是代码硬编码分支，应通过数据结构改造消除。区分"数据双语"和"代码分支"可避免过度重构 [来源:blindfold-chess @2026-05-22] | replay.js / data/games.js |
| | **测试中断言的具体文本值是重构的敏感点**：当翻译源从"模块内联字典"切换到"全局字典"时，即使语义相同，具体字符串也可能不同（如 `"再来一局"` → `"再玩一局"`）。重构前应先审计测试中的文本断言，预估需要调整的范围 [来源:blindfold-chess @2026-05-22] | 测试维护 |
| | **数据文件中的引号嵌套是极易被忽视的语法陷阱**：`data/games.js` 中的 `'Rubinstein's Immortal'` 在 Node 测试环境中不会触发（因为该文件仅被浏览器加载），但在真实浏览器中会抛出 `SyntaxError` 并阻断后续脚本执行 [来源:blindfold-chess @2026-05-22] | data/games.js |
| | **Node 测试全过 ≠ 浏览器表现正常**：`data/games.js` 的语法错误在 Node 测试中被完全绕过（Node 测试不加载该文件），必须用 headless 浏览器（playwright）才能捕获 [来源:blindfold-chess @2026-05-22] | 测试策略 |
| | **playwright 是定位浏览器特有 bug 的有效手段**：通过 `page.add_init_script` 注入错误监听器 + `page.on('pageerror')`，可以精确定位到出错的文件、行号和列号 [来源:blindfold-chess @2026-05-22] | 调试工具 |
| | **通用配置层设计能降低新增模式的边际成本**：将"选择阵营 + 难度"抽象为 `gameSetupScreen`，由 `WelcomeModule` 维护 `_pendingMode`，新增对局模式时只需加一行 `else if` 分发逻辑，无需重复造 DOM/CSS [来源:blindfold-chess @2026-05-22] | 架构设计 |
| | **向后兼容接口设计能减少重构的连锁反应**：`BlindfoldModule.init('medium')` 继续工作，内部映射为 `{side:'w', elo:1400}`，所有旧测试和外部调用点无需改动 [来源:blindfold-chess @2026-05-22] | API 设计 |
| | 浏览器集成测试阶段发现 welcome.js / replay.js / stats.js 的 DOM 事件绑定遗漏 [来源:blindfold-chess @2026-05-22] |  |
| | Node 测试覆盖逻辑，浏览器测试覆盖 DOM 集成，两者互补 [来源:blindfold-chess @2026-05-22] |  |
| | `AGENTS.md` 定义触发词和行为约束，`STATE.md`（现 status.md）记录动态进度，分工明确 [来源:blindfold-chess @2026-05-22] |  |
| | 每批次开发完成后同步更新进度文档，避免新会话迷路 [来源:blindfold-chess @2026-05-22] |  |
| | **手工构建100条结构化数据不现实**：经典棋局的 PGN 分散在各网站，无统一免费 API；手动录入100盘完整 PGN 工作量巨大且易出错 [来源:blindfold-chess @2026-05-22] |  |
| | **WriteFile 不适合超大特殊字符内容**：含大量引号/换行的长文本会因 JSON 转义失败；应改用本地 Python/Node 脚本生成，或提前准备好数据文件 [来源:blindfold-chess @2026-05-22] |  |
| | **Shell here-document 在 Windows git bash 中不可靠**：含引号的多行复杂脚本会被截断或解析错误；应先 `WriteFile` 写脚本，再 `Shell` 执行 [来源:blindfold-chess @2026-05-22] |  |
| | **翻译检查必须是独立任务，不能依赖"开发时顺手做"**：本次检查发现 25+ 处遗漏，分布在 HTML、JS 字典、硬编码三个层面。分批迭代时，每新增一个 `data-i18n` 或用户可见字符串，必须同步到唯一字典源，否则必然遗漏。 [来源:blindfold-chess @2026-05-22] |  |
| | **涉及 7+ 文件读改测的架构重构，应新开会话执行**：当前会话在查漏补缺后已承载大量上下文，继续塞进系统性重构容易触发窗口压缩，导致信息丢失。 [来源:blindfold-chess @2026-05-22] |  |
| | GitHub Pages 国内访问需代理；unpkg CDN 加载 Stockfish 可能超时，需考虑离线备选方案 [来源:blindfold-chess @2026-05-22] |  |
| | Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚本时优先用正斜杠或 `path.join` [来源:blindfold-chess @2026-05-22] |  |
| | TAG:build-env TAG:testing [来源:vibe-coding-project-sop @2026-05-22] | INFO | 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 Web Worker（如 Stockfish）时必须启 HTTP 服务器 [来源:blindfold-chess @2026-05-21] | EngineModule |
| | TAG:dom TAG:api-design [来源:vibe-coding-project-sop @2026-05-22] | WARNING | 手写 IIFE 模块时，用 `window.ModuleName = Module` 暴露 API，内部私有变量用下划线前缀，避免全局泄漏 [来源:blindfold-chess @2026-05-21] | 所有 js/*.js |
| | TAG:data TAG:api-design [来源:vibe-coding-project-sop @2026-05-22] | INFO | PGN 解析器对空/无效输入返回 `[]`（空数组）而非 `null`，调用方需区分"无走法"和"解析失败" [来源:blindfold-chess @2026-05-21] | ReplayModule |
| | TAG:dom TAG:ux [来源:vibe-coding-project-sop @2026-05-22] | WARNING | 屏幕切换导航不能只隐藏上一个屏幕，必须遍历 `.screen` 全部隐藏后再显示目标，否则多层屏幕重叠 [来源:blindfold-chess @2026-05-21] | 全局导航 |
| | TAG:ai-workflow [来源:vibe-coding-project-sop @2026-05-22] | INFO | 项目文档结构会随时间进化，"存档"或"恢复"操作前应先 `ls`/`glob` 确认当前文件系统现状，避免按历史路径写入已不存在的文件 [来源:blindfold-chess @2026-05-21] | 文档维护 |
| | TAG:i18n [来源:vibe-coding-project-sop @2026-05-22] | CRITICAL | **i18n 分散架构必然导致翻译遗漏**：当项目同时存在"全局字典 + 模块私有字典 + 硬编码"三种翻译方式时，新增功能几乎必然漏掉其中一种或多种。唯一可持续的方案是"单一字典源" [来源:blindfold-chess @2026-05-21] | 全站 i18n |
| | TAG:i18n TAG:architecture [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **模块内部字典若从不主动更新 DOM，则纯属冗余**：welcome.js 有 `_i18n` 和 `_t()`，但从不调用，完全依赖 common.js 的 `updateTexts()`。这种"假私有字典"不仅没用，还会给维护者造成"这里已经翻译了"的错觉 [来源:blindfold-chess @2026-05-21] | welcome.js |
| | TAG:i18n TAG:dom [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **settings.js 的独立字典与 common.js 的全局扫描存在竞争**：settings panel 的元素带 `data-i18n`，settings.js 自己 `_updateAllTexts()` 会覆盖，但 common.js 的 `updateTexts()` 也会扫到，如果 common.js 缺键，用户会看到 key 名闪一下才被正确文本覆盖 [来源:blindfold-chess @2026-05-21] | settings.js / common.js |
| | TAG:testing TAG:architecture [来源:vibe-coding-project-sop @2026-05-22] | CRITICAL | **删除生产代码的 fallback 函数前，必须先评估测试环境是否提供了该依赖**：架构统一重构必须同时改代码+测试，只改一边会导致测试雪崩 [来源:blindfold-chess @2026-05-21] | 全站 i18n |
| | TAG:dom TAG:i18n [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **全局 `updateTexts()` 与模块私有 `_updateXxx()` 可能存在 DOM 竞争**：两者操作同一 DOM 元素。测试必须验证最终渲染结果，而非中间状态 [来源:blindfold-chess @2026-05-21] | settings.js / common.js |
| | TAG:ux TAG:architecture [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **UI 风格不一致的根因通常是「硬编码颜色」**：引入统一的「棋盘风格配置源」后，所有棋盘自动同步，消除不一致的根因 [来源:blindfold-chess @2026-05-21] | BoardRenderer / coordinate.js |
| | TAG:data TAG:architecture [来源:vibe-coding-project-sop @2026-05-22] | INFO | **数据层的双语字段与代码层的硬编码分支是两个问题**：区分"数据双语"和"代码分支"可避免过度重构 [来源:blindfold-chess @2026-05-21] | replay.js / data/games.js |
| | TAG:data TAG:build-env [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **数据文件中的引号嵌套是极易被忽视的语法陷阱**：在真实浏览器中会抛出 `SyntaxError` 并阻断后续脚本执行 [来源:blindfold-chess @2026-05-21] | data/games.js |
| | TAG:testing TAG:debugging [来源:vibe-coding-project-sop @2026-05-22] | INFO | **playwright 是定位浏览器特有 bug 的有效手段**：通过 `page.add_init_script` 注入错误监听器 + `page.on('pageerror')`，可以精确定位到出错的文件、行号和列号 [来源:blindfold-chess @2026-05-21] | 调试工具 |
| | TAG:testing TAG:dom [来源:vibe-coding-project-sop @2026-05-22] | INFO | 浏览器集成测试阶段发现 welcome.js / replay.js / stats.js 的 DOM 事件绑定遗漏 [来源:blindfold-chess @2026-05-21] | |
| | TAG:cross-platform TAG:ai-workflow [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **Shell here-document 在 Windows git bash 中不可靠**：含引号的多行复杂脚本会被截断或解析错误；应先 `WriteFile` 写脚本，再 `Shell` 执行 [来源:blindfold-chess @2026-05-21] | |
| | TAG:i18n TAG:ai-workflow [来源:vibe-coding-project-sop @2026-05-22] | WARNING | **翻译检查必须是独立任务，不能依赖"开发时顺手做"**：本次检查发现 25+ 处遗漏，分布在 HTML、JS 字典、硬编码三个层面 [来源:blindfold-chess @2026-05-21] | |
| | TAG:state-management [来源:vibe-coding-project-sop @2026-05-22] | CRITICAL | **绝对不要**在 `setState` 的 updater 函数内部调用 `dispatch()` 或其他 setState，会触发 React "渲染时更新" 警告 [来源:french-exit @2026-05-21] | `ResultsPage.tsx` |
| | TAG:cross-platform TAG:build-env [来源:vibe-coding-project-sop @2026-05-22] | CRITICAL | 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/french-exit`）后编译 [来源:french-exit @2026-05-21] | |
| | TAG:testing TAG:cross-platform [来源:vibe-coding-project-sop @2026-05-22] | INFO | **`#[cfg(not(test))]` 隔离问题代码**是零副作用的修复手法：release 构建完全不受影响，测试逻辑移至独立模块继续跑 [来源:french-exit @2026-05-21] | |
| | TAG:data TAG:performance [来源:vibe-coding-project-sop @2026-05-22] | WARNING | 不要一次性加载所有完整 `TraceItem` 到前端（内存 + DOM 渲染压力大） [来源:french-exit @2026-05-21] | |
| | TAG:architecture TAG:data [来源:vibe-coding-project-sop @2026-05-22] | INFO | 正确做法：后端提供**轻量摘要接口**（只返回 id + category + suggested_action），前端用它批量生成 decisions [来源:french-exit @2026-05-21] | |
| | TAG:pagination TAG:architecture [来源:vibe-coding-project-sop @2026-05-22] | WARNING | 用户实际浏览仍按分页，但"全选全部"走轻量接口，两者解耦 [来源:french-exit @2026-05-21] | |
| | TAG:pagination TAG:state-management TAG:security [来源:vibe-coding-project-sop @2026-05-22] | CRITICAL | **事故经过**：ResultsPage 默认自动勾选所有扫描结果 → 用户点击"全选全部"（以为是全选当前页，实际是全选全部）→ 确认页看到"将删除 17,706 个文件"但未警觉 → 执行后大量文件丢失 [来源:french-exit @2026-05-21] | |
| | TAG:security TAG:ux [来源:vibe-coding-project-sop @2026-05-22] | CRITICAL | **教训**：涉及删除的安全工具，**默认安全 > 默认便利**。所有选择必须用户显式操作，任何"帮你选好"的设计都需反复审视 [来源:french-exit @2026-05-21] | |
| | 测试驱动开发能在手工测试无法触及的边界条件下发现 bug（如"恰好取消所有勾选"触发死循环）[来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] |  |
| | `AGENTS.md` 定义触发词和行为约束，`status.md` 记录动态进度，两者分工明确，新会话读 2 份文件即可开工 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] |  |
| | 涉及 7+ 文件读改测的架构重构，应新开会话执行，避免上下文压缩导致信息丢失 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] |  |
| | 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径后编译 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] |  |
| | `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test --no-run` 同理 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] |  |
| | Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚本时优先用正斜杠或 `path.join` [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] |  |
|

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

### 默认勾选是严重 UX 陷阱

"方便用户"的默认勾选设计，在涉及删除操作的安全工具中是致命陷阱：

- **事故经过**：ResultsPage 默认自动勾选所有扫描结果 → 用户点击"全选全部"（以为是全选当前页，实际是全选全部）→ 确认页看到"将删除 17,706 个文件"但未警觉 → 执行后大量文件丢失
- **根因链**：默认勾选 × deselectAll 只清当前页 × ConfirmPage 遍历 scanResults（分页未加载完整）= 三重 bug 叠加
- **教训**：涉及删除的安全工具，**默认安全 > 默认便利**。所有选择必须用户显式操作，任何"帮你选好"的设计都需反复审视

### `deselectAll` 只清当前页不是真正的"取消全选"

当系统支持"全选全部"（跨分页选中所有数据）时，取消全选必须清空**全部**选中状态，不能只操作当前可见页：

- **原实现**：`deselectAll` 只遍历 `searchedItems`（当前页数据），从 `selectedIds` 中移除 → 其他分页的选中状态仍保留
- **修复**：`deselectAll` 清空 `selectedIds` 为 `new Set()`，同时 `dispatch({ type: "SET_DECISIONS", payload: new Map() })` 清空全部 decisions
- **教训**：跨分页操作时，"取消"必须与"全选"的对称——全选影响多大范围，取消就必须影响多大范围

### 分页加载场景下，确认页必须遍历 `decisions` 而非 `scanResults`

前端采用分页加载时，`scanResults` 只包含已加载的分页数据，而 `decisions` 是用户全部选择决策的完整集合：

- **原实现**：ConfirmPage 遍历 `state.scanResults`，过滤出选中的项 → 分页未加载的项完全丢失
- **修复**：遍历 `state.decisions`，每项在 `scanResults` 中查找详细信息，找不到时用 `name: id` 兜底
- **教训**：在分页/懒加载架构中，**用户操作集合（decisions）是主数据源，展示数据（scanResults）是从属数据源**。确认/汇总逻辑必须基于操作集合

### 开发预览工作流：Vite 服务器替代 `cargo tauri dev`

Tauri 的 `cargo tauri dev` 无法在后台任务/SSH/无头环境中启动（需 Windows 桌面 GUI 上下文）。前端独立预览方案：

- **启动**：`npm run dev`（Vite 服务器）→ 浏览器访问 `http://localhost:1420`
- **优势**：HMR 热更新、即时预览、不依赖 Rust 编译
- **限制**：IPC 调用会失败，需通过 mock 数据或调试导航面板 bypass
- **完整功能验证**：仍需本地 `cargo tauri dev` 或双击 release `.exe`

---

### 前端 `fixed` 定位元素必须加 `pointer-events-none` 避免遮挡交互

`App.tsx` 的大 Logo 使用 `fixed z-50` 覆盖页面中央，未设置 `pointer-events-none`，导致用户点击"开始使用"按钮时实际点击到了 Logo `div` 上：

- **根因**：CSS `fixed` + `z-50` 的元素默认接收鼠标事件，即使视觉上看起来透明也会拦截点击
- **修复**：给所有非交互性的 `fixed` 装饰元素统一添加 `pointer-events-none`
- **教训**：任何使用 `fixed`/`absolute` + 高 `z-index` 的纯展示元素，必须默认视为点击拦截器

### E2E 测试适配前端 UI 变更的系统方法

前端 UI 迭代（DatePicker 重构、默认勾选移除、ReportPage 重构等）导致 E2E 大面积失效，修复遵循以下模式：

| 变更类型 | 适配策略 | 示例 |
|---|---|---|
| DOM 结构变更（`#start-date` → DatePicker） | 提取辅助函数封装新交互 | `fillDatePicker(page, dateStr)` |
| 产品逻辑变更（默认勾选 → 全不勾选） | 更新测试期望，在关键路径手动触发操作 | 点击"下一步"前手动勾选 checkbox |
| 页面跳转机制变更（直接返回值 → 事件驱动） | 补充 `emitEvent` 模拟后端事件 | `ExecutionCompleted` 事件触发 ReportPage 跳转 |
| 文案/样式变更 | 使用更稳定的定位策略（文案匹配替代 CSS class） | `text=已删除 2 条` 替代 `.text-green-600` |

- **教训**：E2E 测试不是写一次就完，它是前端契约测试。UI 迭代时必须同步评估对 selector、交互流程、状态断言的影响

### `progress_cb` 是扫描器内部实现暂停的有效检查点

Scanner trait 的 `scan()` 方法签名包含 `pause_rx`，但所有实现都标记为 `_pause_rx`（未使用）。要在扫描**进行中**支持暂停，最轻量的方案是在 `progress_cb` 闭包中插入暂停检查：

- **方案**：`ScannerRegistry::scan_impl` 的 `progress_cb` 在每次上报进度前 `while *pause_rx.borrow() { sleep(100ms) }`
- **优势**：零侵入 scanner 实现，不需要修改 7 个具体 scanner 的代码
- **局限**：如果 scanner 长时间不调用 `progress`（如读取超大文件），暂停会有延迟
- **教训**：对于已成型的大型 trait 实现体系，优先在调度层（registry）而非实现层（scanner）插入横切关注点

*最后更新：2026-05-22*

---

## 进度条全局化设计：多任务并行场景下的进度计算模式

**适用场景**：后端有多个并行任务（Scanner/Worker），每个任务报告自己的局部进度，前端需要展示全局进度条。

**反模式（不要这样做）**：
- ❌ 直接把每个任务的局部 `current/total` 当作全局百分比
- ❌ 前端"只增不减"机制配合局部进度 = 轻量任务瞬间把进度锁死在 100%

**正确模式**：
1. **后端计算全局进度**：调度层（Registry/Coordinator）为每个任务分配权重，计算加权平均
2. **权重反映预估耗时**：不是均分，而是按任务实际工作量分配（如全盘扫描 50%，轻量检测 5%）
3. **通过独立字段传递**：`ScanProgress { current, total, global_percent: Option<u8> }`，前端优先使用 `global_percent`
4. **保留回退路径**：前端若未收到全局进度，回退到局部计算（兼容旧版本/测试环境）

**French Exit 实践**：
- 7 个 Scanner 并行，权重分配：fs 50% + browser 15% + system 15% + 其他各 5%
- 修改范围：Rust `ScanProgress` / `ProgressEvent` 结构 → `ScannerRegistry::scan_impl` 加权计算 → 前端 `ScanPage.tsx` 优先使用
- 测试：后端 129 测、前端 51 测全绿

**可复用性**：★★★★★ —— 任何多任务并行 + 进度条展示的场景都适用

*最后更新：2026-05-22*
