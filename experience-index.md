# 经验索引

> 本文件由 `scripts/build-experience-index.py` 自动生成。
> 覆盖 troubleshooting / lessons-learned / decisions，统一搜索入口。

> 当前收录 **242** 条记录（问题 106 + 经验 98 + 决策 38）。

---

## 快速搜索表

| 关键词 | 类型 | 分类 | 来源 | 状态 | 定位 |
|--------|------|------|------|------|------|
| AI 重复实现已有组件（棋盘/网格类 UI） | 问题 | 未分类 | blindfold-chess | 待修复 | troubleshooting.md#L8 |
| Stockfish 加载超时 / 引擎不启动 | 问题 | 未分类 | blindfold-chess | 已知限制 | troubleshooting.md#L19 |
| GitHub Pages 国内打不开 | 问题 | 未分类 | blindfold-chess | 已知限制 | troubleshooting.md#L28 |
| Node.js 测试运行时 chess.js 未定义 | 问题 | 开发/测试 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L41 |
| CLI subAgent 并行超时 | 问题 | 开发/测试 | blindfold-chess | 已修复（改为 IDE 串行） | troubleshooting.md#L50 |
| 设置面板一闪而过 | 问题 | 开发/测试 | blindfold-chess | 已修复 | troubleshooting.md#L59 |
| 旧代码与新模块冲突 | 问题 | 运行时 | blindfold-chess | 已知未修复（逐步迁移中） | troubleshooting.md#L72 |
| 引擎候选走法未集成 | 问题 | 运行时 | blindfold-chess | 已知未修复 | troubleshooting.md#L81 |
| JS 数据文件嵌套单引号导致 `Unexpected identifier` | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L92 |
| 设置面板点击无反应（panel toggle 测试失败） | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L102 |
| mock DOM 中 `querySelector` / `querySelectorAll` 缺失 | 问题 | 运行时 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L112 |
| `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 | 问题 | 运行时 | french-exit | — | troubleshooting.md#L124 |
| `cargo check --lib` 报错：`FILETIME` 未定义 | 问题 | 运行时 | french-exit | — | troubleshooting.md#L132 |
| `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） | 问题 | 运行时错误 | french-exit | ✅ 已修复 | troubleshooting.md#L144 |
| 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime... | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L156 |
| 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L164 |
| `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L172 |
| vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"` | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L182 |
| vitest 报错：`act is not a function` | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L191 |
| vitest 报错：React 警告 `Cannot update a component while renderin... | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L199 |
| checkbox 点击后状态不变化 | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L208 |
| 中文路径下编译失败 | 问题 | 环境问题 | french-exit | — | troubleshooting.md#L221 |
| cargo tauri dev 在后台任务中崩溃 | 问题 | 环境问题 | french-exit | — | troubleshooting.md#L232 |
| PowerShell 执行中文脚本报 "UnexpectedToken" | 问题 | 存档提示 | vibe-coding-project-sop | — | troubleshooting.md#L253 |
| GitHub push 报错 `Permission denied (publickey)` | 问题 | 存档提示 | vibe-coding-project-sop | 已修复 | troubleshooting.md#L265 |
| `gh auth login` 超时：`read tcp ... operation timed out` | 问题 | 存档提示 | vibe-coding-project-sop | 已修复 | troubleshooting.md#L274 |
| HuggingFace 模型下载连接超时 `curl: (28) Could not connect to server... | 问题 | 存档提示 | vibe-coding-project-sop | 已解决 | troubleshooting.md#L283 |
| PowerShell 添加防火墙规则权限不足 `Access is denied` | 问题 | 存档提示 | vibe-coding-project-sop | 已知限制 | troubleshooting.md#L292 |
| Stockfish 加载超时 / 引擎不启动 | 问题 | 存档提示 | blindfold-chess | 已知限制 | troubleshooting.md#L302 |
| GitHub Pages 国内打不开 | 问题 | 存档提示 | blindfold-chess | 已知限制 | troubleshooting.md#L311 |
| Node.js 测试运行时 chess.js 未定义 | 问题 | 开发/测试 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L324 |
| CLI subAgent 并行超时 | 问题 | 开发/测试 | blindfold-chess | 已修复（改为 IDE 串行） | troubleshooting.md#L333 |
| 设置面板一闪而过 | 问题 | 开发/测试 | blindfold-chess | 已修复 | troubleshooting.md#L342 |
| 旧代码与新模块冲突 | 问题 | 运行时 | blindfold-chess | 已知未修复（逐步迁移中） | troubleshooting.md#L355 |
| 引擎候选走法未集成 | 问题 | 运行时 | blindfold-chess | 已知未修复 | troubleshooting.md#L364 |
| JS 数据文件嵌套单引号导致 `Unexpected identifier` | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L375 |
| 设置面板点击无反应（panel toggle 测试失败） | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L385 |
| Stockfish 加载超时 / 引擎不启动 | 问题 | 运行时 | blindfold-chess | 已知限制 | troubleshooting.md#L395 |
| GitHub Pages 国内打不开 | 问题 | 运行时 | blindfold-chess | 已知限制 | troubleshooting.md#L404 |
| Node.js 测试运行时 chess.js 未定义 | 问题 | 开发/测试 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L417 |
| CLI subAgent 并行超时 | 问题 | 开发/测试 | blindfold-chess | 已修复（改为 IDE 串行） | troubleshooting.md#L426 |
| 设置面板一闪而过 | 问题 | 开发/测试 | blindfold-chess | 已修复 | troubleshooting.md#L435 |
| 旧代码与新模块冲突 | 问题 | 运行时 | blindfold-chess | 已知未修复（逐步迁移中） | troubleshooting.md#L448 |
| 引擎候选走法未集成 | 问题 | 运行时 | blindfold-chess | 已知未修复 | troubleshooting.md#L457 |
| JS 数据文件嵌套单引号导致 `Unexpected identifier` | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L468 |
| 设置面板点击无反应（panel toggle 测试失败） | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L478 |
| `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 | 问题 | 运行时 | french-exit | — | troubleshooting.md#L488 |
| `cargo check --lib` 报错：`FILETIME` 未定义 | 问题 | 运行时 | french-exit | — | troubleshooting.md#L496 |
| `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） | 问题 | 运行时错误 | french-exit | ✅ 已修复 | troubleshooting.md#L508 |
| 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime... | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L520 |
| 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L528 |
| `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L536 |
| vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"` | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L546 |
| vitest 报错：`act is not a function` | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L555 |
| vitest 报错：React 警告 `Cannot update a component while renderin... | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L563 |
| checkbox 点击后状态不变化 | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L572 |
| 中文路径下编译失败 | 问题 | 环境问题 | french-exit | — | troubleshooting.md#L585 |
| cargo tauri dev 在后台任务中崩溃 | 问题 | 环境问题 | french-exit | — | troubleshooting.md#L596 |
| PowerShell 执行中文脚本报 "UnexpectedToken" | 问题 | 存档提示 | vibe-coding-project-sop | — | troubleshooting.md#L617 |
| GitHub push 报错 `Permission denied (publickey)` | 问题 | 存档提示 | vibe-coding-project-sop | 已修复 | troubleshooting.md#L629 |
| `gh auth login` 超时：`read tcp ... operation timed out` | 问题 | 存档提示 | vibe-coding-project-sop | 已修复 | troubleshooting.md#L638 |
| HuggingFace 模型下载连接超时 `curl: (28) Could not connect to server... | 问题 | 存档提示 | vibe-coding-project-sop | 已解决 | troubleshooting.md#L647 |
| PowerShell 添加防火墙规则权限不足 `Access is denied` | 问题 | 存档提示 | vibe-coding-project-sop | 已知限制 | troubleshooting.md#L656 |
| Node.js 报 SyntaxError: Unexpected identifier（i18n 中文字符串） | 问题 | 存档提示 | blindfold-chess | 已解决 | troubleshooting.md#L665 |
| sed 批量修改误改结构体定义 | 问题 | 存档提示 | french-exit | — | troubleshooting.md#L675 |
| French Exit 进程锁定 exe 导致复制失败 | 问题 | 存档提示 | french-exit | — | troubleshooting.md#L684 |
| Stockfish 加载超时 / 引擎不启动 | 问题 | 存档提示 | blindfold-chess | 已知限制 | troubleshooting.md#L702 |
| GitHub Pages 国内打不开 | 问题 | 存档提示 | blindfold-chess | 已知限制 | troubleshooting.md#L711 |
| Node.js 测试运行时 chess.js 未定义 | 问题 | 开发/测试 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L724 |
| CLI subAgent 并行超时 | 问题 | 开发/测试 | blindfold-chess | 已修复（改为 IDE 串行） | troubleshooting.md#L733 |
| 设置面板一闪而过 | 问题 | 开发/测试 | blindfold-chess | 已修复 | troubleshooting.md#L742 |
| 旧代码与新模块冲突 | 问题 | 运行时 | blindfold-chess | 已知未修复（逐步迁移中） | troubleshooting.md#L755 |
| 引擎候选走法未集成 | 问题 | 运行时 | blindfold-chess | 已知未修复 | troubleshooting.md#L764 |
| JS 数据文件嵌套单引号导致 `Unexpected identifier` | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L775 |
| 设置面板点击无反应（panel toggle 测试失败） | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L785 |
| Stockfish 加载超时 / 引擎不启动 | 问题 | 运行时 | blindfold-chess | 已知限制 | troubleshooting.md#L795 |
| GitHub Pages 国内打不开 | 问题 | 运行时 | blindfold-chess | 已知限制 | troubleshooting.md#L804 |
| Node.js 测试运行时 chess.js 未定义 | 问题 | 开发/测试 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L817 |
| CLI subAgent 并行超时 | 问题 | 开发/测试 | blindfold-chess | 已修复（改为 IDE 串行） | troubleshooting.md#L826 |
| 设置面板一闪而过 | 问题 | 开发/测试 | blindfold-chess | 已修复 | troubleshooting.md#L835 |
| 旧代码与新模块冲突 | 问题 | 运行时 | blindfold-chess | 已知未修复（逐步迁移中） | troubleshooting.md#L848 |
| 引擎候选走法未集成 | 问题 | 运行时 | blindfold-chess | 已知未修复 | troubleshooting.md#L857 |
| JS 数据文件嵌套单引号导致 `Unexpected identifier` | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L868 |
| 设置面板点击无反应（panel toggle 测试失败） | 问题 | 运行时 | blindfold-chess | 已修复 | troubleshooting.md#L878 |
| mock DOM 中 `querySelector` / `querySelectorAll` 缺失 | 问题 | 运行时 | blindfold-chess | 已修复（测试已适配） | troubleshooting.md#L888 |
| `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 | 问题 | 运行时 | french-exit | — | troubleshooting.md#L900 |
| `cargo check --lib` 报错：`FILETIME` 未定义 | 问题 | 运行时 | french-exit | — | troubleshooting.md#L908 |
| `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） | 问题 | 运行时错误 | french-exit | ✅ 已修复 | troubleshooting.md#L920 |
| 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime... | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L932 |
| 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L940 |
| `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) | 问题 | 运行时错误 | french-exit | — | troubleshooting.md#L948 |
| vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"` | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L958 |
| vitest 报错：`act is not a function` | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L967 |
| vitest 报错：React 警告 `Cannot update a component while renderin... | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L975 |
| checkbox 点击后状态不变化 | 问题 | 测试错误 | french-exit | — | troubleshooting.md#L984 |
| 中文路径下编译失败 | 问题 | 环境问题 | french-exit | — | troubleshooting.md#L997 |
| cargo tauri dev 在后台任务中崩溃 | 问题 | 环境问题 | french-exit | — | troubleshooting.md#L1008 |
| Cookie 过期 / 401 认证失败 | 问题 | 存档提示 | qianniu_business_analytics | 已修复（有自动刷新机制） | troubleshooting.md#L1029 |
| Digest 刷新失败 | 问题 | 存档提示 | qianniu_business_analytics | 已知处理方案 | troubleshooting.md#L1038 |
| auth/jycm.json 缺失字段 | 问题 | 存档提示 | qianniu_business_analytics | 已修复（有写入验证） | troubleshooting.md#L1047 |
| 日期区间多返回一天 | 问题 | 取数相关 | qianniu_business_analytics | 已修复（有强制约束） | troubleshooting.md#L1060 |
| getAllShopList 返回空数组 | 问题 | 取数相关 | qianniu_business_analytics | 已知未修复（业务侧问题） | troubleshooting.md#L1069 |
| createAndDownload 返回失败 | 问题 | 取数相关 | qianniu_business_analytics | 已知未修复（需具体 case 分析） | troubleshooting.md#L1078 |
| openpyxl 未安装 | 问题 | 报告相关 | qianniu_business_analytics | 临时绕过 | troubleshooting.md#L1091 |
| 钉钉推送失败 | 问题 | 报告相关 | qianniu_business_analytics | 已知未修复（需用户侧配置） | troubleshooting.md#L1100 |
| Windows Git Bash LF/CRLF 警告 | 问题 | 环境相关 | qianniu_business_analytics | 已知未修复（不影响功能） | troubleshooting.md#L1113 |
| 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 Web Worker（如... | 经验 | build-env / testing | blindfold-chess | INFO | lessons-learned.md#L14 |
| 手写 IIFE 模块时，用 `window.ModuleName = Module` 暴露 API，内部私有变量用下划线... | 经验 | dom / api-design | blindfold-chess | WARNING | lessons-learned.md#L15 |
| 浏览器集成测试用 TestRunner（自定义极简框架），保持与 Node 测试同一套断言 API，降低切换成本 | 经验 | testing | blindfold-chess | INFO | lessons-learned.md#L16 |
| Canvas 图表渲染在浏览器中测试，Node 环境用 Mock 2D context 跳过绘制验证，各测其责 | 经验 | testing | blindfold-chess | INFO | lessons-learned.md#L17 |
| PGN 解析器对空/无效输入返回 `[]`（空数组）而非 `null`，调用方需区分"无走法"和"解析失败" | 经验 | data / api-design | blindfold-chess | INFO | lessons-learned.md#L18 |
| `cloneNode(true)` 替换含 SVG 的按钮会导致 SVG 渲染异常（显示不完整）；移除事件监听器应优先用... | 经验 | dom | blindfold-chess | WARNING | lessons-learned.md#L19 |
| 匿名事件监听器无法被后续代码移除；需要动态解除绑定的监听器必须用命名函数（暴露到 `window` 或模块内部变量） | 经验 | dom | blindfold-chess | WARNING | lessons-learned.md#L20 |
| 屏幕切换导航不能只隐藏上一个屏幕，必须遍历 `.screen` 全部隐藏后再显示目标，否则多层屏幕重叠 | 经验 | dom / ux | blindfold-chess | WARNING | lessons-learned.md#L21 |
| SVG path 中密集参数（如 `a2 2 0 0 1-2.83 0`）在某些浏览器中可能解析异常，命令与参数间保留空... | 经验 | dom | blindfold-chess | INFO | lessons-learned.md#L22 |
| `document` 级事件监听器若引用了某个 DOM 元素，该元素被替换后监听器仍会按旧引用判断，导致逻辑错误（如点击... | 经验 | dom | blindfold-chess | WARNING | lessons-learned.md#L23 |
| 项目文档结构会随时间进化，"存档"或"恢复"操作前应先 `ls`/`glob` 确认当前文件系统现状，避免按历史路径写入... | 经验 | ai-workflow | blindfold-chess | INFO | lessons-learned.md#L24 |
| **UI 布局/样式不要猜测用户意图**：候选走法开关经历了 5 次位置/样式反复，每次修改后用户都不满意；应在设计阶段... | 经验 | ux | blindfold-chess | CRITICAL | lessons-learned.md#L25 |
| **引擎候选走法的调用时机决定产品逻辑正确性**：用户走完后立即 `goMultiPv` 分析的是对手局面；若要提示用户... | 经验 | api-design | blindfold-chess | CRITICAL | lessons-learned.md#L26 |
| **引擎返回 UCI（e2e4），用户界面必须用 SAN（e4）**：`goMultiPv` 回调中的 `move` 是... | 经验 | data / api-design | blindfold-chess | WARNING | lessons-learned.md#L27 |
| **静态 HTML 结构与动态渲染模块的 DOM 冲突**：`index.html` 中预置了完整棋盘结构，而 `Boa... | 经验 | dom | blindfold-chess | WARNING | lessons-learned.md#L28 |
| **删除功能必须同步删除对应测试**：移除 `showHints` / `multiPvSetting` 后，相关测试会... | 经验 | testing | blindfold-chess | WARNING | lessons-learned.md#L29 |
| **焦点管理是盲棋产品的核心体验**：进入对局自动 `input.focus()`、引擎走完后恢复焦点、全局 Enter... | 经验 | ux | blindfold-chess | WARNING | lessons-learned.md#L30 |
| **i18n 分散架构必然导致翻译遗漏**：当项目同时存在"全局字典 + 模块私有字典 + 硬编码"三种翻译方式时，新增... | 经验 | i18n | blindfold-chess | CRITICAL | lessons-learned.md#L31 |
| **JS 中的硬编码人类可读字符串是翻译遗漏的重灾区**：HTML 中的 `data-i18n` 至少能被肉眼扫描到，但... | 经验 | i18n | blindfold-chess | WARNING | lessons-learned.md#L32 |
| **复制粘贴是 i18n 错误的常见来源**：将中文值直接粘贴进英文字典，或反之，属于低级但高频的疏忽 | 经验 | i18n | blindfold-chess | WARNING | lessons-learned.md#L33 |
| **模块内部字典若从不主动更新 DOM，则纯属冗余**：welcome.js 有 `_i18n` 和 `_t()`，但从... | 经验 | i18n / architecture | blindfold-chess | WARNING | lessons-learned.md#L34 |
| **settings.js 的独立字典与 common.js 的全局扫描存在竞争**：settings panel 的元... | 经验 | i18n / dom | blindfold-chess | WARNING | lessons-learned.md#L35 |
| **已删除的 JS 文件若不从 index.html 移除引用，会导致 404**：功能清理和引用清理必须是同一任务 | 经验 | build-env | blindfold-chess | WARNING | lessons-learned.md#L36 |
| **Node 测试不对 UI 文本做断言，无法捕获翻译错误**：只测 API 形状和数值，不检查按钮文字、提示语等人类可... | 经验 | testing | blindfold-chess | WARNING | lessons-learned.md#L37 |
| **删除生产代码的 fallback 函数前，必须先评估测试环境是否提供了该依赖**：架构统一重构必须同时改代码+测试，... | 经验 | testing / architecture | blindfold-chess | CRITICAL | lessons-learned.md#L38 |
| **`localStorage` mock 必须支持 `setItem` 持久化**：测试中 `global.local... | 经验 | testing | blindfold-chess | WARNING | lessons-learned.md#L39 |
| **全局 `updateTexts()` 与模块私有 `_updateXxx()` 可能存在 DOM 竞争**：两者操作... | 经验 | dom / i18n | blindfold-chess | WARNING | lessons-learned.md#L40 |
| **配置类设置项用「弹窗选择」优于「循环切换」**：循环切换隐藏了全部选项，用户不知道有哪些风格；弹窗一次展示所有选项+... | 经验 | ux | blindfold-chess | INFO | lessons-learned.md#L41 |
| **`cloneNode(true)` 无法移除旧事件监听器，它只是复制了 DOM 结构**：真正安全的解绑是 `rem... | 经验 | dom | blindfold-chess | WARNING | lessons-learned.md#L42 |
| **UI 风格不一致的根因通常是「硬编码颜色」**：引入统一的「棋盘风格配置源」后，所有棋盘自动同步，消除不一致的根因。... | 经验 | ux / architecture | blindfold-chess | WARNING | lessons-learned.md#L43 |
| **功能入口迁移需要同步更新「正向路径」和「反向路径」**：不仅要添加新入口，还要移除旧入口，否则用户会在两个地方看到同... | 经验 | ux / architecture | blindfold-chess | WARNING | lessons-learned.md#L44 |
| **数据层的双语字段与代码层的硬编码分支是两个问题**：区分"数据双语"和"代码分支"可避免过度重构 | 经验 | data / architecture | blindfold-chess | INFO | lessons-learned.md#L45 |
| **测试中断言的具体文本值是重构的敏感点**：重构前应先审计测试中的文本断言，预估需要调整的范围 | 经验 | testing | blindfold-chess | WARNING | lessons-learned.md#L46 |
| **数据文件中的引号嵌套是极易被忽视的语法陷阱**：在真实浏览器中会抛出 `SyntaxError` 并阻断后续脚本执行 | 经验 | data / build-env | blindfold-chess | WARNING | lessons-learned.md#L47 |
| **Node 测试全过 ≠ 浏览器表现正常**：必须用 headless 浏览器（playwright）才能捕获浏览器特... | 经验 | testing | blindfold-chess | WARNING | lessons-learned.md#L48 |
| **playwright 是定位浏览器特有 bug 的有效手段**：通过 `page.add_init_script` ... | 经验 | testing / debugging | blindfold-chess | INFO | lessons-learned.md#L49 |
| **通用配置层设计能降低新增模式的边际成本**：新增对局模式时只需加一行 `else if` 分发逻辑，无需重复造 DO... | 经验 | architecture | blindfold-chess | INFO | lessons-learned.md#L50 |
| **向后兼容接口设计能减少重构的连锁反应**：旧接口继续工作，内部映射新参数，所有旧测试和外部调用点无需改动 | 经验 | api-design | blindfold-chess | INFO | lessons-learned.md#L51 |
| 浏览器集成测试阶段发现 welcome.js / replay.js / stats.js 的 DOM 事件绑定遗漏 | 经验 | testing / dom | blindfold-chess | INFO | lessons-learned.md#L52 |
| Node 测试覆盖逻辑，浏览器测试覆盖 DOM 集成，两者互补 | 经验 | testing | blindfold-chess | INFO | lessons-learned.md#L53 |
| `AGENTS.md` 定义触发词和行为约束，`status.md` 记录动态进度，分工明确，新会话读 2 份文件即可开... | 经验 | ai-workflow | french-exit | INFO | lessons-learned.md#L54 |
| 每批次开发完成后同步更新进度文档，避免新会话迷路 | 经验 | ai-workflow | blindfold-chess | INFO | lessons-learned.md#L55 |
| **手工构建100条结构化数据不现实**：经典棋局的 PGN 分散在各网站，无统一免费 API；手动录入工作量巨大且易出... | 经验 | data | blindfold-chess | INFO | lessons-learned.md#L56 |
| **WriteFile 不适合超大特殊字符内容**：含大量引号/换行的长文本会因 JSON 转义失败；应改用本地 Pyt... | 经验 | ai-workflow | blindfold-chess | WARNING | lessons-learned.md#L57 |
| **Shell here-document 在 Windows git bash 中不可靠**：含引号的多行复杂脚本会被... | 经验 | cross-platform / ai-workflow | blindfold-chess | WARNING | lessons-learned.md#L58 |
| **翻译检查必须是独立任务，不能依赖"开发时顺手做"**：本次检查发现 25+ 处遗漏，分布在 HTML、JS 字典、硬... | 经验 | i18n / ai-workflow | blindfold-chess | WARNING | lessons-learned.md#L59 |
| **涉及 7+ 文件读改测的架构重构，应新开会话执行**：继续塞进系统性重构容易触发窗口压缩，导致信息丢失 | 经验 | ai-workflow | blindfold-chess | INFO | lessons-learned.md#L60 |
| GitHub Pages 国内访问需代理；unpkg CDN 加载 Stockfish 可能超时，需考虑离线备选方案 | 经验 | build-env | blindfold-chess | INFO | lessons-learned.md#L61 |
| Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚本时优先用正斜杠或 ... | 经验 | cross-platform | blindfold-chess | WARNING | lessons-learned.md#L62 |
| `windows-rs` 0.61 的错误处理统一用 `.map_err( | 经验 | api-design |  | INFO | lessons-learned.md#L63 |
| `GetDiskFreeSpaceExW` 传 `&HSTRING` 作为路径参数，`Option<&mut u64>`... | 经验 | api-design | french-exit | INFO | lessons-learned.md#L64 |
| CPU% 精确计算只需 `GetProcessTimes` + wall clock elapsed，不需要 `GetS... | 经验 | api-design | french-exit | INFO | lessons-learned.md#L65 |
| `FILETIME` 转 u64：`((high as u64) << 32) | 经验 | api-design |  | INFO | lessons-learned.md#L66 |
| `Arc<dyn Fn(...) + Send + Sync>` 是 Rust 中给同步结构体注入回调的标准方式 | 经验 | api-design | french-exit | INFO | lessons-learned.md#L67 |
| Tauri 前端用 vitest + jsdom 测试时，必须在 `setup.ts` 中 `vi.mock()` 所有... | 经验 | testing | french-exit | INFO | lessons-learned.md#L68 |
| 若 `@tauri-apps/api/xxx` 模块不存在（如 v2 移除了 `fs`），用 **vite alias*... | 经验 | build-env | french-exit | INFO | lessons-learned.md#L69 |
| Controlled checkbox 的测试用 `@testing-library/user-event` 的 `us... | 经验 | testing | french-exit | INFO | lessons-learned.md#L70 |
| `tokio::sync::mpsc::Sender::try_send()` 适合非阻塞的进度回调，避免 Scanne... | 经验 | api-design | french-exit | INFO | lessons-learned.md#L71 |
| **绝对不要**在 `setState` 的 updater 函数内部调用 `dispatch()` 或其他 setSt... | 经验 | state-management | french-exit | CRITICAL | lessons-learned.md#L72 |
| `useEffect` 依赖 `state.xxx.size === 0` 作为触发条件时，容易形成死循环（用户操作 →... | 经验 | state-management | french-exit | CRITICAL | lessons-learned.md#L73 |
| `useRef` 作为"只执行一次"的标志，比依赖数组更可靠，尤其涉及批量初始化逻辑时 | 经验 | state-management | french-exit | WARNING | lessons-learned.md#L74 |
| **测试驱动暴露 Bug**：ResultsPage 的默认勾选死循环是在写单元测试时发现的，手工测试几乎不可能复现（需... | 经验 | testing | french-exit | INFO | lessons-learned.md#L75 |
| **结论**：前端状态管理类的 bug，单元测试是最有效的发现手段，远超手工测试 | 经验 | testing | french-exit | INFO | lessons-learned.md#L76 |
| `prompt-next-session.md` 的问题：每次都要重写环境初始化、模块速查表等**不变内容** | 经验 | ai-workflow | french-exit | INFO | lessons-learned.md#L77 |
| **改进**：`status.md`（活文档，只记录变化）+ `AGENTS.md`（固定规则） | 经验 | ai-workflow | french-exit | INFO | lessons-learned.md#L78 |
| **收益**：新会话读 2 份文件即可开工，维护成本降低 80% | 经验 | ai-workflow | french-exit | INFO | lessons-learned.md#L79 |
| **横跨工具层和应用层的词汇必须确认语境**。用户问"一个项目多个终端能否实现同步处理进度"——"终端"可以指并行 ex... | 经验 | ai-workflow | french-exit | WARNING | lessons-learned.md#L80 |
| **工具硬性限制不要绕圈分析可行性**。Kimi CLI 多窗口无 IPC、无共享内存、无实时同步——回答应直接给结论 ... | 经验 | ai-workflow | french-exit | WARNING | lessons-learned.md#L81 |
| **从 SOP 模板采纳更新时，必须逐字核对关键字段，不要凭记忆改写**。触发词「存档」错误抄写为「存储」，原因是未逐字... | 经验 | ai-workflow | french-exit | WARNING | lessons-learned.md#L82 |
| 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/french-exit`）后... | 经验 | cross-platform / build-env | french-exit | CRITICAL | lessons-learned.md#L83 |
| `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test --no-run` 同... | 经验 | cross-platform / build-env | french-exit | INFO | lessons-learned.md#L84 |
| **`0xc0000139` 不一定是 UCRT/MinGW 兼容性 issue**。先跑一个**最简单 lib 测试*... | 经验 | debugging | french-exit | INFO | lessons-learned.md#L85 |
| **`cargo test --bin` 能过、`cargo test --lib` 崩溃** → 问题出在**仅被 l... | 经验 | debugging | french-exit | WARNING | lessons-learned.md#L86 |
| **定位代码的最快方法**：清空 `lib.rs` 只保留一个空测试，逐步 `pub mod` 添加模块，直到崩溃复现。... | 经验 | debugging | french-exit | INFO | lessons-learned.md#L87 |
| **`tauri::AppHandle` 出现在 `async fn` 签名中 + MinGW = `STATUS_EN... | 经验 | cross-platform / build-env | french-exit | CRITICAL | lessons-learned.md#L88 |
| **`#[cfg(not(test))]` 隔离问题代码**是零副作用的修复手法：release 构建完全不受影响，测试... | 经验 | testing / cross-platform | french-exit | INFO | lessons-learned.md#L89 |
| `useRef` + `mousedown` 监听实现点击外部关闭 | 经验 | dom / ux | french-exit | INFO | lessons-learned.md#L90 |
| CSS `@keyframes dropdownIn` 实现淡入+位移动画 | 经验 | ux | french-exit | INFO | lessons-learned.md#L91 |
| 年月日联动限制（如今年只显示到当前月） | 经验 | ux | french-exit | INFO | lessons-learned.md#L92 |
| `cargo tauri dev` 必须在**交互式 Windows 桌面会话**中运行，无法通过远程/后台任务启动（W... | 经验 | build-env | french-exit | INFO | lessons-learned.md#L93 |
| **替代方案**：`npm run dev` 启动 Vite 服务器 → 浏览器访问 `http://localhost... | 经验 | build-env | french-exit | INFO | lessons-learned.md#L94 |
| **完整功能验证**：仍需本地运行 `cargo tauri dev` 或双击 release `.exe` | 经验 | build-env | french-exit | INFO | lessons-learned.md#L95 |
| 不要一次性加载所有完整 `TraceItem` 到前端（内存 + DOM 渲染压力大） | 经验 | data / performance | french-exit | WARNING | lessons-learned.md#L96 |
| 正确做法：后端提供**轻量摘要接口**（只返回 id + category + suggested_action），前端... | 经验 | architecture / data | french-exit | INFO | lessons-learned.md#L97 |
| 用户实际浏览仍按分页，但"全选全部"走轻量接口，两者解耦 | 经验 | pagination / architecture | french-exit | WARNING | lessons-learned.md#L98 |
| **事故经过**：ResultsPage 默认自动勾选所有扫描结果 → 用户点击"全选全部"（以为是全选当前页，实际是全... | 经验 | pagination / state-management / security | french-exit | CRITICAL | lessons-learned.md#L99 |
| **根因链**：默认勾选 × deselectAll 只清当前页 × ConfirmPage 遍历 scanResult... | 经验 | pagination / state-management | french-exit | CRITICAL | lessons-learned.md#L100 |
| **教训**：涉及删除的安全工具，**默认安全 > 默认便利**。所有选择必须用户显式操作，任何"帮你选好"的设计都需反... | 经验 | security / ux | french-exit | CRITICAL | lessons-learned.md#L101 |
| **原实现**：`deselectAll` 只遍历 `searchedItems`（当前页数据），从 `selected... | 经验 | pagination / state-management | french-exit | WARNING | lessons-learned.md#L102 |
| **修复**：`deselectAll` 清空 `selectedIds` 为 `new Set()`，同时 `disp... | 经验 | pagination / state-management | french-exit | INFO | lessons-learned.md#L103 |
| **教训**：跨分页操作时，"取消"必须与"全选"的对称——全选影响多大范围，取消就必须影响多大范围 | 经验 | pagination / state-management | french-exit | WARNING | lessons-learned.md#L104 |
| **原实现**：ConfirmPage 遍历 `state.scanResults`，过滤出选中的项 → 分页未加载的项... | 经验 | pagination / state-management | french-exit | WARNING | lessons-learned.md#L105 |
| **修复**：遍历 `state.decisions`，每项在 `scanResults` 中查找详细信息，找不到时用 ... | 经验 | pagination / state-management | french-exit | INFO | lessons-learned.md#L106 |
| **教训**：在分页/懒加载架构中，**用户操作集合（decisions）是主数据源，展示数据（scanResults）... | 经验 | pagination / architecture | french-exit | WARNING | lessons-learned.md#L107 |
| Git for Windows 的 bash `/tmp` 与 PowerShell `$env:TEMP` 指向同一物... | 经验 | build-env |  | INFO | lessons-learned.md#L170 |
| 国内下载 HuggingFace 模型时，ModelScope 是比 hf-mirror 更可靠的 fallback（后... | 经验 | build-env |  | INFO | lessons-learned.md#L171 |
| Windows 非管理员运行 PowerShell 脚本时，`New-NetFirewallRule` 会失败，但 `l... | 经验 | build-env |  | WARNING | lessons-learned.md#L172 |
| `gh api --paginate --slurp` 返回嵌套数组 `[page1, page2, ...]`（每页一... | 经验 | api-design |  | WARNING | lessons-learned.md#L204 |
| ADR-001：前端技术栈选型 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L8 |
| ADR-002：测试框架选型 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L22 |
| ADR-003：AI 开发方式与批次划分 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L36 |
| ADR-004：棋盘渲染与棋子方案 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L50 |
| ADR-005：数据持久化方案 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L64 |
| ADR-006：通用对局配置层设计 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L78 |
| ADR-007：难度选择从离散按钮改为连续滑块 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L92 |
| ADR-008：棋盘风格设置交互方案 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L106 |
| ADR-009：盲棋复盘入口位置 | 决策 | 架构决策 | blindfold-chess | — | decisions.md#L120 |
| ADR-017：GitHub 认证从 SSH 切换到 GitHub CLI + HTTPS | 决策 | 架构决策 | vibe-coding-project-sop | — | decisions.md#L132 |
| ADR-001：为什么用 Tauri（Rust + WebView2）而非 Electron？ | 决策 | 架构决策 | french-exit | — | decisions.md#L147 |
| ADR-002：为什么前端用 React（而非 Vue/Svelte）？ | 决策 | 架构决策 | french-exit | — | decisions.md#L160 |
| ADR-003：为什么 CPU% 用 `GetProcessTimes` 而非 `sysinfo` crate？ | 决策 | 架构决策 | french-exit | — | decisions.md#L174 |
| ADR-004：为什么 Scanner 进度用 `mpsc::channel` 而非 `tokio::sync::wat... | 决策 | 架构决策 | french-exit | — | decisions.md#L188 |
| ADR-005：为什么加密文件回调用同步 `Fn` 而非 `async`？ | 决策 | 架构决策 | french-exit | — | decisions.md#L202 |
| ADR-006：为什么用 `status.md` + `session-log.md` 替代 `prompt-next-... | 决策 | 架构决策 | french-exit | — | decisions.md#L216 |
| ADR-007：WebView2 分发策略——放弃 NSIS bootstrapper，改用携带 DLL | 决策 | 架构决策 | french-exit | — | decisions.md#L228 |
| ADR-008：默认深色主题而非跟随系统 | 决策 | 架构决策 | french-exit | — | decisions.md#L240 |
| ADR-009：全选全部功能的技术方案 | 决策 | 架构决策 | french-exit | — | decisions.md#L252 |
| ADR-010：路径交互设计 — 文本可点击 vs 独立按钮 | 决策 | 架构决策 | french-exit | — | decisions.md#L264 |
| ADR-011：删除策略从 DoD 安全擦除改为普通删除 | 决策 | 架构决策 | french-exit | — | decisions.md#L276 |
| ADR-012：扫描范围从 Desktop/Downloads 扩展为全盘扫描 | 决策 | 架构决策 | french-exit | — | decisions.md#L288 |
| ADR-013：移除 ResultsPage 默认自动勾选 | 决策 | 架构决策 | french-exit | — | decisions.md#L300 |
| ADR-014：同步脚本优先使用仓库 default_branch | 决策 | 架构决策 | vibe-coding-project-sop | — | decisions.md#L314 |
| ADR-015：syncFrom 配置实现聚合/分发双模式 | 决策 | 架构决策 | vibe-coding-project-sop | — | decisions.md#L326 |
| ADR-016：母库 AGENTS 与其他项目 AGENTS 物理分离 | 决策 | 架构决策 | vibe-coding-project-sop | — | decisions.md#L338 |
| ADR-009：Troubleshooting 索引采用独立文件 + 行号链接 | 决策 | 架构决策 |  | — | decisions.md#L392 |
| ADR-017：init-skeleton.py 保持 Python 3.9 兼容 | 决策 | 架构决策 |  | — | decisions.md#L416 |
| ADR-017：假删除模式通过环境变量 `FRENCH_EXIT_DRY_RUN` 控制 [来源:french-exit... | 决策 | 架构决策 | french-exit | — | decisions.md#L442 |
| ADR-017：扫描进度条采用后端全局加权进度计算 | 决策 | 架构决策 | french-exit | — | decisions.md#L458 |
| ADR-018：个人目录全量扫描 + 文件类型分类 | 决策 | 架构决策 | french-exit | — | decisions.md#L473 |
| ADR-018：WebView2 安装模式选择 downloadBootstrapper | 决策 | 架构决策 | french-exit | — | decisions.md#L491 |
| ADR-001：技术栈选型（Python 3 + requests） | 决策 | 架构决策 | qianniu_business_analytics | — | decisions.md#L510 |
| ADR-002：渠道范围限定（仅淘系生意参谋） | 决策 | 架构决策 | qianniu_business_analytics | — | decisions.md#L524 |
| ADR-003：Cookie 刷新策略（Digest 自动刷新，不问用户） | 决策 | 架构决策 | qianniu_business_analytics | — | decisions.md#L538 |
| ADR-004：日期时间格式（T00:00:00+08:00，禁用 23:59:59） | 决策 | 架构决策 | qianniu_business_analytics | — | decisions.md#L552 |
| ADR-005：报告形态（Markdown 四段式，单店/多店统一） | 决策 | 架构决策 | qianniu_business_analytics | — | decisions.md#L566 |
| ADR-017：聚焦 Excel 驱动流，API 驱动流暂不投入 [来源:qianniu_business_analyt... | 决策 | 架构决策 | qianniu_business_analytics | — | decisions.md#L584 |

---

## 按技术栈分组

> 一个条目可能同时属于多个技术栈。

### Rust / Tauri

- [问题] `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 — `运行时` → troubleshooting.md#L124
- [问题] `cargo check --lib` 报错：`FILETIME` 未定义 — `运行时` → troubleshooting.md#L132
- [问题] `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） — `运行时错误` → troubleshooting.md#L144
- [问题] 运行 `french-exit.exe` 报错：`Could not find the WebVie — `运行时错误` → troubleshooting.md#L156
- [问题] 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` — `运行时错误` → troubleshooting.md#L164
- [问题] `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32 — `运行时错误` → troubleshooting.md#L172
- [问题] vitest 报错：`Failed to resolve import "@tauri-apps/a — `测试错误` → troubleshooting.md#L182
- [问题] cargo tauri dev 在后台任务中崩溃 — `环境问题` → troubleshooting.md#L232
- [问题] `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 — `运行时` → troubleshooting.md#L488
- [问题] `cargo check --lib` 报错：`FILETIME` 未定义 — `运行时` → troubleshooting.md#L496
- [问题] `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） — `运行时错误` → troubleshooting.md#L508
- [问题] 运行 `french-exit.exe` 报错：`Could not find the WebVie — `运行时错误` → troubleshooting.md#L520
- [问题] 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` — `运行时错误` → troubleshooting.md#L528
- [问题] `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32 — `运行时错误` → troubleshooting.md#L536
- [问题] vitest 报错：`Failed to resolve import "@tauri-apps/a — `测试错误` → troubleshooting.md#L546
- [问题] cargo tauri dev 在后台任务中崩溃 — `环境问题` → troubleshooting.md#L596
- [问题] `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 — `运行时` → troubleshooting.md#L900
- [问题] `cargo check --lib` 报错：`FILETIME` 未定义 — `运行时` → troubleshooting.md#L908
- [问题] `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） — `运行时错误` → troubleshooting.md#L920
- [问题] 运行 `french-exit.exe` 报错：`Could not find the WebVie — `运行时错误` → troubleshooting.md#L932
- [问题] 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` — `运行时错误` → troubleshooting.md#L940
- [问题] `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32 — `运行时错误` → troubleshooting.md#L948
- [问题] vitest 报错：`Failed to resolve import "@tauri-apps/a — `测试错误` → troubleshooting.md#L958
- [问题] cargo tauri dev 在后台任务中崩溃 — `环境问题` → troubleshooting.md#L1008
- [经验] `GetDiskFreeSpaceExW` 传 `&HSTRING` 作为路径参数，`Option< — `api-design` → lessons-learned.md#L64
- [经验] `FILETIME` 转 u64：`((high as u64) << 32) — `api-design` → lessons-learned.md#L66
- [经验] `Arc<dyn Fn(...) + Send + Sync>` 是 Rust 中给同步结构体注入回 — `api-design` → lessons-learned.md#L67
- [经验] Tauri 前端用 vitest + jsdom 测试时，必须在 `setup.ts` 中 `vi. — `testing` → lessons-learned.md#L68
- [经验] 若 `@tauri-apps/api/xxx` 模块不存在（如 v2 移除了 `fs`），用 **v — `build-env` → lessons-learned.md#L69
- [经验] 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/fren — `cross-platform / build-env` → lessons-learned.md#L83
- [经验] `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test - — `cross-platform / build-env` → lessons-learned.md#L84
- [经验] **`0xc0000139` 不一定是 UCRT/MinGW 兼容性 issue**。先跑一个**最 — `debugging` → lessons-learned.md#L85
- [经验] **`cargo test --bin` 能过、`cargo test --lib` 崩溃** →  — `debugging` → lessons-learned.md#L86
- [经验] **`tauri::AppHandle` 出现在 `async fn` 签名中 + MinGW =  — `cross-platform / build-env` → lessons-learned.md#L88
- [经验] `cargo tauri dev` 必须在**交互式 Windows 桌面会话**中运行，无法通过远 — `build-env` → lessons-learned.md#L93
- [经验] **完整功能验证**：仍需本地运行 `cargo tauri dev` 或双击 release `. — `build-env` → lessons-learned.md#L95
- [决策] ADR-001：为什么用 Tauri（Rust + WebView2）而非 Electron？ — `架构决策` → decisions.md#L147
- [决策] ADR-003：为什么 CPU% 用 `GetProcessTimes` 而非 `sysinfo`  — `架构决策` → decisions.md#L174
- [决策] ADR-007：WebView2 分发策略——放弃 NSIS bootstrapper，改用携带 D — `架构决策` → decisions.md#L228
- [决策] ADR-018：WebView2 安装模式选择 downloadBootstrapper — `架构决策` → decisions.md#L491

### JavaScript / React / Vitest

- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L41
- [问题] JS 数据文件嵌套单引号导致 `Unexpected identifier` — `运行时` → troubleshooting.md#L92
- [问题] mock DOM 中 `querySelector` / `querySelectorAll` 缺失 — `运行时` → troubleshooting.md#L112
- [问题] vitest 报错：`Failed to resolve import "@tauri-apps/a — `测试错误` → troubleshooting.md#L182
- [问题] vitest 报错：`act is not a function` — `测试错误` → troubleshooting.md#L191
- [问题] vitest 报错：React 警告 `Cannot update a component whil — `测试错误` → troubleshooting.md#L199
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L324
- [问题] JS 数据文件嵌套单引号导致 `Unexpected identifier` — `运行时` → troubleshooting.md#L375
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L417
- [问题] JS 数据文件嵌套单引号导致 `Unexpected identifier` — `运行时` → troubleshooting.md#L468
- [问题] vitest 报错：`Failed to resolve import "@tauri-apps/a — `测试错误` → troubleshooting.md#L546
- [问题] vitest 报错：`act is not a function` — `测试错误` → troubleshooting.md#L555
- [问题] vitest 报错：React 警告 `Cannot update a component whil — `测试错误` → troubleshooting.md#L563
- [问题] Node.js 报 SyntaxError: Unexpected identifier（i18n  — `存档提示` → troubleshooting.md#L665
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L724
- [问题] JS 数据文件嵌套单引号导致 `Unexpected identifier` — `运行时` → troubleshooting.md#L775
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L817
- [问题] JS 数据文件嵌套单引号导致 `Unexpected identifier` — `运行时` → troubleshooting.md#L868
- [问题] mock DOM 中 `querySelector` / `querySelectorAll` 缺失 — `运行时` → troubleshooting.md#L888
- [问题] vitest 报错：`Failed to resolve import "@tauri-apps/a — `测试错误` → troubleshooting.md#L958
- [问题] vitest 报错：`act is not a function` — `测试错误` → troubleshooting.md#L967
- [问题] vitest 报错：React 警告 `Cannot update a component whil — `测试错误` → troubleshooting.md#L975
- [经验] 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 We — `build-env / testing` → lessons-learned.md#L14
- [经验] 手写 IIFE 模块时，用 `window.ModuleName = Module` 暴露 API， — `dom / api-design` → lessons-learned.md#L15
- [经验] 浏览器集成测试用 TestRunner（自定义极简框架），保持与 Node 测试同一套断言 API， — `testing` → lessons-learned.md#L16
- [经验] Canvas 图表渲染在浏览器中测试，Node 环境用 Mock 2D context 跳过绘制验证 — `testing` → lessons-learned.md#L17
- [经验] `cloneNode(true)` 替换含 SVG 的按钮会导致 SVG 渲染异常（显示不完整）；移 — `dom` → lessons-learned.md#L19
- [经验] 匿名事件监听器无法被后续代码移除；需要动态解除绑定的监听器必须用命名函数（暴露到 `window`  — `dom` → lessons-learned.md#L20
- [经验] 屏幕切换导航不能只隐藏上一个屏幕，必须遍历 `.screen` 全部隐藏后再显示目标，否则多层屏幕重 — `dom / ux` → lessons-learned.md#L21
- [经验] SVG path 中密集参数（如 `a2 2 0 0 1-2.83 0`）在某些浏览器中可能解析异常 — `dom` → lessons-learned.md#L22
- [经验] `document` 级事件监听器若引用了某个 DOM 元素，该元素被替换后监听器仍会按旧引用判断， — `dom` → lessons-learned.md#L23
- [经验] **静态 HTML 结构与动态渲染模块的 DOM 冲突**：`index.html` 中预置了完整棋 — `dom` → lessons-learned.md#L28
- [经验] **模块内部字典若从不主动更新 DOM，则纯属冗余**：welcome.js 有 `_i18n` 和 — `i18n / architecture` → lessons-learned.md#L34
- [经验] **settings.js 的独立字典与 common.js 的全局扫描存在竞争**：setting — `i18n / dom` → lessons-learned.md#L35
- [经验] **Node 测试不对 UI 文本做断言，无法捕获翻译错误**：只测 API 形状和数值，不检查按钮 — `testing` → lessons-learned.md#L37
- [经验] **全局 `updateTexts()` 与模块私有 `_updateXxx()` 可能存在 DOM — `dom / i18n` → lessons-learned.md#L40
- [经验] **`cloneNode(true)` 无法移除旧事件监听器，它只是复制了 DOM 结构**：真正安 — `dom` → lessons-learned.md#L42
- [经验] **Node 测试全过 ≠ 浏览器表现正常**：必须用 headless 浏览器（playwrigh — `testing` → lessons-learned.md#L48
- [经验] **通用配置层设计能降低新增模式的边际成本**：新增对局模式时只需加一行 `else if` 分发逻 — `architecture` → lessons-learned.md#L50
- [经验] 浏览器集成测试阶段发现 welcome.js / replay.js / stats.js 的 DO — `testing / dom` → lessons-learned.md#L52
- [经验] Node 测试覆盖逻辑，浏览器测试覆盖 DOM 集成，两者互补 — `testing` → lessons-learned.md#L53
- [经验] **WriteFile 不适合超大特殊字符内容**：含大量引号/换行的长文本会因 JSON 转义失败 — `ai-workflow` → lessons-learned.md#L57
- [经验] Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚 — `cross-platform` → lessons-learned.md#L62
- [经验] Tauri 前端用 vitest + jsdom 测试时，必须在 `setup.ts` 中 `vi. — `testing` → lessons-learned.md#L68
- [经验] 若 `@tauri-apps/api/xxx` 模块不存在（如 v2 移除了 `fs`），用 **v — `build-env` → lessons-learned.md#L69
- [经验] Controlled checkbox 的测试用 `@testing-library/user-ev — `testing` → lessons-learned.md#L70
- [经验] **绝对不要**在 `setState` 的 updater 函数内部调用 `dispatch()` — `state-management` → lessons-learned.md#L72
- [经验] `useRef` + `mousedown` 监听实现点击外部关闭 — `dom / ux` → lessons-learned.md#L90
- [经验] **替代方案**：`npm run dev` 启动 Vite 服务器 → 浏览器访问 `http:/ — `build-env` → lessons-learned.md#L94
- [经验] 不要一次性加载所有完整 `TraceItem` 到前端（内存 + DOM 渲染压力大） — `data / performance` → lessons-learned.md#L96
- [决策] ADR-002：为什么前端用 React（而非 Vue/Svelte）？ — `架构决策` → decisions.md#L160

### Python

- [经验] **WriteFile 不适合超大特殊字符内容**：含大量引号/换行的长文本会因 JSON 转义失败 — `ai-workflow` → lessons-learned.md#L57
- [决策] ADR-017：init-skeleton.py 保持 Python 3.9 兼容 — `架构决策` → decisions.md#L416
- [决策] ADR-001：技术栈选型（Python 3 + requests） — `架构决策` → decisions.md#L510

### AI 工具链 / LLM

- [问题] HuggingFace 模型下载连接超时 `curl: (28) Could not connect — `存档提示` → troubleshooting.md#L283
- [问题] HuggingFace 模型下载连接超时 `curl: (28) Could not connect — `存档提示` → troubleshooting.md#L647
- [经验] 国内下载 HuggingFace 模型时，ModelScope 是比 hf-mirror 更可靠的  — `build-env` → lessons-learned.md#L171
- [经验] Windows 非管理员运行 PowerShell 脚本时，`New-NetFirewallRule — `build-env` → lessons-learned.md#L172

### Git / GitHub

- [问题] GitHub Pages 国内打不开 — `未分类` → troubleshooting.md#L28
- [问题] GitHub push 报错 `Permission denied (publickey)` — `存档提示` → troubleshooting.md#L265
- [问题] `gh auth login` 超时：`read tcp ... operation timed o — `存档提示` → troubleshooting.md#L274
- [问题] GitHub Pages 国内打不开 — `存档提示` → troubleshooting.md#L311
- [问题] GitHub Pages 国内打不开 — `运行时` → troubleshooting.md#L404
- [问题] GitHub push 报错 `Permission denied (publickey)` — `存档提示` → troubleshooting.md#L629
- [问题] `gh auth login` 超时：`read tcp ... operation timed o — `存档提示` → troubleshooting.md#L638
- [问题] GitHub Pages 国内打不开 — `存档提示` → troubleshooting.md#L711
- [问题] GitHub Pages 国内打不开 — `运行时` → troubleshooting.md#L804
- [问题] Windows Git Bash LF/CRLF 警告 — `环境相关` → troubleshooting.md#L1113
- [经验] **Shell here-document 在 Windows git bash 中不可靠**：含引 — `cross-platform / ai-workflow` → lessons-learned.md#L58
- [经验] GitHub Pages 国内访问需代理；unpkg CDN 加载 Stockfish 可能超时，需 — `build-env` → lessons-learned.md#L61
- [经验] Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚 — `cross-platform` → lessons-learned.md#L62
- [经验] Git for Windows 的 bash `/tmp` 与 PowerShell `$env:T — `build-env` → lessons-learned.md#L170
- [决策] ADR-017：GitHub 认证从 SSH 切换到 GitHub CLI + HTTPS — `架构决策` → decisions.md#L132

### 网络 / 环境 / 权限

- [问题] GitHub Pages 国内打不开 — `未分类` → troubleshooting.md#L28
- [问题] 中文路径下编译失败 — `环境问题` → troubleshooting.md#L221
- [问题] HuggingFace 模型下载连接超时 `curl: (28) Could not connect — `存档提示` → troubleshooting.md#L283
- [问题] PowerShell 添加防火墙规则权限不足 `Access is denied` — `存档提示` → troubleshooting.md#L292
- [问题] GitHub Pages 国内打不开 — `存档提示` → troubleshooting.md#L311
- [问题] GitHub Pages 国内打不开 — `运行时` → troubleshooting.md#L404
- [问题] 中文路径下编译失败 — `环境问题` → troubleshooting.md#L585
- [问题] HuggingFace 模型下载连接超时 `curl: (28) Could not connect — `存档提示` → troubleshooting.md#L647
- [问题] PowerShell 添加防火墙规则权限不足 `Access is denied` — `存档提示` → troubleshooting.md#L656
- [问题] GitHub Pages 国内打不开 — `存档提示` → troubleshooting.md#L711
- [问题] GitHub Pages 国内打不开 — `运行时` → troubleshooting.md#L804
- [问题] 中文路径下编译失败 — `环境问题` → troubleshooting.md#L997
- [经验] SVG path 中密集参数（如 `a2 2 0 0 1-2.83 0`）在某些浏览器中可能解析异常 — `dom` → lessons-learned.md#L22
- [经验] **UI 布局/样式不要猜测用户意图**：候选走法开关经历了 5 次位置/样式反复，每次修改后用户都 — `ux` → lessons-learned.md#L25
- [经验] **i18n 分散架构必然导致翻译遗漏**：当项目同时存在"全局字典 + 模块私有字典 + 硬编码" — `i18n` → lessons-learned.md#L31
- [经验] **JS 中的硬编码人类可读字符串是翻译遗漏的重灾区**：HTML 中的 `data-i18n` 至 — `i18n` → lessons-learned.md#L32
- [经验] **UI 风格不一致的根因通常是「硬编码颜色」**：引入统一的「棋盘风格配置源」后，所有棋盘自动同步 — `ux / architecture` → lessons-learned.md#L43
- [经验] **数据层的双语字段与代码层的硬编码分支是两个问题**：区分"数据双语"和"代码分支"可避免过度重构 — `data / architecture` → lessons-learned.md#L45
- [经验] **翻译检查必须是独立任务，不能依赖"开发时顺手做"**：本次检查发现 25+ 处遗漏，分布在 HT — `i18n / ai-workflow` → lessons-learned.md#L59
- [经验] GitHub Pages 国内访问需代理；unpkg CDN 加载 Stockfish 可能超时，需 — `build-env` → lessons-learned.md#L61
- [经验] Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚 — `cross-platform` → lessons-learned.md#L62
- [经验] 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/fren — `cross-platform / build-env` → lessons-learned.md#L83
- [经验] `cargo check --lib` 不需要链接，可以在中文路径直接跑；`cargo test - — `cross-platform / build-env` → lessons-learned.md#L84
- [经验] 国内下载 HuggingFace 模型时，ModelScope 是比 hf-mirror 更可靠的  — `build-env` → lessons-learned.md#L171
- [经验] Windows 非管理员运行 PowerShell 脚本时，`New-NetFirewallRule — `build-env` → lessons-learned.md#L172

### Windows / PowerShell

- [问题] `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） — `运行时错误` → troubleshooting.md#L144
- [问题] 运行 `french-exit.exe` 报错：`Could not find the WebVie — `运行时错误` → troubleshooting.md#L156
- [问题] 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` — `运行时错误` → troubleshooting.md#L164
- [问题] PowerShell 执行中文脚本报 "UnexpectedToken" — `存档提示` → troubleshooting.md#L253
- [问题] PowerShell 添加防火墙规则权限不足 `Access is denied` — `存档提示` → troubleshooting.md#L292
- [问题] `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） — `运行时错误` → troubleshooting.md#L508
- [问题] 运行 `french-exit.exe` 报错：`Could not find the WebVie — `运行时错误` → troubleshooting.md#L520
- [问题] 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` — `运行时错误` → troubleshooting.md#L528
- [问题] PowerShell 执行中文脚本报 "UnexpectedToken" — `存档提示` → troubleshooting.md#L617
- [问题] PowerShell 添加防火墙规则权限不足 `Access is denied` — `存档提示` → troubleshooting.md#L656
- [问题] French Exit 进程锁定 exe 导致复制失败 — `存档提示` → troubleshooting.md#L684
- [问题] `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） — `运行时错误` → troubleshooting.md#L920
- [问题] 运行 `french-exit.exe` 报错：`Could not find the WebVie — `运行时错误` → troubleshooting.md#L932
- [问题] 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` — `运行时错误` → troubleshooting.md#L940
- [问题] Windows Git Bash LF/CRLF 警告 — `环境相关` → troubleshooting.md#L1113
- [经验] **Shell here-document 在 Windows git bash 中不可靠**：含引 — `cross-platform / ai-workflow` → lessons-learned.md#L58
- [经验] Windows 路径在 git bash / Node.js / cmd 中转义规则不同，写跨平台脚 — `cross-platform` → lessons-learned.md#L62
- [经验] `windows-rs` 0.61 的错误处理统一用 `.map_err( — `api-design` → lessons-learned.md#L63
- [经验] **横跨工具层和应用层的词汇必须确认语境**。用户问"一个项目多个终端能否实现同步处理进度"——"终 — `ai-workflow` → lessons-learned.md#L80
- [经验] 中文路径 + MinGW = 链接器失败。解决方案：复制到纯 ASCII 路径（如 `/c/fren — `cross-platform / build-env` → lessons-learned.md#L83
- [经验] **`0xc0000139` 不一定是 UCRT/MinGW 兼容性 issue**。先跑一个**最 — `debugging` → lessons-learned.md#L85
- [经验] **`tauri::AppHandle` 出现在 `async fn` 签名中 + MinGW =  — `cross-platform / build-env` → lessons-learned.md#L88
- [经验] `cargo tauri dev` 必须在**交互式 Windows 桌面会话**中运行，无法通过远 — `build-env` → lessons-learned.md#L93
- [经验] **完整功能验证**：仍需本地运行 `cargo tauri dev` 或双击 release `. — `build-env` → lessons-learned.md#L95
- [经验] Git for Windows 的 bash `/tmp` 与 PowerShell `$env:T — `build-env` → lessons-learned.md#L170
- [经验] Windows 非管理员运行 PowerShell 脚本时，`New-NetFirewallRule — `build-env` → lessons-learned.md#L172
- [决策] ADR-007：WebView2 分发策略——放弃 NSIS bootstrapper，改用携带 D — `架构决策` → decisions.md#L228

### Chess / 引擎

- [问题] Stockfish 加载超时 / 引擎不启动 — `未分类` → troubleshooting.md#L19
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L41
- [问题] Stockfish 加载超时 / 引擎不启动 — `存档提示` → troubleshooting.md#L302
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L324
- [问题] Stockfish 加载超时 / 引擎不启动 — `运行时` → troubleshooting.md#L395
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L417
- [问题] Stockfish 加载超时 / 引擎不启动 — `存档提示` → troubleshooting.md#L702
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L724
- [问题] Stockfish 加载超时 / 引擎不启动 — `运行时` → troubleshooting.md#L795
- [问题] Node.js 测试运行时 chess.js 未定义 — `开发/测试` → troubleshooting.md#L817
- [经验] 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 We — `build-env / testing` → lessons-learned.md#L14
- [经验] PGN 解析器对空/无效输入返回 `[]`（空数组）而非 `null`，调用方需区分"无走法"和"解 — `data / api-design` → lessons-learned.md#L18
- [经验] `cloneNode(true)` 替换含 SVG 的按钮会导致 SVG 渲染异常（显示不完整）；移 — `dom` → lessons-learned.md#L19
- [经验] **引擎候选走法的调用时机决定产品逻辑正确性**：用户走完后立即 `goMultiPv` 分析的是对 — `api-design` → lessons-learned.md#L26
- [经验] **引擎返回 UCI（e2e4），用户界面必须用 SAN（e4）**：`goMultiPv` 回调中 — `data / api-design` → lessons-learned.md#L27
- [经验] **`cloneNode(true)` 无法移除旧事件监听器，它只是复制了 DOM 结构**：真正安 — `dom` → lessons-learned.md#L42
- [经验] **手工构建100条结构化数据不现实**：经典棋局的 PGN 分散在各网站，无统一免费 API；手动 — `data` → lessons-learned.md#L56
- [经验] GitHub Pages 国内访问需代理；unpkg CDN 加载 Stockfish 可能超时，需 — `build-env` → lessons-learned.md#L61

### 其他

- [问题] AI 重复实现已有组件（棋盘/网格类 UI） — `未分类` → troubleshooting.md#L8
- [问题] CLI subAgent 并行超时 — `开发/测试` → troubleshooting.md#L50
- [问题] 设置面板一闪而过 — `开发/测试` → troubleshooting.md#L59
- [问题] 旧代码与新模块冲突 — `运行时` → troubleshooting.md#L72
- [问题] 引擎候选走法未集成 — `运行时` → troubleshooting.md#L81
- [问题] 设置面板点击无反应（panel toggle 测试失败） — `运行时` → troubleshooting.md#L102
- [问题] checkbox 点击后状态不变化 — `测试错误` → troubleshooting.md#L208
- [问题] CLI subAgent 并行超时 — `开发/测试` → troubleshooting.md#L333
- [问题] 设置面板一闪而过 — `开发/测试` → troubleshooting.md#L342
- [问题] 旧代码与新模块冲突 — `运行时` → troubleshooting.md#L355
- [问题] 引擎候选走法未集成 — `运行时` → troubleshooting.md#L364
- [问题] 设置面板点击无反应（panel toggle 测试失败） — `运行时` → troubleshooting.md#L385
- [问题] CLI subAgent 并行超时 — `开发/测试` → troubleshooting.md#L426
- [问题] 设置面板一闪而过 — `开发/测试` → troubleshooting.md#L435
- [问题] 旧代码与新模块冲突 — `运行时` → troubleshooting.md#L448
- [问题] 引擎候选走法未集成 — `运行时` → troubleshooting.md#L457
- [问题] 设置面板点击无反应（panel toggle 测试失败） — `运行时` → troubleshooting.md#L478
- [问题] checkbox 点击后状态不变化 — `测试错误` → troubleshooting.md#L572
- [问题] sed 批量修改误改结构体定义 — `存档提示` → troubleshooting.md#L675
- [问题] CLI subAgent 并行超时 — `开发/测试` → troubleshooting.md#L733
- [问题] 设置面板一闪而过 — `开发/测试` → troubleshooting.md#L742
- [问题] 旧代码与新模块冲突 — `运行时` → troubleshooting.md#L755
- [问题] 引擎候选走法未集成 — `运行时` → troubleshooting.md#L764
- [问题] 设置面板点击无反应（panel toggle 测试失败） — `运行时` → troubleshooting.md#L785
- [问题] CLI subAgent 并行超时 — `开发/测试` → troubleshooting.md#L826
- [问题] 设置面板一闪而过 — `开发/测试` → troubleshooting.md#L835
- [问题] 旧代码与新模块冲突 — `运行时` → troubleshooting.md#L848
- [问题] 引擎候选走法未集成 — `运行时` → troubleshooting.md#L857
- [问题] 设置面板点击无反应（panel toggle 测试失败） — `运行时` → troubleshooting.md#L878
- [问题] checkbox 点击后状态不变化 — `测试错误` → troubleshooting.md#L984
- [问题] Cookie 过期 / 401 认证失败 — `存档提示` → troubleshooting.md#L1029
- [问题] Digest 刷新失败 — `存档提示` → troubleshooting.md#L1038
- [问题] auth/jycm.json 缺失字段 — `存档提示` → troubleshooting.md#L1047
- [问题] 日期区间多返回一天 — `取数相关` → troubleshooting.md#L1060
- [问题] getAllShopList 返回空数组 — `取数相关` → troubleshooting.md#L1069
- [问题] createAndDownload 返回失败 — `取数相关` → troubleshooting.md#L1078
- [问题] openpyxl 未安装 — `报告相关` → troubleshooting.md#L1091
- [问题] 钉钉推送失败 — `报告相关` → troubleshooting.md#L1100
- [经验] 项目文档结构会随时间进化，"存档"或"恢复"操作前应先 `ls`/`glob` 确认当前文件系统现状 — `ai-workflow` → lessons-learned.md#L24
- [经验] **删除功能必须同步删除对应测试**：移除 `showHints` / `multiPvSettin — `testing` → lessons-learned.md#L29
- [经验] **焦点管理是盲棋产品的核心体验**：进入对局自动 `input.focus()`、引擎走完后恢复焦 — `ux` → lessons-learned.md#L30
- [经验] **复制粘贴是 i18n 错误的常见来源**：将中文值直接粘贴进英文字典，或反之，属于低级但高频的疏 — `i18n` → lessons-learned.md#L33
- [经验] **已删除的 JS 文件若不从 index.html 移除引用，会导致 404**：功能清理和引用清 — `build-env` → lessons-learned.md#L36
- [经验] **删除生产代码的 fallback 函数前，必须先评估测试环境是否提供了该依赖**：架构统一重构必 — `testing / architecture` → lessons-learned.md#L38
- [经验] **`localStorage` mock 必须支持 `setItem` 持久化**：测试中 `gl — `testing` → lessons-learned.md#L39
- [经验] **配置类设置项用「弹窗选择」优于「循环切换」**：循环切换隐藏了全部选项，用户不知道有哪些风格；弹 — `ux` → lessons-learned.md#L41
- [经验] **功能入口迁移需要同步更新「正向路径」和「反向路径」**：不仅要添加新入口，还要移除旧入口，否则用 — `ux / architecture` → lessons-learned.md#L44
- [经验] **测试中断言的具体文本值是重构的敏感点**：重构前应先审计测试中的文本断言，预估需要调整的范围 — `testing` → lessons-learned.md#L46
- [经验] **数据文件中的引号嵌套是极易被忽视的语法陷阱**：在真实浏览器中会抛出 `SyntaxError` — `data / build-env` → lessons-learned.md#L47
- [经验] **playwright 是定位浏览器特有 bug 的有效手段**：通过 `page.add_ini — `testing / debugging` → lessons-learned.md#L49
- [经验] **向后兼容接口设计能减少重构的连锁反应**：旧接口继续工作，内部映射新参数，所有旧测试和外部调用点 — `api-design` → lessons-learned.md#L51
- [经验] `AGENTS.md` 定义触发词和行为约束，`status.md` 记录动态进度，分工明确，新会话 — `ai-workflow` → lessons-learned.md#L54
- [经验] 每批次开发完成后同步更新进度文档，避免新会话迷路 — `ai-workflow` → lessons-learned.md#L55
- [经验] **涉及 7+ 文件读改测的架构重构，应新开会话执行**：继续塞进系统性重构容易触发窗口压缩，导致信 — `ai-workflow` → lessons-learned.md#L60
- [经验] CPU% 精确计算只需 `GetProcessTimes` + wall clock elapsed — `api-design` → lessons-learned.md#L65
- [经验] `tokio::sync::mpsc::Sender::try_send()` 适合非阻塞的进度回调 — `api-design` → lessons-learned.md#L71
- [经验] `useEffect` 依赖 `state.xxx.size === 0` 作为触发条件时，容易形成 — `state-management` → lessons-learned.md#L73
- [经验] `useRef` 作为"只执行一次"的标志，比依赖数组更可靠，尤其涉及批量初始化逻辑时 — `state-management` → lessons-learned.md#L74
- [经验] **测试驱动暴露 Bug**：ResultsPage 的默认勾选死循环是在写单元测试时发现的，手工测 — `testing` → lessons-learned.md#L75
- [经验] **结论**：前端状态管理类的 bug，单元测试是最有效的发现手段，远超手工测试 — `testing` → lessons-learned.md#L76
- [经验] `prompt-next-session.md` 的问题：每次都要重写环境初始化、模块速查表等**不 — `ai-workflow` → lessons-learned.md#L77
- [经验] **改进**：`status.md`（活文档，只记录变化）+ `AGENTS.md`（固定规则） — `ai-workflow` → lessons-learned.md#L78
- [经验] **收益**：新会话读 2 份文件即可开工，维护成本降低 80% — `ai-workflow` → lessons-learned.md#L79
- [经验] **工具硬性限制不要绕圈分析可行性**。Kimi CLI 多窗口无 IPC、无共享内存、无实时同步— — `ai-workflow` → lessons-learned.md#L81
- [经验] **从 SOP 模板采纳更新时，必须逐字核对关键字段，不要凭记忆改写**。触发词「存档」错误抄写为「 — `ai-workflow` → lessons-learned.md#L82
- [经验] **定位代码的最快方法**：清空 `lib.rs` 只保留一个空测试，逐步 `pub mod` 添加 — `debugging` → lessons-learned.md#L87
- [经验] **`#[cfg(not(test))]` 隔离问题代码**是零副作用的修复手法：release 构 — `testing / cross-platform` → lessons-learned.md#L89
- [经验] CSS `@keyframes dropdownIn` 实现淡入+位移动画 — `ux` → lessons-learned.md#L91
- [经验] 年月日联动限制（如今年只显示到当前月） — `ux` → lessons-learned.md#L92
- [经验] 正确做法：后端提供**轻量摘要接口**（只返回 id + category + suggested_ — `architecture / data` → lessons-learned.md#L97
- [经验] 用户实际浏览仍按分页，但"全选全部"走轻量接口，两者解耦 — `pagination / architecture` → lessons-learned.md#L98
- [经验] **事故经过**：ResultsPage 默认自动勾选所有扫描结果 → 用户点击"全选全部"（以为是 — `pagination / state-management / security` → lessons-learned.md#L99
- [经验] **根因链**：默认勾选 × deselectAll 只清当前页 × ConfirmPage 遍历  — `pagination / state-management` → lessons-learned.md#L100
- [经验] **教训**：涉及删除的安全工具，**默认安全 > 默认便利**。所有选择必须用户显式操作，任何"帮 — `security / ux` → lessons-learned.md#L101
- [经验] **原实现**：`deselectAll` 只遍历 `searchedItems`（当前页数据），从 — `pagination / state-management` → lessons-learned.md#L102
- [经验] **修复**：`deselectAll` 清空 `selectedIds` 为 `new Set() — `pagination / state-management` → lessons-learned.md#L103
- [经验] **教训**：跨分页操作时，"取消"必须与"全选"的对称——全选影响多大范围，取消就必须影响多大范围 — `pagination / state-management` → lessons-learned.md#L104
- [经验] **原实现**：ConfirmPage 遍历 `state.scanResults`，过滤出选中的项 — `pagination / state-management` → lessons-learned.md#L105
- [经验] **修复**：遍历 `state.decisions`，每项在 `scanResults` 中查找详 — `pagination / state-management` → lessons-learned.md#L106
- [经验] **教训**：在分页/懒加载架构中，**用户操作集合（decisions）是主数据源，展示数据（sc — `pagination / architecture` → lessons-learned.md#L107
- [经验] `gh api --paginate --slurp` 返回嵌套数组 `[page1, page2, — `api-design` → lessons-learned.md#L204
- [决策] ADR-001：前端技术栈选型 — `架构决策` → decisions.md#L8
- [决策] ADR-002：测试框架选型 — `架构决策` → decisions.md#L22
- [决策] ADR-003：AI 开发方式与批次划分 — `架构决策` → decisions.md#L36
- [决策] ADR-004：棋盘渲染与棋子方案 — `架构决策` → decisions.md#L50
- [决策] ADR-005：数据持久化方案 — `架构决策` → decisions.md#L64
- [决策] ADR-006：通用对局配置层设计 — `架构决策` → decisions.md#L78
- [决策] ADR-007：难度选择从离散按钮改为连续滑块 — `架构决策` → decisions.md#L92
- [决策] ADR-008：棋盘风格设置交互方案 — `架构决策` → decisions.md#L106
- [决策] ADR-009：盲棋复盘入口位置 — `架构决策` → decisions.md#L120
- [决策] ADR-004：为什么 Scanner 进度用 `mpsc::channel` 而非 `tokio: — `架构决策` → decisions.md#L188
- [决策] ADR-005：为什么加密文件回调用同步 `Fn` 而非 `async`？ — `架构决策` → decisions.md#L202
- [决策] ADR-006：为什么用 `status.md` + `session-log.md` 替代 `pr — `架构决策` → decisions.md#L216
- [决策] ADR-008：默认深色主题而非跟随系统 — `架构决策` → decisions.md#L240
- [决策] ADR-009：全选全部功能的技术方案 — `架构决策` → decisions.md#L252
- [决策] ADR-010：路径交互设计 — 文本可点击 vs 独立按钮 — `架构决策` → decisions.md#L264
- [决策] ADR-011：删除策略从 DoD 安全擦除改为普通删除 — `架构决策` → decisions.md#L276
- [决策] ADR-012：扫描范围从 Desktop/Downloads 扩展为全盘扫描 — `架构决策` → decisions.md#L288
- [决策] ADR-013：移除 ResultsPage 默认自动勾选 — `架构决策` → decisions.md#L300
- [决策] ADR-014：同步脚本优先使用仓库 default_branch — `架构决策` → decisions.md#L314
- [决策] ADR-015：syncFrom 配置实现聚合/分发双模式 — `架构决策` → decisions.md#L326
- [决策] ADR-016：母库 AGENTS 与其他项目 AGENTS 物理分离 — `架构决策` → decisions.md#L338
- [决策] ADR-009：Troubleshooting 索引采用独立文件 + 行号链接 — `架构决策` → decisions.md#L392
- [决策] ADR-017：假删除模式通过环境变量 `FRENCH_EXIT_DRY_RUN` 控制 [来源:f — `架构决策` → decisions.md#L442
- [决策] ADR-017：扫描进度条采用后端全局加权进度计算 — `架构决策` → decisions.md#L458
- [决策] ADR-018：个人目录全量扫描 + 文件类型分类 — `架构决策` → decisions.md#L473
- [决策] ADR-002：渠道范围限定（仅淘系生意参谋） — `架构决策` → decisions.md#L524
- [决策] ADR-003：Cookie 刷新策略（Digest 自动刷新，不问用户） — `架构决策` → decisions.md#L538
- [决策] ADR-004：日期时间格式（T00:00:00+08:00，禁用 23:59:59） — `架构决策` → decisions.md#L552
- [决策] ADR-005：报告形态（Markdown 四段式，单店/多店统一） — `架构决策` → decisions.md#L566
- [决策] ADR-017：聚焦 Excel 驱动流，API 驱动流暂不投入 [来源:qianniu_busin — `架构决策` → decisions.md#L584

---

## 按类型分组

### 问题（106 条）

- AI 重复实现已有组件（棋盘/网格类 UI） → troubleshooting.md#L8
- Stockfish 加载超时 / 引擎不启动 → troubleshooting.md#L19
- GitHub Pages 国内打不开 → troubleshooting.md#L28
- Node.js 测试运行时 chess.js 未定义 → troubleshooting.md#L41
- CLI subAgent 并行超时 → troubleshooting.md#L50
- 设置面板一闪而过 → troubleshooting.md#L59
- 旧代码与新模块冲突 → troubleshooting.md#L72
- 引擎候选走法未集成 → troubleshooting.md#L81
- JS 数据文件嵌套单引号导致 `Unexpected identifier` → troubleshooting.md#L92
- 设置面板点击无反应（panel toggle 测试失败） → troubleshooting.md#L102
- mock DOM 中 `querySelector` / `querySelectorAll` 缺失 → troubleshooting.md#L112
- `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 → troubleshooting.md#L124
- `cargo check --lib` 报错：`FILETIME` 未定义 → troubleshooting.md#L132
- `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） → troubleshooting.md#L144
- 运行 `french-exit.exe` 报错：`Could not find the WebView2 Ru... → troubleshooting.md#L156
- 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` → troubleshooting.md#L164
- `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) → troubleshooting.md#L172
- vitest 报错：`Failed to resolve import "@tauri-apps/api/fs... → troubleshooting.md#L182
- vitest 报错：`act is not a function` → troubleshooting.md#L191
- vitest 报错：React 警告 `Cannot update a component while ren... → troubleshooting.md#L199
- ... 还有 86 条

### 经验（98 条）

- 纯 HTML+CSS+JS 项目无需 npm，双击 `index.html` 即可预览，但涉及 Web Wor... → lessons-learned.md#L14
- 手写 IIFE 模块时，用 `window.ModuleName = Module` 暴露 API，内部私有变... → lessons-learned.md#L15
- 浏览器集成测试用 TestRunner（自定义极简框架），保持与 Node 测试同一套断言 API，降低切换成... → lessons-learned.md#L16
- Canvas 图表渲染在浏览器中测试，Node 环境用 Mock 2D context 跳过绘制验证，各测其责 → lessons-learned.md#L17
- PGN 解析器对空/无效输入返回 `[]`（空数组）而非 `null`，调用方需区分"无走法"和"解析失败" → lessons-learned.md#L18
- `cloneNode(true)` 替换含 SVG 的按钮会导致 SVG 渲染异常（显示不完整）；移除事件监听... → lessons-learned.md#L19
- 匿名事件监听器无法被后续代码移除；需要动态解除绑定的监听器必须用命名函数（暴露到 `window` 或模块内部... → lessons-learned.md#L20
- 屏幕切换导航不能只隐藏上一个屏幕，必须遍历 `.screen` 全部隐藏后再显示目标，否则多层屏幕重叠 → lessons-learned.md#L21
- SVG path 中密集参数（如 `a2 2 0 0 1-2.83 0`）在某些浏览器中可能解析异常，命令与参... → lessons-learned.md#L22
- `document` 级事件监听器若引用了某个 DOM 元素，该元素被替换后监听器仍会按旧引用判断，导致逻辑错... → lessons-learned.md#L23
- 项目文档结构会随时间进化，"存档"或"恢复"操作前应先 `ls`/`glob` 确认当前文件系统现状，避免按历... → lessons-learned.md#L24
- **UI 布局/样式不要猜测用户意图**：候选走法开关经历了 5 次位置/样式反复，每次修改后用户都不满意；应... → lessons-learned.md#L25
- **引擎候选走法的调用时机决定产品逻辑正确性**：用户走完后立即 `goMultiPv` 分析的是对手局面；若... → lessons-learned.md#L26
- **引擎返回 UCI（e2e4），用户界面必须用 SAN（e4）**：`goMultiPv` 回调中的 `mo... → lessons-learned.md#L27
- **静态 HTML 结构与动态渲染模块的 DOM 冲突**：`index.html` 中预置了完整棋盘结构，而... → lessons-learned.md#L28
- **删除功能必须同步删除对应测试**：移除 `showHints` / `multiPvSetting` 后，... → lessons-learned.md#L29
- **焦点管理是盲棋产品的核心体验**：进入对局自动 `input.focus()`、引擎走完后恢复焦点、全局 ... → lessons-learned.md#L30
- **i18n 分散架构必然导致翻译遗漏**：当项目同时存在"全局字典 + 模块私有字典 + 硬编码"三种翻译方... → lessons-learned.md#L31
- **JS 中的硬编码人类可读字符串是翻译遗漏的重灾区**：HTML 中的 `data-i18n` 至少能被肉眼... → lessons-learned.md#L32
- **复制粘贴是 i18n 错误的常见来源**：将中文值直接粘贴进英文字典，或反之，属于低级但高频的疏忽 → lessons-learned.md#L33
- ... 还有 78 条

### 决策（38 条）

- ADR-001：前端技术栈选型 → decisions.md#L8
- ADR-002：测试框架选型 → decisions.md#L22
- ADR-003：AI 开发方式与批次划分 → decisions.md#L36
- ADR-004：棋盘渲染与棋子方案 → decisions.md#L50
- ADR-005：数据持久化方案 → decisions.md#L64
- ADR-006：通用对局配置层设计 → decisions.md#L78
- ADR-007：难度选择从离散按钮改为连续滑块 → decisions.md#L92
- ADR-008：棋盘风格设置交互方案 → decisions.md#L106
- ADR-009：盲棋复盘入口位置 → decisions.md#L120
- ADR-017：GitHub 认证从 SSH 切换到 GitHub CLI + HTTPS → decisions.md#L132
- ADR-001：为什么用 Tauri（Rust + WebView2）而非 Electron？ → decisions.md#L147
- ADR-002：为什么前端用 React（而非 Vue/Svelte）？ → decisions.md#L160
- ADR-003：为什么 CPU% 用 `GetProcessTimes` 而非 `sysinfo` crate... → decisions.md#L174
- ADR-004：为什么 Scanner 进度用 `mpsc::channel` 而非 `tokio::sync... → decisions.md#L188
- ADR-005：为什么加密文件回调用同步 `Fn` 而非 `async`？ → decisions.md#L202
- ADR-006：为什么用 `status.md` + `session-log.md` 替代 `prompt-... → decisions.md#L216
- ADR-007：WebView2 分发策略——放弃 NSIS bootstrapper，改用携带 DLL → decisions.md#L228
- ADR-008：默认深色主题而非跟随系统 → decisions.md#L240
- ADR-009：全选全部功能的技术方案 → decisions.md#L252
- ADR-010：路径交互设计 — 文本可点击 vs 独立按钮 → decisions.md#L264
- ... 还有 18 条
