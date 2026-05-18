# 任务拆分：M08 — scanner-registry-sys（注册表扫描器）

> 职责：扫描 HKEY_CURRENT_USER 下入职日期后修改的键值；按时间 + 键名 + 值内容做启发式推断；每个结果标注 `inferred: true` 和警告文案（RULE-10）。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SRS-01 ~ SRS-03（注册表读取基础）
2. SRS-04 ~ SRS-06（启发式推断与标记）
3. SRS-07（权限降级）
4. SRS-08 ~ SRS-10（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SRS-01** 封装 Windows 注册表读取 API（`winreg` crate）
  - 打开 `HKEY_CURRENT_USER`
  - 递归枚举指定子键下的所有键值
  - 读取键名、值内容、最后修改时间（`RegQueryInfoKey`）
  - 测试点：API 封装不 panic

- [ ] **SRS-02** 实现注册表键值枚举与修改时间读取
  - 目标路径：`Software\`, `Environment`, `SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\` 等
  - 递归深度限制：最大 5 层，避免无限递归
  - 测试点：假注册表数据遍历完整性

- [ ] **SRS-03** 实现按修改时间过滤
  - 只保留 `last_write_time >= start_date` 的键值
  - 修改时间精度为天（忽略时分秒，避免入职当天条目被过滤）
  - 测试点：入职日期前后键值过滤正确

- [ ] **SRS-04** 实现启发式推断算法
  - 键名匹配：`username`, `email`, `phone`, `password`, `token`, `key`, `account`
  - 值内容匹配：邮箱格式、`\d{11}`（手机号）、`wxid_`、GUID 格式
  - 匹配到任一模式即判定为"疑似个人信息"
  - 测试点：构造假键值数据，验证匹配率

- [ ] **SRS-05** 实现 `inferred: true` 标记
  - 所有通过 SRS-04 推断出的条目，`TraceItem.inferred = true`
  - 测试点：`assert!(item.inferred)`

- [ ] **SRS-06** 实现风险文案生成
  - `risk_note = Some("⚠️ 此项由程序自动推断，请仔细确认后再操作")`
  - 附加说明：如匹配到 `token`，追加 `"可能包含 API 访问密钥"`
  - 这是 RULE-10 硬规则
  - 测试点：验证 risk_note 包含预期文案

- [ ] **SRS-07** 实现权限不足时优雅降级
  - 某些键需要管理员权限，读取失败时跳过该键
  - 记录 `tracing::warn!` 日志
  - 不中断整体扫描流程
  - 测试点：模拟权限错误，验证不 panic

- [ ] **SRS-08** 【测试】构造假注册表键值数据，验证推断算法准确性
  - 正例：10 个含个人信息的键，验证被识别
  - 负例：10 个不含个人信息的键，验证不被误报

- [ ] **SRS-09** 【测试】验证所有推断结果的 `inferred = true`
  - 遍历 Scanner 输出，断言 `inferred` 字段

- [ ] **SRS-10** 【测试】验证权限错误被捕获且流程继续
  - mock 注册表读取函数返回 `ERROR_ACCESS_DENIED`

### P2 — 后续迭代

- [ ] **SRS-11** 扩展启发式规则库
  - 支持更多模式：身份证号、`github.com` URL、IP 地址等
  - 引入权重打分机制

- [ ] **SRS-12** 实现注册表操作预览（值内容摘要）
  - 由于值内容可能敏感，摘要显示前 20 字符 + `...`
  - 不展示完整值，避免前端泄露敏感信息

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SRS-08 | 推断算法准确率（正例+负例） | P1 |
| SRS-09 | inferred 标记全覆盖 | P1 |
| SRS-10 | 权限降级不中断 | P1 |
| SRS-11 | 扩展规则库 | P2 |
| SRS-12 | 值内容摘要 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
