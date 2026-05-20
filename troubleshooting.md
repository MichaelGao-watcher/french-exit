# French Exit — 急救手册

> 按错误关键词索引，只给"现象 → 原因 → 解决"，不给背景。
>
> 遇到问题时先搜此文件，再搜 `lessons-learned.md`（背景知识），最后问搜索引擎。

---

## 编译错误

### `cargo check --lib` 报错：`GetDiskFreeSpaceExW` 未定义

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module` |
| **原因** | `windows` crate 的 Cargo.toml features 中未启用 `Win32_Storage_FileSystem` |
| **解决** | 在 `src-tauri/Cargo.toml` 的 `windows` features 中添加 `"Win32_Storage_FileSystem"` |

### `cargo check --lib` 报错：`FILETIME` 未定义

| | 内容 |
|---|---|
| **现象** | `error[E0433]: failed to resolve: use of undeclared crate or module 'FILETIME'` |
| **原因** | 未导入 `windows::Win32::Foundation::FILETIME` |
| **解决** | `use windows::Win32::Foundation::FILETIME;` |

---

## 运行时错误

### `cargo test --lib` 报错 `0xc0000139`（UCRT DLL 缺失）

| | 内容 |
|---|---|
| **状态** | ✅ 已修复 |
| **现象** | 测试编译通过，但运行时弹窗或报错 `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` |
| **原因** | `tauri::AppHandle` 出现在 `async fn` 签名中，与 MinGW UCRT 生成不兼容的 PE 导入表 |
| **解决** | 将含 `AppHandle` 的 async command 函数拆分到 `commands/handlers.rs`，在 `#[cfg(not(test))]` 下条件编译；`lib.rs` 的 `run()` 同样条件编译。测试模式下不链接这些函数，从而绕过 loader 入口点缺失问题 |
| **验证** | `cargo test --lib` 103 测全绿 |

---

### 运行 `french-exit.exe` 报错：`Could not find the WebView2 Runtime`

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示找不到 WebView2 Runtime |
| **原因** | 系统未安装 WebView2 Runtime（某些重装系统或企业阉割镜像） |
| **解决** | 从 NuGet 包 `Microsoft.Web.WebView2` 提取 `WebView2Loader.dll`，配置 `tauri.conf.json` 的 `bundle.resources` 自动打包到 `.exe` 同目录；同时程序启动时自动检测系统 EdgeCore 作为 WebView2 内核回退 |

### 运行 `french-exit.exe` 报错：`找不到 WebView2Loader.dll`

| | 内容 |
|---|---|
| **现象** | 双击 `.exe` 弹窗提示 `由于找不到 WebView2Loader.dll，无法继续执行代码` |
| **原因** | 系统有 EdgeCore（Edge 浏览器内核）但缺少 WebView2 Runtime 的加载入口 DLL |
| **解决** | 将 `WebView2Loader.dll`（可从 NuGet 提取）与 `.exe` 一起分发。Tauri `bundle.resources` 会自动将其复制到输出目录 |

### `cargo tauri build` 失败：`另一个程序正在使用此文件` (os error 32)

| | 内容 |
|---|---|
| **现象** | `cargo tauri build` 报错，提示无法访问 `french-exit.exe` 或 `target/release/` 下的文件 |
| **原因** | `french-exit.exe` 仍在后台运行，锁定了构建产物 |
| **解决** | `taskkill //F //IM french-exit.exe` 强制结束进程后再构建 |

## 测试错误

### vitest 报错：`Failed to resolve import "@tauri-apps/api/fs"`

| | 内容 |
|---|---|
| **现象** | `Error: Failed to resolve import "@tauri-apps/api/fs" from "src/pages/ResultsPage.tsx"` |
| **原因** | Tauri v2 已移除 `@tauri-apps/api/fs` 模块，改为 `@tauri-apps/plugin-fs`（需单独安装） |
| **解决** | 1. 若只需要测试通过：在 `vite.config.ts` 中配置 alias，将 `@tauri-apps/api/fs` 指向本地 mock 文件<br>2. 若需要真功能：安装 `@tauri-apps/plugin-fs` 并修改所有导入路径 |
| **本项目做法** | `vite.config.ts` 中：<br>`"@tauri-apps/api/fs": path.resolve(__dirname, "./src/test/mocks/tauri-fs.ts")` |

### vitest 报错：`act is not a function`

| | 内容 |
|---|---|
| **现象** | `TypeError: act is not a function` |
| **原因** | 从 `vitest` 导入 `act`，但 `act` 实际来自 `@testing-library/react` |
| **解决** | `import { act } from "@testing-library/react"` 而非 `from "vitest"` |

### vitest 报错：React 警告 `Cannot update a component while rendering`

| | 内容 |
|---|---|
| **现象** | `Warning: Cannot update a component (AppProvider) while rendering a different component (ResultsPage)` |
| **原因** | `dispatch()` 在 `setState` 的 updater 函数内部被调用，React 认为这是"渲染时更新" |
| **解决** | 将 `dispatch` 移出 updater 函数：先计算新状态，再分别调用 `setState` 和 `dispatch` |
| **本项目修复** | `ResultsPage.tsx` 中的 `toggleItem` 函数已修复 |

### checkbox 点击后状态不变化

| | 内容 |
|---|---|
| **现象** | `fireEvent.click(checkbox)` 后 `checked` 状态未变 |
| **原因** | React controlled checkbox 的 `onChange` 可能不响应 `click` 事件 |
| **解决** | 用 `@testing-library/user-event` 的 `user.click(checkbox)` 替代 `fireEvent.click` |
| **安装** | `npm install -D @testing-library/user-event` |

---

## 环境问题

### 中文路径下编译失败

| | 内容 |
|---|---|
| **现象** | MinGW 链接器报错，无法生成 `.exe` |
| **原因** | 工作目录含中文（如 `E:/工作文件/...`），MinGW 工具链对 Unicode 路径支持差 |
| **解决** | 1. `rm -rf /c/french-exit && cp -r "/e/工作文件/vs-code/french-exit" /c/french-exit`<br>2. `cd /c/french-exit/src-tauri && cargo check --lib` |
| **注意** | `cargo check --lib` 和 `cargo test --no-run` 不需要链接，可在中文路径直接运行 |

---

### cargo tauri dev 在后台任务中崩溃

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
