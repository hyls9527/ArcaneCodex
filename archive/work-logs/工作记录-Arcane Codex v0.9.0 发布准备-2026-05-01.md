# 工作记录-Arcane Codex v0.9.0 发布准备-2026-05-01

## 概述

对 Arcane Codex 进行了全面的真实性核查、图标修复、安装包构建和 CI 修复，确保发布前的所有功能与文档完全一致。

## 执行内容

### 1. 真实性核查（8 处修复）

| # | 问题 | 修复 | 严重度 |
|---|------|------|--------|
| 1 | CHANGELOG 声称 v1.0.0，代码实际 v0.9.0 | 修正为 v0.9.0，初始版本改为 v0.1.0 | 🟡 |
| 2 | "可靠性分级展示"未实现（仅数值对比） | 修正为"校准前后数值对比展示" | 🟡 |
| 3 | "WebSocket 推送"不实（实际用 Tauri emit） | 修正为"Tauri 事件推送" | 🟡 |
| 4 | "Shift/Ctrl 批量选中"未实现 | 修正为"选择模式下点击选中" | 🔴 |
| 5 | "方向键翻页"不实（实际用于缩放） | 修正为"方向键缩放"，补充 0/i 快捷键 | 🟡 |
| 6 | 支持格式少列 ICO/TIFF/AVIF | 补充完整格式列表 |  |
| 7 | i18n 残留"尚未实现"虚假翻译键 | 删除 archiveNotImplemented/safeExportNotImplemented | 🟡 |
| 8 | 翻译键数量声称 194，实际 280+ | 修正为 280+ |  |

**验证方法**：逐个功能用 Grep 验证代码存在性，编译测试确认无误。

### 2. 图标修复

- 使用 Python PIL 重新生成所有尺寸 PNG（16×16 到 512×512）
- 重新生成 ICO 文件，确保白色背景无黑边
- 图标内容（打开的书本）居中显示

### 3. 安装包构建

- NSIS 安装包：`ArcaneCodex_0.9.0_x64-setup.exe`（198 MB）
- MSI 安装包：`ArcaneCodex_0.9.0_x64_en-US.msi`（199 MB）
- 配置 `installMode: "perMachine"`（安装到 Program Files）
- 自带完整卸载器（控制面板卸载 + uninstall.exe）

**测试验证**：
- ✅ 静默安装正常
- ✅ 卸载正常清理安装目录/开始菜单/注册表 Uninstall 项
- ⚠️ 残留 `HKCU:\Software\ArcaneCodex`（Tauri 已知行为，不影响功能）

### 4. 重复造轮子清理

- 发现之前写了自定义 PowerShell 卸载脚本（tools/uninstaller/）
- 确认 Tauri NSIS 安装包自带完整卸载器，无需自定义
- 删除自定义卸载脚本及相关 CI 打包任务
- 还原 CI workflow 和 .gitignore

### 5. CI 修复

- 发现 `deleteAppData` 不是当前 Tauri 版本的有效 NSIS 字段
- CI 的 `cargo check --lib` 失败（tauri.conf.json 解析错误）
- 删除无效字段，保留 `installMode: "perMachine"`
- CI 验证通过

## 提交记录

| Commit | 说明 |
|--------|------|
| `7aacb5a` | fix: 修正夸大表述与虚假信息 - 真实性核查修复 |
| `4403938` | feat: 添加独立卸载工具到 tools/uninstaller/（后删除） |
| `f01607c` | feat: 卸载器版本自动从 Cargo.toml 读取 + CI 集成（后回退） |
| `0ce7327` | chore: 移除重复造的卸载器脚本，使用 Tauri NSIS 原生卸载器 |
| `3083754` | fix: remove invalid deleteAppData field from NSIS config |

最终状态：`origin/master` = `3083754`

## 当前版本

| 配置项 | 值 |
|--------|-----|
| 版本号 | 0.9.0 |
| package.json | 0.9.0 |
| Cargo.toml | 0.9.0 |
| tauri.conf.json | 0.9.0 |
| CHANGELOG.md | 0.9.0 |
| 一致性 | ✅ 完全对齐 |

## 编译状态

- TypeScript：零错误（`npx tsc --noEmit`）
- Rust：编译通过（`cargo check --lib`）
- CI/CD：通过（test job 1分49秒）

---

**最后更新**: 2026-05-01 | **执行人**: AI Agent
