# 任务拆分：M03 — orchestrator（流程调度器）

> 职责：维护全局状态机（FSM），调度 Scanner / Executor / Reporter 按序执行；管理暂停/恢复/取消信号；扫描阶段分批聚合结果。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）
- [ ] M04 scanner-registry（`ScannerRegistry` 可用）
- [ ] M12 executor-delete（`DeleteExecutor` 可用）
- [ ] M13 executor-pack（`PackExecutor` 可用）
- [ ] M14 executor-preserve（`PreserveExecutor` 可用）
- [ ] M16 reporter（`Reporter` 可用）
- [ ] M18 temp-store（`TempStore` 可用）
- [ ] M17 resource-controller（`ResourceController` 可用）

## 推荐开发顺序

1. OR-01 ~ OR-02（状态机与类型）
2. OR-03 ~ OR-04（扫描与暂停）
3. OR-05 ~ OR-06（执行计划与调度）
4. OR-07 ~ OR-11（进度与状态转换）
5. OR-12 ~ OR-15（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **OR-01** 定义 `SessionState` 和 `SessionId` 类型
  - `SessionId = Uuid`
  - `SessionState` 枚举：`Idle`, `Scanning { scan_id }`, `Paused`, `Scanned { item_count }`, `Confirming`, `Executing`, `Completed { report }`, `Failed { reason }`
  - 测试点：状态可序列化

- [ ] **OR-02** 实现 FSM 状态转换规则与校验
  - 合法转换：
    - `Idle → Scanning`（start_scan）
    - `Scanning ↔ Paused`（pause/resume）
    - `Scanning → Scanned`（扫描完成）
    - `Scanned → Confirming`（前端加载结果）
    - `Confirming → Executing`（submit_decisions）
    - `Executing → Completed`（执行完成）
  - 非法转换返回 `OrchestratorError::InvalidStateTransition`
  - 测试点：非法转换被阻止

- [ ] **OR-03** 实现 `start_scan()` 创建会话并调度 `ScannerRegistry`
  - 构造 `ScanContext`，传入 `start_date` 和 `user_home`
  - 创建 `watch::channel<bool>` 作为 pause 信号
  - 创建 `mpsc::channel<ProgressEvent>` 作为进度通道
  - 在 tokio task 中执行 `scanner_registry.run_selected()`
  - 状态：`Idle → Scanning`
  - 测试点：调用后状态为 Scanning

- [ ] **OR-04** 实现 `pause_session()` / `resume_session()` 信号传递
  - `pause` → 发送 `true` 到 watch channel
  - `resume` → 发送 `false`
  - Scanner 通过 `pause_rx` 接收并暂停/继续
  - 状态转换：`Scanning ↔ Paused`
  - 测试点：pause 后 Scanner 停止产生新结果

- [ ] **OR-05** 实现 `plan_execution()` 生成 `ExecutionPlan`
  - 接收用户 `Decision[]`
  - 按 `action` 分组为 Delete / Pack / Preserve 三个列表
  - 验证所有 `item_id` 存在于当前会话的扫描结果中
  - 状态：`Scanned → Confirming`
  - 测试点：无效 item_id 返回错误

- [ ] **OR-06** 实现 `execute_plan()` 调度三个 Executor
  - 并行/串行执行 Delete / Pack / Preserve
  - PackExecutor 需要 `finalize()` 调用
  - 收集所有 `ExecutionResult` 生成 `ExecutionReport`
  - 状态：`Confirming → Executing → Completed`
  - 测试点：三种 action 都被正确分发

- [ ] **OR-07** 实现进度事件聚合与推送
  - 扫描阶段：转发 ScannerRegistry 的 `ScanProgress`
  - 执行阶段：汇总 Executor 的进度（如 "正在安全擦除 3/10"）
  - 通过 `progress_tx` 推送到 Frontend
  - 测试点：Frontend 能收到连续进度事件

- [ ] **OR-08** 实现扫描完成 → `Scanned` 状态转换
  - ScannerRegistry 返回后，统计总 item 数
  - 状态：`Scanning → Scanned`
  - 测试点：状态正确转换

- [ ] **OR-09** 实现用户提交 decisions → `Confirming → Executing` 转换
  - 仅在 `Confirming` 状态接受 `submit_decisions`
  - 提交后立即进入 `Executing`
  - 这是 RULE-01 硬规则的执行层面保障
  - 测试点：非 Confirming 状态提交返回错误

- [ ] **OR-10** 实现执行完成 → `Completed` 状态转换
  - 所有 Executor 完成后，调用 M16 Reporter
  - 生成 HTML 并打开浏览器
  - 调用 M18 TempStore::self_destruct()
  - 状态：`Executing → Completed`
  - 测试点：TempStore 自毁被调用

- [ ] **OR-11** 实现异常取消/错误处理路径
  - 用户可随时取消（从任何状态 → `Idle`，清理临时数据）
  - Scanner/Executor panic 时捕获并转为 `Failed` 状态
  - 失败时仍尝试调用 `TempStore::self_destruct()`
  - 测试点：panic 后被捕获，不导致程序崩溃

- [ ] **OR-12** 【测试】构造假 Scanner/Executor 验证状态机 `Idle → Scanning → Scanned`
  - mock Scanner 立即返回 10 条结果
  - 验证状态序列正确

- [ ] **OR-13** 【测试】验证 pause/resume 信号正确传递到 Scanner
  - mock Scanner 计数运行次数，pause 后计数停止，resume 后继续

- [ ] **OR-14** 【测试】验证未提交 decisions 时无法进入 Executing
  - 从 `Scanned` 直接调用 `execute_plan` → 期望错误

- [ ] **OR-15** 【测试】验证加密文件回调从 Orchestrator 透传到 Frontend
  - mock PackExecutor 的 `on_encrypted` 回调，验证 Orchestrator 正确转发

### P2 — 后续迭代

- [ ] **OR-16** 实现扫描阶段的部分结果实时推送
  - Scanner 每完成一批就推送到 Frontend
  - 前端可提前浏览已扫描到的结果

- [ ] **OR-17** 实现执行阶段的细粒度进度
  - 大文件安全擦除的逐文件进度
  - 大目录打包的逐文件进度

- [ ] **OR-18** 实现会话超时自动清理
  - 扫描会话超过 30 分钟无活动，自动取消并清理

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| OR-12 | 状态机 Idle→Scanning→Scanned | P1 |
| OR-13 | pause/resume 信号传递 | P1 |
| OR-14 | 未确认无法执行 | P1 |
| OR-15 | 加密文件回调透传 | P1 |
| OR-16 | 部分结果实时推送 | P2 |
| OR-17 | 细粒度执行进度 | P2 |

---

## 依赖关系

```
被依赖方：M02 commands（所有后端操作的入口）
依赖方：M04 scanner-registry, M12~M14 executors, M16 reporter, M17 resource-ctl, M18 temp-store
```
