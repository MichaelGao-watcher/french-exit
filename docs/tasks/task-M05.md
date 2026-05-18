# 任务拆分：M05 — scanner-fs（文件系统扫描器）

> 职责：扫描 Desktop、Downloads 中入职日期后的新增/修改文件；特殊处理微信聊天记录目录（整目录标记为建议处理）。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SF-01 ~ SF-02（基础结构与 Desktop 扫描）
2. SF-03 ~ SF-05（Downloads + 微信特殊处理）
3. SF-06 ~ SF-07（过滤与排除）
4. SF-08（P2 扩展）
5. SF-09 ~ SF-12（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SF-01** 实现 `FileSystemScanner` 结构体并实现 `Scanner` trait
  - `id() -> "scanner-fs"`
  - `category() -> TraceCategory::FileSystem`
  - `display_name() -> "个人文件"`
  - 测试点：trait 实现编译通过

- [ ] **SF-02** 实现 Desktop 目录扫描 + 入职日期过滤
  - 路径：`ScanContext.user_home / "Desktop"`
  - 递归扫描子目录
  - 过滤条件：`modified_at >= start_date`
  - 每条生成 `TraceItem`，`category = FileSystem`
  - 测试点：tempdir 构造文件树，验证结果完整

- [ ] **SF-03** 实现 Downloads 目录扫描 + 入职日期过滤
  - 路径：`ScanContext.user_home / "Downloads"`
  - 与 Desktop 共用同一递归扫描逻辑
  - 测试点：同上

- [ ] **SF-04** 实现文件 MIME 类型推断
  - 基于文件扩展名映射（不用 magic number，避免读取大文件）
  - 常用类型：image/*, video/*, audio/*, application/pdf, application/zip 等
  - 写入 `TraceItem` 的 metadata 供前端展示图标
  - 测试点：常见扩展名映射正确

- [ ] **SF-05** 实现微信聊天记录目录检测
  - 检测路径：`%USERPROFILE%\Documents\WeChat Files\{wxid}\`（常见路径）
  - 备选路径：从注册表或文件系统探测
  - 发现微信目录后，整个目录作为**单个** `TraceItem` 返回
  - 不递归列出目录内每个文件
  - 测试点：虚拟微信目录被识别为单条记录

- [ ] **SF-06** 实现微信目录 `suggested_action = Some(Action::DeleteOrPack)`
  - 这是 RULE-03 硬规则
  - `risk_note = Some("微信聊天记录属于私人内容，建议处理")`
  - 测试点：验证微信 TraceItem 的 suggested_action

- [ ] **SF-07** 实现排除规则
  - 排除 Windows 系统目录：`C:\Windows`, `C:\Program Files`, `C:\ProgramData`
  - 排除 French Exit 自身目录（避免自扫）
  - 排除隐藏的系统文件/目录（`$Recycle.Bin`, `pagefile.sys` 等）
  - 测试点：构造被排除目录下的文件，验证不出现在结果中

- [ ] **SF-08** 【测试】在 tempdir 构造已知文件树，验证扫描结果完整性
  - 构造 Desktop-like 结构，含文件、子目录、空目录
  - 验证返回的 `TraceItem` 数量、路径、大小、修改时间正确

- [ ] **SF-09** 【测试】验证入职日期前文件被过滤
  - 创建 3 个文件：入职日期前 1 天、当天、后 1 天
  - 验证只返回后 2 个

- [ ] **SF-10** 【测试】验证微信目录被整目录标记（不拆分内部文件）
  - 构造虚拟 `WeChat Files/wxid_xxx/` 含多个子文件
  - 验证只返回 1 条 TraceItem，path 指向根目录

- [ ] **SF-11** 【测试】验证系统目录和自身目录被排除
  - 扫描路径包含 French Exit 自身可执行文件所在目录
  - 验证结果中无自身文件

### P2 — 后续迭代

- [ ] **SF-12** 实现 Documents 目录扫描（不逐条展示，仅作为汇总信息）
  - 按文件类型统计数量/大小，返回**单条**汇总 `TraceItem`
  - 前端展示为"文档文件夹中有 X 个文件，共 Y GB"，用户可决定去留

- [ ] **SF-13** 实现自定义包含/排除路径配置
  - `FileSystemScanner { include_paths, exclude_paths }`
  - 供高级用户扩展扫描范围

- [ ] **SF-14** 实现文件大小阈值过滤（跳过 < 1KB 的日志类文件）
  - 减少结果噪音

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SF-08 | 已知文件树扫描完整性 | P1 |
| SF-09 | 入职日期过滤 | P1 |
| SF-10 | 微信整目录标记 | P1 |
| SF-11 | 排除规则生效 | P1 |
| SF-12 | Documents 汇总扫描 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
