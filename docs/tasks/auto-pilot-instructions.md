# French Exit — 全自动推进指令（无需用户参与）

> 本指令供 AI 助手在**无用户监督**的情况下自主推进剩余工作。
> 
> 适用场景：用户暂时离开，AI 自动完成测试补完 + 细节打磨。

---

## 0. 核心原则

| 原则 | 说明 |
|------|------|
| **只补不推翻** | 绝不重写已有代码，只补充缺失部分 |
| **最小修改** | 每轮只修改/新增一个文件，降低冲突风险 |
| **错误即停** | 遇到无法自洽的问题，记录 TODO 而非强行推进 |
| **文档同步** | 每轮结束更新 `task-progress.md` |

---

## 1. 全自动推进顺序（6轮）

### 第1轮：前端预览弹窗 + 搜索过滤
**目标**：提升 ResultsPage 用户体验

**交付物**：
- 修改 `src/pages/ResultsPage.tsx`：
  - 每条 TraceItem 右侧添加"预览"按钮
  - 点击弹出模态框（Apple Design：毛玻璃背景 + 圆角 + 淡入动画）
  - 文本文件：读取 path，展示前 4KB 内容（调用 `fs.readTextFile` 或后端 command）
  - 图片文件：展示图片（`<img src={convertFileSrc(path)} />`，Tauri 的 convertFileSrc）
  - 不支持：显示"暂不支持预览此类型"
- 修改 `src/pages/ResultsPage.tsx`：
  - 顶部添加搜索输入框，按 `item.name` 和 `item.path` 过滤
  - 搜索实时过滤，不调用后端（前端内存过滤已加载的数据）

**约束**：
- 不安装新 npm 包
- 使用现有 Tailwind 样式

---

### 第2轮：前端进度实时推送（Tauri Event）
**目标**：替代 ScanPage 的轮询机制

**交付物**：
- 修改 `src-tauri/src/orchestrator/mod.rs`：
  - `start_scan()` 中，将 `progress_callback` 改为通过 `tauri::Emitter` 发送事件到前端
  - 引入 `tauri::Manager` 或 `tauri::AppHandle`，在 scan task 中 emit `ProgressEvent`
- 修改 `src-tauri/src/lib.rs`：
  - 确保 AppHandle 在初始化时可用（可能需要在 AppState 中保存 AppHandle）
- 修改 `src/pages/ScanPage.tsx`：
  - 添加 `listen('scan_progress', handler)` 替代轮询
  - 保留轮询作为 fallback

**约束**：
- 如果不确定 Tauri v2 Event API 的精确用法，记录 TODO 而不是写可能错误的代码
- `tauri::Emitter` 在 Tauri v2 中是 `tauri::Manager::emit()` 或 `@tauri-apps/api/event` 的 `listen`

---

### 第3轮：Rust 测试补完（Batch 1: M05-M08）
**目标**：为文件系统、浏览器、聊天、注册表扫描器补测试

**交付物**：
- `src/scanner/fs.rs` 末尾添加 `#[cfg(test)]` 模块：
  - test_fs_scanner_trait_compiles：验证 Scanner trait 实现
  - test_fs_modified_date_filter：构造临时文件，验证日期过滤
  - test_fs_excludes_system_paths：验证系统目录排除
- `src/scanner/browser.rs` 末尾添加测试：
  - test_browser_scanner_trait_compiles
  - test_chrome_time_conversion：验证 Chrome 时间戳转换正确
  - test_firefox_time_conversion：验证 Firefox 时间戳转换正确
- `src/scanner/chat.rs` 末尾添加测试：
  - test_chat_scanner_trait_compiles
  - test_chat_detects_qq_path：验证 QQ 路径检测
- `src/scanner/registry_sys.rs` 末尾添加测试：
  - test_registry_scanner_trait_compiles
  - test_registry_inferred_flag：验证所有返回项 inferred=true

**约束**：
- 测试不依赖外部文件系统状态（用 tempfile crate 构造临时环境）
- 由于 cargo 不可用，无法运行验证，但必须保证代码结构正确

---

### 第4轮：Rust 测试补完（Batch 2: M09-M13 + M16 + M03）
**目标**：完成剩余模块的测试

**交付物**：
- `src/scanner/system.rs`：测试 trait 编译 + Temp 目录过滤 + 最近文档检测
- `src/scanner/devtools.rs`：测试 trait 编译 + SSH 私钥识别 + Git 配置检测
- `src/scanner/env.rs`：测试 trait 编译 + TOKEN 识别 + PATH 拆分
- `src/executor/pack.rs`：测试去重逻辑 + zip 路径生成
- `src/executor/delete.rs`：测试 DeleteExecutor 对不同 category 的处理
- `src/reporter/mod.rs`：测试 format_bytes + format_number + html_escape
- `src/orchestrator/mod.rs`：测试状态转换合法性 + 非法转换被拒绝

**约束**：
- 同上，代码结构正确即可

---

### 第5轮：M02 Commands 测试 + 集成框架
**目标**：补完 IPC 层测试

**交付物**：
- `src/commands/mod.rs` 末尾添加测试：
  - test_start_scan_date_validation：验证非法日期返回 INVALID_DATE
  - test_submit_decisions_dedup：验证重复 item_id 被拒绝
  - test_resource_config_validation：验证非法 CPU 百分比被拒绝
- 新增 `src/commands/tests.rs`（可选）：mock AppState 验证 command 调用

---

### 第6轮：审查 + 文档更新 + 最终汇总
**目标**：确保项目状态清晰，无遗留问题

**交付物**：
1. 更新 `docs/tasks/task-progress.md`：
   - 将所有"核心代码完成，测试待补"更新为"已完成"
   - 更新百分比
2. 审查所有 `TODO` 注释：
   - 如果 TODO 是 P2/P3 级别，保留并标注优先级
   - 如果 TODO 是 P1 但未完成，记录到 `docs/tasks/remaining-p1.md`
3. 生成 `docs/tasks/final-report.md`：
   - 模块完成度总览
   - 代码行数统计
   - 测试覆盖率估算
   - 已知问题清单

---

## 2. 自主决策规则

### 遇到以下情况时的处理策略

| 情况 | 策略 |
|------|------|
| 不确定 API 用法 | 查 `docs/high-Level Design.md` → 查已有代码中的类似用法 → 仍不确定则写 TODO |
| 需要修改已有代码才能继续 | 评估修改范围：小于 5 行可直接改，大于 5 行记录 TODO 等待用户决策 |
| 发现已有代码有 bug | 记录到 TODO 列表，不修（避免在全自动模式下引入新 bug） |
| 轮次超时（15分钟） | 保存当前进度，记录"未完成：XXX"，跳到下一轮 |
| 上下文即将耗尽 | 停止推进，生成最终状态报告，告知用户"上下文不足，建议新会话继续" |

---

## 3. 禁止做的事

1. **不要运行 `git commit` / `git push`**
2. **不要修改 `docs/proposal.md` / `docs/brief.md`**（这些是已确认决策，不可变）
3. **不要重写 M03 Orchestrator 或 M02 Commands 的核心逻辑**
4. **不要安装新的 Cargo 依赖**（除非绝对必要且无法替代）
5. **不要删除已有文件**（即使认为是旧代码）

---

## 4. 每轮结束时的标准动作

```
1. 更新 task-progress.md 对应模块的进度
2. 在 notes.md 追加一行："[时间] 完成第 X 轮：YYY"
3. 检查是否有未提交的 TODO 需要记录
```

---

*本指令生成时间：2026-05-18*
*适用前提：核心代码已完成，进入测试/打磨阶段*
