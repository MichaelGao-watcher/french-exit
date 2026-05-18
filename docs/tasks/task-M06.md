# 任务拆分：M06 — scanner-browser（浏览器扫描器）

> 职责：检测 Chrome、Edge、Firefox 等浏览器；扫描历史记录、Cookie、保存密码、缓存；识别浏览器账号登录状态。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SB-01 ~ SB-04（浏览器检测与路径定位）
2. SB-05 ~ SB-09（各数据类型扫描）
3. SB-10 ~ SB-12（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SB-01** 实现浏览器检测逻辑
  - Chrome：检测 `%LOCALAPPDATA%\Google\Chrome\User Data\`
  - Edge：检测 `%LOCALAPPDATA%\Microsoft\Edge\User Data\`
  - Firefox：检测 `%APPDATA%\Mozilla\Firefox\Profiles\`
  - 检测标志：`Preferences` 文件存在、或 `History` 数据库存在
  - 返回检测到的浏览器列表
  - 测试点：虚拟目录结构验证检测正确

- [ ] **SB-02** 实现 Chrome 用户数据目录解析
  - 读取 `Local State` JSON 获取 Profile 列表
  - 定位每个 Profile 的 `History`, `Cookies`, `Login Data`, `Cache` 路径
  - 测试点：构造假 `Local State` 和 Profile 目录

- [ ] **SB-03** 实现 Edge 用户数据目录解析
  - 逻辑与 Chrome 基本一致（同 Chromium 内核）
  - 路径前缀不同
  - 测试点：同上

- [ ] **SB-04** 实现 Firefox 用户数据目录解析
  - 读取 `profiles.ini` 获取 Profile 路径
  - 定位 `places.sqlite`（历史）、`cookies.sqlite`、`logins.json`、`cache2`
  - 测试点：构造假 `profiles.ini` 和 Profile 目录

- [ ] **SB-05** 实现浏览器历史记录扫描
  - Chrome/Edge：读取 `History` SQLite 数据库的 `urls` 表
  - Firefox：读取 `places.sqlite` 的 `moz_places` 表
  - 按 `last_visit_time` 过滤（入职日期后）
  - 注意：浏览器可能正在运行导致数据库被锁定，捕获 `SQLITE_BUSY`
  - 测试点：构造假 SQLite 数据库验证读取

- [ ] **SB-06** 实现 Cookie 扫描
  - Chrome/Edge：读取 `Cookies` SQLite 数据库
  - Firefox：读取 `cookies.sqlite`
  - 返回域名级别的汇总（不逐条 Cookie，避免结果爆炸）
  - 按创建/修改时间过滤
  - 测试点：同上

- [ ] **SB-07** 实现保存密码扫描
  - Chrome/Edge：`Login Data` 数据库中 `logins` 表
  - Firefox：`logins.json` 文件
  - 只返回存在保存密码的**域名列表**和**数量**，不返回密码明文
  - 标注 `risk_note = "包含保存的账号密码信息"`
  - 测试点：验证不泄露明文密码

- [ ] **SB-08** 实现缓存扫描
  - 定位 `Cache` / `Code Cache` / `cache2` 目录
  - 返回缓存目录总大小，不逐条列出
  - 测试点：验证大小计算正确

- [ ] **SB-09** 实现浏览器账号登录状态识别
  - Chrome/Edge：检查 `Preferences` 中 `account_info` 或 `google` 字段
  - Firefox：检查 `signedInUser.json`
  - 生成 "待退出账号" 提示数据（供 M16 reporter 使用）
  - 测试点：构造含账号信息的配置文件验证识别

- [ ] **SB-10** 【测试】构造假浏览器配置目录，验证各浏览器检测逻辑
  - 创建完整的模拟 Chrome/Edge/Firefox 目录结构

- [ ] **SB-11** 【测试】验证历史记录数据库被锁定时标记 `risk_note` 而非 panic
  - 用文件锁模拟数据库占用

- [ ] **SB-12** 【测试】验证不存在的浏览器被优雅跳过
  - 系统中无 Firefox 时，结果中无 Firefox 相关条目

### P2 — 后续迭代

- [ ] **SB-13** 支持 360 安全浏览器、QQ 浏览器等国产浏览器
  - 基于 Chromium 的国产浏览器路径探测

- [ ] **SB-14** 实现浏览器缓存的安全清理（调用 M15 安全擦除）
  - 当前 P1 仅扫描列出，P2 支持一键清理缓存

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SB-10 | 假浏览器目录检测 | P1 |
| SB-11 | 数据库锁定容错 | P1 |
| SB-12 | 不存在浏览器跳过 | P1 |
| SB-13 | 国产浏览器支持 | P2 |
| SB-14 | 缓存安全清理 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
