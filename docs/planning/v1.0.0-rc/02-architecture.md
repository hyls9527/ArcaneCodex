# 系统架构设计 (System Architecture)

## 1. 技术栈 (Tech Stack)

### 1.1 前端 (Frontend)
- **框架**: React 18 + TypeScript
- **UI 样式**: Tailwind CSS 3.4 + Framer Motion 11
- **状态管理**: Zustand 4.5 + React Query 5
- **虚拟滚动**: @tanstack/react-virtual 3.5
- **国际化**: i18next 23 + react-i18next 14
- **图标**: Lucide React 0.378
- **组件库**: Radix UI (Dialog/DropdownMenu/Toast)

### 1.2 后端 (Rust - Tauri Core)
- **运行时**: Tauri 2.x + tokio 1
- **数据库**: SQLite (rusqlite 0.31 bundled)
- **图像处理**: image 0.25 + img_hash 0.4
- **HTTP 客户端**: reqwest 0.12
- **中文分词**: jieba-rs 0.7
- **任务队列**: async-channel 2.2 + tokio::sync::Semaphore
- **错误处理**: thiserror 1 + anyhow 1
- **日志**: tracing 0.1 + tracing-subscriber 0.3

### 1.3 AI 推理 (Local)
- **服务**: LM Studio (localhost:1234/v1)
- **模型**: Qwen2.5-VL-7B-Instruct (GGUF)
- **协议**: OpenAI Chat Completions API 兼容

### 1.4 质量保障 (QA Stack)
- **单元测试**: cargo test (Rust) + Vitest (前端)
- **组件测试**: React Testing Library + Vitest
- **E2E 测试**: Playwright
- **打包**: Tauri 内置 NSIS (Windows)

## 2. 整体架构图 (Single-Binary Architecture)

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Tauri 2.x Desktop App                           │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Frontend Layer (WebView)                        │   │
│  │  ┌──────────┐ ┌──────────────┐ ┌──────────┐                │   │
│  │  │ React 18 │ │ Zustand +    │ │ React    │                │   │
│  │  │ + TSX    │ │ React Query  │ │ Virtual  │                │   │
│  │  │          │ │ (State)      │ │ Grid     │                │   │
│  │  └────┬─────┘ └──────┬───────┘ └────┬─────┘                │   │
│  │       └──────────────┼───────────────┘                      │   │
│  │              ▼                                              │   │
│  │  ┌──────────────────────────────────────────────────────┐  │   │
│  │  │           Tauri IPC Bridge (Commands + Events)        │  │   │
│  │  └──────────────────────┬───────────────────────────────┘  │   │
│  └─────────────────────────┼──────────────────────────────────┘   │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │              Backend Layer (Rust Core)                       │   │
│  │                                                             │   │
│  │  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐  │   │
│  │  │ File       │  │ Image      │  │ AI Task Queue       │  │   │
│  │  │ Manager    │  │ Processor  │  │ (async_channel)     │  │   │
│  │  │ (tokio/fs) │  │ (image)    │  │ + Semaphore (3)     │  │   │
│  │  └─────┬──────┘  └─────┬──────┘  └──────────┬──────────┘  │   │
│  │        │               │                     │              │   │
│  │        ▼               ▼                     ▼              │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │              Data Layer (SQLite)                     │   │   │
│  │  │  ┌──────────────┐  ┌─────────────────────────────┐  │   │   │
│  │  │  │ rusqlite     │  │ search_index (jieba 分词)    │  │   │   │
│  │  │  │ (Images)     │  │ (Inverted Index)            │  │   │   │
│  │  │  └──────────────┘  └─────────────────────────────┘  │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └────────────────────────────────────────────────────────────┘   │
│                            │                                      │
│                            ▼ (HTTP reqwest)                       │
│              ┌───────────────────────────────┐                   │
│              │    LM Studio Server (Local)    │                   │
│              │    localhost:1234/v1           │                   │
│              │    Qwen2.5-VL-7B-Instruct      │                   │
│              └───────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
```

## 3. 目录结构规范

```
e:\knowledge base\
├── .trae/
│   └── rules/
│       └── project_rules.md
│
├── src-tauri/                         # Rust 后端
│   ├── src/
│   │   ├── main.rs                    # Tauri 入口
│   │   ├── commands/                  # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── images.rs              # 图片导入/管理
│   │   │   ├── ai.rs                  # AI 分析
│   │   │   ├── search.rs              # 检索
│   │   │   └── export.rs              # 导出
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   ├── db.rs                  # 数据库连接 + Migration
│   │   │   ├── image.rs               # 图像处理 (缩略图/pHash)
│   │   │   ├── ai_queue.rs            # 任务队列
│   │   │   └── lm_studio.rs           # LM Studio 客户端
│   │   ├── models/                    # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── image.rs
│   │   │   └── task.rs
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── hash.rs                # SHA256/pHash
│   │       └── error.rs               # 错误类型
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── icons/
│
├── frontend/                          # React 前端
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.tsx
│   │   │   │   ├── TopBar.tsx
│   │   │   │   └── MainLayout.tsx
│   │   │   ├── gallery/
│   │   │   │   ├── ImageGrid.tsx
│   │   │   │   ├── ImageCard.tsx
│   │   │   │   └── ImageViewer.tsx
│   │   │   ├── ai/
│   │   │   │   ├── AIProgressPanel.tsx
│   │   │   │   └── AITaskQueue.tsx
│   │   │   └── ui/
│   │   ├── stores/
│   │   │   ├── imageStore.ts
│   │   │   └── aiStore.ts
│   │   ├── hooks/
│   │   │   ├── useImages.ts
│   │   │   └── useAI.ts
│   │   ├── i18n/
│   │   │   ├── index.ts
│   │   │   ├── en.json
│   │   │   └── zh.json
│   │   └── styles/
│   │       ├── globals.css
│   │       └── theme.ts
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   └── vite.config.ts
│
├── test_assets/
│   ├── landscape.jpg
│   ├── portrait.jpg
│   └── object.jpg
│
├── docs/
│   └── planning/
│       └── v1.0.0-rc/
│           ├── 01-requirements.md
│           ├── 02-architecture.md
│           ├── 04-ralph-tasks.md
│           ├── 05-test-plan.md
│           └── 06-learnings.md
│
├── final-architecture.md
├── pre-requirements.md
├── RALPH_STATE.md
└── README.md
```

## 4. 数据模型 (Data Model)

### 4.1 images (图片主表)
| 字段名 | 类型 | 必填 | 说明 |
|---|---|---|---|
| id | INTEGER | 是 | 主键，自增 |
| file_path | TEXT | 是 | 唯一，文件绝对路径 |
| file_name | TEXT | 是 | 文件名 |
| file_size | INTEGER | 是 | 文件大小 (bytes) |
| file_hash | TEXT | 否 | SHA256 (精确去重) |
| mime_type | TEXT | 否 | MIME 类型 |
| width | INTEGER | 否 | 图片宽度 (px) |
| height | INTEGER | 否 | 图片高度 (px) |
| thumbnail_path | TEXT | 否 | 缩略图路径 |
| phash | TEXT | 否 | 感知哈希 (相似去重) |
| exif_data | JSON | 否 | EXIF 元数据 |
| ai_status | TEXT | 是 | pending/processing/completed/failed |
| ai_tags | JSON | 否 | AI 生成标签数组 |
| ai_description | TEXT | 否 | AI 自然语言描述 |
| ai_category | TEXT | 否 | AI 分类 |
| ai_confidence | REAL | 否 | 置信度 0.0-1.0 |
| ai_model | TEXT | 否 | 模型版本标识 |
| ai_processed_at | DATETIME | 否 | AI 处理时间 |
| ai_error_message | TEXT | 否 | 错误信息 |
| ai_retry_count | INTEGER | 是 | 重试次数，默认 0 |
| source | TEXT | 是 | 来源 (manual/import/watcher/agent) |
| created_at | DATETIME | 是 | 创建时间 |
| updated_at | DATETIME | 是 | 更新时间 |

### 4.2 tags (标签索引表)
| 字段名 | 类型 | 必填 | 说明 |
|---|---|---|---|
| id | INTEGER | 是 | 主键 |
| name | TEXT | 是 | 标签名 (UNIQUE, NOCASE) |
| count | INTEGER | 是 | 使用次数 |

### 4.3 image_tags (图片-标签关联表)
| 字段名 | 类型 | 必填 | 说明 |
|---|---|---|---|
| image_id | INTEGER | 是 | 外键 → images.id |
| tag_id | INTEGER | 是 | 外键 → tags.id |

### 4.4 search_index (搜索倒排索引表)
| 字段名 | 类型 | 必填 | 说明 |
|---|---|---|---|
| id | INTEGER | 是 | 主键 |
| term | TEXT | 是 | jieba 分词后的词条 |
| image_id | INTEGER | 是 | 外键 → images.id |
| field | TEXT | 是 | 'description'/'tags'/'category' |
| position | INTEGER | 否 | 词条位置 (用于高亮) |

### 4.5 task_queue (任务队列表)
| 字段名 | 类型 | 必填 | 说明 |
|---|---|---|---|
| id | INTEGER | 是 | 主键 |
| image_id | INTEGER | 是 | 外键 → images.id |
| task_type | TEXT | 是 | auto_tag/dedup/export |
| status | TEXT | 是 | pending/running/completed/failed |
| priority | INTEGER | 是 | 优先级 (0=普通, 1=高) |
| retry_count | INTEGER | 是 | 已重试次数 |
| max_retries | INTEGER | 是 | 最大重试次数，默认 3 |
| error_message | TEXT | 否 | 错误信息 |
| created_at | DATETIME | 是 | 创建时间 |
| started_at | DATETIME | 否 | 开始时间 |
| completed_at | DATETIME | 否 | 完成时间 |

### 4.6 app_config (系统配置表)
| 字段名 | 类型 | 必填 | 说明 |
|---|---|---|---|
| key | TEXT | 是 | 配置键 (主键) |
| value | TEXT | 是 | 配置值 |
| updated_at | DATETIME | 是 | 更新时间 |

## 5. Tauri Commands API 定义

### 5.1 图片管理模块

#### 5.1.1 import_images
- **Command**: `import_images(file_paths: Vec<String>) -> Result<ImportResult>`
- **描述**: 导入单个或多个图片文件
- **Request**:
```rust
{
    "file_paths": ["C:\\Photos\\sunset.jpg", "C:\\Photos\\beach.jpg"]
}
```
- **Response**:
```rust
{
    "success_count": 2,
    "duplicate_count": 0,
    "error_count": 0,
    "image_ids": [1, 2]
}
```

#### 5.1.2 get_images
- **Command**: `get_images(page: u32, page_size: u32, filters: Option<Filters>) -> Result<Vec<ImageDTO>>`
- **描述**: 分页获取图片列表
- **Response**: 返回图片元数据 + 缩略图路径数组

#### 5.1.3 get_image_detail
- **Command**: `get_image_detail(id: i64) -> Result<ImageDetailDTO>`
- **描述**: 获取单张图片详情 (含 AI 分析结果)

#### 5.1.4 delete_images
- **Command**: `delete_images(ids: Vec<i64>) -> Result<usize>`
- **描述**: 批量删除图片 (同时清理数据库记录)

### 5.2 AI 分析模块

#### 5.2.1 start_ai_processing
- **Command**: `start_ai_processing(concurrency: Option<u32>) -> Result<()>`
- **描述**: 启动 AI 批量处理 (后台异步)
- **参数**: 并发数，默认 3

#### 5.2.2 pause_ai_processing
- **Command**: `pause_ai_processing() -> Result<()>`
- **描述**: 暂停 AI 处理队列

#### 5.2.3 resume_ai_processing
- **Command**: `resume_ai_processing() -> Result<()>`
- **描述**: 恢复 AI 处理队列

#### 5.2.4 get_ai_status
- **Command**: `get_ai_status() -> Result<AIStatusDTO>`
- **描述**: 获取 AI 处理实时状态
- **Response**:
```rust
{
    "status": "processing",
    "total": 5231,
    "completed": 3298,
    "failed": 142,
    "retrying": 15,
    "eta_seconds": 720
}
```

#### 5.2.5 retry_failed_ai
- **Command**: `retry_failed_ai() -> Result<usize>`
- **描述**: 重新提交所有失败任务

### 5.3 搜索模块

#### 5.3.1 semantic_search
- **Command**: `semantic_search(query: String, limit: u32, filters: Option<SearchFilters>) -> Result<Vec<SearchResultDTO>>`
- **描述**: 语义搜索 (jieba 分词 + 倒排索引)
- **Response**: 返回图片列表 + 相关性分数

### 5.4 去重模块

#### 5.4.1 scan_duplicates
- **Command**: `scan_duplicates(threshold: u32) -> Result<Vec<DuplicateGroup>>`
- **描述**: 扫描相似图片 (pHash 相似度)
- **参数**: 相似度阈值 (70-99)，默认 90

#### 5.4.2 delete_duplicates
- **Command**: `delete_duplicates(keep_ids: Vec<i64>) -> Result<usize>`
- **描述**: 删除重复项，保留指定 ID

### 5.5 导出模块

#### 5.5.1 export_data
- **Command**: `export_data(format: String, image_ids: Option<Vec<i64>>, output_path: String) -> Result<String>`
- **描述**: 导出图片元数据 (JSON/CSV)

## 6. 关键流程设计

### 6.1 图片导入流程
1. 用户拖拽图片到主界面 → 触发 `import_images` Command
2. Rust 端验证文件 (大小 ≤ 50MB，格式检查)
3. 计算 SHA256 → 检查是否重复
4. 生成缩略图 (tokio::spawn_blocking, 300x200)
5. 计算 pHash (感知哈希)
6. 提取 EXIF 元数据
7. 插入 `images` 表 (ai_status = 'pending')
8. 创建 `task_queue` 记录
9. emit("import_complete", image_ids) → 前端刷新

### 6.2 AI 自动打标流程
1. task_queue 扫描器查询 status='pending' 任务 (LIMIT 100)
2. 批量推入 async_channel (背压队列，容量 1000)
3. Worker 获取 Semaphore permit (并发控制，默认 3)
4. 读取图片路径 → 编码为 base64
5. POST `http://localhost:1234/v1/chat/completions`
6. 解析 AI 响应 (JSON: tags/description/category)
7. 更新 `images` 表 + `tags` 表 (多对多)
8. jieba 分词 → 写入 `search_index` 倒排索引
9. emit("ai_completed", image_id) → 前端刷新
10. 错误处理: 失败 → retry_count++，最多 3 次，指数退避

### 6.3 语义搜索流程
1. 用户输入查询 (如"日落的海滩")
2. jieba-rs 对查询分词 → ["日落", "海滩"]
3. SQL: `SELECT DISTINCT image_id, COUNT(*) as score FROM search_index WHERE term IN ('日落', '海滩') GROUP BY image_id ORDER BY score DESC`
4. JOIN `images` 表获取完整元数据
5. 返回结果 + 相关性分数 (匹配词条数)
