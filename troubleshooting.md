# vibe-coding-project-sop — 跨项目问题母库

> 本文件聚合自多个项目的问题与解决方案。所有条目均标注来源。
> 母库自身问题标注 `[母库]`。

---

### AI 重复实现已有组件（棋盘/网格类 UI） [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 待修复 |
| **现象** | 新增模块（如坐标练习 coordinate.js）手写棋盘网格创建、高亮、抖动、坐标标注等逻辑，而不是复用已有的 `BoardRenderer` 组件。guide.js 的 mini-board 也是部分重复实现 |
| **原因** | AI 没有在写代码前搜索项目中已有的同类组件；lessons-learned 中的描述粒度太粗（只提颜色统一，没提组件复用） |
| **解决** | 1. 新增模块前，先用 grep/搜索 `experience-index.md` 查找可复用的 UI 组件<br>2. 棋盘/网格类 UI 必须通过 `BoardRenderer.create()` 创建，禁止手写<br>3. 如需不同尺寸，在 `BoardRenderer` 中增加 `squareSize` 参数，不要另起炉灶<br>4. `BoardRenderer` 已支持：网格创建、高亮、清除高亮、抖动动画、坐标标注、样式切换、点击回调 |

---

### Stockfish 加载超时 / 引擎不启动 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 页面打开后引擎状态一直显示"Loading"，或走子后无响应 |
| **原因** | Stockfish 从 unpkg CDN 加载，国内网络可能超时；或本地使用 `file://` 协议打开导致 Web Worker 被浏览器拦截 |
| **解决** | 1. 确认使用 HTTP 服务器访问（如 `python -m http.server 8000`），不要用 `file://` 直接打开<br>2. 国内用户可尝试代理或切换网络环境<br>3. 检查控制台是否有 `SharedArrayBuffer` 或 CORS 相关报错 |

### GitHub Pages 国内打不开 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 部署地址 https://michaelgao-watcher.github.io/blindfold-chess/ 在国内无法访问 |
| **原因** | GitHub Pages 域名被 DNS 污染或墙 |
| **解决** | 使用代理/VPN 访问，或考虑国内镜像部署（如 Gitee Pages、Vercel 国内节点） |

---

## 开发/测试

### Node.js 测试运行时 chess.js 未定义 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | ReplayModule 的 `_applyMoves()` 只返回初始位置，导致 navigate/verifyMove 测试失败 |
| **原因** | chess.js 使用 `var Chess = function(){}` + `exports.Chess = Chess`，Node `require` 后需手动 `global.Chess = require('chess.js').Chess` 才能被 replay.js 识别 |
| **解决** | 浏览器环境中 `<script src>` 会自动暴露全局 `Chess`；Node 测试环境手动设置 `global.Chess = require('./chess.js').Chess` |

### CLI subAgent 并行超时 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已修复（改为 IDE 串行） |
| **现象** | Kimi Code CLI 并行运行多个 subAgent 时，默认 15 分钟超时导致任务中断 |
| **原因** | 复杂模块（如 SettingsModule 51 个测试）在 15 分钟内无法完成代码+测试+调试 |
| **解决** | 改用 IDE 串行执行，或显式将 subAgent timeout 调高到 3600s |

### 设置面板一闪而过 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 点击齿轮图标后设置面板闪现立刻消失 |
| **原因** | 事件冒泡或 CSS transition 与 class 切换时序冲突 |
| **解决** | 检查 `settings-toggle` 的 click 事件是否阻止冒泡；确认 `.hidden` 和 `.active` 的切换使用 `requestAnimationFrame` 保证时序 |

---

## 运行时

### 旧代码与新模块冲突 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已知未修复（逐步迁移中） |
| **现象** | `common.js` / `game.js` 中的旧全局函数与新 IIFE 模块并存，可能出现变量名冲突或重复初始化 |
| **原因** | 早期原型代码未模块化，新模块逐步替换中 |
| **解决** | 修改代码前检查同名全局变量；迁移完成后删除旧文件 |

### 引擎候选走法未集成 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已知未修复 |
| **现象** | EngineModule 支持 `goMultiPv`，但 BlindfoldModule UI 未展示 Top 3 候选走法 |
| **原因** | 功能已实现于引擎层，UI 层未接入 |
| **解决** | 在 BlindfoldModule 中添加候选走法面板 + 用户开关（默认关闭），调用 `EngineModule.goMultiPv` 获取数据 |

---

### JS 数据文件嵌套单引号导致 `Unexpected identifier` [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 页面打开后点击按钮完全无反应，浏览器控制台报错 `Uncaught SyntaxError: Unexpected identifier 's'` |
| **原因** | `data/games.js` 中 `en: 'Rubinstein's Immortal'` 的单引号提前结束了字符串，`s Immortal'` 被解析为标识符 |
| **解决** | 将包含单引号的字符串改用双引号包裹：`en: "Rubinstein's Immortal"` |
| **预防** | 数据文件中包含英文撇号（如 `It's`、`Rubinstein's`）时，必须检查外层引号类型；建议此类字符串统一使用双引号 |

### 设置面板点击无反应（panel toggle 测试失败） [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `test-settings-node.js` 中 panel toggle 测试突然失败，`settingsPanel` 未获得 `show` class |
| **原因** | `settings.js` 的 `init()` 每次被调用都会给 `settingsToggle` 新增一个 `click` 监听器，旧监听器从未移除。多次 init 后监听器数量累积为奇数/偶数，导致 `classList.toggle('show')` 的最终状态不可预期 |
| **解决** | 保存 panel toggle 和 document click 的匿名监听器引用到 `window._settingsPanelToggleHandler` / `window._settingsDocumentHandler`，下次 `init()` 时先 `removeEventListener` 再重新绑定 |
| **预防** | 需要动态解除绑定的事件监听器必须用命名函数（或保存引用），绝对不能用匿名函数直接 `addEventListener` |

### mock DOM 中 `querySelector` / `querySelectorAll` 缺失 [来源:blindfold-chess @2026-05-21]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | `boardStyleModal.querySelector('[data-style="blue"]')` 抛出 `TypeError: modal.querySelector is not a function` |
| **原因** | `test-settings-node.js` 的 `MockElement` 只实现了 `getElementById`，未实现 `querySelector`/`querySelectorAll` |
| **解决** | 测试代码中改用 `document.getElementById` + 遍历 `children` 的方式查找目标元素；或给 mock DOM 补充 `querySelectorAll` 实现 |

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*
---

### `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module` |
| **原因** | `windows` crate 的 Cargo.toml features 中未启用 `Win32_Storage_FileSystem` |
| **解决** | 在 `src-tauri/Cargo.toml` 的 `windows` features 中添加 `"Win32_Storage_FileSystem"` |

### `cargo check --lib` 报错：`FILETIME` 未定义 [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module 'FILETIME'` |
| **原因** | 未导入 `windows::Win32::Foundation::FILETIME` |
| **解决** | `use windows::Win32::Foundation::FILETIME;` |

---

## 运行时错误

### `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **状态** | ✅ 已修复 |
| **现象** | 测试编译通过，但运行时弹窗或报错 `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` |
| **原因** | `tauri::AppHandle` 出现在 `async fn` 签名中，与 MinGW UCRT 生成不兼容的 PE 导入表 |
| **解决** | 将含 `AppHandle` 的 async command 函数拆分到 `commands/handlers.rs`，在 `#[cfg(not(test))]` 下条件编译；`lib.rs` 的 `run()` 同样条件编译。测试模式下不链接这些函数，从而绕过 loader 入口点缺失问题 |
| **验证** | `cargo test --lib` 103 测全绿 |

---

### 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime` [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示找不到 WebView2 Runtime |
| **原因** | 系统未安装 WebView2 Runtime（某些重装系统或企业阉割镜像） |
| **解决** | 从 NuGet 包 `Microsoft.Web.WebView2` 提取 `WebView2Loader.dll`，配置 `tauri.conf.json` 的 `bundle.resources` 自动打包到 `.exe` 同目录；同时程序启动时自动检测系统 EdgeCore 作为 WebView2 内核回退 |

### 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示 `由于找不到 WebView2Loader.dll，无法继续执行代码` |
| **原因** | 系统有 EdgeCore（Edge 浏览器内核）但缺少 WebView2 Runtime 的加载入口 DLL |
| **解决** | 将 `WebView2Loader.dll`（可从 NuGet 提取）与 `.exe` 一起分发。Tauri `bundle.resources` 会自动将其复制到输出目录 |

### `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `cargo tauri build` 报错，提示无法访问 `french-exit.exe` 或 `target/release/` 下的文件 |
| **原因** | `french-exit.exe` 仍在后台运行，锁定了构建产物 |
| **解决** | `taskkill //F //IM french-exit.exe` 强制结束进程后再构建 |

## 测试错误

### vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"` [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `Error: Failed to resolve import "@tauri-apps/api/fs" from "src/pages/ResultsPage.tsx"` |
| **原因** | Tauri v2 已移除 `@tauri-apps/api/fs` 模块，改为 `@tauri-apps/plugin-fs`（需单独安装） |
| **解决** | 1. 若只需要测试通过：在 `vite.config.ts` 中配置 alias，将 `@tauri-apps/api/fs` 指向本地 mock 文件<br>2. 若需要真功能：安装 `@tauri-apps/plugin-fs` 并修改所有导入路径 |
| **本项目做法** | `vite.config.ts` 中：<br>`"@tauri-apps/api/fs": path.resolve(__dirname, "./src/test/mocks/tauri-fs.ts")` |

### vitest 报错：`act is not a function` [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `TypeError: act is not a function` |
| **原因** | 从 `vitest` 导入 `act`，但 `act` 实际来自 `@testing-library/react` |
| **解决** | `import { act } from "@testing-library/react"` 而非 `from "vitest"` |

### vitest 报错：React 警告 `Cannot update a component while rendering` [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `Warning: Cannot update a component (AppProvider) while rendering a different component (ResultsPage)` |
| **原因** | `dispatch()` 在 `setState` 的 updater 函数内部被调用，React 认为这是"渲染时更新" |
| **解决** | 将 `dispatch` 移出 updater 函数：先计算新状态，再分别调用 `setState` 和 `dispatch` |
| **本项目修复** | `ResultsPage.tsx` 中的 `toggleItem` 函数已修复 |

### checkbox 点击后状态不变化 [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `fireEvent.click(checkbox)` 后 `checked` 状态未变 |
| **原因** | React controlled checkbox 的 `onChange` 可能不响应 `click` 事件 |
| **解决** | 用 `@testing-library/user-event` 的 `user.click(checkbox)` 替代 `fireEvent.click` |
| **安装** | `npm install -D @testing-library/user-event` |

---

## 环境问题

### 中文路径下编译失败 [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | MinGW 链接器报错，无法生成 `.exe` |
| **原因** | 工作目录含中文（如 `E:/工作文件/...`），MinGW 工具链对 Unicode 路径支持差 |
| **解决** | 1. `rm -rf /c/french-exit && cp -r "/e/工作文件/vs-code/french-exit" /c/french-exit`<br>2. `cd /c/french-exit/src-tauri && cargo check --lib` |
| **注意** | `cargo check --lib` 和 `cargo test --no-run` 不需要链接，可在中文路径直接运行 |

---

### cargo tauri dev 在后台任务中崩溃 [来源:french-exit @2026-05-21]

| | 内容 |
|---|---|
| **现象** | `cargo tauri dev` 启动后编译成功，但运行时报 `exit code: 0xc0000005, STATUS_ACCESS_VIOLATION`，随后 Segmentation fault |
| **原因** | Tauri 应用需要创建 WebView2 GUI 窗口，而 background task / SSH / 无头环境缺少 Windows 桌面会话和显示上下文 |
| **解决** | 1. 在本地交互式终端（PowerShell/CMD）中手动运行 `cargo tauri dev`<br>2. 或改用 `npm run dev` 仅启动 Vite 前端服务器，在浏览器中预览 UI（IPC 功能不可用） |
| **注意** | 此限制仅影响 GUI 启动方式，不影响 `cargo tauri build` 构建产物 |

---

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否有新的具体报错需要记入本文件。有则按模板追加；没有则跳过。

---

### PowerShell 执行中文脚本报 "UnexpectedToken" [来源:vibe-coding-project-sop @2026-05-22]

| | 内容 |
|---|---|
| **现象** | .\start-llm-server.ps1 执行时报错：表达式或语句中包含意外的标记"}"，行号指向 `}` |
| **原因** | PowerShell 5.1 默认以 Windows-1252 编码读取无 BOM 的 UTF-8 文件，中文字符被错误解码后破坏了字符串引号匹配，导致解析器认为 `}` 位置不对 |
| **解决** | 给脚本文件添加 UTF-8 BOM（文件头添加字节 EF BB BF）：`printf '\xef\xbb\xbf' > file.ps1 && cat original.ps1 >> file.ps1` |
| **注意** | PowerShell 7+ 默认支持 UTF-8 无 BOM，但 Windows 10 自带的 PowerShell 5.1 仍受此限制 |


---

### GitHub push 报错 `Permission denied (publickey)` [来源:vibe-coding-project-sop @2026-05-23]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `git push` 失败，`ssh -T git@github.com` 返回 `Permission denied (publickey)` |
| **原因** | 1. SSH agent 未加载私钥（`ssh-add -l` 显示 `no identities`）<br>2. GitHub 账户未添加对应公钥 |
| **解决** | 方案 A（SSH）：`ssh-add ~/.ssh/id_ed25519`，将公钥添加到 GitHub Settings → SSH Keys<br>方案 B（推荐）：改用 HTTPS + GitHub CLI 管理凭证（见下方 `gh auth login` 条目） |

### `gh auth login` 超时：`read tcp ... operation timed out` [来源:vibe-coding-project-sop @2026-05-23]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `gh auth login` 报错 `Post "https://github.com/login/device/code": read tcp ...: read: operation timed out` |
| **原因** | `gh` 底层是 Go 程序，默认直连 GitHub API，不走系统代理。国内网络环境下 GitHub API 可能超时 |
| **解决** | 设置环境变量后运行：`HTTPS_PROXY=http://127.0.0.1:7897 HTTP_PROXY=http://127.0.0.1:7897 gh auth login` |

### HuggingFace 模型下载连接超时 `curl: (28) Could not connect to server` [来源:vibe-coding-project-sop @2026-05-24]

| | 内容 |
|---|---|
| **状态** | 已解决 |
| **现象** | `curl https://huggingface.co/.../resolve/main/...gguf` 长时间无响应后报错 `Failed to connect to huggingface.co port 443 after 21073 ms` |
| **原因** | 国内网络环境下 HuggingFace 主站被墙或 DNS 污染 |
| **解决** | 改用国内镜像源：ModelScope（`https://modelscope.cn/models/<namespace>/<model>/resolve/master/<file>.gguf`），实测速度 2.5MB/s+ |

### PowerShell 添加防火墙规则权限不足 `Access is denied` [来源:vibe-coding-project-sop @2026-05-24]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 非管理员身份运行 `start-llm-server.ps1` 时，`New-NetFirewallRule` 报错 `Access is denied` |
| **原因** | Windows 防火墙规则修改需要管理员权限 |
| **解决** | 1. 以管理员身份运行一次 PowerShell 执行脚本，添加规则后后续无需管理员<br>2. 或手动在 Windows 防火墙高级设置中添加 11434 TCP 入站规则 |
---

### Stockfish 加载超时 / 引擎不启动 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 页面打开后引擎状态一直显示"Loading"，或走子后无响应 |
| **原因** | Stockfish 从 unpkg CDN 加载，国内网络可能超时；或本地使用 `file://` 协议打开导致 Web Worker 被浏览器拦截 |
| **解决** | 1. 确认使用 HTTP 服务器访问（如 `python -m http.server 8000`），不要用 `file://` 直接打开<br>2. 国内用户可尝试代理或切换网络环境<br>3. 检查控制台是否有 `SharedArrayBuffer` 或 CORS 相关报错 |

### GitHub Pages 国内打不开 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 部署地址 https://michaelgao1999.github.io/blindfold-chess/ 在国内无法访问 |
| **原因** | GitHub Pages 域名被 DNS 污染或墙 |
| **解决** | 使用代理/VPN 访问，或考虑国内镜像部署（如 Gitee Pages、Vercel 国内节点） |

---

## 开发/测试

### Node.js 测试运行时 chess.js 未定义 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | ReplayModule 的 `_applyMoves()` 只返回初始位置，导致 navigate/verifyMove 测试失败 |
| **原因** | chess.js 使用 `var Chess = function(){}` + `exports.Chess = Chess`，Node `require` 后需手动 `global.Chess = require('chess.js').Chess` 才能被 replay.js 识别 |
| **解决** | 浏览器环境中 `<script src>` 会自动暴露全局 `Chess`；Node 测试环境手动设置 `global.Chess = require('./chess.js').Chess` |

### CLI subAgent 并行超时 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（改为 IDE 串行） |
| **现象** | Kimi Code CLI 并行运行多个 subAgent 时，默认 15 分钟超时导致任务中断 |
| **原因** | 复杂模块（如 SettingsModule 51 个测试）在 15 分钟内无法完成代码+测试+调试 |
| **解决** | 改用 IDE 串行执行，或显式将 subAgent timeout 调高到 3600s |

### 设置面板一闪而过 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 点击齿轮图标后设置面板闪现立刻消失 |
| **原因** | 事件冒泡或 CSS transition 与 class 切换时序冲突 |
| **解决** | 检查 `settings-toggle` 的 click 事件是否阻止冒泡；确认 `.hidden` 和 `.active` 的切换使用 `requestAnimationFrame` 保证时序 |

---

## 运行时

### 旧代码与新模块冲突 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（逐步迁移中） |
| **现象** | `common.js` / `game.js` 中的旧全局函数与新 IIFE 模块并存，可能出现变量名冲突或重复初始化 |
| **原因** | 早期原型代码未模块化，新模块逐步替换中 |
| **解决** | 修改代码前检查同名全局变量；迁移完成后删除旧文件 |

### 引擎候选走法未集成 [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复 |
| **现象** | EngineModule 支持 `goMultiPv`，但 BlindfoldModule UI 未展示 Top 3 候选走法 |
| **原因** | 功能已实现于引擎层，UI 层未接入 |
| **解决** | 在 BlindfoldModule 中添加候选走法面板 + 用户开关（默认关闭），调用 `EngineModule.goMultiPv` 获取数据 |

---

### JS 数据文件嵌套单引号导致 `Unexpected identifier` [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 页面打开后点击按钮完全无反应，浏览器控制台报错 `Uncaught SyntaxError: Unexpected identifier 's'` |
| **原因** | `data/games.js` 中 `en: 'Rubinstein's Immortal'` 的单引号提前结束了字符串，`s Immortal'` 被解析为标识符 |
| **解决** | 将包含单引号的字符串改用双引号包裹：`en: "Rubinstein's Immortal"` |
| **预防** | 数据文件中包含英文撇号（如 `It's`、`Rubinstein's`）时，必须检查外层引号类型；建议此类字符串统一使用双引号 |

### 设置面板点击无反应（panel toggle 测试失败） [来源:blindfold-chess @2026-05-19] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `test-settings-node.js` 中 panel toggle 测试突然失败，`settingsPanel` 未获得 `show` class |
| **原因** | `settings.js` 的 `init()` 每次被调用都会给 `settingsToggle` 新增一个 `click` 监听器，旧监听器从未移除。多次 init 后监听器数量累积为奇数/偶数，导致 `classList.toggle('show')` 的最终状态不可预期 |
| **解决** | 保存 panel toggle 和 document click 的匿名监听器引用到 `window._settingsPanelToggleHandler` / `window._settingsDocumentHandler`，下次 `init()` 时先 `removeEventListener` 再重新绑定 |
| **预防** | 需要动态解除绑定的事件监听器必须用命名函数（或保存引用），绝对不能用匿名函数直接 `addEventListener` |

### Stockfish 加载超时 / 引擎不启动 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 页面打开后引擎状态一直显示"Loading"，或走子后无响应 |
| **原因** | Stockfish 从 unpkg CDN 加载，国内网络可能超时；或本地使用 `file://` 协议打开导致 Web Worker 被浏览器拦截 |
| **解决** | 1. 确认使用 HTTP 服务器访问（如 `python -m http.server 8000`），不要用 `file://` 直接打开<br>2. 国内用户可尝试代理或切换网络环境<br>3. 检查控制台是否有 `SharedArrayBuffer` 或 CORS 相关报错 |

### GitHub Pages 国内打不开 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 部署地址 https://michaelgao-watcher.github.io/blindfold-chess/ 在国内无法访问 |
| **原因** | GitHub Pages 域名被 DNS 污染或墙 |
| **解决** | 使用代理/VPN 访问，或考虑国内镜像部署（如 Gitee Pages、Vercel 国内节点） |

---

## 开发/测试

### Node.js 测试运行时 chess.js 未定义 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | ReplayModule 的 `_applyMoves()` 只返回初始位置，导致 navigate/verifyMove 测试失败 |
| **原因** | chess.js 使用 `var Chess = function(){}` + `exports.Chess = Chess`，Node `require` 后需手动 `global.Chess = require('chess.js').Chess` 才能被 replay.js 识别 |
| **解决** | 浏览器环境中 `<script src>` 会自动暴露全局 `Chess`；Node 测试环境手动设置 `global.Chess = require('./chess.js').Chess` |

### CLI subAgent 并行超时 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（改为 IDE 串行） |
| **现象** | Kimi Code CLI 并行运行多个 subAgent 时，默认 15 分钟超时导致任务中断 |
| **原因** | 复杂模块（如 SettingsModule 51 个测试）在 15 分钟内无法完成代码+测试+调试 |
| **解决** | 改用 IDE 串行执行，或显式将 subAgent timeout 调高到 3600s |

### 设置面板一闪而过 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 点击齿轮图标后设置面板闪现立刻消失 |
| **原因** | 事件冒泡或 CSS transition 与 class 切换时序冲突 |
| **解决** | 检查 `settings-toggle` 的 click 事件是否阻止冒泡；确认 `.hidden` 和 `.active` 的切换使用 `requestAnimationFrame` 保证时序 |

---

## 运行时

### 旧代码与新模块冲突 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（逐步迁移中） |
| **现象** | `common.js` / `game.js` 中的旧全局函数与新 IIFE 模块并存，可能出现变量名冲突或重复初始化 |
| **原因** | 早期原型代码未模块化，新模块逐步替换中 |
| **解决** | 修改代码前检查同名全局变量；迁移完成后删除旧文件 |

### 引擎候选走法未集成 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复 |
| **现象** | EngineModule 支持 `goMultiPv`，但 BlindfoldModule UI 未展示 Top 3 候选走法 |
| **原因** | 功能已实现于引擎层，UI 层未接入 |
| **解决** | 在 BlindfoldModule 中添加候选走法面板 + 用户开关（默认关闭），调用 `EngineModule.goMultiPv` 获取数据 |

---

### JS 数据文件嵌套单引号导致 `Unexpected identifier` [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 页面打开后点击按钮完全无反应，浏览器控制台报错 `Uncaught SyntaxError: Unexpected identifier 's'` |
| **原因** | `data/games.js` 中 `en: 'Rubinstein's Immortal'` 的单引号提前结束了字符串，`s Immortal'` 被解析为标识符 |
| **解决** | 将包含单引号的字符串改用双引号包裹：`en: "Rubinstein's Immortal"` |
| **预防** | 数据文件中包含英文撇号（如 `It's`、`Rubinstein's`）时，必须检查外层引号类型；建议此类字符串统一使用双引号 |

### 设置面板点击无反应（panel toggle 测试失败） [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `test-settings-node.js` 中 panel toggle 测试突然失败，`settingsPanel` 未获得 `show` class |
| **原因** | `settings.js` 的 `init()` 每次被调用都会给 `settingsToggle` 新增一个 `click` 监听器，旧监听器从未移除。多次 init 后监听器数量累积为奇数/偶数，导致 `classList.toggle('show')` 的最终状态不可预期 |
| **解决** | 保存 panel toggle 和 document click 的匿名监听器引用到 `window._settingsPanelToggleHandler` / `window._settingsDocumentHandler`，下次 `init()` 时先 `removeEventListener` 再重新绑定 |
| **预防** | 需要动态解除绑定的事件监听器必须用命名函数（或保存引用），绝对不能用匿名函数直接 `addEventListener` |

### `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module` |
| **原因** | `windows` crate 的 Cargo.toml features 中未启用 `Win32_Storage_FileSystem` |
| **解决** | 在 `src-tauri/Cargo.toml` 的 `windows` features 中添加 `"Win32_Storage_FileSystem"` |

### `cargo check --lib` 报错：`FILETIME` 未定义 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module 'FILETIME'` |
| **原因** | 未导入 `windows::Win32::Foundation::FILETIME` |
| **解决** | `use windows::Win32::Foundation::FILETIME;` |

---

## 运行时错误

### `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | ✅ 已修复 |
| **现象** | 测试编译通过，但运行时弹窗或报错 `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` |
| **原因** | `tauri::AppHandle` 出现在 `async fn` 签名中，与 MinGW UCRT 生成不兼容的 PE 导入表 |
| **解决** | 将含 `AppHandle` 的 async command 函数拆分到 `commands/handlers.rs`，在 `#[cfg(not(test))]` 下条件编译；`lib.rs` 的 `run()` 同样条件编译。测试模式下不链接这些函数，从而绕过 loader 入口点缺失问题 |
| **验证** | `cargo test --lib` 103 测全绿 |

---

### 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示找不到 WebView2 Runtime |
| **原因** | 系统未安装 WebView2 Runtime（某些重装系统或企业阉割镜像） |
| **解决** | 从 NuGet 包 `Microsoft.Web.WebView2` 提取 `WebView2Loader.dll`，配置 `tauri.conf.json` 的 `bundle.resources` 自动打包到 `.exe` 同目录；同时程序启动时自动检测系统 EdgeCore 作为 WebView2 内核回退 |

### 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示 `由于找不到 WebView2Loader.dll，无法继续执行代码` |
| **原因** | 系统有 EdgeCore（Edge 浏览器内核）但缺少 WebView2 Runtime 的加载入口 DLL |
| **解决** | 将 `WebView2Loader.dll`（可从 NuGet 提取）与 `.exe` 一起分发。Tauri `bundle.resources` 会自动将其复制到输出目录 |

### `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `cargo tauri build` 报错，提示无法访问 `french-exit.exe` 或 `target/release/` 下的文件 |
| **原因** | `french-exit.exe` 仍在后台运行，锁定了构建产物 |
| **解决** | `taskkill //F //IM french-exit.exe` 强制结束进程后再构建 |

## 测试错误

### vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `Error: Failed to resolve import "@tauri-apps/api/fs" from "src/pages/ResultsPage.tsx"` |
| **原因** | Tauri v2 已移除 `@tauri-apps/api/fs` 模块，改为 `@tauri-apps/plugin-fs`（需单独安装） |
| **解决** | 1. 若只需要测试通过：在 `vite.config.ts` 中配置 alias，将 `@tauri-apps/api/fs` 指向本地 mock 文件<br>2. 若需要真功能：安装 `@tauri-apps/plugin-fs` 并修改所有导入路径 |
| **本项目做法** | `vite.config.ts` 中：<br>`"@tauri-apps/api/fs": path.resolve(__dirname, "./src/test/mocks/tauri-fs.ts")` |

### vitest 报错：`act is not a function` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `TypeError: act is not a function` |
| **原因** | 从 `vitest` 导入 `act`，但 `act` 实际来自 `@testing-library/react` |
| **解决** | `import { act } from "@testing-library/react"` 而非 `from "vitest"` |

### vitest 报错：React 警告 `Cannot update a component while rendering` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `Warning: Cannot update a component (AppProvider) while rendering a different component (ResultsPage)` |
| **原因** | `dispatch()` 在 `setState` 的 updater 函数内部被调用，React 认为这是"渲染时更新" |
| **解决** | 将 `dispatch` 移出 updater 函数：先计算新状态，再分别调用 `setState` 和 `dispatch` |
| **本项目修复** | `ResultsPage.tsx` 中的 `toggleItem` 函数已修复 |

### checkbox 点击后状态不变化 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `fireEvent.click(checkbox)` 后 `checked` 状态未变 |
| **原因** | React controlled checkbox 的 `onChange` 可能不响应 `click` 事件 |
| **解决** | 用 `@testing-library/user-event` 的 `user.click(checkbox)` 替代 `fireEvent.click` |
| **安装** | `npm install -D @testing-library/user-event` |

---

## 环境问题

### 中文路径下编译失败 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | MinGW 链接器报错，无法生成 `.exe` |
| **原因** | 工作目录含中文（如 `E:/工作文件/...`），MinGW 工具链对 Unicode 路径支持差 |
| **解决** | 1. `rm -rf /c/french-exit && cp -r "/e/工作文件/vs-code/french-exit" /c/french-exit`<br>2. `cd /c/french-exit/src-tauri && cargo check --lib` |
| **注意** | `cargo check --lib` 和 `cargo test --no-run` 不需要链接，可在中文路径直接运行 |

---

### cargo tauri dev 在后台任务中崩溃 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `cargo tauri dev` 启动后编译成功，但运行时报 `exit code: 0xc0000005, STATUS_ACCESS_VIOLATION`，随后 Segmentation fault |
| **原因** | Tauri 应用需要创建 WebView2 GUI 窗口，而 background task / SSH / 无头环境缺少 Windows 桌面会话和显示上下文 |
| **解决** | 1. 在本地交互式终端（PowerShell/CMD）中手动运行 `cargo tauri dev`<br>2. 或改用 `npm run dev` 仅启动 Vite 前端服务器，在浏览器中预览 UI（IPC 功能不可用） |
| **注意** | 此限制仅影响 GUI 启动方式，不影响 `cargo tauri build` 构建产物 |

---

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否有新的具体报错需要记入本文件。有则按模板追加；没有则跳过。

---

### PowerShell 执行中文脚本报 "UnexpectedToken" [来源:vibe-coding-project-sop @2026-05-22] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **现象** | .\start-llm-server.ps1 执行时报错：表达式或语句中包含意外的标记"}"，行号指向 `}` |
| **原因** | PowerShell 5.1 默认以 Windows-1252 编码读取无 BOM 的 UTF-8 文件，中文字符被错误解码后破坏了字符串引号匹配，导致解析器认为 `}` 位置不对 |
| **解决** | 给脚本文件添加 UTF-8 BOM（文件头添加字节 EF BB BF）：`printf '\xef\xbb\xbf' > file.ps1 && cat original.ps1 >> file.ps1` |
| **注意** | PowerShell 7+ 默认支持 UTF-8 无 BOM，但 Windows 10 自带的 PowerShell 5.1 仍受此限制 |


---

### GitHub push 报错 `Permission denied (publickey)` [来源:vibe-coding-project-sop @2026-05-23] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `git push` 失败，`ssh -T git@github.com` 返回 `Permission denied (publickey)` |
| **原因** | 1. SSH agent 未加载私钥（`ssh-add -l` 显示 `no identities`）<br>2. GitHub 账户未添加对应公钥 |
| **解决** | 方案 A（SSH）：`ssh-add ~/.ssh/id_ed25519`，将公钥添加到 GitHub Settings → SSH Keys<br>方案 B（推荐）：改用 HTTPS + GitHub CLI 管理凭证（见下方 `gh auth login` 条目） |

### `gh auth login` 超时：`read tcp ... operation timed out` [来源:vibe-coding-project-sop @2026-05-23] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `gh auth login` 报错 `Post "https://github.com/login/device/code": read tcp ...: read: operation timed out` |
| **原因** | `gh` 底层是 Go 程序，默认直连 GitHub API，不走系统代理。国内网络环境下 GitHub API 可能超时 |
| **解决** | 设置环境变量后运行：`HTTPS_PROXY=http://127.0.0.1:7897 HTTP_PROXY=http://127.0.0.1:7897 gh auth login` |

### HuggingFace 模型下载连接超时 `curl: (28) Could not connect to server` [来源:vibe-coding-project-sop @2026-05-24] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已解决 |
| **现象** | `curl https://huggingface.co/.../resolve/main/...gguf` 长时间无响应后报错 `Failed to connect to huggingface.co port 443 after 21073 ms` |
| **原因** | 国内网络环境下 HuggingFace 主站被墙或 DNS 污染 |
| **解决** | 改用国内镜像源：ModelScope（`https://modelscope.cn/models/<namespace>/<model>/resolve/master/<file>.gguf`），实测速度 2.5MB/s+ |

### PowerShell 添加防火墙规则权限不足 `Access is denied` [来源:vibe-coding-project-sop @2026-05-24] [来源:vibe-coding-project-sop @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 非管理员身份运行 `start-llm-server.ps1` 时，`New-NetFirewallRule` 报错 `Access is denied` |
| **原因** | Windows 防火墙规则修改需要管理员权限 |
| **解决** | 1. 以管理员身份运行一次 PowerShell 执行脚本，添加规则后后续无需管理员<br>2. 或手动在 Windows 防火墙高级设置中添加 11434 TCP 入站规则 |

### Node.js 报 SyntaxError: Unexpected identifier（i18n 中文字符串） [来源:blindfold-chess @2026-05-29] [来源:blindfold-chess @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已解决 |
| **现象** | `node -e "require('./js/common.js')"` 报错 `SyntaxError: Unexpected identifier '日'` 或 `Invalid or unexpected token`，指向中文 i18n 字符串 |
| **原因** | 中文全角引号 `""`（U+201C/U+201D）在某些编辑器/环境下被存储为 ASCII 双引号 `"`（U+0022），导致 JS 字符串被意外截断。如 `"走"日"字形"` 被解析为 `"走"` + `日` + `"字形"` |
| **解决** | 将中文引号替换为其他标点（如 `「」`），或使用 Unicode 转义 `\u201c\u201d` |
---

### sed 批量修改误改结构体定义 [来源:french-exit @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `cargo tauri build` 报错 `missing field global_percent in initializer of ScanProgress`，但 `scanner/mod.rs` 中并未主动添加该字段 |
| **原因** | 使用 `sed -i '/ScanProgress {/...'` 批量给所有 `ScanProgress` 实例添加字段时，也匹配到了 `pub struct ScanProgress {` 结构体定义行，在定义中插入了 `global_percent: None,` |
| **解决** | 1. `git checkout mod.rs` 还原结构体定义<br>2. 改用更精确的匹配条件，或手动逐个文件修改 |
| **注意** | 批量文本替换时，结构体定义和实例化共用同一关键字，需额外排除定义行 |

### French Exit 进程锁定 exe 导致复制失败 [来源:french-exit @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `cp` 报错 `cannot create regular file: Device or resource busy` |
| **原因** | French Exit 正在运行，Windows 锁定 exe 文件句柄 |
| **解决** | 1. `taskkill //F //IM french-exit.exe` 强制终止进程<br>2. `rm -f release/french-exit.exe` 强制删除旧文件<br>3. 重新复制新构建的 exe |
| **注意** | Tauri 构建时的 bundle patching 步骤也会因文件锁定而报 `os error 32`，不影响 exe 本身 |

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否有新的具体报错需要记入本文件。有则按模板追加；没有则跳过。
---

### Stockfish 加载超时 / 引擎不启动 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 页面打开后引擎状态一直显示"Loading"，或走子后无响应 |
| **原因** | Stockfish 从 unpkg CDN 加载，国内网络可能超时；或本地使用 `file://` 协议打开导致 Web Worker 被浏览器拦截 |
| **解决** | 1. 确认使用 HTTP 服务器访问（如 `python -m http.server 8000`），不要用 `file://` 直接打开<br>2. 国内用户可尝试代理或切换网络环境<br>3. 检查控制台是否有 `SharedArrayBuffer` 或 CORS 相关报错 |

### GitHub Pages 国内打不开 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 部署地址 https://michaelgao1999.github.io/blindfold-chess/ 在国内无法访问 |
| **原因** | GitHub Pages 域名被 DNS 污染或墙 |
| **解决** | 使用代理/VPN 访问，或考虑国内镜像部署（如 Gitee Pages、Vercel 国内节点） |

---

## 开发/测试

### Node.js 测试运行时 chess.js 未定义 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | ReplayModule 的 `_applyMoves()` 只返回初始位置，导致 navigate/verifyMove 测试失败 |
| **原因** | chess.js 使用 `var Chess = function(){}` + `exports.Chess = Chess`，Node `require` 后需手动 `global.Chess = require('chess.js').Chess` 才能被 replay.js 识别 |
| **解决** | 浏览器环境中 `<script src>` 会自动暴露全局 `Chess`；Node 测试环境手动设置 `global.Chess = require('./chess.js').Chess` |

### CLI subAgent 并行超时 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（改为 IDE 串行） |
| **现象** | Kimi Code CLI 并行运行多个 subAgent 时，默认 15 分钟超时导致任务中断 |
| **原因** | 复杂模块（如 SettingsModule 51 个测试）在 15 分钟内无法完成代码+测试+调试 |
| **解决** | 改用 IDE 串行执行，或显式将 subAgent timeout 调高到 3600s |

### 设置面板一闪而过 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 点击齿轮图标后设置面板闪现立刻消失 |
| **原因** | 事件冒泡或 CSS transition 与 class 切换时序冲突 |
| **解决** | 检查 `settings-toggle` 的 click 事件是否阻止冒泡；确认 `.hidden` 和 `.active` 的切换使用 `requestAnimationFrame` 保证时序 |

---

## 运行时

### 旧代码与新模块冲突 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（逐步迁移中） |
| **现象** | `common.js` / `game.js` 中的旧全局函数与新 IIFE 模块并存，可能出现变量名冲突或重复初始化 |
| **原因** | 早期原型代码未模块化，新模块逐步替换中 |
| **解决** | 修改代码前检查同名全局变量；迁移完成后删除旧文件 |

### 引擎候选走法未集成 [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复 |
| **现象** | EngineModule 支持 `goMultiPv`，但 BlindfoldModule UI 未展示 Top 3 候选走法 |
| **原因** | 功能已实现于引擎层，UI 层未接入 |
| **解决** | 在 BlindfoldModule 中添加候选走法面板 + 用户开关（默认关闭），调用 `EngineModule.goMultiPv` 获取数据 |

---

### JS 数据文件嵌套单引号导致 `Unexpected identifier` [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 页面打开后点击按钮完全无反应，浏览器控制台报错 `Uncaught SyntaxError: Unexpected identifier 's'` |
| **原因** | `data/games.js` 中 `en: 'Rubinstein's Immortal'` 的单引号提前结束了字符串，`s Immortal'` 被解析为标识符 |
| **解决** | 将包含单引号的字符串改用双引号包裹：`en: "Rubinstein's Immortal"` |
| **预防** | 数据文件中包含英文撇号（如 `It's`、`Rubinstein's`）时，必须检查外层引号类型；建议此类字符串统一使用双引号 |

### 设置面板点击无反应（panel toggle 测试失败） [来源:blindfold-chess @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `test-settings-node.js` 中 panel toggle 测试突然失败，`settingsPanel` 未获得 `show` class |
| **原因** | `settings.js` 的 `init()` 每次被调用都会给 `settingsToggle` 新增一个 `click` 监听器，旧监听器从未移除。多次 init 后监听器数量累积为奇数/偶数，导致 `classList.toggle('show')` 的最终状态不可预期 |
| **解决** | 保存 panel toggle 和 document click 的匿名监听器引用到 `window._settingsPanelToggleHandler` / `window._settingsDocumentHandler`，下次 `init()` 时先 `removeEventListener` 再重新绑定 |
| **预防** | 需要动态解除绑定的事件监听器必须用命名函数（或保存引用），绝对不能用匿名函数直接 `addEventListener` |

### Stockfish 加载超时 / 引擎不启动 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 页面打开后引擎状态一直显示"Loading"，或走子后无响应 |
| **原因** | Stockfish 从 unpkg CDN 加载，国内网络可能超时；或本地使用 `file://` 协议打开导致 Web Worker 被浏览器拦截 |
| **解决** | 1. 确认使用 HTTP 服务器访问（如 `python -m http.server 8000`），不要用 `file://` 直接打开<br>2. 国内用户可尝试代理或切换网络环境<br>3. 检查控制台是否有 `SharedArrayBuffer` 或 CORS 相关报错 |

### GitHub Pages 国内打不开 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知限制 |
| **现象** | 部署地址 https://michaelgao-watcher.github.io/blindfold-chess/ 在国内无法访问 |
| **原因** | GitHub Pages 域名被 DNS 污染或墙 |
| **解决** | 使用代理/VPN 访问，或考虑国内镜像部署（如 Gitee Pages、Vercel 国内节点） |

---

## 开发/测试

### Node.js 测试运行时 chess.js 未定义 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | ReplayModule 的 `_applyMoves()` 只返回初始位置，导致 navigate/verifyMove 测试失败 |
| **原因** | chess.js 使用 `var Chess = function(){}` + `exports.Chess = Chess`，Node `require` 后需手动 `global.Chess = require('chess.js').Chess` 才能被 replay.js 识别 |
| **解决** | 浏览器环境中 `<script src>` 会自动暴露全局 `Chess`；Node 测试环境手动设置 `global.Chess = require('./chess.js').Chess` |

### CLI subAgent 并行超时 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（改为 IDE 串行） |
| **现象** | Kimi Code CLI 并行运行多个 subAgent 时，默认 15 分钟超时导致任务中断 |
| **原因** | 复杂模块（如 SettingsModule 51 个测试）在 15 分钟内无法完成代码+测试+调试 |
| **解决** | 改用 IDE 串行执行，或显式将 subAgent timeout 调高到 3600s |

### 设置面板一闪而过 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 点击齿轮图标后设置面板闪现立刻消失 |
| **原因** | 事件冒泡或 CSS transition 与 class 切换时序冲突 |
| **解决** | 检查 `settings-toggle` 的 click 事件是否阻止冒泡；确认 `.hidden` 和 `.active` 的切换使用 `requestAnimationFrame` 保证时序 |

---

## 运行时

### 旧代码与新模块冲突 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（逐步迁移中） |
| **现象** | `common.js` / `game.js` 中的旧全局函数与新 IIFE 模块并存，可能出现变量名冲突或重复初始化 |
| **原因** | 早期原型代码未模块化，新模块逐步替换中 |
| **解决** | 修改代码前检查同名全局变量；迁移完成后删除旧文件 |

### 引擎候选走法未集成 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复 |
| **现象** | EngineModule 支持 `goMultiPv`，但 BlindfoldModule UI 未展示 Top 3 候选走法 |
| **原因** | 功能已实现于引擎层，UI 层未接入 |
| **解决** | 在 BlindfoldModule 中添加候选走法面板 + 用户开关（默认关闭），调用 `EngineModule.goMultiPv` 获取数据 |

---

### JS 数据文件嵌套单引号导致 `Unexpected identifier` [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | 页面打开后点击按钮完全无反应，浏览器控制台报错 `Uncaught SyntaxError: Unexpected identifier 's'` |
| **原因** | `data/games.js` 中 `en: 'Rubinstein's Immortal'` 的单引号提前结束了字符串，`s Immortal'` 被解析为标识符 |
| **解决** | 将包含单引号的字符串改用双引号包裹：`en: "Rubinstein's Immortal"` |
| **预防** | 数据文件中包含英文撇号（如 `It's`、`Rubinstein's`）时，必须检查外层引号类型；建议此类字符串统一使用双引号 |

### 设置面板点击无反应（panel toggle 测试失败） [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复 |
| **现象** | `test-settings-node.js` 中 panel toggle 测试突然失败，`settingsPanel` 未获得 `show` class |
| **原因** | `settings.js` 的 `init()` 每次被调用都会给 `settingsToggle` 新增一个 `click` 监听器，旧监听器从未移除。多次 init 后监听器数量累积为奇数/偶数，导致 `classList.toggle('show')` 的最终状态不可预期 |
| **解决** | 保存 panel toggle 和 document click 的匿名监听器引用到 `window._settingsPanelToggleHandler` / `window._settingsDocumentHandler`，下次 `init()` 时先 `removeEventListener` 再重新绑定 |
| **预防** | 需要动态解除绑定的事件监听器必须用命名函数（或保存引用），绝对不能用匿名函数直接 `addEventListener` |

### mock DOM 中 `querySelector` / `querySelectorAll` 缺失 [来源:blindfold-chess @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（测试已适配） |
| **现象** | `boardStyleModal.querySelector('[data-style="blue"]')` 抛出 `TypeError: modal.querySelector is not a function` |
| **原因** | `test-settings-node.js` 的 `MockElement` 只实现了 `getElementById`，未实现 `querySelector`/`querySelectorAll` |
| **解决** | 测试代码中改用 `document.getElementById` + 遍历 `children` 的方式查找目标元素；或给 mock DOM 补充 `querySelectorAll` 实现 |

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*
---

### `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module` |
| **原因** | `windows` crate 的 Cargo.toml features 中未启用 `Win32_Storage_FileSystem` |
| **解决** | 在 `src-tauri/Cargo.toml` 的 `windows` features 中添加 `"Win32_Storage_FileSystem"` |

### `cargo check --lib` 报错：`FILETIME` 未定义 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module 'FILETIME'` |
| **原因** | 未导入 `windows::Win32::Foundation::FILETIME` |
| **解决** | `use windows::Win32::Foundation::FILETIME;` |

---

## 运行时错误

### `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失） [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **状态** | ✅ 已修复 |
| **现象** | 测试编译通过，但运行时弹窗或报错 `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` |
| **原因** | `tauri::AppHandle` 出现在 `async fn` 签名中，与 MinGW UCRT 生成不兼容的 PE 导入表 |
| **解决** | 将含 `AppHandle` 的 async command 函数拆分到 `commands/handlers.rs`，在 `#[cfg(not(test))]` 下条件编译；`lib.rs` 的 `run()` 同样条件编译。测试模式下不链接这些函数，从而绕过 loader 入口点缺失问题 |
| **验证** | `cargo test --lib` 103 测全绿 |

---

### 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示找不到 WebView2 Runtime |
| **原因** | 系统未安装 WebView2 Runtime（某些重装系统或企业阉割镜像） |
| **解决** | 从 NuGet 包 `Microsoft.Web.WebView2` 提取 `WebView2Loader.dll`，配置 `tauri.conf.json` 的 `bundle.resources` 自动打包到 `.exe` 同目录；同时程序启动时自动检测系统 EdgeCore 作为 WebView2 内核回退 |

### 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示 `由于找不到 WebView2Loader.dll，无法继续执行代码` |
| **原因** | 系统有 EdgeCore（Edge 浏览器内核）但缺少 WebView2 Runtime 的加载入口 DLL |
| **解决** | 将 `WebView2Loader.dll`（可从 NuGet 提取）与 `.exe` 一起分发。Tauri `bundle.resources` 会自动将其复制到输出目录 |

### `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32) [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `cargo tauri build` 报错，提示无法访问 `french-exit.exe` 或 `target/release/` 下的文件 |
| **原因** | `french-exit.exe` 仍在后台运行，锁定了构建产物 |
| **解决** | `taskkill //F //IM french-exit.exe` 强制结束进程后再构建 |

## 测试错误

### vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `Error: Failed to resolve import "@tauri-apps/api/fs" from "src/pages/ResultsPage.tsx"` |
| **原因** | Tauri v2 已移除 `@tauri-apps/api/fs` 模块，改为 `@tauri-apps/plugin-fs`（需单独安装） |
| **解决** | 1. 若只需要测试通过：在 `vite.config.ts` 中配置 alias，将 `@tauri-apps/api/fs` 指向本地 mock 文件<br>2. 若需要真功能：安装 `@tauri-apps/plugin-fs` 并修改所有导入路径 |
| **本项目做法** | `vite.config.ts` 中：<br>`"@tauri-apps/api/fs": path.resolve(__dirname, "./src/test/mocks/tauri-fs.ts")` |

### vitest 报错：`act is not a function` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `TypeError: act is not a function` |
| **原因** | 从 `vitest` 导入 `act`，但 `act` 实际来自 `@testing-library/react` |
| **解决** | `import { act } from "@testing-library/react"` 而非 `from "vitest"` |

### vitest 报错：React 警告 `Cannot update a component while rendering` [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `Warning: Cannot update a component (AppProvider) while rendering a different component (ResultsPage)` |
| **原因** | `dispatch()` 在 `setState` 的 updater 函数内部被调用，React 认为这是"渲染时更新" |
| **解决** | 将 `dispatch` 移出 updater 函数：先计算新状态，再分别调用 `setState` 和 `dispatch` |
| **本项目修复** | `ResultsPage.tsx` 中的 `toggleItem` 函数已修复 |

### checkbox 点击后状态不变化 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `fireEvent.click(checkbox)` 后 `checked` 状态未变 |
| **原因** | React controlled checkbox 的 `onChange` 可能不响应 `click` 事件 |
| **解决** | 用 `@testing-library/user-event` 的 `user.click(checkbox)` 替代 `fireEvent.click` |
| **安装** | `npm install -D @testing-library/user-event` |

---

## 环境问题

### 中文路径下编译失败 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | MinGW 链接器报错，无法生成 `.exe` |
| **原因** | 工作目录含中文（如 `E:/工作文件/...`），MinGW 工具链对 Unicode 路径支持差 |
| **解决** | 1. `rm -rf /c/french-exit && cp -r "/e/工作文件/vs-code/french-exit" /c/french-exit`<br>2. `cd /c/french-exit/src-tauri && cargo check --lib` |
| **注意** | `cargo check --lib` 和 `cargo test --no-run` 不需要链接，可在中文路径直接运行 |

---

### cargo tauri dev 在后台任务中崩溃 [来源:french-exit @2026-05-21] [来源:vibe-coding-project-sop @2026-05-22] [来源:french-exit @2026-05-29]

| | 内容 |
|---|---|
| **现象** | `cargo tauri dev` 启动后编译成功，但运行时报 `exit code: 0xc0000005, STATUS_ACCESS_VIOLATION`，随后 Segmentation fault |
| **原因** | Tauri 应用需要创建 WebView2 GUI 窗口，而 background task / SSH / 无头环境缺少 Windows 桌面会话和显示上下文 |
| **解决** | 1. 在本地交互式终端（PowerShell/CMD）中手动运行 `cargo tauri dev`<br>2. 或改用 `npm run dev` 仅启动 Vite 前端服务器，在浏览器中预览 UI（IPC 功能不可用） |
| **注意** | 此限制仅影响 GUI 启动方式，不影响 `cargo tauri build` 构建产物 |

---

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否有新的具体报错需要记入本文件。有则按模板追加；没有则跳过。
---
---

### Cookie 过期 / 401 认证失败 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（有自动刷新机制） |
| **现象** | 接口返回 401 或 302 重定向到登录页；`product.json` 返回 `success !== true` |
| **原因** | `jycmOpenApiCookie` 过期，需要刷新 |
| **解决** | 1. 读取 `auth/jycm.json` 中的 AK/SK<br>2. POST `openapi/employee/authToken.json` + Digest 认证获取新 requestCode<br>3. 回写 `auth/jycm.json`（只更新 `jycmOpenApiCookie`，保留 AK/SK）<br>4. 用新 Cookie 重试原请求。详见 `auth.md` 步骤 3 |

### Digest 刷新失败 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知处理方案 |
| **现象** | POST authToken.json 返回非 200 或网络超时 |
| **原因** | 网络问题 / 服务端限流 / AK/SK 被服务端拒绝 |
| **解决** | 1. 自动重试 1 次<br>2. 仍失败 → 明确报 `FAILURE：Digest 刷新失败` + 真实原因<br>3. 若 AK/SK 被服务端拒绝 → 报 `AK/SK 不可用，需删除凭证文件后重新提供 Token Key` |

### auth/jycm.json 缺失字段 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（有写入验证） |
| **现象** | 初始化后 Digest 刷新永远失败 |
| **原因** | 首次写入时只写了 `jycmOpenApiCookie`，漏写 `accessKey` / `secretKey` |
| **解决** | 1. 删除 `auth/jycm.json`<br>2. 重新引导用户提供 Token Key<br>3. 初始化时必须写入全部三字段，写入后立即读取验证非空 |

---

## 取数相关

### 日期区间多返回一天 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已修复（有强制约束） |
| **现象** | 请求 4/20-4/26，结果包含 4/27 的数据 |
| **原因** | `endDate` 使用了 `T23:59:59.999+08:00` 或 `Z`（UTC）后缀 |
| **解决** | 强制使用 `T00:00:00+08:00` 格式。详见 `SKILL.md` → 强约束 → 日期区间 |

### getAllShopList 返回空数组 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（业务侧问题） |
| **现象** | 绑店检查显示「尚未绑任何店铺」 |
| **原因** | 用户未完成店铺绑定流程 |
| **解决** | 引导用户按 [绑店操作指南](https://alidocs.dingtalk.com/i/nodes/qnYMoO1rWxrkmoj2IznlmLDmJ47Z3je9) 完成绑定 |

### createAndDownload 返回失败 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（需具体 case 分析） |
| **现象** | 步骤 G 返回 `code !== 0` 或 `success !== true` |
| **原因** | 参数错误 / 店铺无数据 / 指标不存在 / 日期范围超限 |
| **解决** | 1. 检查 `shopIds` 是否为 `List<String>`（非数字数组）<br>2. 检查 `channelName` / `dataPlatform` / `dataType` / `dataDimension` 是否匹配<br>3. 检查日期范围是否超限（如超过 90 天）<br>4. 检查 `indicators` 是否为步骤 F 返回的有效 key |

---

## 报告相关

### openpyxl 未安装 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 临时绕过 |
| **现象** | 分析 Excel 时报 `ModuleNotFoundError: No module named 'openpyxl'` |
| **原因** | 依赖未声明或环境未安装 |
| **解决** | `pip install openpyxl`。建议后续补充 `requirements.txt` |

### 钉钉推送失败 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（需用户侧配置） |
| **现象** | `dingtalk_send_markdown.py` 返回 HTTP 400/403 |
| **原因** | Webhook token 错误 / 机器人被禁 / 内容含敏感词 |
| **解决** | 1. 检查 `DINGTALK_WEBHOOK` 环境变量是否正确<br>2. 检查钉钉机器人是否被禁言<br>3. 检查 Markdown 内容是否含敏感关键词<br>4. 超长内容（>18000 字符）会自动截断，检查截断后是否格式破坏 |

---

## 环境相关

### Windows Git Bash LF/CRLF 警告 [来源:qianniu_business_analytics @2026-05-29]

| | 内容 |
|---|---|
| **状态** | 已知未修复（不影响功能） |
| **现象** | `git add` 时提示 `LF will be replaced by CRLF` |
| **原因** | Windows 默认 `core.autocrlf=true` |
| **解决** | 不影响功能；跨平台协作前建议统一配置 `git config --global core.autocrlf true` |

---

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否有新的具体报错需要记入本文件。有则按模板追加；没有则跳过。
---
