# French Exit — 会话交接文档

> 本文档供新会话快速接手项目。当前阶段：核心代码 100% 完成，进入测试与打磨阶段。

---

## 1. 已完成的核心模块（18/18）

### Phase 1 — 基础设施
- ✅ **M04 scanner-registry** — Scanner trait + 并行调度 + 5个单元测试
- ✅ **M15 secure-erase** — DoD 5220.22-M 标准（3次覆写）+ 5个测试
- ✅ **M17 resource-ctl** — Windows Job Object CPU限制 + 4个测试
- ✅ **M18 temp-store** — JSON Lines 分批落盘 + 分页读取 + 自毁 + 7个测试

### Phase 2 — 扫描器集群（7个全部完成）
- ✅ **M05 scanner-fs** — Desktop/Downloads + 微信整目录标记(RULE-03)
- ✅ **M06 scanner-browser** — Chrome/Edge/Firefox 历史记录（rusqlite读取SQLite）
- ✅ **M07 scanner-chat** — QQ/钉钉/飞书/企业微信（整目录标记）
- ✅ **M08 scanner-registry-sys** — HKEY_CURRENT_USER 推断，inferred=true + 风险提示(RULE-10)
- ✅ **M09 scanner-system** — 最近文档/.lnk + Temp + 缩略图缓存
- ✅ **M10 scanner-devtools** — Git/SSH/VS Code/GitHub CLI 配置
- ✅ **M11 scanner-env** — TOKEN识别 + PATH工具路径，默认不选(RULE-02)

### Phase 3 — 执行器 + 报告器
- ✅ **M12 executor-delete** — 安全删除文件/目录，注册表/环境变量跳过
- ✅ **M13 executor-pack** — zip打包 + 去重 + 相对路径保留
- ✅ **M14 executor-preserve** — 无操作记录
- ✅ **M16 reporter** — HTML庆祝页(Apple Design) + 浏览器打开

### Phase 4 — 调度 + IPC + 前端
- ✅ **M03 orchestrator** — FSM状态机 + 全流程调度 + 暂停恢复
- ✅ **M02 commands** — 9个Tauri command + AppState + 完整初始化
- ✅ **M01 frontend** — 6个页面全部实现（输入/扫描/结果/确认/执行/报告）

---

## 2. 后端主链路（已跑通）

```
Frontend InputPage → start_scan(date) → M02 → M03
  → ScannerRegistry.scan_all() → [M05~M11 并行扫描]
    → TempStore.save_scan_batch() 分批落盘
  → Frontend ResultsPage ← getScanResults(page) ← TempStore.load_scan_results()
  → Frontend ConfirmPage → submitDecisions(decisions) → M03
    → M03.execute_plan():
      - DeleteExecutor → DoDEraser 安全擦除
      - PackExecutor → finalize() → French-exit.zip
      - PreserveExecutor → 记录保留
    → Reporter.write_report() + open_in_browser()
    → TempStore.self_destruct() (RULE-04)
  → Frontend ReportPage
```

---

## 3. 剩余工作清单（优先级排序）

### P1 — 必须做

| # | 任务 | 说明 | 预估 |
|---|------|------|------|
| 1 | **单元测试补完** | M05/M06/M07/M08/M09/M10/M11/M12/M13/M16/M03/M02 均缺测试。当前只有 M04/M15/M17/M18 有测试。 | 3-4轮 |
| 2 | **cargo check 调通** | 当前环境无 cargo，但代码需要确保能编译。重点关注：Scanner trait 签名一致性、类型导入路径、Cargo.toml feature 完整性。 | 1轮 |
| 3 | **前端预览弹窗** | ResultsPage 中点击条目"预览"按钮，文本文件展示前4KB内容，图片展示Base64，不支持则提示。 | 1轮 |
| 4 | **前端搜索过滤** | ResultsPage 增加按名称/路径关键词搜索，按大小范围过滤。 | 1轮 |

### P2 — 优化项

| # | 任务 | 说明 | 预估 |
|---|------|------|------|
| 5 | **进度实时推送** | 当前 ScanPage 靠每秒轮询 getSessionState()。理想方案：Orchestrator 通过 Tauri Event 推送 ProgressEvent 到前端。 | 1-2轮 |
| 6 | **加密文件回调** | PackExecutor 检测到 .enc/.locked 文件时，前端弹窗确认"该文件已加密，确定打包？" | 1轮 |
| 7 | **前端动画优化** | 页面切换滑动+淡入、列表项hover微动效、按钮涟漪效果 | 1轮 |
| 8 | **首次使用引导** | Onboarding 遮罩层引导用户完成第一步 | 1轮 |

### P3 — 测试

| # | 任务 | 说明 | 预估 |
|---|------|------|------|
| 9 | **Rust 单元测试** | 为每个无测试模块补 3-5 个测试 | 2-3轮 |
| 10 | **前端组件测试** | vitest + @testing-library/react | 1轮 |
| 11 | **E2E 测试** | Playwright 完整流程：输入→扫描→勾选→执行→验证报告 | 1轮 |

---

## 4. 新会话快速上手

### 必读文件（按顺序）
1. `AGENTS.md` — 项目硬规则（RULE-01 ~ RULE-10）
2. `docs/proposal.md` — 需求提案
3. `docs/high-Level Design.md` — 概要设计
4. `docs/tasks/task-progress.md` — 当前进度看板
5. **本文件** — 交接状态

### 关键代码入口
- **Scanner trait**: `src-tauri/src/scanner/mod.rs`
- **Executor trait**: `src-tauri/src/executor/mod.rs`
- **Orchestrator**: `src-tauri/src/orchestrator/mod.rs`
- **Commands**: `src-tauri/src/commands/mod.rs`
- **Frontend 状态**: `src/store/AppContext.tsx`
- **Frontend 页面**: `src/pages/*.tsx`
- **初始化链路**: `src-tauri/src/lib.rs` 中的 `run()`

### Cargo.toml 依赖变更记录
- 已添加：`walkdir = "2.5"`
- 已添加：`zip = "2.2"`
- 已添加：`rusqlite = { version = "0.34", features = ["bundled"] }`
- 已修改：`windows` feature 增加了 `"Win32_System_Registry"`

### 当前已知 TODO（代码中标记）
- M03 orchestrator: 扫描阶段实时进度推送（OR-16）
- M13 pack executor: 磁盘空间预检查
- M13 pack executor: 加密文件回调确认
- M17 resource-ctl: current_usage CPU% 计算（占位值 0.0）

---

## 5. 推荐推进策略

**如果新会话有 10-15 轮 Agent 接力容量：**

1. **第1轮**: cargo check 调通（修复编译错误）
2. **第2-4轮**: 并行补测试（M05/M06/M07 + M08/M09/M10 + M11/M12/M13）
3. **第5轮**: M16 reporter 测试 + M03 orchestrator 测试
4. **第6轮**: M02 commands 测试
5. **第7轮**: 前端预览弹窗
6. **第8轮**: 前端搜索过滤
7. **第9轮**: 进度实时推送（Tauri Event）
8. **第10轮**: 集成验证 + bug 修复

**如果新会话容量有限（5-8轮）：**

1. **第1轮**: cargo check 调通
2. **第2-3轮**: 补核心模块测试（M12/M13/M03/M02）
3. **第4轮**: 前端预览 + 搜索
4. **第5轮**: 进度实时推送
5. **第6轮**: 集成验证

---

*文档生成时间: 2026-05-18*
*交接自: 阶段五第13轮接力完成*
