# 任务拆分：M15 — secure-erase（安全擦除）

> 职责：对文件进行多次覆写（DoD 5220.22-M 标准），覆写后重命名并删除，确保数据不可恢复。

---

## 前置条件

- [ ] INF-05（全局错误类型定义完成）

## 推荐开发顺序

1. SE-01 ~ SE-02（trait 与结构体）
2. SE-03 ~ SE-05（核心算法）
3. SE-06 ~ SE-09（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SE-01** 定义 `SecureEraser` trait 和 `EraseError` 枚举
  - `fn erase_file(&self, path: &Path) -> Result<(), EraseError>`
  - `fn erase_directory(&self, path: &Path) -> Result<(), EraseError>`
  - `EraseError` 包含：`IoError`, `PathNotFound`, `PermissionDenied`
  - 测试点：trait object 可正常构造

- [ ] **SE-02** 实现 `DoDEraser` 结构体
  - 配置字段：`passes: u8`（默认 3，对应 DoD 标准）
  - 实现 `SecureEraser` trait

- [ ] **SE-03** 实现 `DoDEraser::erase_file()` 覆写逻辑
  - Pass 1：覆写 `0x00`
  - Pass 2：覆写 `0xFF`
  - Pass 3：覆写随机数据（`rand::rngs::OsRng`）
  - 每次覆写后调用 `fsync`
  - 使用固定大小缓冲区（如 64KB）流式写入，避免大文件 OOM
  - 测试点：覆写后读取文件内容，验证非原始数据

- [ ] **SE-04** 实现覆写后重命名为随机名并删除
  - 重命名：原文件名 → 16 字节随机十六进制字符串（无扩展名）
  - 重命名后调用 `std::fs::remove_file`
  - 测试点：验证最终文件不存在

- [ ] **SE-05** 实现 `DoDEraser::erase_directory()`
  - 递归安全擦除所有子文件（调用 `erase_file`）
  - 子目录递归处理
  - 所有文件清理后，删除空目录
  - 测试点：验证目录及所有子内容被完全移除

- [ ] **SE-06** 【测试】创建 1MB 测试文件，覆写后读取验证内容非原始数据
  - 直接读取重命名前的文件（在重命名前 hook）验证已覆写

- [ ] **SE-07** 【测试】验证覆写后文件被成功删除（文件不存在）

- [ ] **SE-08** 【测试】验证嵌套目录（3 层深，每层多个文件）被完全擦除

- [ ] **SE-09** 【测试】验证 100MB 大文件覆写不 OOM（流式缓冲区工作正常）
  - 监控测试进程内存占用峰值

### P2 — 后续迭代

- [ ] **SE-10** 支持可配置的覆写次数（1/3/7/35 pass）
  - 7 pass 为 Gutmann 方法，35 pass 为极致安全

- [ ] **SE-11** 对 SSD/NVMe 添加 TRIM 提示（不依赖，仅发送提示）
  - Windows `DeviceIoControl` + `FSCTL_SET_ZERO_DATA`

- [ ] **SE-12** 添加进度回调（每完成一个 pass 或每 1% 进度）
  - 供 M03 orchestrator 的大文件擦除进度展示

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SE-06 | 覆写后内容非原始 | P1 |
| SE-07 | 文件最终被删除 | P1 |
| SE-08 | 递归目录完整擦除 | P1 |
| SE-09 | 大文件流式写入不 OOM | P1 |
| SE-10 | 可配置 pass 数 | P2 |

---

## 依赖关系

```
被依赖方：M12 executor-delete（文件类型删除调用）
依赖方：无（最底层工具模块）
```
