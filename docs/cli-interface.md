# ArcaneCodex CLI Interface Design

> **Design Principle**: Making ArcaneCodex Agent-Native through structured CLI interface.
> **Inspired by**: CLI-Anything (HKUDS) - "Today's Software Serves Humans. Tomorrow's Users will be Agents."

## CLI Command Groups

### 1. Image Management (`ac image`)

```bash
# Import images
ac image import --path ./photos --recursive
ac image import --paths ./photo1.jpg ./photo2.jpg
ac image import --folder D:\Pictures\2025

# List/query images
ac image list --page 1 --limit 50 --format json
ac image list --filter "date:2025-01-01..2025-12-31"
ac image list --filter "category:风景"
ac image get --id 12345 --format json

# Delete images
ac image delete --id 12345
ac image delete --ids 12345,12346,12347
```

### 2. AI Processing (`ac ai`)

```bash
# Start AI processing
ac ai start --concurrency 3 --timeout 60
ac ai start --target all --model llava
ac ai start --target pending --resume

# Control queue
ac ai pause
ac ai resume
ac ai cancel

# Check status
ac ai status --format json
ac ai status --verbose

# Retry failed
ac ai retry --failed-only
ac ai retry --id 12345
```

### 3. Search (`ac search`)

```bash
# Semantic search
ac search --query "日落 海滩" --limit 20
ac search --query "family vacation" --filter "date:2024"
ac search --query "猫" --category "动物" --format json

# Rebuild index
ac search rebuild --all
ac search rebuild --modified-since 2025-01-01
```

### 4. Deduplication (`ac dedup`)

```bash
# Scan duplicates
ac dedup scan --threshold 90
ac dedup scan --threshold 85 --batch-size 1000
ac dedup scan --output ./duplicates.json

# Delete duplicates
ac dedup delete --keep-highest-res --confirm
ac dedup delete --strategy keep-oldest --dry-run
```

### 5. System (`ac system`)

```bash
# Database operations
ac system db-info --format json
ac system db-backup --output ./backup.zip
ac system db-restore --input ./backup.zip

# Configuration
ac system config get --key lm_studio_url
ac system config set --key lm_studio_url --value http://localhost:1234
ac system config list --format json

# Health check
ac system health
ac system health --verbose
```

## JSON Output Format (Agent-Friendly)

All commands support `--format json` for structured output:

```json
{
  "command": "image list",
  "status": "success",
  "data": {
    "images": [...],
    "total": 1500,
    "page": 1,
    "per_page": 50
  },
  "meta": {
    "execution_time_ms": 45,
    "timestamp": "2025-04-27T14:30:00Z"
  }
}
```

## Error Handling

Standard error format for Agent consumption:

```json
{
  "command": "image import",
  "status": "error",
  "error": {
    "code": "FILE_NOT_FOUND",
    "message": "File not found: D:\\photos\\missing.jpg",
    "details": {
      "file_path": "D:\\photos\\missing.jpg",
      "suggestion": "Check file path and permissions"
    }
  }
}
```

## REPL Mode

Interactive mode for human testing:

```bash
$ ac repl
ArcaneCodex CLI v0.1.0 - Agent-Native Interface
> image list --limit 5
Found 5 images...
> search --query "日落"
Found 23 matching images...
> exit
```

## SKILL.md (Agent Discovery)

```yaml
---
name: arcanecodex
description: "Local image knowledge base with AI tagging, semantic search, and deduplication"
commands:
  - name: image import
    description: "Import images from path/folder"
    args: [--path, --recursive]
  - name: image list
    description: "List images with pagination and filters"
    args: [--page, --limit, --filter, --format]
  - name: ai start
    description: "Start AI auto-tagging queue"
    args: [--concurrency, --timeout]
  - name: search
    description: "Semantic search with jieba tokenization"
    args: [--query, --limit, --filter]
  - name: dedup scan
    description: "Scan for duplicate images using pHash"
    args: [--threshold]
---
```

## Implementation Strategy

### Phase 1: Core CLI (Priority)
1. Implement `ac image list` + `ac image import`
2. Implement `ac search`
3. Add `--format json` support

### Phase 2: AI Integration
4. Implement `ac ai start/status/pause/resume`
5. Add Tauri Event streaming for progress

### Phase 3: Advanced Features
6. Implement `ac dedup scan/delete`
7. Add REPL mode
8. Generate SKILL.md

## Agent Workflow Example

```bash
# Agent-driven workflow
ac image import --path ./new-photos --recursive
ac ai start --concurrency 3
# Wait for completion via event stream
ac search --query "风景" --format json > ./results.json
ac dedup scan --threshold 90
ac dedup delete --keep-highest-res --confirm
ac system db-backup --output ./backup.zip
```

This transforms ArcaneCodex from a GUI-only app to an **Agent-Native platform** where AI can fully control all operations through deterministic CLI commands.
