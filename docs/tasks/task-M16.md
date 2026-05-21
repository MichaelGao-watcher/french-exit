# 任务拆分：M16 — reporter（报告生成器）

> 职责：汇总操作结果，生成文本摘要 + HTML 庆祝页；HTML 保存位置：有打包放 zip 同目录，无打包放桌面（RULE-09）；自动调用系统浏览器打开。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] INF-05（全局错误类型定义完成）

## 推荐开发顺序

1. RP-01 ~ RP-02（结构与摘要）
2. RP-03 ~ RP-06（HTML 模板与渲染）
3. RP-07 ~ RP-08（文件输出与浏览器调用）
4. RP-09 ~ RP-11（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **RP-01** 实现 `Reporter` 结构体
  - 无状态，纯函数集合
  - 测试点：可实例化

- [ ] **RP-02** 实现 `generate_summary(report: &ExecutionReport) -> String`
  - 格式示例："已删除 1,247 条痕迹（共 3.2GB），打包 56 个文件至 D:\\Backup\\French-exit.zip，保留 12 条"
  - 数字格式化：千分位分隔
  - 测试点：注入假报告，验证文案包含预期数字

- [ ] **RP-03** 设计 HTML 庆祝页静态模板
  - 主文案："您已完成 French Exit，现在去享受生活吧"（RULE-14，不可改）
  - 布局：居中大字 + 统计卡片（删除/打包/保留数量） + 操作明细列表
  - 样式：Apple Design 风格（圆角、毛玻璃、系统颜色模式跟随）
  - 文件：单文件 HTML（内嵌 CSS，无外联资源）
  - 测试点：模板文件存在且非空

- [ ] **RP-04** 实现 HTML 模板渲染引擎
  - 使用 `tera` / `handlebars` / 或 Rust 字符串 format
  - 替换变量：`main_text`, `deleted_count`, `deleted_bytes`, `packed_count`, `packed_bytes`, `preserved_count`, `pack_path`
  - 测试点：注入变量后 HTML 包含替换值

- [ ] **RP-05** 实现清单明细列表渲染
  - 按类别分组：删除列表、打包列表、保留列表
  - 每条显示：名称、路径（截断）、大小
  - 超过 50 条折叠，提供"展开全部"按钮（前端 JS 实现）
  - 测试点：50 条以上时折叠逻辑正确

- [ ] **RP-06** 实现"待退出账号"清单和跳转链接
  - 数据来源：M06 scanner-browser 和 M07 scanner-chat 检测到的账号
  - 为每个账号生成"点击跳转退出页面"链接
  - 链接列表放在 HTML 底部
  - 测试点：链接 URL 包含对应服务的退出登录地址

- [ ] **RP-07** 实现 `generate_celebration_html()` 文件写入
  - 参数：`output_dir: &Path`
  - 文件名：`French-exit-report.html`
  - 有打包 → 放 zip 同目录；无打包 → `output_dir` 传桌面路径
  - 这是 RULE-09 硬规则
  - 测试点：验证文件写入指定目录

- [ ] **RP-08** 实现 `open_in_browser(path: &Path)`
  - Windows：`cmd /c start "" "path"` 或 `ShellExecuteW`
  - 失败时返回 `ReporterError::BrowserOpenFailed`
  - 测试点：mock 系统调用验证参数正确

- [ ] **RP-09** 【测试】注入假 `ExecutionReport`，验证 HTML 包含预期文案
  - 断言 HTML 字符串包含主文案
  - 断言包含删除数量、打包数量

- [ ] **RP-10** 【测试】验证 HTML 路径逻辑（有打包 vs 无打包）
  - 有 `pack_file_path` → HTML 与 zip 同目录
  - 无 `pack_file_path` → HTML 在桌面

- [ ] **RP-11** 【测试】验证浏览器调用参数正确
  - mock `ShellExecuteW` 或 `Command::new`，验证传入路径正确

### P2 — 后续迭代

- [ ] **RP-12** 实现深色/浅色模式 CSS 媒体查询
  - `prefers-color-scheme` 自动适配
  - 与 M01 frontend 的主题风格一致

- [ ] **RP-13** 实现 HTML 页面交互动画
  - 进入动画：淡入 + 轻微上浮
  - 统计卡片数字滚动动画（从 0 滚动到目标值）

- [ ] **RP-14** 生成文本版报告（`.txt`）
  - 供用户复制粘贴或存档
  - 与 HTML 内容一致，纯文本格式

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| RP-09 | HTML 包含主文案和统计数据 | P1 |
| RP-10 | HTML 保存路径逻辑 | P1 |
| RP-11 | 浏览器调用参数 | P1 |
| RP-12 | 深色/浅色模式适配 | P2 |
| RP-13 | 交互动画 | P2 |

---

## 依赖关系

```
被依赖方：M03 orchestrator（执行完成后调用）
依赖方：无（纯生成模块）
```
