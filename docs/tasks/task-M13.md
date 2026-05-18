# 任务拆分：M13 — executor-pack（打包执行器）

> 职责：收集标记为 `Action::Pack` 的条目，打包为 `French-exit.zip`；保留原始目录结构；加密文件回调确认；输出到用户指定目录（RULE-06）。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）
- [ ] M04 scanner-registry（`Executor` trait 或等效调度接口）

## 推荐开发顺序

1. EP-01 ~ EP-02（结构与收集）
2. EP-03 ~ EP-04（zip 创建与路径保留）
3. EP-05 ~ EP-06（加密文件处理）
4. EP-07 ~ EP-08（空间检查与 finalize）
5. EP-09 ~ EP-11（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **EP-01** 实现 `PackExecutor` 结构体
  - 字段：`output_path: PathBuf`, `items: Vec<TraceItem>`, `on_encrypted: Box<dyn Fn(&Path) -> bool + Send + Sync>`
  - 测试点：编译通过

- [ ] **EP-02** 实现条目收集与去重
  - `execute()` 每次调用将 `TraceItem` 加入内部 Vec
  - 去重：同一 `path` 不重复添加
  - 测试点：重复路径只打包一次

- [ ] **EP-03** 实现 zip 文件创建（使用 `zip` crate）
  - 创建 `ZipWriter`，输出到 `output_path / "French-exit.zip"`
  - 压缩方法：`Deflated`
  - 测试点：zip 文件能被标准解压工具打开

- [ ] **EP-04** 实现保留原始目录结构（相对路径）
  - 文件在 zip 内的路径 = 相对于用户主目录的路径
  - 例：`C:\Users\Alice\Desktop\photo.jpg` → `Desktop/photo.jpg`
  - 测试点：解压后目录结构与原始一致

- [ ] **EP-05** 实现加密文件检测与 `on_encrypted` 回调
  - 加密文件判定：扩展名为 `.enc`, `.locked`；或文件头含加密标志；或读取失败且非权限问题
  - 触发回调：`if !(on_encrypted)(path) { return Skipped }`
  - 测试点：验证回调被触发

- [ ] **EP-06** 实现用户取消后标记 `Skipped`
  - 回调返回 `false` → 该文件不加入 zip，生成 `ExecutionResult { status: Skipped("用户取消加密文件") }`
  - 测试点：验证 `Skipped` 状态正确

- [ ] **EP-07** 实现磁盘空间预检查
  - `finalize()` 前计算所有待打包文件总大小 × 1.1（压缩余量）
  - 与目标目录可用空间比较
  - 不足时返回 `ExecutionError::InsufficientDiskSpace { required, available }`
  - 测试点：模拟空间不足场景

- [ ] **EP-08** 实现 `finalize()` 生成最终 zip
  - 遍历收集的所有条目，写入 zip
  - 关闭 `ZipWriter`，返回最终文件路径
  - 测试点：finalize 后文件存在且非空

- [ ] **EP-09** 【测试】虚拟文件打包后解压验证内容完整性
  - 构造 10 个文件的目录树，打包后解压，逐字节对比 MD5

- [ ] **EP-10** 【测试】验证加密文件触发回调
  - 构造 `.enc` 文件，验证 `on_encrypted` 被调用且参数正确

- [ ] **EP-11** 【测试】验证空间不足时提前报错
  - mock 磁盘空间为 1KB，尝试打包 1MB 文件，验证返回 `InsufficientDiskSpace`

### P2 — 后续迭代

- [ ] **EP-12** 实现 zip 文件密码加密（可选）
  - 用户可选择设置 zip 密码保护
  - 使用 `zip` crate 的 AES 加密功能

- [ ] **EP-13** 实现打包进度回调
  - 每完成一个大文件（> 100MB）回传进度
  - 供前端展示"正在打包 xxx / 共 yyy"

- [ ] **EP-14** 实现打包后的文件校验（SHA-256 清单）
  - zip 内附带 `manifest.sha256` 文件，列明每个文件的哈希

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| EP-09 | 打包解压内容完整性 | P1 |
| EP-10 | 加密文件回调触发 | P1 |
| EP-11 | 磁盘空间不足提前报错 | P1 |
| EP-12 | zip 密码加密 | P2 |
| EP-13 | 打包进度回调 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（执行阶段调用）
依赖方：无（独立模块，但依赖 zip crate）
```
