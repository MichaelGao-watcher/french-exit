# 任务拆分：M10 — scanner-devtools（开发工具扫描器）

> 职责：扫描 Git 配置、SSH 密钥、IDE 配置、GitHub CLI 配置；区分"安全清除"和"有风险"标记。

---

## 前置条件

- [ ] INF-04（共享数据结构定义完成）
- [ ] M04 scanner-registry（Scanner trait 定义完成）

## 推荐开发顺序

1. SD-01 ~ SD-05（各工具扫描）
2. SD-06（分类标记）
3. SD-07（P2 扩展）
4. SD-08 ~ SD-09（测试）

---

## 子任务清单

### P1 — 必须做

- [ ] **SD-01** 实现 Git 全局配置扫描
  - 路径：`%USERPROFILE%\.gitconfig`
  - 提取 `user.name`, `user.email`
  - 如果文件存在且包含个人信息，生成 `TraceItem`
  - `suggested_action = Delete`（安全清除，仅配置文件）
  - 测试点：构造假 `.gitconfig` 验证提取

- [ ] **SD-02** 实现 SSH 密钥扫描
  - 路径：`%USERPROFILE%\.ssh\`
  - 检测文件：`id_rsa`, `id_ed25519`, `id_ecdsa`, `*.pub`, `config`, `known_hosts`
  - 私钥文件标注 `"SSH 私钥，删除后需重新生成"`
  - `known_hosts` 标注 `"包含您连接过的服务器记录"`
  - 测试点：虚拟 `.ssh` 目录验证

- [ ] **SD-03** 实现 VS Code 配置扫描
  - 路径：`%APPDATA%\Code\User\`
  - 检测：`settings.json`, `keybindings.json`, `globalStorage\`（含插件数据）
  - 特别关注：Kimi Code / Cursor / GitHub Copilot 等插件历史
  - 测试点：虚拟 VS Code 配置目录验证

- [ ] **SD-04** 实现 JetBrains 配置扫描
  - 路径：`%APPDATA%\JetBrains\`
  - 检测各 IDE（IDEA, PyCharm, WebStorm 等）的配置目录
  - 提取 `idea.key`（许可证信息）、`options/` 目录
  - 测试点：虚拟 JetBrains 目录验证

- [ ] **SD-05** 实现 GitHub CLI 配置扫描
  - 路径：`%LOCALAPPDATA%\GitHub CLI\`
  - 或 `%APPDATA%\GitHub CLI\`
  - 检测 `config.yml`, `hosts.yml`, 凭证缓存
  - 标注 `"安全清除：仅删除本地配置文件"`
  - 测试点：虚拟 GitHub CLI 目录验证

- [ ] **SD-06** 实现"安全清除"与"有风险"分类标记
  - **安全清除**：Git 配置、GitHub CLI 配置、IDE 配置（仅本地文件，不涉及系统级修改）
  - **有风险**：SSH 私钥（删除后影响 Git 连接）、IDE 许可证（可能需重新激活）
  - 有风险条目附加 `risk_note` 说明具体影响
  - 测试点：验证分类逻辑正确

- [ ] **SD-07** 【测试】虚拟配置文件验证各工具扫描结果
  - 构造 `.gitconfig`, `.ssh/`, `Code/User/`, `JetBrains/`, `GitHub CLI/` 全套假数据
  - 验证返回的 `TraceItem` 数量和字段正确

- [ ] **SD-08** 【测试】验证 GitHub CLI 配置被正确分类为"安全清除"
  - 断言 `risk_note` 不含"可能与其他工具共用"

- [ ] **SD-09** 【测试】验证未安装的工具被跳过
  - 无 `.gitconfig` 时，结果中无 Git 相关条目

### P2 — 后续迭代

- [ ] **SD-10** 实现 Docker 容器数据检测
  - 路径：`%USERPROFILE%\.docker\`
  - 检测容器、镜像、卷的磁盘占用
  - 标注 `"Docker 数据可能包含开发环境敏感信息"`

- [ ] **SD-11** 实现虚拟机镜像检测（VMware / VirtualBox / WSL）
  - 检测 `.vmdk`, `.vdi`, `ext4.vhdx` 等文件位置
  - 返回总大小汇总

---

## 测试点汇总

| 编号 | 测试内容 | 优先级 |
|------|---------|--------|
| SD-07 | 全套虚拟配置扫描 | P1 |
| SD-08 | GitHub CLI 安全分类 | P1 |
| SD-09 | 未安装工具跳过 | P1 |
| SD-10 | Docker 数据检测 | P2 |
| SD-11 | 虚拟机镜像检测 | P2 |

---

## 依赖关系

```
被依赖方：M04 scanner-registry（注册为 Scanner 实例）
依赖方：M04（Scanner trait）
```
