# Arcane Codex 开发规范

## 技术栈
- **前端**: React 18 + TypeScript + Vite + Tailwind CSS + motion
- **状态管理**: Zustand
- **后端**: Rust + Tauri 2.0 + tokio + SQLite
- **数据库**: SQLite + rusqlite + r2d2 连接池
- **图像处理**: image + kamadak-exif
- **测试**: Vitest (前端), cargo test (Rust)

## 代码规范

### Rust
- 所有异步函数返回 `Result<T, AppError>`
- Tauri Command 必须用 `#[tauri::command]` 宏
- 缩略图生成必须在 `tokio::spawn_blocking` 中执行
- 统一错误处理：使用 `thiserror` + `anyhow`

### TypeScript/React
- 函数式组件 + Hooks
- 统一错误处理：使用 `errorMap` 映射
- 组件文件命名：`PascalCase.tsx`
- 状态管理：Zustand（避免 Context 滥用）

## 关键路径
- 前端源码：`frontend/src/`
- 后端源码：`src-tauri/src/`
- 国际化：`frontend/src/i18n/{zh,en}.json`
- 缩略图存储：`%APPDATA%\ArcaneCodex\thumbnails\`

## 常用命令

```bash
# 开发
npm run tauri dev

# 构建
npm run build

# 前端测试
cd frontend && npm run test

# Rust 测试
cd src-tauri && cargo test

# 类型检查
npx tsc --noEmit

# Rust 格式化
cargo fmt
```

## 项目规则
- 禁止编造不存在的依赖包或 CLI 参数
- 所有外部 API 调用必须搜索官方文档验证
- 修改范围外文件前必须说明理由并获得批准
- 每个子任务完成后运行 Lint → Typecheck → Test

## 数据库迁移版本
- v1: 初始 schema（images、tags、image_tags、search_index、task_queue、app_config）
- v2: ComfyUI 生成支持
- v3: 叙事锚点（narratives、semantic_edges）
- v4: 多 Provider 支持（ai_provider、settings 表）
- v5: AI 标签状态分级（ai_tag_status、calibration 相关表）

## AI 推理配置
- 默认 Provider: LM Studio (http://127.0.0.1:1234)
- 默认模型：Qwen2.5-VL-7B-Instruct
- 超时：60 秒
- 重试策略：最多 3 次，指数退避（2s → 4s → 8s）
