# ArcaneCodex CLI Harness Architecture

## Overview
ArcaneCodex 是一个**本地图像知识管理**应用，需要将其转化为 Agent 原生工具。

## 核心能力映射

### 1. 图像管理 (Image Management)
- **导入**: `ac import --path ./photos --recursive`
- **查询**: `ac list --filter "category:风景" --format json`
- **删除**: `ac delete --ids 123,456`

### 2. AI 处理 (AI Processing)
- **自动标签**: `ac ai tag --concurrency 3`
- **状态查询**: `ac ai status --format json`
- **控制队列**: `ac ai pause/resume/cancel`

### 3. 语义搜索 (Semantic Search)
- **搜索**: `ac search --query "日落 海滩" --limit 20`
- **重建索引**: `ac search rebuild --all`

### 4. 去重 (Deduplication)
- **扫描**: `ac dedup scan --threshold 90`
- **删除**: `ac dedup delete --keep-highest-res --confirm`

### 5. 系统管理 (System)
- **配置**: `ac config get/set`
- **数据库**: `ac db backup/restore/info`
- **健康检查**: `ac system health`

## 技术实现策略

### 与 Tauri 后端集成
- 通过 Tauri 的 `invoke` 命令调用 Rust 后端
- 或使用 SQLite 直接查询（只读操作）
- 状态变更操作通过 Tauri Event 流式返回进度

### JSON 输出格式
所有命令支持 `--format json` 输出结构化数据：
```json
{
  "command": "search",
  "status": "success",
  "data": { "images": [...] },
  "meta": { "total": 1500, "page": 1 }
}
```

### 错误处理
标准化错误输出：
```json
{
  "command": "import",
  "status": "error",
  "error": {
    "code": "FILE_NOT_FOUND",
    "message": "路径不存在",
    "suggestion": "检查路径和权限"
  }
}
```

## 实施计划

### Phase 1: 基础 CLI (核心)
1. 实现 `ac list` + `ac import`
2. 实现 `ac search`
3. 添加 `--format json` 支持

### Phase 2: AI 集成
4. 实现 `ac ai start/status/pause/resume`
5. 添加 Tauri Event 流式进度

### Phase 3: 高级功能
6. 实现 `ac dedup scan/delete`
7. 添加 REPL 模式
8. 生成 SKILL.md

## Agent 工作流示例

```bash
# Agent 驱动的完整工作流
ac import --path ./new-photos --recursive
ac ai tag --concurrency 3
# 等待完成（通过事件流）
ac search --query "风景" --format json > ./results.json
ac dedup scan --threshold 90
ac dedup delete --keep-highest-res --confirm
ac db backup --output ./backup.zip
```

这将把 ArcaneCodex 从纯 GUI 应用转变为**Agent 原生平台**，AI 可以通过确定性 CLI 命令完全控制所有操作。
