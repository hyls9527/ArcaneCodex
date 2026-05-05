# Contributing to Arcane Codex

感谢你对 Arcane Codex 的兴趣！这个项目始于个人需求，但欢迎所有愿意一起完善的贡献者。

---

## 开发环境

### 前置要求

| 工具 | 版本 | 用途 |
|------|------|------|
| Node.js | 20+ | 前端构建 |
| Rust | 1.75+ | 后端编译 |
| Windows | 10+ | 当前主要支持平台 |

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/hyls9527/ArcaneCodex.git
cd ArcaneCodex

# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

---

## 开发工作流

### 分支命名规范

```
feature/简短描述    # 新功能
fix/问题描述        # Bug 修复
docs/文档内容       # 文档更新
perf/优化内容       # 性能优化
refactor/重构内容   # 代码重构
```

### 提交信息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型说明：**

| 类型 | 用途 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档更新 |
| `style` | 代码格式（不影响功能） |
| `refactor` | 重构 |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建/工具链 |

**示例：**

```
feat(ai): add OpenRouter provider support

- Implement OpenRouter client in inference module
- Add provider configuration UI
- Update settings validation

Closes #123
```

---

## 代码规范

### Rust

- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 检查（项目启用 `-D warnings`）
- 错误处理使用 `thiserror` / `anyhow`
- 异步函数返回 `Result<T, AppError>`
- Tauri Command 函数必须标注 `#[tauri::command]`

### TypeScript / React

- 使用项目 ESLint 配置
- 函数组件 + TypeScript，避免 `any`
- 状态管理优先使用 Zustand
- 组件 props 接口命名：`ComponentNameProps`

### 通用

- 注释使用中文
- 新功能必须包含测试
- 修改公共 API 必须更新 `docs/api.md`

---

## 提交前检查清单

```bash
# 1. Rust 编译检查
cd src-tauri && cargo check --all-targets

# 2. TypeScript 类型检查
cd frontend && npx tsc --noEmit

# 3. 代码格式化
cd src-tauri && cargo fmt --all
cd frontend && npm run lint

# 4. 运行测试
cd src-tauri && cargo test
cd frontend && npx vitest run
```

---

## 报告 Bug

使用 [Bug Report 模板](https://github.com/hyls9527/ArcaneCodex/issues/new?template=bug_report.md)，包含：

- 清晰的问题描述
- 复现步骤
- 期望 vs 实际行为
- 环境信息（Windows 版本、应用版本、AI 服务类型）
- 相关日志或截图

---

## 功能建议

使用 [Feature Request 模板](https://github.com/hyls9527/ArcaneCodex/issues/new?template=feature_request.md)，说明：

- 需求场景
- 为什么需要
- 可能的实现思路

---

## Pull Request 流程

1. Fork 仓库并创建功能分支
2. 开发并确保所有检查通过
3. 更新相关文档
4. 提交 PR，填写模板中的检查清单
5. 等待 CI 通过和代码审查

---

## 许可证

所有贡献均遵循 [MIT License](LICENSE)。

---

有问题？直接开 Issue 或 Discussion，不用客气。
