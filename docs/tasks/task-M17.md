# 任务拆分：M17 — resource-controller（资源控制器）

> 职责：默认限制当前进程及其子进程的 CPU 使用率 ≤30%；使用 Windows Job Object API；提供解除限制模式。

---

## 前置条件

- [ ] INF-05（全局错误类型定义完成）

## 推荐开发顺序

1. RC-01 ~ RC-02（类型与 API 封装）
2. RC-03 ~ RC-05（核心逻辑）
3. RC-06 ~ RC-08（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **RC-01** 定义 `ResourceConfig` 和 `ResourceUsage` 结构体
  - `ResourceConfig { cpu_limit_percent: u8, unlimited: bool }`
  - `ResourceUsage { cpu_percent: f32, memory_mb: u64 }`
  - 默认值：`cpu_limit_percent = 30`, `unlimited = false`
  - 测试点：默认值符合 RULE-05

- [ ] **RC-02** 封装 Windows Job Object 基础 API（`windows` crate）
  - `CreateJobObjectW`
  - `SetInformationJobObject` + `JOBOBJECT_BASIC_LIMIT_INFORMATION`
  - `AssignProcessToJobObject`
  - 失败时返回 `ResourceError::JobObjectFailed`
  - 测试点：API 封装函数参数正确，不泄漏 Handle

- [ ] **RC-03** 实现 `ResourceController::apply_limits(config: ResourceConfig)`
  - 若 `unlimited = true`，直接返回 Ok
  - 否则计算 `Affinity` 或 `PerProcessUserTimeLimit` 实现 CPU 限制
  - 将当前进程加入 Job Object
  - 测试点：验证 `unlimited=true` 时跳过限制

- [ ] **RC-04** 实现 `ResourceController::remove_limits()`
  - 终止 Job Object 限制或设置 `unlimited = true`
  - 测试点：调用后 `current_usage` 反映无限制状态

- [ ] **RC-05** 实现 `ResourceController::current_usage() -> ResourceUsage`
  - 读取当前进程 CPU 和内存占用
  - 测试点：返回值在合理范围内（CPU 0~100%，内存 > 0）

- [ ] **RC-06** 【测试】验证 `apply_limits` 时 Job Object 配置参数正确（mock Windows API 或验证行为）
  - 由于真实限制需要观察，单元测试以 API 参数校验为主

- [ ] **RC-07** 【测试】验证 `remove_limits` 后限制被清除
  - 通过再次查询 Job Object 信息验证

- [ ] **RC-08** 【测试】验证默认值 `unlimited = false`
  - 这是 RULE-05 硬规则，必须测试

### P2 — 后续迭代

- [ ] **RC-09** 实现内存上限软限制（超过阈值时通知 Orchestrator 降速）
  - 非硬性 kill，而是发送信号让 Scanner 降低并发

- [ ] **RC-10** 实现 CPU 限制的可配置档位（10% / 30% / 50% / 无限制）
  - 供 M01 frontend 的滑块控件使用

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| RC-06 | Job Object API 参数正确 | P1 |
| RC-07 | remove_limits 生效 | P1 |
| RC-08 | 默认 unlimited=false | P1 |
| RC-09 | 内存软限制信号 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（启动时调用 apply_limits）、M02 commands（get/set 接口暴露）
依赖方：无（独立模块）
```
