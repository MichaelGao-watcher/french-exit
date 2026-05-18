# 任务拆分：M12 — executor-delete（删除执行器）

> 职责：对标记为 `Action::Delete` 的条目调用安全擦除；文件调用 M15，注册表调用 Windows API；记录操作结果。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）
- [ ] M15 secure-erase（`SecureEraser` trait 可用）
- [ ] M04 scanner-registry（`Executor` trait 或等效调度接口）

## 推荐开发顺序

1. ED-01 ~ ED-02（trait 与结构体）
2. ED-03 ~ ED-05（核心逻辑）
3. ED-06 ~ ED-08（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **ED-01** 定义 `Executor` trait（如尚未定义）
  - `fn execute(&self, item: &TraceItem) -> Result<ExecutionResult, ExecutionError>`
  - 若 M03 已定义，则复用
  - 测试点：trait object 可构造

- [ ] **ED-02** 实现 `DeleteExecutor` 结构体
  - 字段：`secure_eraser: Arc<dyn SecureEraser>`
  - 构造函数注入 `SecureEraser` 实例
  - 测试点：编译通过

- [ ] **ED-03** 实现文件类型 `TraceItem` 的安全删除
  - `item.path` 为 `Some(path)` 且路径存在 → 调用 `secure_eraser.erase_file()`
  - 目录 → 调用 `secure_eraser.erase_directory()`
  - 返回 `ExecutionResult { status: Success, detail: Some(path) }`
  - 测试点：tempdir 创建文件，执行后验证不可恢复

- [ ] **ED-04** 实现注册表类型 `TraceItem` 的删除
  - `item.category == TraceCategory::Registry`
  - 调用 `winreg` 或 Windows API 删除键值
  - 需要管理员权限的键值，失败后标记 `Failed("需要管理员权限")`
  - 测试点：mock Windows API 验证调用参数

- [ ] **ED-05** 实现操作结果记录
  - 所有执行结果（成功/失败/跳过）生成 `ExecutionResult`
  - 失败时 `detail` 包含具体错误信息
  - 测试点：验证失败条目的 `ExecutionResult` 字段完整

- [ ] **ED-06** 【测试】tempdir 创建文件，执行删除后验证不可恢复
  - 创建文件 → DeleteExecutor 执行 → 尝试读取 → 期望报错 `NotFound`

- [ ] **ED-07** 【测试】验证注册表删除操作参数正确（使用 mock 或假注册表）
  - 构造 `TraceItem` 含注册表路径，验证 API 收到正确键名

- [ ] **ED-08** 【测试】验证失败时返回 `Failed` 状态而非 panic
  - 传入不存在的路径，验证返回 `Failed` 而非 unwrap panic

### P2 — 后续迭代

- [ ] **ED-09** 实现删除前二次确认回调
  - 对超大文件（> 1GB）或重要系统路径，执行前回调 Frontend 确认
  - 当前 P1 依赖 M03 的 Orchestrator 做统一确认，P2 可在 Executor 层加细粒度控制

- [ ] **ED-10** 实现删除进度回调
  - 大目录删除时，每完成一个文件回传进度
  - 供前端展示"正在安全擦除 xxx / 共 yyy"

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| ED-06 | 文件删除后不可恢复 | P1 |
| ED-07 | 注册表删除参数正确 | P1 |
| ED-08 | 失败返回 Failed 不 panic | P1 |
| ED-09 | 超大文件二次确认 | P2 |
| ED-10 | 删除进度回调 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（执行阶段调用）
依赖方：M15 secure-erase（文件删除底层实现）
```
