# 任务拆分：M09 — scanner-system（系统痕迹扫描器）

> 职责：扫描最近打开文档列表、Temp 文件夹、事件查看器日志、搜索索引、缩略图缓存、休眠文件、系统还原点；按入职日期过滤。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SS-01 ~ SS-05（核心扫描项）
2. SS-06 ~ SS-07（P2 扩展项）
3. SS-08 ~ SS-10（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SS-01** 实现最近打开文档列表扫描
  - 路径：`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs`
  - 以及 `%APPDATA%\Microsoft\Windows\Recent\`
  - 解析 `.lnk` 快捷方式指向的原始文件
  - 按文件修改时间过滤
  - 测试点：构造假 `.lnk` 和注册表项验证

- [ ] **SS-02** 实现 Temp 文件夹扫描 + 入职日期过滤
  - 路径：`%TEMP%`（用户级 Temp）和 `%WINDIR%\Temp`
  - 递归扫描文件，按修改时间过滤
  - 只返回总大小 > 1MB 的汇总，或单个 > 10MB 的文件
  - 避免结果爆炸（Temp 文件通常极多）
  - 测试点：验证过滤和汇总逻辑

- [ ] **SS-03** 实现 Windows 事件查看器日志扫描（轻量版）
  - 只扫描 `Security` 和 `System` 日志中用户相关事件
  - 使用 `wevtapi` 或读取 `.evtx` 文件
  - 由于事件日志解析复杂，P1 仅检测日志文件大小和存在性
  - 标注 `inferred: true` + `"系统日志可能记录你的操作历史"`
  - 测试点：验证日志文件被检测到

- [ ] **SS-04** 实现搜索索引扫描
  - 路径：`%PROGRAMDATA%\Microsoft\Search\Data\Applications\Windows\`
  - 检测 `Windows.edb` 文件大小
  - 该文件包含你搜索过的关键词索引
  - 返回单条汇总 `TraceItem`
  - 测试点：验证索引文件检测

- [ ] **SS-05** 实现缩略图缓存扫描
  - 路径：`%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db`
  - 返回缓存文件列表和总大小
  - 标注 `"包含你浏览过的图片缩略图"`
  - 测试点：虚拟缩略图缓存文件验证

- [ ] **SS-06** 【测试】mock 系统数据验证各扫描逻辑
  - 用 tempdir 模拟 `%TEMP%`, `%LOCALAPPDATA%` 等结构

- [ ] **SS-07** 【测试】验证入职日期过滤生效
  - 创建修改时间不同的文件，验证过滤结果

- [ ] **SS-08** 【测试】验证权限不足时标记风险而非 panic
  - 模拟系统目录不可读场景

### P2 — 后续迭代

- [ ] **SS-09** 实现休眠文件（`hiberfil.sys`）检测
  - 检测 `C:\hiberfil.sys` 存在性和大小
  - 标注 `"休眠文件可能包含内存中的敏感数据"`
  - 由于需要管理员权限删除，仅检测列出

- [ ] **SS-10** 实现系统还原点检测
  - 使用 `WMI` 或 `srclient` 查询还原点
  - 返回入职日期后创建的还原点列表
  - 标注 `"系统还原点可能包含你的个人数据快照"`

- [ ] **SS-11** 实现事件查看器日志的深度解析（P2）
  - 解析 `.evtx` 提取用户登录/注销/文件访问事件
  - 使用 `evtx` crate 或 Windows API

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SS-06 | mock 系统数据扫描 | P1 |
| SS-07 | 入职日期过滤 | P1 |
| SS-08 | 权限降级容错 | P1 |
| SS-09 | 休眠文件检测 | P2 |
| SS-10 | 系统还原点检测 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
