# French Exit — Agent 启动指令

> 本文件供 AI 开发助手读取。接手本项目时，**必须先读完下面列出的上下文文件**，再开始任何代码操作。

---

## 1. 项目定位

French Exit 是一款面向**非技术背景职场人**的 Windows 离职清理工具。

- **形态**：绿色免安装单文件，双击即运行，用完即走
- **技术栈**：Tauri（Rust backend + WebView2 frontend），完全离线
- **UI 风格**：Apple Design，简洁圆角毛玻璃，跟随系统深色/浅色模式
- **目标用户**：零编程背景的普通白领，只会基础电脑操作

---

## 2. 必读上下文（按顺序，5 分钟）

1. `AGENTS.md` — 本文件（硬规则）
2. `status.md` — 当前进度、待办清单
3. `session-log.md` — 前几轮怎么走到这里的
4. `docs/high-Level Design.md` — 概要设计（如需改架构或接口）

**禁止**：在未阅读 `status.md` 前直接写代码。

> 各文件详细职责见 `status.md` 开头的文档体系说明。

---

## 3. 核心约束（硬规则，不可违反）

以下规则优先级高于任何技术便利。如果代码逻辑和规则冲突，**改代码，不改规则**。

| 规则 ID | 规则内容 | 违反后果 |
|---------|---------|---------|
| RULE-01 | 所有 `Action::Delete` 必须在用户提交最终决策清单后执行 | 零容忍误删 |
| RULE-02 | 来源于 `scanner-env`（环境变量）的 `TraceItem`，默认决策状态为**未选中** | 避免误删共享 TOKEN |
| RULE-03 | 微信相关 `TraceItem` 的 `suggested_action = DeleteOrPack`，前端默认选中 | 用户已确认微信直接建议处理 |
| RULE-04 | 程序退出时必须调用 `TempStore::self_destruct()`，但 HTML 报告路径**必须排除** | 庆祝页是用户唯一保留物 |
| RULE-05 | 默认启用 CPU ≤30% 限制（`ResourceConfig::unlimited = false`） | 保证用户办公不卡顿 |
| RULE-06 | 打包输出文件名固定为 `French-exit.zip` | 用户已确认 |
| RULE-07 | 扫描结果不按"工作/私人"区分，按类型列出 | 用户确认无法区分 |
| RULE-08 | 需要询问的文件范围**仅限 Desktop 和 Downloads** | 其他目录不逐条展示 |
| RULE-09 | HTML 庆祝页保存位置：有打包则放 zip 同目录，无打包则放桌面 | 用户已确认 |
| RULE-10 | 所有注册表/系统日志的推断结果必须标注 `inferred: true` 和风险提示 | 用户看不懂，需要程序兜底 |

---

## 4. 模块速查表

开发前先确认你要改的是哪个模块，再去看 `high-Level Design.md` 的详细接口。

| ID | 模块名 | 一句话职责 | 技术要点 |
|----|--------|-----------|---------|
| M01 | frontend | 渲染 5 个页面，管理状态 | TypeScript + React（阶段三确认） |
| M02 | commands | Tauri IPC 入口，参数校验 | `#[tauri::command]` |
| M03 | orchestrator | 状态机驱动全流程 | FSM: Idle→Scanning→Scanned→Confirming→Executing→Completed |
| M04 | scanner-registry | 管理所有 Scanner，按类别调度 | `Vec<Box<dyn Scanner>>` |
| M05 | scanner-fs | 扫 Desktop、Downloads、微信记录目录 | 过滤入职日期后文件，微信整目录标记 |
| M06 | scanner-browser | 扫浏览器历史/Cookie/密码/缓存 | 检测 Chrome/Edge/Firefox |
| M07 | scanner-chat | 扫微信/QQ/钉钉/飞书/企业微信 | 微信 → `suggested_action: DeleteOrPack` |
| M08 | scanner-registry-sys | 扫注册表，启发式推断个人信息 | `HKEY_CURRENT_USER`，标注 `inferred` |
| M09 | scanner-system | 扫系统日志/最近文档/Temp/搜索索引 | 时间过滤 |
| M10 | scanner-devtools | 扫 Git/SSH/IDE/GitHub CLI 配置 | 区分"安全清"和"有风险" |
| M11 | scanner-env | 扫用户级环境变量 | 默认不勾选，标注风险 |
| M12 | executor-delete | 执行删除，调用安全擦除 | 分发到 secure-erase |
| M13 | executor-pack | 打包为 French-exit.zip | 加密文件回调确认 |
| M14 | executor-preserve | 执行保留（无操作，仅记录） | |
| M15 | secure-erase | DoD 标准安全擦除（3 次覆写） | 覆写 → 重命名 → 删除 |
| M16 | reporter | 生成 HTML 庆祝页 + 调用浏览器打开 | HTML 不在临时目录 |
| M17 | resource-ctl | 限制 CPU ≤30%，可手动解除 | Windows Job Object |
| M18 | temp-store | 临时数据管理 + 自毁 | JSON Lines 分批落盘，`Drop` 时清理 |

---

## 5. 关键数据流（开发前必须理解）

```
扫描阶段：
  Frontend → Commands → Orchestrator → ScannerRegistry → [M05-M11 并行扫描]
                                                           ↓
                                                        TempStore (分批落盘)

确认阶段：
  Frontend ← Commands ← Orchestrator ← TempStore (分页读取)
  Frontend → Commands → Orchestrator (提交 Decisions)

执行阶段：
  Orchestrator → [M12 Delete → M15 SecureErase]
               → [M13 Pack → French-exit.zip]
               → [M14 Preserve (记录)]
               → M16 Reporter (HTML + 打开浏览器)
               → M18 TempStore::self_destruct()
```

---

## 6. 常见陷阱（前序 AI 踩过的坑）

1. **不要在前端直接操作文件系统** — 必须通过 Tauri Commands 调用 Rust 后端。
2. **不要在扫描阶段就删除任何东西** — 扫描只收集信息，执行必须等用户最终确认。
3. **不要把 HTML 庆祝页放到 TempStore 目录** — 它会被自毁清理掉。
4. **不要假设用户有管理员权限** — 注册表/系统级操作失败时要优雅降级，标记风险而非崩溃。
5. **不要在 Rust 里用 `std::fs::remove_file` 代替安全擦除** — 普通删除可恢复，必须用 `M15`。

---

## 7. 更新记录

| 日期 | 更新内容 | 更新人 |
|------|---------|--------|
| 2026-05-18 | 初始版本，基于阶段二设计文档 | AI Assistant |
