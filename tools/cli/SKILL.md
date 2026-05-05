# ArcaneCodex CLI - SKILL.md

## 📋 技能描述
ArcaneCodex CLI 是一个 Agent 原生的本地图像知识管理工具，通过结构化命令行接口让 AI Agent 能够完全控制图像导入、AI 标签、语义搜索和去重等操作。

## ️ 命令组

### 图像管理 (`ac image`)
- `ac image import --path <路径> [--recursive]` - 导入图像
- `ac image list [--page <页码>] [--limit <数量>] [--filter <过滤>]` - 列出图像
- `ac image delete --id <ID> [--confirm]` - 删除图像

### AI 处理 (`ac ai`)
- `ac ai start [--concurrency <数量>] [--target <目标>]` - 启动 AI 标签
- `ac ai status` - 查看处理状态
- `ac ai pause` / `ac ai resume` - 控制处理队列

### 语义搜索 (`ac search`)
- `ac search --query <查询> [--limit <数量>] [--category <类别>]` - 搜索图像
- `ac search rebuild --all` - 重建搜索索引

### 去重 (`ac dedup`)
- `ac dedup scan --threshold <阈值>` - 扫描重复图像
- `ac dedup delete --strategy <策略> [--confirm]` - 删除重复图像

### 系统管理 (`ac system`)
- `ac system health [--verbose]` - 健康检查
- `ac system config [--get|--set|--list]` - 配置管理
- `ac system db-backup --output <路径>` - 数据库备份

## 📝 使用示例

### Agent 工作流
```bash
# 1. 导入新图像
ac image import --path ./new-photos --recursive

# 2. 启动 AI 自动标签
ac ai start --concurrency 3

# 3. 等待处理完成后搜索
ac search --query "日落 海滩" --format json > ./results.json

# 4. 去重扫描
ac dedup scan --threshold 90

# 5. 删除重复项
ac dedup delete --strategy keep-highest-res --confirm

# 6. 备份数据库
ac system db-backup --output ./backup.zip
```

### JSON 输出 (Agent 友好)
所有命令支持 `--format json` 输出结构化数据：
```json
{
  "command": "search",
  "status": "success",
  "data": { "results": [...] },
  "meta": { "execution_time_ms": 45 }
}
```

## 🤖 Agent 使用指南
1. **发现能力**: 使用 `ac --help` 查看所有可用命令
2. **结构化输出**: 始终使用 `--format json` 获取机器可读数据
3. **错误处理**: 检查返回 JSON 中的 `status` 字段
4. **进度跟踪**: 使用 `ac ai status` 监控长时间运行的任务
5. **安全操作**: 删除操作需要 `--confirm` 参数确认

## ⚙️ 安装
```bash
cd arcane-codex-cli
pip install -e .
```
