# Changelog

## [1.0.0-rc] - 2026-05-03

### 🎉 首个候选发布版本

Arcane Codex 是一个本地优先的图片知识库应用，支持 AI 自动打标、智能搜索和多推理源切换。

---

### 🏗️ 核心架构

- **Tauri v2** 桌面应用框架，Rust 后端 + React/TypeScript 前端
- **SQLite** 本地数据库，含完整 migration 系统 (v1→v5)
- **r2d2** 数据库连接池
- **Tokio** 异步运行时

### 🤖 AI 推理引擎

- 多推理源支持：LM Studio / Ollama / Hermes / 智谱 / OpenAI / OpenRouter
- 模型自动发现：扫描本地服务端口，无需手动输入模型名
- AI 打标三级状态：verified / provisional / rejected
- 置信度校准：数据库表已创建（calibration_samples/reports/curves），ECE 计算逻辑待实现
- 独立验证层：CLIP embedding + ONNX Runtime（已集成到主流程，需 ONNX 模型文件运行；知识图谱引擎依赖此模块）
- 用户反馈闭环：修正记录 API + 错误模式库
- 数据库 v5 migration：新增校准/修正/错误模式表

### 🔒 安全加固

- API Key AES-256-GCM 加密存储
- Zip Slip 攻击防护（部分实现，解压路径验证存在但未显式检查路径逃逸）
- 路径清理与验证
- CSP 内容安全策略强化
- Tauri 安全配置

### ⚡ 性能优化

- 搜索缓存（HashMap + TTL 5 分钟）
- 磁盘空间预检
- 高效智能去重算法
- 虚拟滚动 (大数据量图片列表)
- 响应式网格布局
- Vite 生产构建优化 + Rust release profile 优化 (LTO + codegen-units)

### 🎨 前端

- React 18 + TypeScript
- 国际化 (i18n) 支持，翻译键补全
- 控制台清理，生产环境零日志泄漏
- 组件 Props 一致性修复
- Store 错误处理完善

### 🛠️ 代码质量

- ESLint 错误/警告修复 27+
- 硬编码中文替换 (aria-label/组件) 15+
- Rust unwrap() 消除 (生产代码) 5 处
- Rust 测试编译修复 3 文件
- UTF-8 编码修复
- Format string 修复
- 重复代码移除

### 📦 依赖管理

- zip 0.6 → 2.x 迁移
- rust-version 1.70 → 1.75
- mockito HTTP mock 测试支持
- Cargo release profile 优化

### 🔧 功能模块

- 图片导入与管理
- AI 自动打标
- 智能搜索与过滤
- 数据备份与恢复 (含版本兼容性检查)
- 图片浏览 (虚拟滚动 + 响应式网格)
- 设置页面 (AI 配置 / 存储配置)
- 错误处理与降级模式
- 边界场景覆盖

### ⚠️ 已知限制

- rustc 1.95.0 + LLVM 22.1.2 在 Windows 上存在 STATUS_ACCESS_VIOLATION bug，cargo check/test 暂时阻塞
- CLIP Python sidecar 集成架构已预留，待实现
- 部分测试依赖 LM Studio 本地服务
