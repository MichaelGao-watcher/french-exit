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
2. `status.md` — 当前进度、待办清单、环境备忘（版本/构建/测试命令）
3. `session-log.md` — 前几轮怎么走到这里的
4. `docs/high-Level Design.md` — 概要设计（如需改架构或接口）

> **新项目启动时额外阅读**：`docs/vibe-coding-sop.md` — 五阶段 Vibe Coding 工作流 SOP（需求→设计→任务→Prompt→开发）。

**禁止**：在未阅读 `status.md` 前直接写代码。

> 详细文档体系说明见 `README.md`（各文件职责边界、接力流程）。

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
| **ARCHIVE-01** | **用户说「存档」时，执行标准存档流程，不要提前结束会话** | 文档与 Git 状态不一致，接力丢失上下文 |
| **ARCHIVE-02** | **「存档」触发后必须先输出确认清单，等待用户二次确认** | 误触导致非预期提交 |

---

## 3.5 存档指令（「存档」）

**触发词**：`存档`（去除标点后精确等于这两个字）

**防误触**：
- 消息精确匹配「存档」→ 进入存档确认流程
- 消息包含「存档」但还有其他内容（如「存档数据」）→ 视为正常对话，不触发

**确认流程**：
```
即将执行存档（含 Git 全量提交）：
[文档] 更新 status.md
[文档] 追加 session-log.md
[文档] 评估并追加 troubleshooting.md（如本轮有报错）
[文档] 评估并追加 lessons-learned.md（如本轮有可复用经验）
[文档] 评估并追加 decisions.md（如本轮有关键决策）
[Git]  git add -A → commit → push
确认执行？（y / yes / 确认）
```

**标准动作序列（用户确认后执行）**：
1. 回顾本轮内容，生成 session-log 草稿（作为原材料）
2. 【强制】更新 `status.md`
3. 【评估】有具体报错？→ 追加 `troubleshooting.md`
4. 【评估】有可复用经验？→ 追加 `lessons-learned.md`
5. 【评估】有关键决策？→ 追加 `decisions.md`
6. 【强制】定稿并追加 `session-log.md`
7. 【强制】Git 全量提交：`git add -A` → `git commit -m "[session] 摘要"` → `git push`
8. 汇报完成，提示可以关闭会话

**Git 错误处理**：
- 无 `.git` 目录 → 跳过 Git 步骤，仅更新文档
- 无变更 → 跳过 commit，直接提示完成
- push 失败 → 报错暂停，提示用户手动处理

---

## 3.6 恢复指令（「恢复」）

**触发词**：`恢复`（去除标点后精确等于这两个字）

**防误触**：
- 消息精确匹配「恢复」→ 执行恢复流程
- 消息包含「恢复」但还有其他内容（如「恢复默认设置」）→ 视为正常对话，不触发

**核心原则**：
> **恢复摘要以 `status.md` 为主，`session-log.md` 为辅。**
> `status.md` 回答"现在在哪、下一步去哪"。`session-log.md` 只用于验证一致性，或补充 status.md 未写的细节。
> **不要复述上轮历史。**

**标准动作序列**：
1. AI 已自动读取 AGENTS.md（会话启动时必读）
2. **读取 `status.md`（主数据源）**：
   - 当前阶段、进度百分比
   - 待办列表（按优先级）
   - 阻塞项/进行中项
3. **读取 `session-log.md` 最后一条（辅数据源）**：
   - 只取「遗留问题/下轮开始点」一句话
   - 用于和 status.md 交叉验证"停在哪里"
4. **内容有效性判断**：
   - `status.md` 关键字段是否为占位符？→ 视为未初始化
   - `session-log.md` 是否只有模板无实际记录？→ 标记为"无历史"
5. **AI 综合分析，给出建议**：
   - 结合 `status.md` 的待办优先级、`session-log` 的遗留问题/下轮开始点
   - 判断依赖关系（如 P2 功能是否依赖 P1 阻塞项）
   - 评估技术可行性（如外部接口是否已就绪）
   - 选出最优路径作为「推荐项」，附一句话推荐理由
6. 向用户汇报恢复摘要
7. 等待用户给出下一步指令

**汇报格式**：

**A. status.md 已初始化且有历史记录**：
```
【恢复摘要】
📍 当前阶段：[status.md 阶段名] [进度]% 完成
   停在这里：[status.md 待办中的阻塞/进行中项，或 session-log 的下轮开始点]

🔧 可以做什么：
   1. [status.md P1 待办第一项]
   2. [status.md P1 待办第二项]
   3. [status.md P2 待办第一项]
   4. 其他 — 请直接告诉我新指令
```

**B. 无历史记录（status.md 和 session-log 均为模板）**：
```
【恢复摘要】
📍 当前项目尚未启动，暂无进度记录。

🔧 可以做什么：
   1. 从需求/设计阶段开始
   2. 先完善 status.md 初始化项目信息
   3. 其他 — 请直接告诉我新指令
```

**C. status.md 未初始化，但 session-log 有记录**：
```
【恢复摘要】
📍 当前阶段：未设置（status.md 尚未初始化）
   停在这里：[session-log 最后一条的下轮开始点]

🔧 可以做什么：
   1. 先完善 status.md 建立项目看板
   2. 继续按 session-log 的下轮开始点执行
   3. 其他 — 请直接告诉我新指令
```

**字段填充规则**：
| 字段 | 数据来源 | 占位符处理 |
|------|---------|-----------|
| 当前阶段 | `status.md` | `[阶段名]` → **未设置** |
| 进度 | `status.md` | `[百分比]` → **0** |
| 停在这里 | `status.md` 优先，session-log 补充 | 未填写 → **无明确阻塞** |
| 选项 A/B/C | `status.md` 待办列表 | `[待办事项]` → **暂无待办** |

**约束**：
- 只读，无副作用，不需要二次确认
- **严禁输出空模板**：`[xxx]` 占位符必须替换为"未设置/暂无"，禁止原样输出
- **不要复述上轮历史**：不输出"上轮完成了xxx"，只回答当前进度、停在哪、能做什么
- 选项不超过 4 个，优先取 status.md 中高优先级待办
- **严禁越界编辑**：只操作当前工作区内的文件，禁止修改其他项目/工作区的文件

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

## 5. 环境备忘索引

> 编译命令、PATH、已知限制见 `status.md` → 环境备忘

---

## 6. 关键数据流（开发前必须理解）

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

## 7. 常见陷阱（前序 AI 踩过的坑）

1. **不要在前端直接操作文件系统** — 必须通过 Tauri Commands 调用 Rust 后端。
2. **不要在扫描阶段就删除任何东西** — 扫描只收集信息，执行必须等用户最终确认。
3. **不要把 HTML 庆祝页放到 TempStore 目录** — 它会被自毁清理掉。
4. **不要假设用户有管理员权限** — 注册表/系统级操作失败时要优雅降级，标记风险而非崩溃。
5. **不要在 Rust 里用 `std::fs::remove_file` 代替安全擦除** — 普通删除可恢复，必须用 `M15`。

---

## 8. 更新记录

| 日期 | 更新内容 | 更新人 |
|------|---------|--------|
| 2026-05-18 | 初始版本，基于阶段二设计文档 | AI Assistant |
