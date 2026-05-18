# 任务拆分：M07 — scanner-chat（聊天软件扫描器）

> 职责：检测并扫描微信、QQ、钉钉、飞书、企业微信的本地数据；微信整目录标记为建议处理（RULE-03）。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SC-01 ~ SC-05（各聊天软件检测）
2. SC-06 ~ SC-08（特殊处理与数据提取）
3. SC-09 ~ SC-11（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SC-01** 实现微信检测与本地数据目录定位
  - 检测路径 1：`%USERPROFILE%\Documents\WeChat Files\`
  - 检测路径 2：`%USERPROFILE%\Documents\WeChatData\`
  - 检测标志：目录存在且包含以 `wxid_` 开头的子目录
  - 读取子目录列表，每个 wxid 对应一个账号
  - 测试点：虚拟微信目录结构验证检测

- [ ] **SC-02** 实现 QQ 检测与本地数据目录定位
  - 检测路径：`%USERPROFILE%\Documents\Tencent Files\`
  - 或从注册表 `HKEY_CURRENT_USER\Software\Tencent\QQ` 读取
  - 测试点：虚拟 QQ 目录验证

- [ ] **SC-03** 实现钉钉检测与本地数据目录定位
  - 检测路径：`%LOCALAPPDATA%\DingTalk\`
  - 测试点：虚拟目录验证

- [ ] **SC-04** 实现飞书检测与本地数据目录定位
  - 检测路径：`%LOCALAPPDATA%\Feishu\`
  - 或 `%LOCALAPPDATA%\Lark\`
  - 测试点：虚拟目录验证

- [ ] **SC-05** 实现企业微信检测与本地数据目录定位
  - 检测路径：`%USERPROFILE%\Documents\WXWork\`
  - 测试点：虚拟目录验证

- [ ] **SC-06** 实现微信整目录标记为 `DeleteOrPack`
  - 每个 wxid 目录生成**一条** `TraceItem`
  - `category = TraceCategory::Chat`
  - `suggested_action = Some(Action::DeleteOrPack)`
  - `risk_note = Some("微信聊天记录属于私人内容，建议处理")`
  - 计算目录总大小（递归）
  - 这是 RULE-03 硬规则，必须测试
  - 测试点：验证微信 TraceItem 字段

- [ ] **SC-07** 实现其他聊天软件按文件类型扫描
  - QQ / 钉钉 / 飞书 / 企业微信：列出数据库文件、图片/视频缓存、接收文件
  - 按入职日期过滤修改时间
  - 每条文件生成独立 `TraceItem`
  - 测试点：虚拟文件验证扫描结果

- [ ] **SC-08** 实现文件传输助手接收文件识别
  - 微信：`WeChat Files\wxid_xxx\FileStorage\File\{年月}\`
  - QQ：`Tencent Files\QQ号\FileRecv\`
  - 单独标注 `risk_note = "可能包含工作文件，请确认后再处理"`
  - 测试点：验证 FileRecv 路径被正确识别

- [ ] **SC-09** 【测试】验证微信 `suggested_action = DeleteOrPack`
  - 直接断言 `assert_eq!(item.suggested_action, Some(Action::DeleteOrPack))`

- [ ] **SC-10** 【测试】验证聊天软件未安装时被跳过
  - 无对应目录时，Scanner 返回空 Vec

- [ ] **SC-11** 【测试】验证数据库/目录被占用时标记 `risk_note` 而非 panic
  - 用文件锁模拟微信运行中

### P2 — 后续迭代

- [ ] **SC-12** 实现聊天软件账号退出清单数据生成
  - 检测到的每个聊天软件账号加入 "待退出账号" 列表
  - 供 M16 reporter 生成跳转链接

- [ ] **SC-13** 实现聊天图片/视频预览支持
  - 为 M01 frontend 的 preview 功能提供解密/解码支持
  - 微信图片解密（如需要）属于 P2 范畴

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SC-09 | 微信 suggested_action = DeleteOrPack | P1 |
| SC-10 | 未安装软件跳过 | P1 |
| SC-11 | 文件占用容错 | P1 |
| SC-12 | 账号退出清单数据 | P2 |
| SC-13 | 图片预览支持 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
