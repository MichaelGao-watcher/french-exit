# 任务拆分：M04 — scanner-registry（扫描器注册中心）

> 职责：管理所有 Scanner 实例，按类别分组调度，聚合扫描结果，单个扫描器失败不中断整体流程。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）
- [ ] M18 temp-store（大结果落盘接口）

## 推荐开发顺序

1. SR-01 ~ SR-03（trait 与注册中心）
2. SR-04 ~ SR-07（调度与容错）
3. SR-08 ~ SR-10（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SR-01** 定义 `Scanner` trait
  - `fn id(&self) -> &'static str`
  - `fn category(&self) -> TraceCategory`
  - `fn display_name(&self) -> &'static str`
  - `fn scan(&self, ctx: &ScanContext, pause_rx: &watch::Receiver<bool>, progress: &dyn Fn(ScanProgress)) -> Result<Vec<TraceItem>, ScanError>`
  - 测试点：trait object 可动态分发

- [ ] **SR-02** 定义 `ScanResultBundle` 和 `ScanProgress` 结构体
  - `ScanResultBundle { scanner_id: String, items: Vec<TraceItem>, error: Option<ScanError> }`
  - `ScanProgress { current: usize, total: Option<usize>, message: String }`

- [ ] **SR-03** 实现 `ScannerRegistry` 结构体及 `register()`
  - 内部 `Vec<Box<dyn Scanner>>`
  - 提供 `register()` 方法注册扫描器实例
  - 测试点：注册后可通过 id 查询

- [ ] **SR-04** 实现 `ScannerRegistry::run_selected()`
  - 参数：`categories: &[TraceCategory]`，过滤出需要运行的 Scanner
  - 使用 `tokio::task::spawn_blocking` 或 `rayon` 并行执行多个 Scanner
  - 收集所有 `ScanResultBundle`
  - 测试点：注入 3 个 mock Scanner，验证全部被执行

- [ ] **SR-05** 实现单个 Scanner 失败不中断整体流程
  - 某个 Scanner panic 或返回 Err 时，记录错误，继续执行其他 Scanner
  - 错误 Scanner 的 `ScanResultBundle.error` 为 `Some(...)`
  - 测试点：故意让 1 个 Scanner 失败，验证另外 2 个仍完成

- [ ] **SR-06** 实现进度聚合与转发
  - 每个 Scanner 的 `ScanProgress` 通过 channel 回传
  - 包装为 `ProgressEvent::ScanProgress { scanner_id, ... }`
  - 测试点：验证前端能收到带 scanner_id 的进度事件

- [ ] **SR-07** 实现扫描结果自动分批落盘（调用 M18 TempStore）
  - 每个 Scanner 完成后立即 `save_scan_batch`
  - 不等待全部 Scanner 完成再落盘
  - 测试点：验证大结果集内存占用可控

- [ ] **SR-08** 【测试】注入 mock Scanner 验证调度顺序和聚合结果
  - mock 返回已知 `TraceItem` 列表

- [ ] **SR-09** 【测试】验证单个 Scanner panic 不中断其他 Scanner（使用 `std::panic::catch_unwind` 或 `tokio::spawn` 的 JoinHandle）

- [ ] **SR-10** 【测试】验证类别过滤只运行选中 Scanner
  - 注册 5 个 Scanner，只选 2 个类别，验证只执行 2 个

### P2 — 后续迭代

- [ ] **SR-11** 实现 Scanner 优先级与串行调度策略
  - 轻量 Scanner（如 M11 env）先跑，重 Scanner（如 M05 fs）后跑
  - 或提供自定义调度策略接口

- [ ] **SR-12** 实现 Scanner 执行超时保护
  - 单个 Scanner 超过 5 分钟未返回，强制取消并标记超时

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SR-08 | mock Scanner 调度与聚合 | P1 |
| SR-09 | 单点失败不中断 | P1 |
| SR-10 | 类别过滤生效 | P1 |
| SR-11 | 优先级调度 | P2 |
| SR-12 | 超时保护 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（调用 run_selected 启动扫描）
依赖方：M18 temp-store（结果落盘）
下游：M05~M11（具体 Scanner 实现，注册到 Registry）
```
