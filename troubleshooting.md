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
| **状态** | ⚠️ 已知未修复 |
| **现象** | 测试编译通过，但运行时弹窗或报错 `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)` |
| **原因** | `x86_64-pc-windows-gnu` toolchain 与 Windows 10 (19041) 存在已知兼容性 issue，非单纯 DLL 缺失 |
| **解决** | 1. 更新 Windows Update / 安装最新 VC++ 2015-2022 Redistributable<br>2. 或升级 MinGW-w64 到最新版<br>3. 或切换 Rust 默认 toolchain 为 `x86_64-pc-windows-msvc`（需配置 MSVC linker） |
| **验证** | `cargo test --no-run` 可通过（仅编译），`cargo test --lib` 需要运行时才报错 |

---

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

*新增条目时复制上方模板，按"错误关键词"作为标题，便于快速搜索。*

---

## 存档提示

**用户说「存储」时**，AI 应回顾本轮会话内容，评估是否有新的具体报错需要记入本文件。有则按模板追加；没有则跳过。
