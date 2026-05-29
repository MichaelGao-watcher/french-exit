# vibe-coding-project-sop — 跨项目经验母库

> 本文件聚合自多个项目的经验沉淀。所有条目均标注来源。
> 新增经验时按项目区分；母库自身经验标注 `[母库]`。
>
> **标签速查**：TAG:pagination | TAG:state-management | TAG:i18n | TAG:testing | TAG:dom | TAG:api-design | TAG:build-env | TAG:ux | TAG:security | TAG:data | TAG:architecture | TAG:debugging | TAG:ai-workflow | TAG:cross-platform | TAG:performance

---

## 技术经验

| # | 标签 | 严重度 | 经验 | 来源模块 |
|---|------|--------|------|---------|
| 1 | TAG:build-env TAG:testing | INFO | 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 Web Worker（如 Stockfish）时必须启 HTTP 服务器 [来源:blindfold-chess @2026-05-21] | EngineModule |
| 2 | TAG:dom TAG:api-design | WARNING | 手写 IIFE 模块时，用 `window.ModuleName = Module` 暴露 API，内部私有变量用下划线前缀，避免全局泄漏 [来源:blindfold-chess @2026-05-21] | 所有 js/*.js |
| 3 | TAG:testing | INFO | 浏览器集成测试用 TestRunner（自定义极简框架），保持与 Node 测试同一套断言 API，降低切换成本 [来源:blindfold-chess @2026-05-21] | docs/tests/ |
| 4 | TAG:testing | INFO | Canvas 图表渲染在浏览器中测试，Node 环境用 Mock 2D context 跳过绘制验证，各测其责 [来源:blindfold-chess @2026-05-21] | StatsModule |
| 5 | TAG:data TAG:api-design | INFO | PGN 解析器对空/无效输入返回 `[]`（空数组）而非 `null`，调用方需区分"无走法"和"解析失败" [来源:blindfold-chess @2026-05-21] | ReplayModule |
| 6 | TAG:dom | WARNING | `cloneNode(true)` 替换含 SVG 的按钮会导致 SVG 渲染异常（显示不完整）；移除事件监听器应优先用 `removeEventListener` + 命名函数，避免替换 DOM 元素 [来源:blindfold-chess @2026-05-21] | SettingsModule |
| 7 | TAG:dom | WARNING | 匿名事件监听器无法被后续代码移除；需要动态解除绑定的监听器必须用命名函数（暴露到 `window` 或模块内部变量） [来源:blindfold-chess @2026-05-21] | common.js / settings.js |
| 8 | TAG:dom TAG:ux | WARNING | 屏幕切换导航不能只隐藏上一个屏幕，必须遍历 `.screen` 全部隐藏后再显示目标，否则多层屏幕重叠 [来源:blindfold-chess @2026-05-21] | 全局导航 |
| 9 | TAG:dom | INFO | SVG path 中密集参数（如 `a2 2 0 0 1-2.83 0`）在某些浏览器中可能解析异常，命令与参数间保留空格更稳妥 [来源:blindfold-chess @2026-05-21] | index.html SVG |
| 10 | TAG:dom | WARNING | `document` 级事件监听器若引用了某个 DOM 元素，该元素被替换后监听器仍会按旧引用判断，导致逻辑错误（如点击新按钮被误判为"点击外部"） [来源:blindfold-chess @2026-05-21] | settings.js / common.js |
| 11 | TAG:ai-workflow | INFO | 项目文档结构会随时间进化，"存档"或"恢复"操作前应先 `ls`/`glob` 确认当前文件系统现状，避免按历史路径写入已不存在的文件 [来源:blindfold-chess @2026-05-21] | 文档维护 |
| 12 | TAG:ux | CRITICAL | **UI 布局/样式不要猜测用户意图**：候选走法开关经历了 5 次位置/样式反复，每次修改后用户都不满意；应在设计阶段出草图或描述供用户确认，再编码 [来源:blindfold-chess @2026-05-21] | BlindfoldModule UI |
| 13 | TAG:api-design | CRITICAL | **引擎候选走法的调用时机决定产品逻辑正确性**：用户走完后立即 `goMultiPv` 分析的是对手局面；若要提示用户，必须在引擎执行完走法后、轮到白方时再调用 [来源:blindfold-chess @2026-05-21] | EngineModule / BlindfoldModule |
| 14 | TAG:data TAG:api-design | WARNING | **引擎返回 UCI（e2e4），用户界面必须用 SAN（e4）**：`goMultiPv` 回调中的 `move` 是 UCI 坐标格式，展示前需映射为 SAN，否则用户无法阅读 [来源:blindfold-chess @2026-05-21] | BlindfoldModule |
| 15 | TAG:dom | WARNING | **静态 HTML 结构与动态渲染模块的 DOM 冲突**：`index.html` 中预置了完整棋盘结构，而 `BoardRenderer.create()` 会重新创建完整结构，导致两组行标注同时存在；应只保留空容器让渲染器全权负责 [来源:blindfold-chess @2026-05-21] | BoardRenderer / index.html |
| 16 | TAG:testing | WARNING | **删除功能必须同步删除对应测试**：移除 `showHints` / `multiPvSetting` 后，相关测试会立即失败；功能清理和测试清理应视为同一任务 [来源:blindfold-chess @2026-05-21] | 测试维护 |
| 17 | TAG:ux | WARNING | **焦点管理是盲棋产品的核心体验**：进入对局自动 `input.focus()`、引擎走完后恢复焦点、全局 Enter 键将焦点拉回输入框——三者缺一不可 [来源:blindfold-chess @2026-05-21] | BlindfoldModule UX |
| 18 | TAG:i18n | CRITICAL | **i18n 分散架构必然导致翻译遗漏**：当项目同时存在"全局字典 + 模块私有字典 + 硬编码"三种翻译方式时，新增功能几乎必然漏掉其中一种或多种。唯一可持续的方案是"单一字典源" [来源:blindfold-chess @2026-05-21] | 全站 i18n |
| 19 | TAG:i18n | WARNING | **JS 中的硬编码人类可读字符串是翻译遗漏的重灾区**：HTML 中的 `data-i18n` 至少能被肉眼扫描到，但 JS 逻辑里直接写的用户可见字符串没有显式标记，切换语言时完全失效 [来源:blindfold-chess @2026-05-21] | common.js / coordinate.js / blindfold.js |
| 20 | TAG:i18n | WARNING | **复制粘贴是 i18n 错误的常见来源**：将中文值直接粘贴进英文字典，或反之，属于低级但高频的疏忽 [来源:blindfold-chess @2026-05-21] | common.js |
| 21 | TAG:i18n TAG:architecture | WARNING | **模块内部字典若从不主动更新 DOM，则纯属冗余**：welcome.js 有 `_i18n` 和 `_t()`，但从不调用，完全依赖 common.js 的 `updateTexts()`。这种"假私有字典"不仅没用，还会给维护者造成"这里已经翻译了"的错觉 [来源:blindfold-chess @2026-05-21] | welcome.js |
| 22 | TAG:i18n TAG:dom | WARNING | **settings.js 的独立字典与 common.js 的全局扫描存在竞争**：settings panel 的元素带 `data-i18n`，settings.js 自己 `_updateAllTexts()` 会覆盖，但 common.js 的 `updateTexts()` 也会扫到，如果 common.js 缺键，用户会看到 key 名闪一下才被正确文本覆盖 [来源:blindfold-chess @2026-05-21] | settings.js / common.js |
| 23 | TAG:build-env | WARNING | **已删除的 JS 文件若不从 index.html 移除引用，会导致 404**：功能清理和引用清理必须是同一任务 [来源:blindfold-chess @2026-05-21] | 代码清理 |
| 24 | TAG:testing | WARNING | **Node 测试不对 UI 文本做断言，无法捕获翻译错误**：只测 API 形状和数值，不检查按钮文字、提示语等人类可读内容。翻译质量必须靠人工检查或专门的 UI 测试覆盖 [来源:blindfold-chess @2026-05-21] | 测试策略 |
| 25 | TAG:testing TAG:architecture | CRITICAL | **删除生产代码的 fallback 函数前，必须先评估测试环境是否提供了该依赖**：架构统一重构必须同时改代码+测试，只改一边会导致测试雪崩 [来源:blindfold-chess @2026-05-21] | 全站 i18n |
| 26 | TAG:testing | WARNING | **`localStorage` mock 必须支持 `setItem` 持久化**：测试中 `global.localStorage = { getItem: () => null }` 会让 `t()` 永远读取默认语言，导致语言切换测试失效 [来源:blindfold-chess @2026-05-21] | 测试基础设施 |
| 27 | TAG:dom TAG:i18n | WARNING | **全局 `updateTexts()` 与模块私有 `_updateXxx()` 可能存在 DOM 竞争**：两者操作同一 DOM 元素。测试必须验证最终渲染结果，而非中间状态 [来源:blindfold-chess @2026-05-21] | settings.js / common.js |
| 28 | TAG:ux | INFO | **配置类设置项用「弹窗选择」优于「循环切换」**：循环切换隐藏了全部选项，用户不知道有哪些风格；弹窗一次展示所有选项+预览，认知负荷更低 [来源:blindfold-chess @2026-05-21] | SettingsModule UI |
| 29 | TAG:dom | WARNING | **`cloneNode(true)` 无法移除旧事件监听器，它只是复制了 DOM 结构**：真正安全的解绑是 `removeEventListener` + 保存引用 [来源:blindfold-chess @2026-05-21] | settings.js |
| 30 | TAG:ux TAG:architecture | WARNING | **UI 风格不一致的根因通常是「硬编码颜色」**：引入统一的「棋盘风格配置源」后，所有棋盘自动同步，消除不一致的根因。**更关键的是：新增模块前必须搜索项目中是否有可复用的同类组件**（如 `BoardRenderer`），禁止手写已有功能 [来源:blindfold-chess @2026-05-21] | BoardRenderer / coordinate.js |
| 31 | TAG:ux TAG:architecture | WARNING | **功能入口迁移需要同步更新「正向路径」和「反向路径」**：不仅要添加新入口，还要移除旧入口，否则用户会在两个地方看到同一功能 [来源:blindfold-chess @2026-05-21] | WelcomeModule / index.html |
| 32 | TAG:data TAG:architecture | INFO | **数据层的双语字段与代码层的硬编码分支是两个问题**：区分"数据双语"和"代码分支"可避免过度重构 [来源:blindfold-chess @2026-05-21] | replay.js / data/games.js |
| 33 | TAG:testing | WARNING | **测试中断言的具体文本值是重构的敏感点**：重构前应先审计测试中的文本断言，预估需要调整的范围 [来源:blindfold-chess @2026-05-21] | 测试维护 |
| 34 | TAG:data TAG:build-env | WARNING | **数据文件中的引号嵌套是极易被忽视的语法陷阱**：在真实浏览器中会抛出 `SyntaxError` 并阻断后续脚本执行 [来源:blindfold-chess @2026-05-21] | data/games.js |
| 35 | TAG:testing | WARNING | **Node 测试全过 ≠ 浏览器表现正常**：必须用 headless 浏览器（playwright）才能捕获浏览器特有错误 [来源:blindfold-chess @2026-05-21] | 测试策略 |
| 36 | TAG:testing TAG:debugging | INFO | **playwright 是定位浏览器特有 bug 的有效手段**：通过 `page.add_init_script` 注入错误监听器 + `page.on('pageerror')`，可以精确定位到出错的文件、行号和列号 [来源:blindfold-chess @2026-05-21] | 调试工具 |
| 37 | TAG:architecture | INFO | **通用配置层设计能降低新增模式的边际成本**：新增对局模式时只需加一行 `else if` 分发逻辑，无需重复造 DOM/CSS [来源:blindfold-chess @2026-05-21] | 架构设计 |
| 38 | TAG:api-design | INFO | **向后兼容接口设计能减少重构的连锁反应**：旧接口继续工作，内部映射新参数，所有旧测试和外部调用点无需改动 [来源:blindfold-chess @2026-05-21] | API 设计 |
| 39 | TAG:testing TAG:dom | INFO | 浏览器集成测试阶段发现 welcome.js / replay.js / stats.js 的 DOM 事件绑定遗漏 [来源:blindfold-chess @2026-05-21] |  |
| 40 | TAG:testing | INFO | Node 测试覆盖逻辑，浏览器测试覆盖 DOM 集成，两者互补 [来源:blindfold-chess @2026-05-21] |  |
| 41 | TAG:ai-workflow | INFO | `AGENTS.md` 定义触发词和行为约束，`status.md` 记录动态进度，分工明确，新会话读 2 份文件即可开工 [来源:french-exit @2026-05-21] |  |
| 42 | TAG:ai-workflow | INFO | 每批次开发完成后同步更新进度文档，避免新会话迷路 [来源:blindfold-chess @2026-05-21] |  |
| 43 | TAG:data | INFO | **手工构建100条结构化数据不现实**：经典棋局的 PGN 分散在各网站，无统一免费 API；手动录入工作量巨大且易出错 [来源:blindfold-chess @2026-05-21] |  |
| 44 | TAG:ai-workflow | WARNING | **WriteFile 不适合超大特殊字符内容**：含大量引号/换行的长文本会因 JSON 转义失败；应改用本地 Python/Node 脚本生成，或提前准备好数据文件 [来源:blindfold-chess @2026-05-21] |  |
| 45 | TAG:cross-platform TAG:ai-workflow | WARNING | **Shell here-document 在 Windows git bash 中不可靠**：含引号的多行复杂脚本会被截断或解析错误；应先 `WriteFile` 写脚本，再 `Shell` 执行 [来源:blindfold-chess @2026-05-21] |  |
| 46 | TAG:i18n TAG:ai-workflow | WARNING | **翻译检查必须是独立任务，不能依赖"开发时顺手做"**：本次检查发现 25+ 处遗漏，分布在 HTML、JS 字典、硬编码三个层面 [来源:blindfold-chess @2026-05-21] |  |
| 47 | TAG:ai-workflow | INFO | **涉及 7+ 文件读改测的架构重构，应新开会话执行**：继续塞进系统性重构容易触发窗口压缩，导致信息丢失 [来源:blindfold-chess @2026-05-21] |  |
| 48 | TAG:build-env | INFO | GitHub Pages 国内访问需代理；unpkg CDN 加载 Stockfish 可能超时，需考虑离线备选方案 [来源:blindfold-chess @2026-05-21] |  |
| 49 | TAG:cross-platform | WARNING | Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚本时优先用正斜杠或 `path.join` [来源:blindfold-chess @2026-05-21] |  |
| 50 | TAG:api-design | INFO | `windows-rs` 0.61 的错误处理统一用 `.map_err(|e| ...)`，其中 `e` 是 `windows::core::Error` [来源:french-exit @2026-05-21] | `resource/controller.rs` |
| 51 | TAG:api-design | INFO | `GetDiskFreeSpaceExW` 传 `&HSTRING` 作为路径参数，`Option<&mut u64>` 接收可用字节 [来源:french-exit @2026-05-21] | `executor/pack.rs` |
| 52 | TAG:api-design | INFO | CPU% 精确计算只需 `GetProcessTimes` + wall clock elapsed，不需要 `GetSystemTimes` [来源:french-exit @2026-05-21] | `resource/controller.rs` |
| 53 | TAG:api-design | INFO | `FILETIME` 转 u64：`((high as u64) << 32) | (low as u64)`，单位是 100ns [来源:french-exit @2026-05-21] | `resource/controller.rs` |
| 54 | TAG:api-design | INFO | `Arc<dyn Fn(...) + Send + Sync>` 是 Rust 中给同步结构体注入回调的标准方式 [来源:french-exit @2026-05-21] | `executor/pack.rs` |
| 55 | TAG:testing | INFO | Tauri 前端用 vitest + jsdom 测试时，必须在 `setup.ts` 中 `vi.mock()` 所有 `@tauri-apps/api/*` 模块 [来源:french-exit @2026-05-21] | `src/test/setup.ts` |
| 56 | TAG:build-env | INFO | 若 `@tauri-apps/api/xxx` 模块不存在（如 v2 移除了 `fs`），用 **vite alias** 指向本地 mock，而非试图安装 [来源:french-exit @2026-05-21] | `vite.config.ts` |
| 57 | TAG:testing | INFO | Controlled checkbox 的测试用 `@testing-library/user-event` 的 `user.click()`，不要用 `fireEvent.click()` [来源:french-exit @2026-05-21] | `ResultsPage.test.tsx` |
| 58 | TAG:api-design | INFO | `tokio::sync::mpsc::Sender::try_send()` 适合非阻塞的进度回调，避免 Scanner 被 channel 阻塞 [来源:french-exit @2026-05-21] | `orchestrator/mod.rs` |
| 59 | TAG:state-management | CRITICAL | **绝对不要**在 `setState` 的 updater 函数内部调用 `dispatch()` 或其他 setState，会触发 React "渲染时更新" 警告 [来源:french-exit @2026-05-21] | `ResultsPage.tsx` |
| 60 | TAG:state-management | CRITICAL | `useEffect` 依赖 `state.xxx.size === 0` 作为触发条件时，容易形成死循环（用户操作 → size 变 0 → effect 重设 → 又变回来） [来源:french-exit @2026-05-21] | `ResultsPage.tsx` |
| 61 | TAG:state-management | WARNING | `useRef` 作为"只执行一次"的标志，比依赖数组更可靠，尤其涉及批量初始化逻辑时 [来源:french-exit @2026-05-21] | `ResultsPage.tsx` |
| 62 | TAG:testing | INFO | **测试驱动暴露 Bug**：ResultsPage 的默认勾选死循环是在写单元测试时发现的，手工测试几乎不可能复现（需要恰好取消所有勾选） [来源:french-exit @2026-05-21] |  |
| 63 | TAG:testing | INFO | **结论**：前端状态管理类的 bug，单元测试是最有效的发现手段，远超手工测试 [来源:french-exit @2026-05-21] |  |
| 64 | TAG:ai-workflow | INFO | `prompt-next-session.md` 的问题：每次都要重写环境初始化、模块速查表等**不变内容** [来源:french-exit @2026-05-21] |  |
| 65 | TAG:ai-workflow | INFO | **改进**：`status.md`（活文档，只记录变化）+ `AGENTS.md`（固定规则） [来源:french-exit @2026-05-21] |  |
| 66 | TAG:ai-workflow | INFO | **收益**：新会话读 2 份文件即可开工，维护成本降低 80% [来源:french-exit @2026-05-21] |  |
| 67 | TAG:ai-workflow | WARNING | **横跨工具层和应用层的词汇必须确认语境**。用户问"一个项目多个终端能否实现同步处理进度"——"终端"可以指并行 executor、Kimi CLI 多窗口、或多会话。默认跳到代码层面分析可能完全偏题 [来源:french-exit @2026-05-21] |  |
| 68 | TAG:ai-workflow | WARNING | **工具硬性限制不要绕圈分析可行性**。Kimi CLI 多窗口无 IPC、无共享内存、无实时同步——回答应直接给结论 + 风险 + 替代方案，省掉技术可行性分析 [来源:french-exit @2026-05-21] |  |
| 69 | TAG:ai-workflow | WARNING | **从 SOP 模板采纳更新时，必须逐字核对关键字段，不要凭记忆改写**。触发词「存档」错误抄写为「存储」，原因是未逐字比对就按直觉填写 [来源:french-exit @2026-05-21] |  |
| 70 | TAG:cross-platform TAG:build-env | CRITICAL | 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/french-exit`）后编译 [来源:french-exit @2026-05-21] |  |
| 71 | TAG:cross-platform TAG:build-env | INFO | `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test --no-run` 同理 [来源:french-exit @2026-05-21] |  |
| 72 | TAG:debugging | INFO | **`0xc0000139` 不一定是 UCRT/MinGW 兼容性 issue**。先跑一个**最简单 lib 测试**（空 crate + `cargo test --lib`），如果能过，就说明工具链没问题，问题在项目的特定代码中 [来源:french-exit @2026-05-21] |  |
| 73 | TAG:debugging | WARNING | **`cargo test --bin` 能过、`cargo test --lib` 崩溃** → 问题出在**仅被 lib 测试链接的代码**中（bin 测试做了死代码消除，没链接到问题代码）。这是极强的定位信号 [来源:french-exit @2026-05-21] |  |
| 74 | TAG:debugging | INFO | **定位代码的最快方法**：清空 `lib.rs` 只保留一个空测试，逐步 `pub mod` 添加模块，直到崩溃复现。比分析 PE 导入表快 10 倍 [来源:french-exit @2026-05-21] |  |
| 75 | TAG:cross-platform TAG:build-env | CRITICAL | **`tauri::AppHandle` 出现在 `async fn` 签名中 + MinGW = `STATUS_ENTRYPOINT_NOT_FOUND`**。workaround：把这些函数拆到子模块，用 `#[cfg(not(test))]` 条件编译，测试模式下不链接 [来源:french-exit @2026-05-21] |  |
| 76 | TAG:testing TAG:cross-platform | INFO | **`#[cfg(not(test))]` 隔离问题代码**是零副作用的修复手法：release 构建完全不受影响，测试逻辑移至独立模块继续跑 [来源:french-exit @2026-05-21] |  |
| 77 | TAG:dom TAG:ux | INFO | `useRef` + `mousedown` 监听实现点击外部关闭 [来源:french-exit @2026-05-21] |  |
| 78 | TAG:ux | INFO | CSS `@keyframes dropdownIn` 实现淡入+位移动画 [来源:french-exit @2026-05-21] |  |
| 79 | TAG:ux | INFO | 年月日联动限制（如今年只显示到当前月） [来源:french-exit @2026-05-21] |  |
| 80 | TAG:build-env | INFO | `cargo tauri dev` 必须在**交互式 Windows 桌面会话**中运行，无法通过远程/后台任务启动（WebView2 需要 GUI 上下文） [来源:french-exit @2026-05-21] |  |
| 81 | TAG:build-env | INFO | **替代方案**：`npm run dev` 启动 Vite 服务器 → 浏览器访问 `http://localhost:1420` → 可实时预览前端 UI（HMR 热更新），但 IPC 调用会失败 [来源:french-exit @2026-05-21] |  |
| 82 | TAG:build-env | INFO | **完整功能验证**：仍需本地运行 `cargo tauri dev` 或双击 release `.exe` [来源:french-exit @2026-05-21] |  |
| 83 | TAG:data TAG:performance | WARNING | 不要一次性加载所有完整 `TraceItem` 到前端（内存 + DOM 渲染压力大） [来源:french-exit @2026-05-21] |  |
| 84 | TAG:architecture TAG:data | INFO | 正确做法：后端提供**轻量摘要接口**（只返回 id + category + suggested_action），前端用它批量生成 decisions [来源:french-exit @2026-05-21] |  |
| 85 | TAG:pagination TAG:architecture | WARNING | 用户实际浏览仍按分页，但"全选全部"走轻量接口，两者解耦 [来源:french-exit @2026-05-21] |  |
| 86 | TAG:pagination TAG:state-management TAG:security | CRITICAL | **事故经过**：ResultsPage 默认自动勾选所有扫描结果 → 用户点击"全选全部"（以为是全选当前页，实际是全选全部）→ 确认页看到"将删除 17,706 个文件"但未警觉 → 执行后大量文件丢失 [来源:french-exit @2026-05-21] |  |
| 87 | TAG:pagination TAG:state-management | CRITICAL | **根因链**：默认勾选 × deselectAll 只清当前页 × ConfirmPage 遍历 scanResults（分页未加载完整）= 三重 bug 叠加 [来源:french-exit @2026-05-21] |  |
| 88 | TAG:security TAG:ux | CRITICAL | **教训**：涉及删除的安全工具，**默认安全 > 默认便利**。所有选择必须用户显式操作，任何"帮你选好"的设计都需反复审视 [来源:french-exit @2026-05-21] |  |
| 89 | TAG:pagination TAG:state-management | WARNING | **原实现**：`deselectAll` 只遍历 `searchedItems`（当前页数据），从 `selectedIds` 中移除 → 其他分页的选中状态仍保留 [来源:french-exit @2026-05-21] |  |
| 90 | TAG:pagination TAG:state-management | INFO | **修复**：`deselectAll` 清空 `selectedIds` 为 `new Set()`，同时 `dispatch({ type: "SET_DECISIONS", payload: new Map() })` 清空全部 decisions [来源:french-exit @2026-05-21] |  |
| 91 | TAG:pagination TAG:state-management | WARNING | **教训**：跨分页操作时，"取消"必须与"全选"的对称——全选影响多大范围，取消就必须影响多大范围 [来源:french-exit @2026-05-21] |  |
| 92 | TAG:pagination TAG:state-management | WARNING | **原实现**：ConfirmPage 遍历 `state.scanResults`，过滤出选中的项 → 分页未加载的项完全丢失 [来源:french-exit @2026-05-21] |  |
| 93 | TAG:pagination TAG:state-management | INFO | **修复**：遍历 `state.decisions`，每项在 `scanResults` 中查找详细信息，找不到时用 `name: id` 兜底 [来源:french-exit @2026-05-21] |  |
| 94 | TAG:pagination TAG:architecture | WARNING | **教训**：在分页/懒加载架构中，**用户操作集合（decisions）是主数据源，展示数据（scanResults）是从属数据源**。确认/汇总逻辑必须基于操作集合 [来源:french-exit @2026-05-21] |  |

---

## 流程经验

### 问题发现机制
- 测试驱动开发能在手工测试无法触及的边界条件下发现 bug（如"恰好取消所有勾选"触发死循环）[来源:french-exit @2026-05-21]

### 文档维护
- `AGENTS.md` 定义触发词和行为约束，`status.md` 记录动态进度，两者分工明确，新会话读 2 份文件即可开工 [来源:french-exit @2026-05-21]
- 涉及 7+ 文件读改测的架构重构，应新开会话执行，避免上下文压缩导致信息丢失 [来源:blindfold-chess @2026-05-21]

### 环境陷阱
- 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径后编译 [来源:french-exit @2026-05-21]
- `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test --no-run` 同理 [来源:french-exit @2026-05-21]
- Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚本时优先用正斜杠或 `path.join` [来源:blindfold-chess @2026-05-21]

---

*最后更新：2026-05-21 | 来源：blindfold-chess, french-exit | 条目数：94 | CRITICAL: 8 | WARNING: 33 | INFO: 53*
| | **UI 布局/样式不要猜测用户意图**：候选走法开关经历了 5 次位置/样式反复（设置面板 → header 图标 → 滑动开关 → 圆形按钮+标签 → 纯文字 → 下移），每次修改后用户都不满意；应在设计阶段出草图或描述供用户确认，再编码 [来源:blindfold-chess @2026-05-22] | BlindfoldModule UI |
| | **引擎候选走法的调用时机决定产品逻辑正确性**：用户走完后立即 `goMultiPv` 分析的是对手（黑方）局面，展示的是"对手会怎么走"；若要提示用户，必须在引擎执行完走法后、轮到白方时再调用 `goMultiPv` [来源:blindfold-chess @2026-05-22] | EngineModule / BlindfoldModule |
| | **引擎返回 UCI（e2e4），用户界面必须用 SAN（e4）**：`goMultiPv` 回调中的 `move` 是 UCI 坐标格式，展示前需通过 `_game.moves({verbose:true})` 映射为 SAN，否则用户无法阅读 [来源:blindfold-chess @2026-05-22] | BlindfoldModule |
| | **静态 HTML 结构与动态渲染模块的 DOM 冲突**：`index.html` 中预置了完整棋盘结构（含行/列标注），而 `BoardRenderer.create()` 会在容器内重新创建完整结构，导致两组行标注同时存在；应只保留空容器让渲染器全权负责 [来源:blindfold-chess @2026-05-22] | BoardRenderer / index.html |
| | **删除功能必须同步删除对应测试**：移除 `showHints` / `multiPvSetting` 后，`test-settings-node.js` 中相关测试会立即失败；功能清理和测试清理应视为同一任务 [来源:blindfold-chess @2026-05-22] | 测试维护 |
| | **焦点管理是盲棋产品的核心体验**：进入对局自动 `input.focus()`、引擎走完后恢复焦点、全局 Enter 键将焦点拉回输入框——三者缺一不可，否则用户被迫频繁使用鼠标 [来源:blindfold-chess @2026-05-22] | BlindfoldModule UX |
| | **JS 中的硬编码人类可读字符串是翻译遗漏的重灾区**：HTML 中的 `data-i18n` 至少能被肉眼扫描到，但 JS 逻辑里直接写的 `"Time Up!"`、`"✓ 已复制"` 没有显式标记，切换语言时完全失效 [来源:blindfold-chess @2026-05-22] | common.js / coordinate.js / blindfold.js |
| | **复制粘贴是 i18n 错误的常见来源**：将中文值直接粘贴进英文字典（如 `boardToggle: "显示棋盘"`），或反之，属于低级但高频的疏忽 [来源:blindfold-chess @2026-05-22] | common.js |
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
| | **通用配置层设计能降低新增模式的边际成本**：将"选择阵营 + 难度"抽象为 `gameSetupScreen`，由 `WelcomeModule` 维护 `_pendingMode`，新增对局模式时只需加一行 `else if` 分发逻辑，无需重复造 DOM/CSS [来源:blindfold-chess @2026-05-22] | 架构设计 |
| | **向后兼容接口设计能减少重构的连锁反应**：`BlindfoldModule.init('medium')` 继续工作，内部映射为 `{side:'w', elo:1400}`，所有旧测试和外部调用点无需改动 [来源:blindfold-chess @2026-05-22] | API 设计 |
| | `AGENTS.md` 定义触发词和行为约束，`STATE.md`（现 status.md）记录动态进度，分工明确 [来源:blindfold-chess @2026-05-22] |  |
| | **手工构建100条结构化数据不现实**：经典棋局的 PGN 分散在各网站，无统一免费 API；手动录入100盘完整 PGN 工作量巨大且易出错 [来源:blindfold-chess @2026-05-22] |  |
| | **翻译检查必须是独立任务，不能依赖"开发时顺手做"**：本次检查发现 25+ 处遗漏，分布在 HTML、JS 字典、硬编码三个层面。分批迭代时，每新增一个 `data-i18n` 或用户可见字符串，必须同步到唯一字典源，否则必然遗漏。 [来源:blindfold-chess @2026-05-22] |  |
| | **涉及 7+ 文件读改测的架构重构，应新开会话执行**：当前会话在查漏补缺后已承载大量上下文，继续塞进系统性重构容易触发窗口压缩，导致信息丢失。 [来源:blindfold-chess @2026-05-22] |  |
| | **横跨工具层和应用层的词汇必须确认语境**。用户问"一个项目多个终端能否实现同步处理进度"——"终端"可以指 French Exit 的并行 executor、Kimi CLI 的多窗口、或 ai-project-skeleton 的多会话。我默认跳到了代码层面分析并行化，结果完全偏题。 [来源:french-exit @2026-05-22] |  |
| | **工具硬性限制不要绕圈分析可行性**。Kimi CLI 多窗口无 IPC、无共享内存、无实时同步——这不是"有难度"，是"设计上就不支持"。回答应直接给结论 + 风险 + 替代方案，省掉技术可行性分析 [来源:french-exit @2026-05-22] |  |
| | **从 SOP 模板采纳更新时，必须逐字核对关键字段，不要凭记忆改写**。本轮将 `vibe-coding-project-sop/AGENTS.md` 中的触发词「存档」错误抄写为「存储」，原因是未逐字比对就按直觉填写。SOP 模板中的占位符（如 `[项目名]`）在实际项目中需替换，但硬规则（触发词、流程步骤）应原样保留。 [来源:french-exit @2026-05-22] |  |
| | **`tauri::AppHandle` 出现在 `async fn` 签名中 + MinGW = `STATUS_ENTRYPOINT_NOT_FOUND`**。原因未知（PE 导入表生成 bug？），但 workaround 明确：把这些函数拆到子模块，用 `#[cfg(not(test))]` 条件编译，测试模式下不链接 [来源:french-exit @2026-05-22] |  |

| 95 | 在 Kimi Code CLI 上安装 Superpowers 需分 A/B 方案：A 方案替换 CLI 为 feat/hook-inject-prompt 分支实现自动注入；B 方案只克隆 skills 到 ~/.kimi/skills/ 手动调用，零风险 | 工具链配置 | [来源:vibe-coding-project-sop @2026-05-22] |
| 96 | 评估 skill 是否适合当前项目时，不应只看功能描述，而应对比项目形态（纯文档/工具型项目不需要 frontend-design 等前端实现型 skill） | 项目评估 | [来源:vibe-coding-project-sop @2026-05-22] |
| 97 | PowerShell 5.1 默认以 Windows-1252 编码读取无 BOM 的 UTF-8 文件，中文字符被解码为乱码后会破坏引号匹配，导致 UnexpectedToken 语法错误。修复方案：文件头添加 UTF-8 BOM（EF BB BF） | 环境兼容 | [来源:vibe-coding-project-sop @2026-05-22] |
| 98 | ModelScope 首页 HTTP 访问速度（~8KB/s）与实际 CDN 模型下载速度（~2.5MB/s）差异巨大，下载 .gguf 模型文件时应直接测试 CDN 链接而非首页 | 网络诊断 | [来源:vibe-coding-project-sop @2026-05-22] |
| 99 | GitHub 下载完全不可达时，可尝试加速镜像 gh.llkk.cc，实测速度 300KB/s+，支持断点续传 | 网络诊断 | [来源:vibe-coding-project-sop @2026-05-22] |
| 100 | llama.cpp 的 server 模式兼容 OpenAI API 格式（/v1/chat/completions），可直接作为 Ollama 的轻量替代，体积仅 18MB vs Ollama 2GB | 工具选型 | [来源:vibe-coding-project-sop @2026-05-22] |
| 101 | macOS 无 Homebrew 时，GitHub CLI 可直接下载官方 zip 包，解压后放到 `~/bin/` 即可使用，无需 sudo | 工具链配置 | [来源:vibe-coding-project-sop @2026-05-23] |
| 102 | `gh` 底层是 Go 程序，默认不走系统代理。访问 GitHub API 时必须显式设置 `HTTPS_PROXY` 环境变量 | 网络诊断 | [来源:vibe-coding-project-sop @2026-05-23] |
| 103 | 从 SSH 协议切换到 HTTPS + `gh auth setup-git` 后，`git push` 完全由 gh 管理的 Token 驱动，无需维护 SSH 密钥对 | 工具链配置 | [来源:vibe-coding-project-sop @2026-05-23] |

| 104 | TAG:build-env | INFO | Git for Windows 的 bash `/tmp` 与 PowerShell `$env:TEMP` 指向同一物理目录（`C:\Users\<user>\AppData\Local\Temp`），跨 shell 操作文件无需移动 | 环境兼容 | [来源:vibe-coding-project-sop @2026-05-24] |
| 105 | TAG:build-env | INFO | 国内下载 HuggingFace 模型时，ModelScope 是比 hf-mirror 更可靠的 fallback（后者可能 404 或同步延迟） | 网络诊断 | [来源:vibe-coding-project-sop @2026-05-24] |
| 106 | TAG:build-env | WARNING | Windows 非管理员运行 PowerShell 脚本时，`New-NetFirewallRule` 会失败，但 `llama-server` 本身可正常启动；首次运行需提升权限 | 环境兼容 | [来源:vibe-coding-project-sop @2026-05-24] |
| | `/c` 执行完关闭窗口；`/k` 保持窗口打开 [来源:french-exit @2026-05-29] |  |
| | `-WindowStyle Minimized` 最小化不干扰工作 [来源:french-exit @2026-05-29] |  |
| | **根因**：CSS `fixed` + `z-50` 的元素默认接收鼠标事件，即使视觉上看起来透明也会拦截点击 [来源:french-exit @2026-05-29] |  |
| | **修复**：给所有非交互性的 `fixed` 装饰元素统一添加 `pointer-events-none` [来源:french-exit @2026-05-29] |  |
| | **教训**：任何使用 `fixed`/`absolute` + 高 `z-index` 的纯展示元素，必须默认视为点击拦截器 [来源:french-exit @2026-05-29] |  |
| | **教训**：E2E 测试不是写一次就完，它是前端契约测试。UI 迭代时必须同步评估对 selector、交互流程、状态断言的影响 [来源:french-exit @2026-05-29] |  |
| | **方案**：`ScannerRegistry::scan_impl` 的 `progress_cb` 在每次上报进度前 `while *pause_rx.borrow() { sleep(100ms) }` [来源:french-exit @2026-05-29] |  |
| | **局限**：如果 scanner 长时间不调用 `progress`（如读取超大文件），暂停会有延迟 [来源:french-exit @2026-05-29] |  |
| | **教训**：对于已成型的大型 trait 实现体系，优先在调度层（registry）而非实现层（scanner）插入横切关注点 [来源:french-exit @2026-05-29] |  |
| | 7 个 Scanner 并行，权重分配：fs 50% + browser 15% + system 15% + 其他各 5% [来源:french-exit @2026-05-29] |  |
| | 修改范围：Rust `ScanProgress` / `ProgressEvent` 结构 → `ScannerRegistry::scan_impl` 加权计算 → 前端 `ScanPage.tsx` 优先使用 [来源:french-exit @2026-05-29] |  |
| | 测试：后端 129 测、前端 51 测全绿 [来源:french-exit @2026-05-29] |  |
| | **HTTP Digest 认证回写规则**：刷新 token 时只更新 `jycmOpenApiCookie`，必须保留 `accessKey` 和 `secretKey`。若覆盖整个文件丢失 AK/SK，后续 Digest 刷新将永久失败 [来源:qianniu_business_analytics @2026-05-29] | auth.md / auth/jycm.json |
| | **日期区间陷阱**：后端按 "< endDate" 解析，`T23:59:59.999+08:00` 和 `Z`（UTC）后缀都会导致多返回一天数据。必须用 `T00:00:00+08:00` [来源:qianniu_business_analytics @2026-05-29] | SKILL.md / fetch-data.md |
| | **shopIds 类型陷阱**：后端要求 `List<String>`（JSON 字符串数组），传入数字数组会导致参数校验失败 [来源:qianniu_business_analytics @2026-05-29] | jycm_fetch_sycm_shop.py |
| | **Token Key 只问一次**：凭证文件存在但 Cookie 过期时，必须先走 Digest 自动刷新，绝不能直接问用户。这是用户体验的关键红线 [来源:qianniu_business_analytics @2026-05-29] | auth.md |
| | **Markdown 一源多用**：对话交付和钉钉推送使用同一套 Markdown 正文，避免"对话一版、钉钉一版"的信息不一致 [来源:qianniu_business_analytics @2026-05-29] | report-analyze.md |
| | **多店合并 vs 跨平台**：同为淘系的多家店铺可一次 `createAndDownload` 合并取数；但淘系与京东/抖音混用时必须明确拒绝，不得伪造数据 [来源:qianniu_business_analytics @2026-05-29] | SKILL.md | | TAG:build-env TAG:testing [来源:vibe-coding-project-sop @2026-05-22] | INFO | 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 Web Worker（如 Stockfish）时必须启 HTTP 服务器 [来源:blindfold-chess @2026-05-21] | EngineModule |
| | **subprocess → direct import 重构**：`subprocess.run` 调用同目录脚本虽然解耦，但 stdout 解析脆弱、异常传递困难。改为 `sys.path.insert(0, SCRIPT_DIR) + import module` 后直接调用函数，错误栈清晰且可测试 [来源:qianniu_business_analytics @2026-05-29] | jycm_auto_report.py |
| | **多店 DataFrame 合并模式**：为每个店铺 DataFrame 添加内部标识列（如 `_shop_name`），再用 `pd.concat` 合并，可使单店/多店共用同一套分析函数，只需在报告生成层判断 `is_multi_shop` 切换展示逻辑 [来源:qianniu_business_analytics @2026-05-29] | analyze_excel_report.py |
| | **pytest stdin 捕获陷阱**：pytest 默认捕获 stdout/stderr，也会替换 `sys.stdin` 为 `DontReadFromInput`。测试 CLI 脚本中从 stdin 读取的逻辑时，必须用 `-f` 参数或 mock `sys.stdin.read` [来源:qianniu_business_analytics @2026-05-29] | test_dingtalk_send_markdown.py |
| | **归档而非删除空壳代码**：对于含大量 TODO 和模拟数据的脚本，直接删除会丢失已有接口设计；改为文件头标记「已归档」+ `main()` 抛 `NotImplementedError`，既防止误用又保留未来重建的参考 [来源:qianniu_business_analytics @2026-05-29] | qianniu_analytics_orchestrator.py / jycm_fetch_sycm_shop.py |
| | 日期区间多一天的问题是通过**后端实测**发现的（请求 4/20-4/26 返回了 4/27），而非文档说明。API 对接时文档与实际行为可能有偏差，应以实测为准。 [来源:qianniu_business_analytics @2026-05-29] |  |
| | Cookie 活性检测与订购检查**复用同一接口**（`product.json`），避免冗余调用。 [来源:qianniu_business_analytics @2026-05-29] |  |
| | 技能包的核心约束（如时区规则、渠道范围）应在 `SKILL.md` 和 `AGENTS.md` 中**双重声明**：`SKILL.md` 面向功能执行，`AGENTS.md` 面向开发维护，确保不同场景下都不会遗忘。 [来源:qianniu_business_analytics @2026-05-29] |  |
| | `auth.md` 中必须包含完整的 curl 参考示例，方便 Agent 直接复制执行。 [来源:qianniu_business_analytics @2026-05-29] |  |
| | Windows Git Bash 下执行 `git init` 时，所有文本文件会触发 `LF will be replaced by CRLF` 警告。这不会阻断提交，但跨平台协作前建议统一配置 `core.autocrlf`（如 `git config --global core.autocrlf true`），否则 Linux/Mac 协作者可能遇到行尾符混乱。 [来源:qianniu_business_analytics @2026-05-29] |  |
| | Python 脚本中通过 `sys.path.insert` 引用外部技能包路径（如 `jycm-fetch-report-nl`），在独立工作目录或环境变化后会失效。技能包应追求**自包含**（self-contained），避免跨包硬编码路径。 [来源:qianniu_business_analytics @2026-05-29] |  |
| | [ ] `jycm_auto_report.py` 中 `sys.path.insert` 引用的外部路径 `jycm-fetch-report-nl` 在当前独立工作目录下是否已失效 — 计划通过实际运行验证 [来源:qianniu_business_analytics @2026-05-29] |  |
| | [ ] `openpyxl` 是否已在所有运行环境中安装 — 计划补充 `requirements.txt` 并验证 [来源:qianniu_business_analytics @2026-05-29] |  |
| | [ ] 多店合并取数时（`shopIds` 含多个 id），`createAndDownload` 返回的 Excel 列结构是否与单店一致 — 计划通过实测验证 [来源:qianniu_business_analytics @2026-05-29] |  |
| 107 | TAG:api-design | WARNING | `gh api --paginate --slurp` 返回嵌套数组 `[page1, page2, ...]`（每页一个子数组），而非展平的单层数组。调用方需手动展平，否则 `repos[0]` 取到的是第一页列表而非第一个仓库 [母库 @2026-05-29] | sync-knowledge.py |
