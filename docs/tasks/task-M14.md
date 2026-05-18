# 任务拆分：M14 — executor-preserve（保留执行器）

> 职责：对标记为 `Action::Preserve` 的条目执行无操作，仅记录用户选择保留，用于最终报告。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（`Executor` trait 或等效调度接口）

## 推荐开发顺序

1. EPR-01 ~ EPR-02（核心实现）
2. EPR-03 ~ EPR-04（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **EPR-01** 实现 `PreserveExecutor` 结构体并实现 `Executor` trait
  - `execute()` 方法：不修改任何文件/注册表/环境变量
  - 直接返回 `ExecutionResult { item_id, action: Preserve, status: Success, detail: Some("用户选择保留") }`
  - 测试点：编译通过

- [ ] **EPR-02** 实现批量保留记录优化
  - 如果大量连续条目都是 Preserve，合并生成一条汇总记录
  - 减少 `ExecutionReport` 体积
  - 测试点：100 条 Preserve 条目合并为 1 条汇总

- [ ] **EPR-03** 【测试】验证返回 `Preserve` 类型的 `ExecutionResult`
  - 断言 `result.action == Action::Preserve`
  - 断言 `result.status == ExecutionStatus::Success`

- [ ] **EPR-04** 【测试】验证不修改任何文件
  - 传入文件路径，执行前后文件存在且 MD5 不变

### P2 — 后续迭代

- [ ] **EPR-05** 实现保留原因记录（供用户后续查看）
  - 在前端提供"保留原因"输入框（可选）
  - 将原因写入 `ExecutionResult.detail`

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| EPR-03 | 返回 Preserve 类型结果 | P1 |
| EPR-04 | 不修改文件 | P1 |
| EPR-05 | 保留原因记录 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（执行阶段调用）
依赖方：无（最简单执行器）
```
