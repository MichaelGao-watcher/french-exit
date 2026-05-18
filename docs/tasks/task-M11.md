# 任务拆分：M11 — scanner-env（环境变量扫描器）

> 职责：扫描用户级环境变量，识别与已知工具相关的条目；明确标注风险；默认不自动勾选（RULE-02）。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SEV-01 ~ SEV-03（扫描与识别）
2. SEV-04 ~ SEV-05（标记与默认状态）
3. SEV-06 ~ SEV-08（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SEV-01** 实现用户级环境变量读取
  - 使用 `std::env::vars()` 或 Windows API `GetEnvironmentStringsW`
  - 只读取当前用户级变量（不读取系统级）
  - 测试点：读取结果包含已知变量

- [ ] **SEV-02** 实现 TOKEN 类变量识别
  - 关键字匹配：`GH_TOKEN`, `GITHUB_TOKEN`, `AWS_ACCESS_KEY_ID`, `AZURE_TOKEN`, `NPM_TOKEN`, `DOCKER_TOKEN` 等
  - 值内容匹配：长度 > 20 的十六进制/ Base64 字符串
  - 测试点：构造假环境变量验证匹配

- [ ] **SEV-03** 实现 PATH 中工具路径识别
  - 拆分 `PATH` 变量
  - 识别含已知工具名称的路径段：`GitHub CLI`, `nodejs`, `python`, `docker`, `code` 等
  - 每个匹配的路径段生成独立 `TraceItem`
  - 测试点：构造假 PATH 验证拆分和识别

- [ ] **SEV-04** 实现风险文案生成
  - `risk_note = Some("⚠️ 以下环境变量可能与其他工具共用，删除后相关工具将无法使用，确定清除吗？")`
  - 针对 PATH 条目追加 `"删除后命令行将找不到该工具"`
  - 测试点：验证 risk_note 包含预期文案

- [ ] **SEV-05** 实现默认不自动勾选（影响前端默认状态）
  - 这是 RULE-02 硬规则
  - Scanner 本身不控制 UI 勾选状态，但需要通过 `TraceItem` 的字段或约定告知 Frontend
  - 方案：所有 env Scanner 返回的 `TraceItem` 附加 `metadata: {"default_checked": false}`
  - 或 Frontend 根据 `scanner_id = "scanner-env"` 统一处理
  - 测试点：验证 `TraceItem` 中包含默认不勾选标识

- [ ] **SEV-06** 【测试】mock 环境变量验证识别逻辑
  - 构造含 GH_TOKEN、PATH 等变量的假环境
  - 验证返回的 `TraceItem` 数量和类型

- [ ] **SEV-07** 【测试】验证风险文案正确生成
  - 断言 `risk_note.unwrap().contains("可能与其他工具共用")`

- [ ] **SEV-08** 【测试】验证系统级变量不可修改时仅列出
  - 系统级变量不在 `std::env::vars()` 范围内（Windows 下用户级和系统级需区分）
  - 若无法区分，标注 `"如为系统级变量，需要管理员权限才能修改"`

### P2 — 后续迭代

- [ ] **SEV-09** 实现环境变量修改功能（P2）
  - 当前 Scanner 只负责"扫描列出"
  - 实际修改由 M12 executor-delete 或专门模块执行
  - 提供修改所需的原始变量名、当前值、建议操作

- [ ] **SEV-10** 扩展 TOKEN 识别规则库
  - 支持更多云服务商：阿里云、腾讯云、华为云、Google Cloud、Azure 等

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SEV-06 | mock 环境变量识别 | P1 |
| SEV-07 | 风险文案生成 | P1 |
| SEV-08 | 系统级变量仅列出 | P1 |
| SEV-09 | 环境变量修改接口 | P2 |
| SEV-10 | 扩展 TOKEN 规则 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
