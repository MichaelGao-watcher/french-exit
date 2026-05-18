# 任务拆分：M02 — commands（IPC 命令层）

> 职责：作为 Frontend 与 Backend 的唯一通道；所有入参合法性校验；异常转换为前端友好的错误码。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）
- [ ] INF-07（IPC 共享类型同步完成）
- [ ] M03 orchestrator（Orchestrator 实例可用）
- [ ] M17 resource-controller（ResourceController 实例可用）

## 推荐开发顺序

1. CMD-01（AppState 设计）
2. CMD-02 ~ CMD-08（各 command 实现）
3. CMD-09（错误转换）
4. CMD-10 ~ CMD-12（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **CMD-01** 实现 `AppState` 结构体
  - 字段：`orchestrator: Arc<Orchestrator>`, `resource_controller: Arc<ResourceController>`, `temp_store: Arc<TempStore>`
  - 使用 `tauri::State` 管理生命周期
  - Tauri `setup` 钩子中初始化
  - 测试点：AppState 可正确注入到 command 函数

- [ ] **CMD-02** 实现 `start_scan` command
  - 参数：`start_date: String`, `categories: Vec<String>`
  - 校验：`start_date` 必须是 `YYYY-MM-DD` 格式（用 `chrono::NaiveDate::parse_from_str`）
  - 校验：`categories` 每个元素必须是已知 `TraceCategory` 的字符串表示
  - 调用 `orchestrator.start_scan()`
  - 返回 `ScanId` 或 `FrontendError`
  - 测试点：合法参数通过，非法日期返回错误

- [ ] **CMD-03** 实现 `pause_scan` / `resume_scan` command
  - 参数：`scan_id: String`
  - 校验：`scan_id` 是否为合法 UUID
  - 转发给 `orchestrator.pause_session()` / `resume_session()`
  - 测试点：mock Orchestrator 验证调用

- [ ] **CMD-04** 实现 `get_scan_results` command
  - 参数：`scan_id`, `category: Option<String>`, `page: u32`, `page_size: u32`
  - 校验：`page` >= 1, `page_size` 在 10~500 之间
  - 从 `TempStore` 分页读取或从 Orchestrator 会话获取
  - 返回 `PaginatedResult<TraceItem>`
  - 测试点：分页参数边界正确

- [ ] **CMD-05** 实现 `preview_item` command
  - 参数：`item_id: String`
  - 校验：`item_id` 为合法 UUID
  - 从 `TempStore` 或会话中查找对应 `TraceItem`
  - 根据 `category` 和 `path` 返回不同预览结果：
    - 文本文件：读取前 4KB 内容
    - 图片文件：返回 Base64 编码（或临时文件 URL）
    - 不支持的类型：返回 `PreviewResult::Unsupported`
  - 测试点：mock 数据验证预览返回格式

- [ ] **CMD-06** 实现 `submit_decisions` command
  - 参数：`scan_id`, `decisions: Vec<Decision>`
  - 校验：`decisions` 中每个 `item_id` 合法，每个 `action` 为 `Delete`/`Preserve`/`Pack`
  - 校验：无重复 `item_id`
  - 调用 `orchestrator.plan_execution()`
  - 返回 `ExecutionPlan` 或错误
  - 测试点：重复 item_id 返回错误

- [ ] **CMD-07** 实现 `start_execution` command
  - 参数：`plan_id: String`, `output_dir: Option<String>`
  - 校验：`plan_id` 合法
  - 校验：`output_dir` 如提供，必须是合法绝对路径，且目录存在或可创建
  - 调用 `orchestrator.execute_plan()`
  - 测试点：非法路径返回错误

- [ ] **CMD-08** 实现 `get_resource_config` / `set_resource_config` command
  - `get`：读取当前 ResourceController 配置
  - `set`：校验 `cpu_limit_percent` 在 1~100，`unlimited` 为 bool
  - 调用 `resource_controller.apply_limits()`
  - 测试点：非法百分比返回错误

- [ ] **CMD-09** 实现 `FrontendError` 转换与错误码映射
  - `InvalidDateFormat → "INVALID_DATE"`
  - `InvalidCategory → "INVALID_CATEGORY"`
  - `SessionNotFound → "SESSION_NOT_FOUND"`
  - `InvalidState → "INVALID_STATE"`
  - `InsufficientDiskSpace → "DISK_FULL"`
  - 等
  - 测试点：每个 BackendError 变体都能正确映射

- [ ] **CMD-10** 【测试】mock Orchestrator 验证命令参数透传
  - 使用 `mockall` 构造 MockOrchestrator，验证每个 command 调用时参数正确

- [ ] **CMD-11** 【测试】验证非法日期格式返回错误
  - 传入 `"2024/01/01"`, `"not-a-date"`, `""` 等，验证返回 `FrontendError`

- [ ] **CMD-12** 【测试】验证非法路径返回错误
  - 传入 `"C:\\Windows\\System32"`（系统目录），`"..\\..\\etc"`（相对路径 traversal），验证被阻止

### P2 — 后续迭代

- [ ] **CMD-13** 实现 `cancel_scan` command
  - 用户可随时取消当前会话
  - 调用 `orchestrator.cancel_session()`

- [ ] **CMD-14** 实现 command 层面的日志记录
  - 每个 command 调用记录 `tracing::info!`，含参数摘要（不含敏感数据）

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| CMD-10 | mock Orchestrator 参数透传 | P1 |
| CMD-11 | 非法日期格式校验 | P1 |
| CMD-12 | 非法路径校验 | P1 |
| CMD-13 | 取消扫描 command | P2 |
| CMD-14 | command 日志 | P2 |

---

## 依赖关系

```
被依赖方：M01 frontend（所有 IPC 调用方）
依赖方：M03 orchestrator, M17 resource-controller, M18 temp-store
```
