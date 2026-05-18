# 任务拆分：M18 — temp-store（临时数据管理）

> 职责：管理 `%TEMP%/french-exit/` 目录下的所有临时文件；扫描中间结果分批落盘；程序退出时自毁。HTML 庆祝页**不放在此处**。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）

## 推荐开发顺序

1. TS-01 ~ TS-02（基础结构）
2. TS-03 ~ TS-05（核心功能）
3. TS-06（Drop 安全网）
4. TS-07 ~ TS-10（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **TS-01** 定义 `TempStore` 结构体及构造函数
  - 字段：`root: PathBuf`（固定为 `%TEMP%/french-exit/{pid}/`）
  - 构造函数自动创建目录，失败时返回 `BackendError`
  - 测试点：构造后目录存在且可写

- [ ] **TS-02** 实现 `allocate(prefix: &str) -> PathBuf`
  - 在 `root` 下创建带随机后缀的临时文件或子目录
  - 命名规则：`{prefix}_{uuid}`
  - 测试点：分配的文件前缀正确，不重复

- [ ] **TS-03** 实现 `save_scan_batch(batch: &[TraceItem])`
  - 输出路径：`root/results/scan_{batch_id}.jsonl`
  - 格式：JSON Lines（每行一个 `TraceItem`）
  - 自动追加，不覆盖已有批次
  - 测试点：写入后按行读取与原始数据一致

- [ ] **TS-04** 实现 `load_scan_results(offset: usize, limit: usize) -> Vec<TraceItem>`
  - 按批次文件顺序读取，支持跨文件分页
  - 内存友好：不一次性加载全部到内存
  - 测试点：offset/limit 边界正确；空结果返回空 Vec

- [ ] **TS-05** 实现 `self_destruct() -> Result<(), BackendError>`
  - 递归删除 `root` 下所有内容（文件 + 子目录）
  - **不删除** `root` 之外任何文件
  - 返回值：成功 / 失败原因
  - 测试点：执行后目录为空或不存在

- [ ] **TS-06** 为 `TempStore` 实现 `Drop` trait
  - `drop()` 内部调用 `self_destruct()`，忽略错误（用 `let _ = ...`）
  - 测试点：TempStore 离开作用域后目录被清理

- [ ] **TS-07** 【测试】构造包含嵌套子目录的临时结构，验证 `self_destruct` 完全清理
  - 用 `tempfile::TempDir` 作为测试根目录，避免污染真实 `%TEMP%`

- [ ] **TS-08** 【测试】写入 1000+ 条 `TraceItem`，验证 `save_scan_batch` + `load_scan_results` 分页一致性
  - 分 3 个 batch 文件写入，验证跨文件 offset 计算正确

- [ ] **TS-09** 【测试】验证 `Drop` 在 panic 后仍被调用（Rust 作用域规则保证）

- [ ] **TS-10** 【测试】验证 `allocate` 在并发场景下不生成同名文件
  - 使用 `tokio::spawn` 并发调用 100 次

### P2 — 后续迭代

- [ ] **TS-11** 实现预览缓存子目录管理（`root/preview/`）
  - 为 M07 scanner-chat 的图片预览提供临时解压空间
  - 提供 `allocate_preview(key: &str) -> PathBuf`

- [ ] **TS-12** 实现执行中间日志目录（`root/logs/`）
  - 为 M03 orchestrator 的详细操作记录提供落盘
  - 自毁时一并清理

- [ ] **TS-13** 添加 `self_destruct` 的防误删校验（验证 `root` 路径必须包含 `french-exit` 关键字）

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| TS-07 | `self_destruct` 完整清理嵌套结构 | P1 |
| TS-08 | 分页读写 1000+ 条数据一致性 | P1 |
| TS-09 | `Drop` 在异常路径仍触发 | P1 |
| TS-10 | 并发 allocate 无冲突 | P1 |
| TS-11 | 预览缓存隔离管理 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（扫描结果读写）、M05-M11 scanners（大结果落盘）、M16 reporter（不依赖，但 HTML 必须排除）
依赖方：无（最底层模块之一）
```
