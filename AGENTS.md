# French Exit — Agent 启动指令

> 本文件供 AI 开发助手读取。接手本项目时，**必须先读完下面列出的上下文文件**，再开始任何代码操作。

---

## 0. 文档体系说明

> 本项目按 `vibe-coding-project-sop` 骨架搭建。**项目已搭建完成，阶段产出已全量生成**，后续进入修改/维护阶段。以下基础设施文件均已存在且持续维护：

### 基础设施层（通用机制）

| 文件 | 职责 |
|------|------|
| `AGENTS.md` | 项目硬规则 + 模块速查（本文件） |
| `docs/vibe-coding-sop.md` | 五阶段工作流参考 |
| `status.md` | 当前进度、待办清单、环境备忘 |
| `session-log.md` | 会话历史记录 |
| `decisions.md` | 关键设计决策（ADR） |
| `troubleshooting.md` | 问题索引与急救手册 |
| `lessons-learned.md` | 跨项目经验沉淀 |

### 阶段产出层（已全量生成）

| 阶段 | 产出文件 |
|------|---------|
| 阶段一 | `docs/proposal.md` |
| 阶段二 | `docs/high-Level Design.md`、`docs/brief.md` |
| 阶段三 | `docs/tasks/`（任务拆分） |
| 阶段四 | `prompt.md` |
| 阶段五 | `src/`、`e2e/`、`src-tauri/src/`（源码 + 测试） |

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
| RULE-08 | 文件扫描范围为**全盘扫描**（所有可用盘符），系统目录受 `is_system_path` 保护自动排除 | 全盘扫描确保不遗漏个人数据 |
| RULE-09 | HTML 庆祝页保存位置：有打包则放 zip 同目录，无打包则放桌面 | 用户已确认 |
| RULE-10 | 所有注册表/系统日志的推断结果必须标注 `inferred: true` 和风险提示 | 用户看不懂，需要程序兜底 |
| **ARCHIVE-01** | **用户说「存档」时，执行标准存档流程，不要提前结束会话** | 文档与 Git 状态不一致，接力丢失上下文 |
| **ARCHIVE-02** | **「存档」触发后必须先输出确认清单，等待用户二次确认** | 误触导致非预期提交 |

---

### 前端设计约束（硬规则，阶段五不可违反）

> 源自 Anthropic frontend-design skill 经母库语境适配。
> 本项目 UI 遵循「**精密仪器风（Precision Instrument）**」美学方向。

| 规则 ID | 规则内容 | 违反后果 |
|---------|---------|---------|
| **UI-01** | **禁止 generic 配色**：禁用 shadcn 默认 `blue-600` / `red-600` / `gray` 系。主色板固定为：深炭黑背景 `#0a0a0f` + 琥珀色状态指示 `#f59e0b` + 冷白文字 `#e8e8ec`。危险红 `#ef4444` 仅用于不可逆操作确认。 | 项目失去视觉识别度，沦为千篇一律的 AI 默认输出 |
| **UI-02** | **禁止标准居中卡片**：每页布局必须服务于该页核心任务。Welcome 用全屏沉浸、Results 用仪表板网格、Confirm 用分屏对比——不可所有页面同一套 `flex flex-col items-center` 模板。 | 用户审美疲劳，无法通过布局区分功能页面 |
| **UI-03** | **必须使用自定义字体配对**：标题/数据用 JetBrains Mono（等宽字体营造技术感），正文用 Geist（或系统无衬线替代）。禁用 Arial/Inter/Roboto 作为默认字体。 | 缺乏技术设备的精密气质，停留在普通网页感 |
| **UI-04** | **动效必须有层次**：页面切换用淡入+位移（200ms ease-out）；状态变化用琥珀色脉冲 glow；按钮用按压下沉反馈。禁止仅有 `active:scale-95` 的单一动效。 | 界面死板，无法传达"仪器正在响应"的反馈感 |
| **UI-05** | **组件必须有场景特征**：按钮是"仪器开关"（细边框 + 内阴影按压），不是 generic 圆角填充；卡片是"仪表板模块"（细线分隔 + 状态指示灯），不是标准 shadcn Card。 | 组件缺乏项目专属印记，用户无法记住产品 |

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
      理由：[一句话推荐理由，基于依赖关系/技术可行性/阻塞解除情况]
   2. [status.md P1 待办第二项]
      理由：[适合场景，如"如果希望先验证 UI 交互，可优先此项"]
   3. [status.md P2 待办第一项]
      理由：[适合场景，如"P1 完成后可并行启动的后端任务"]
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
- **推荐理由必须具体**：不能写"因为这是高优先级"，必须基于项目上下文（如"此接口是登录流程的前置条件"）
- **推荐项只能有 1 个**：不要标注多个推荐，避免用户困惑

---

## 4. 模块速查表

开发前先确认您要改的是哪个模块，再去看 `high-Level Design.md` 的详细接口。

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

## 3.7 母库经验指令（「母库经验」）

> 本章节为**其他项目专用**。由 `pull.py` 自动拉取母库经验。
> 其他项目从母库 `vibe-coding-project-sop` 获取已沉淀的跨项目经验。
> 母库本身不需要此指令，母库使用「同步知识」（聚合模式）。

**触发词**：`拉取母库`、`母库经验`、`更新经验`（去除标点后精确匹配任一）

**防误触**：
- 消息精确匹配上述任一触发词 → 执行母库经验同步流程
- 消息包含触发词但还有其他内容 → 视为正常对话，不触发

**前置条件**：
- 项目根目录存在 `scripts/pull.py`（或 `scripts/sync-knowledge.py`）
- 项目根目录存在 `config/github-sync.json` 且 `syncFrom` 已填写母库仓库名（如 `vibe-coding-project-sop`）

**确认流程**：
1. 读取 `config/github-sync.json` 中的 `syncFrom` 字段
2. 输出将要同步的母库仓库名和当前项目路径
3. 等待用户二次确认

**标准动作序列**：
1. 检查 `scripts/pull.py` 是否存在
   - 如存在 → 运行 `python scripts/pull.py`
   - 如不存在 → 下载 `https://raw.githubusercontent.com/MichaelGao1999/vibe-coding-project-sop/master/scripts/pull.py`，然后运行
2. 读取脚本输出，汇报同步结果：
   - 母库仓库名
   - 拉取到的文件数
   - 新增条目数 / 全部已存在
3. 如有新增内容，提示用户查看 `decisions.md`、`lessons-learned.md`、`troubleshooting.md`
4. 如脚本报错，按 troubleshooting.md 格式记录问题

**与母库「同步知识」的区别**：
- 母库「同步知识」是**聚合模式**：把多个仓库的经验汇总到当前项目（用于维护母库）
- 本指令是**分发模式**：从指定母库获取经验到当前项目（用于消费经验）
- 两者触发词不同，使用场景不同，不可混淆

**反哺母库**：
如其他项目在开发过程中产生了新的可复用经验，可手动提交到母库仓库，供所有项目共享。

---

## 8. 更新记录

| 日期 | 更新内容 | 更新人 |
|------|---------|--------|
| 2026-05-18 | 初始版本，基于阶段二设计文档 | AI Assistant |
