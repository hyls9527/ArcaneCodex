# 项目经验与教训 (Learnings & Patterns)

> **Agent 必读**: 每次开始工作前，必须阅读此文件。如果在执行中发现了新的"坑"或"最佳实践"，必须追加记录于此。这是你的长期记忆，防止在不同 Session 中重复犯错。

## 1. 代码风格与规范 (Conventions)
- Tauri Command 函数必须使用 `#[tauri::command]` 宏标注
- 所有异步函数返回 `Result<T, AppError>` 统一错误类型
- React 组件统一使用 Functional Component + TypeScript
- 状态管理使用 Zustand，避免 Context 滥用

## 2. 避坑指南 (Gotchas)
- ⚠️ Tauri 2.x 配置文件中 `app.windows` 替代了 v1 的 `tauri.windows`
- ⚠️ `rusqlite` 的 `bundled` 特性会自动编译 SQLite，无需系统安装
- ⚠️ 缩略图生成必须在 `tokio::spawn_blocking` 中执行，避免阻塞 async runtime
- ⚠️ LM Studio API 超时设置为 60 秒，7B 模型推理可能较慢
- ⚠️ Windows 路径使用 `\\` 分隔符，Rust 中用 `PathBuf` 处理跨平台

## 3. 常用命令速查 (Shortcuts)
- 开发模式: `cargo tauri dev`
- 生产构建: `cargo tauri build`
- 前端开发: `cd frontend && pnpm dev`
- Rust 测试: `cd src-tauri && cargo test`
- 前端测试: `cd frontend && pnpm test`
