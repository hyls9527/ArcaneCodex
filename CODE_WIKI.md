# Arcane Codex — Code Wiki

> 本地优先的 AI 图像知识管理系统 | Rust + React + Tauri 2.0 + SQLite
> 版本: v1.0.0 | 许可证: MIT

---

## 目录

1. [项目概览](#1-项目概览)
2. [项目结构](#2-项目结构)
3. [架构设计](#3-架构设计)
4. [后端架构 (Rust / Tauri)](#4-后端架构-rust--tauri)
   - 4.1 [模块总览](#41-模块总览)
   - 4.2 [Commands 层 — Tauri 命令](#42-commands-层--tauri-命令)
   - 4.3 [Core 层 — 业务逻辑](#43-core-层--业务逻辑)
   - 4.4 [Models 层 — 数据模型](#44-models-层--数据模型)
   - 4.5 [Utils 层 — 工具函数](#45-utils-层--工具函数)
5. [前端架构 (React)](#5-前端架构-react)
   - 5.1 [组件体系](#51-组件体系)
   - 5.2 [状态管理 (Zustand)](#52-状态管理-zustand)
   - 5.3 [路由系统](#53-路由系统)
   - 5.4 [API 调用层](#54-api-调用层)
   - 5.5 [国际化 (i18n)](#55-国际化-i18n)
6. [数据库设计 (SQLite)](#6-数据库设计-sqlite)
7. [依赖关系](#7-依赖关系)
8. [构建与运行](#8-构建与运行)
9. [安全机制](#9-安全机制)
10. [关键数据流](#10-关键数据流)

---

## 1. 项目概览

**Arcane Codex** 是一款本地优先的 AI 图像知识管理系统。核心理念：**你的照片，就是记忆**。

### 核心能力

| 能力 | 说明 |
|------|------|
| 图片管理 | 批量导入、拖拽上传、缩略图生成 (WebP 300x200)、EXIF 提取、虚拟滚动 (5000+ 流畅) |
| AI 打标 | 6 种后端 (LM Studio/Ollama/Hermes/智谱/OpenAI/OpenRouter)、多模态分析、任务队列 (并发 3，指数退避重试 3 次) |
| ONNX 推理 | 图像分类、人脸检测/识别、CLIP 向量嵌入 (本地 ONNX Runtime) |
| 智能去重 | BK-Tree + pHash 算法，相似度阈值 70-99% |
| 语义搜索 | jieba-rs 分词 + 倒排索引 + 向量搜索 (HNSW) |
| 知识图谱 | 节点/边/社区管理、语义关联、路径查找 |
| 用户反馈 | 标签修正记录、错误模式识别、一致性校验 |
| 国际化 | 中文/英文切换 |
| 安全导出 | 路径穿越防护、敏感目录保护、磁盘空间预检 |
| XMP 互操作 | XMP 元数据读写、Sidecar 文件管理 |

### 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 框架 | Tauri | 2.x |
| 后端 | Rust | 2021 Edition |
| 前端 | React + TypeScript + Tailwind CSS | React 18, TS 5.5 |
| 数据库 | SQLite (rusqlite + r2d2 连接池) | rusqlite 0.31 |
| 状态管理 | Zustand | 4.5 |
| AI 推理 | ONNX Runtime (ort) | 2.0.0-rc.12 |
| 构建 | Vite | 最新 |

---

## 2. 项目结构

```
e:\ArcaneCodex\
├── src-tauri/                    # Rust Tauri 2.0 后端
│   ├── src/
│   │   ├── main.rs               # 应用入口、命令注册、服务初始化
│   │   ├── lib.rs                # 库入口
│   │   ├── commands/             # Tauri 命令层 (18 个模块)
│   │   │   ├── mod.rs
│   │   │   ├── ai.rs             # AI 处理队列控制
│   │   │   ├── ai_core.rs        # ONNX 推理 (分类/人脸/CLIP/向量)
│   │   │   ├── batch_ops.rs      # 批量操作 (打标/导出/日志/统计)
│   │   │   ├── dedup.rs          # 智能去重
│   │   │   ├── error_patterns.rs # 错误模式识别
│   │   │   ├── export.rs         # 数据导出
│   │   │   ├── file_monitor.rs   # 文件系统监控
│   │   │   ├── images.rs         # 图片导入/管理/导出
│   │   │   ├── inference_settings.rs # 推理提供者配置
│   │   │   ├── knowledge_graph.rs # 知识图谱
│   │   │   ├── narrative.rs      # 叙事锚点
│   │   │   ├── search.rs         # 语义搜索
│   │   │   ├── seed_data.rs      # 示例数据
│   │   │   ├── settings.rs       # 系统设置 (含加密备份/恢复)
│   │   │   ├── tag_correction.rs # 标签修正
│   │   │   └── xmp.rs            # XMP 元数据读写
│   │   ├── core/                 # 业务逻辑层 (19 个模块)
│   │   │   ├── mod.rs
│   │   │   ├── ai_queue.rs       # AI 任务队列 + Worker
│   │   │   ├── bk_tree.rs        # BK-Tree 去重数据结构
│   │   │   ├── clip_embedder.rs  # CLIP 向量嵌入
│   │   │   ├── consistency_checker.rs # 一致性校验
│   │   │   ├── db.rs             # 数据库连接 + 迁移 (v1-v8)
│   │   │   ├── dedup.rs          # 去重逻辑
│   │   │   ├── face_detector.rs  # 人脸检测/识别
│   │   │   ├── file_watcher.rs   # 文件系统监控
│   │   │   ├── image.rs          # 图像处理 (缩略图/pHash/EXIF)
│   │   │   ├── image_classifier.rs # 图像分类
│   │   │   ├── inference.rs      # AI 推理引擎 (多 Provider)
│   │   │   ├── knowledge_graph.rs # 知识图谱引擎
│   │   │   ├── lm_studio.rs      # LM Studio 客户端
│   │   │   ├── onnx_runtime.rs   # ONNX 推理运行时管理
│   │   │   ├── search_index.rs   # 倒排索引 (jieba-rs 分词)
│   │   │   ├── vector_index.rs   # HNSW 向量索引
│   │   │   └── xmp_service.rs    # XMP 元数据服务
│   │   ├── models/               # 数据模型 (3 个模块)
│   │   │   ├── mod.rs
│   │   │   ├── category.rs       # ImageCategory 枚举
│   │   │   ├── image.rs          # Image 模型
│   │   │   └── task.rs           # Task 模型
│   │   └── utils/                # 工具函数 (3 个模块)
│   │       ├── mod.rs
│   │       ├── crypto.rs         # AES-256-GCM 加密
│   │       ├── error.rs          # AppError 统一错误类型
│   │       └── hash.rs           # SHA256/pHash
│   ├── Cargo.toml                # Rust 依赖配置
│   ├── tauri.conf.json           # Tauri 应用配置
│   └── build.rs                  # Tauri 构建脚本
├── frontend/                     # React 前端
│   ├── src/
│   │   ├── components/
│   │   │   ├── layout/           # MainLayout, Sidebar, TopBar
│   │   │   ├── gallery/          # ImageGrid, ImageCard, ImageViewer, DropZone, ImageFilter, NarrativePrompt, ImportProgressBar, SampleDataBanner
│   │   │   ├── ai/              # AIProgressPanel, LMStudioGuide
│   │   │   ├── dedup/           # DedupManager
│   │   │   ├── settings/        # SettingsPage, AIConfig, DisplayConfig, StorageConfig, PrivacyConfig, LanguageSwitcher, LogViewer, NotificationConfig, AboutPage
│   │   │   ├── dashboard/       # AccuracyChart
│   │   │   └── common/          # ErrorBoundary
│   │   ├── pages/               # GalleryPage, AIPage, DedupPage, DashboardPage
│   │   ├── stores/              # useImageStore, useAIStore, useConfigStore, useThemeStore, useDedupStore
│   │   ├── hooks/               # useAIActions, useDedupActions
│   │   ├── router/              # 状态路由系统
│   │   ├── lib/                 # api, ai-integration, api-tester, errorMap, comic-generator, infographic-generator, slide-deck-generator
│   │   ├── i18n/                # zh.json, en.json
│   │   ├── types/               # image.ts
│   │   └── utils/               # cn.ts (clsx 封装)
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
├── arcane-codex-cli/             # Python CLI 工具 (ac 命令)
├── ralph-engine/                 # Ralph Protocol MCP Server
├── package.json                  # 根目录构建脚本
└── .cargo/config.toml            # Rust 编译器配置
```

---

## 3. 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────┐
│                    Tauri 2.0 Shell                   │
│  ┌──────────────────────┐  ┌──────────────────────┐ │
│  │   React Frontend     │  │   Rust Backend        │ │
│  │  ┌────────────────┐  │  │  ┌────────────────┐  │ │
│  │  │  Components    │  │  │  │  Commands      │  │ │
│  │  │  (UI Layer)    │  │  │  │  (API Layer)   │  │ │
│  │  └───────┬────────┘  │  │  └───────┬────────┘  │ │
│  │  ┌───────┴────────┐  │  │  ┌───────┴────────┐  │ │
│  │  │  Stores        │  │  │  │  Core          │  │ │
│  │  │  (State Mgmt)  │  │  │  │  (Business)    │  │ │
│  │  └───────┬────────┘  │  │  └───────┬────────┘  │ │
│  │  ┌───────┴────────┐  │  │  ┌───────┴────────┐  │ │
│  │  │  API Layer     │◄─┼──┼─►│  Models/Utils  │  │ │
│  │  │  (Tauri IPC)   │  │  │  │  (Data/Infra)  │  │ │
│  │  └────────────────┘  │  │  └────────────────┘  │ │
│  └──────────────────────┘  └──────────────────────┘ │
│                          │                           │
│              ┌───────────┴───────────┐               │
│              │     SQLite (WAL)      │               │
│              └───────────────────────┘               │
└─────────────────────────────────────────────────────┘
```

### 分层职责

| 层 | 职责 | 关键约束 |
|----|------|----------|
| **Commands** | Tauri 命令层，接收前端调用，参数校验，调用 Core | 函数签名必须 `#[tauri::command]`，返回 `AppResult<T>` |
| **Core** | 业务逻辑层，所有核心算法和数据操作 | 不直接依赖 Tauri 类型（除 AppHandle 事件发射） |
| **Models** | 数据模型定义，纯数据结构 | 仅定义结构体和枚举，不含业务逻辑 |
| **Utils** | 基础设施工具，错误类型、加密、哈希 | 无状态，纯函数 |

### 前后端通信

前端通过 `@tauri-apps/api` 的 `invoke` 函数调用后端 Tauri 命令，后端通过 `app.emit()` 向前端推送事件：

```
Frontend (invoke) ──► Tauri IPC ──► Commands ──► Core ──► Database
Frontend (listen) ◄── Tauri Event ◄── app.emit() ◄── Worker/AI Queue
```

---

## 4. 后端架构 (Rust / Tauri)

### 4.1 模块总览

| 模块路径 | 文件数 | 职责 |
|----------|--------|------|
| `commands/` | 18 | Tauri 命令层，暴露给前端的 API |
| `core/` | 19 | 业务逻辑层，核心引擎 |
| `models/` | 3 | 数据模型定义 |
| `utils/` | 3 | 工具函数 |

### 4.2 Commands 层 — Tauri 命令

所有命令通过 `#[tauri::command]` 宏标注，在 [main.rs](src-tauri/src/main.rs) 中统一注册。

#### 图片管理 (`commands::images`)

| 命令 | 签名 | 说明 |
|------|------|------|
| `import_images` | `(app, db, file_paths: Vec<String>) -> ImportResult` | 批量导入图片，两阶段处理：快速入库 + 异步元数据 |
| `get_images` | `(db, page, page_size, filters) -> JSON` | 分页获取图片列表，支持多条件筛选 |
| `get_image_detail` | `(db, id) -> JSON` | 获取单张图片详情（含 EXIF、AI 结果） |
| `delete_images` | `(db, ids: Vec<i64>) -> usize` | 批量删除图片（含缩略图和搜索索引清理） |
| `check_broken_links` | `(db) -> CheckBrokenLinksResult` | 检查断链，标记 `broken_link` 状态 |
| `archive_image` | `(app, db, id) -> ArchiveImageResult` | 归档图片到应用数据目录 |
| `safe_export` | `(db, image_ids, dest_dir) -> SafeExportResult` | 安全批量导出（含路径穿越防护、磁盘预检） |

**导入流程** (两阶段设计)：

```
阶段1: 快速入库
  路径展开 → 文件验证(魔术字节) → SHA256 去重 → 插入数据库 → 释放连接

阶段2: 异步元数据
  spawn_blocking {
    缩略图生成 (WebP 300x200)
    pHash 计算 (感知哈希)
    EXIF 提取 (kamadak-exif)
  } → 更新元数据 → 创建 AI 任务
```

#### AI 处理 (`commands::ai`)

| 命令 | 签名 | 说明 |
|------|------|------|
| `start_ai_processing` | `(queue) -> AIStatus` | 启动 AI 处理队列 |
| `pause_ai_processing` | `(queue)` | 暂停队列 |
| `resume_ai_processing` | `(queue)` | 恢复队列 |
| `get_ai_status` | `(queue) -> AIStatus` | 获取实时状态 |
| `retry_failed_ai` | `(queue, image_id?) -> usize` | 重试失败任务 |
| `get_recent_ai_results` | `(db, limit?) -> Vec<AIResult>` | 获取最近 AI 结果 |

#### AI Core — ONNX 推理 (`commands::ai_core`)

| 命令 | 签名 | 说明 |
|------|------|------|
| `get_ai_model_status` | `(state) -> Vec<ModelStatus>` | 获取所有模型加载状态 |
| `load_ai_model` | `(state, model_type, custom_path?) -> ModelLoadResult` | 加载 ONNX 模型 |
| `unload_ai_model` | `(state, model_type) -> bool` | 卸载模型 |
| `classify_image` | `(state, image_path, top_n) -> ImageClassificationResult` | 图像分类 |
| `detect_faces` | `(state, image_path, confidence) -> Vec<FaceDetection>` | 人脸检测 |
| `extract_face_embedding` | `(state, image_path, bbox) -> FaceEmbedding` | 人脸嵌入提取 |
| `register_face` | `(state, image_path, bbox) -> String` | 注册人脸 |
| `recognize_face` | `(state, image_path, bbox, threshold) -> Option<FaceMatch>` | 人脸识别 |
| `get_registered_face_count` | `(state) -> usize` | 已注册人脸数 |
| `embed_image_clip` | `(state, image_path) -> ClipEmbedding` | CLIP 向量嵌入 |
| `insert_vector` | `(state, entry) -> ()` | 插入向量 |
| `search_vectors` | `(state, query, top_k, min_similarity) -> Vec<SearchResult>` | 向量搜索 |
| `delete_vector` | `(state, id) -> bool` | 删除向量 |
| `get_vector_index_stats` | `(state) -> IndexStats` | 向量索引统计 |

**AppState** — AI Core 共享状态：

```rust
pub struct AppState {
    pub onnx_manager: Arc<OnnxRuntimeManager>,
    pub classifier: Arc<ImageClassifier>,
    pub face_detector: Arc<FaceDetector>,
    pub clip_embedder: Arc<ClipEmbedder>,
    pub vector_index: Arc<HnswVectorIndex>,
}
```

**ModelType** 枚举：`ImageClassification | FaceDetection | FaceRecognition | ClipEmbedding`

#### 知识图谱 (`commands::knowledge_graph`)

| 命令 | 说明 |
|------|------|
| `kg_build_graph` | 构建知识图谱 |
| `kg_get_stats` | 获取统计信息 |
| `kg_get_all_nodes` | 获取所有节点 |
| `kg_get_all_edges` | 获取所有边 |
| `kg_get_communities` | 获取社区 |
| `kg_get_community_nodes` | 获取社区节点 |
| `kg_get_neighbors` | 获取邻居节点 |
| `kg_find_path` | 寻找路径 |
| `kg_search_nodes` | 搜索节点 |
| `kg_clear` | 清空图谱 |
| `kg_load_from_db` | 从数据库加载 |
| `kg_save_to_db` | 保存到数据库 |

#### 其他命令

| 模块 | 命令 | 说明 |
|------|------|------|
| `search` | `semantic_search` | 语义搜索 (jieba 分词 + 倒排索引) |
| `dedup` | `scan_duplicates`, `delete_duplicates` | 智能去重 |
| `settings` | `get_config`, `set_config`, `get_all_configs`, `backup_database`, `backup_database_encrypted`, `restore_database`, `restore_database_encrypted`, `test_lm_studio_connection` | 系统设置与备份 |
| `inference_settings` | `get_inference_config`, `set_inference_provider`, `test_inference_connection`, `discover_available_models` | 推理提供者配置 |
| `export` | `export_data` | 数据导出 |
| `narrative` | `write_narrative`, `get_narratives`, `query_associations` | 叙事锚点 |
| `tag_correction` | `record_tag_correction`, `get_tag_correction_history`, `get_all_tag_corrections` | 标签修正 |
| `error_patterns` | `record_error_pattern`, `get_error_patterns`, `check_error_pattern_exists`, `delete_error_pattern`, `get_high_frequency_error_patterns` | 错误模式 |
| `batch_ops` | `start_batch_ai_tag`, `get_batch_ai_status`, `pause_batch_ai_task`, `resume_batch_ai_task`, `cancel_batch_ai_task`, `batch_tag_correction`, `batch_export`, `get_library_stats`, `get_accuracy_trend`, `get_log_entries`, `get_log_stats`, `export_logs`, `clear_logs` | 批量操作 |
| `xmp` | `read_xmp_metadata`, `write_xmp_metadata`, `generate_xmp_sidecar`, `export_as_xmp` | XMP 元数据 |
| `seed_data` | `check_sample_data`, `clear_sample_data`, `load_sample_data` | 示例数据 |
| `file_monitor` | `start_file_monitor`, `stop_file_monitor`, `get_monitor_status` | 文件监控 |

### 4.3 Core 层 — 业务逻辑

#### 数据库 (`core::db`)

```rust
pub struct Database {
    pub db_path: Arc<PathBuf>,
    pool: SqlitePool,  // r2d2 连接池
}
```

**关键方法**：

| 方法 | 说明 |
|------|------|
| `new(app_handle)` | 从 Tauri AppHandle 创建数据库 |
| `open_connection()` | 从连接池获取连接 |
| `run_migrations()` | 执行数据库迁移 (v1-v8) |

**PRAGMA 配置**：`journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5000`

**迁移版本**：

| 版本 | 内容 |
|------|------|
| v1 | 初始 schema (images, tags, image_tags, search_index, task_queue, app_config) |
| v2 | ComfyUI 生成支持 (generation_source, generation_metadata) |
| v3 | 叙事锚点 (narratives, semantic_edges) |
| v4 | 多 Provider 支持 (ai_provider, settings 表) |
| v5 | AI 标签状态分级 (ai_tag_status, calibration_*, tag_corrections, error_patterns) |
| v6 | 统一配置表 (settings 替代 app_config) |
| v7 | XMP Sidecar 支持 (xmp_sidecars) |
| v8 | 知识图谱持久化 (kg_nodes, kg_edges, kg_communities) |

#### AI 任务队列 (`core::ai_queue`)

```rust
pub struct AITaskQueue {
    sender: mpsc::Sender<AITask>,
    receiver: Arc<TokioMutex<Option<mpsc::Receiver<AITask>>>>,
    command_sender: mpsc::Sender<QueueCommand>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    concurrency: usize,          // 默认 3
    pub total_tasks: AtomicUsize,
    pub processed_tasks: AtomicUsize,
    pub failed_tasks: AtomicUsize,
    db: Arc<Database>,
    app_handle: Option<AppHandle>,
}
```

**Worker 处理流程**：

```
Worker 循环:
  1. 检查 is_running / is_paused 状态
  2. 从 receiver 获取任务
  3. 更新 ai_status = "processing"
  4. 查询 ProviderConfig → ProviderFactory::create()
  5. provider.analyze_image() → AIResult
  6. 一致性校验 (ConsistencyChecker) → 确定 tag_status
  7. 更新数据库 (ai_status, ai_tags, ai_description, ai_category, ai_confidence, ai_tag_status)
  8. 构建/跳过搜索索引 (tag_status != "rejected" 时构建)
  9. 发射 ai-progress 事件
```

**重试策略**：最多 3 次，指数退避 (2s → 4s → 8s)

**标签状态分级**：

| 状态 | 条件 | 行为 |
|------|------|------|
| `verified` | confidence ≥ 0.85 且一致性通过 | 参与搜索 |
| `provisional` | 0.5 ≤ confidence < 0.85 且一致性通过 | 标记待验证 |
| `rejected` | confidence < 0.5 或一致性校验失败 | 不参与搜索 |

#### 推理引擎 (`core::inference`)

```rust
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult>;
    async fn health_check(&self) -> AppResult<Vec<String>>;
}
```

**Provider 架构**：

```
InferenceProvider (trait)
├── OpenAICompatibleAdapter  ← LM Studio / Ollama / Hermes (共享 LMStudioClient)
└── OpenAIClient             ← OpenAI / OpenRouter (独立实现)
```

**InferenceProviderType 枚举**：`LMStudio | Ollama | Hermes | OpenAI | OpenRouter`

**ProviderConfig**：

```rust
pub struct ProviderConfig {
    pub provider_type: InferenceProviderType,
    pub base_url: String,        // 默认 "http://127.0.0.1:1234"
    pub model: String,           // 默认 "Qwen2.5-VL-7B-Instruct"
    pub api_key: Option<String>,
    pub timeout_secs: u64,       // 默认 60
}
```

**模型发现服务** (`ModelDiscoveryService`)：扫描 LM Studio (1234)、Ollama (11434)、Hermes (18789) 三个本地服务，检测可用模型和加载状态。

#### 图像处理 (`core::image`)

`ImageProcessor` 提供三个核心方法：

| 方法 | 说明 |
|------|------|
| `generate_thumbnail(src, dest)` | 生成 WebP 缩略图 (300x200) |
| `calculate_phash(path)` | 计算感知哈希 (pHash) |
| `extract_exif(path)` | 提取 EXIF 元数据 |

#### 搜索索引 (`core::search_index`)

`SearchIndexBuilder` 使用 jieba-rs 中文分词构建倒排索引：

- 对 AI 描述、标签、分类进行分词
- 每个词项记录 image_id、字段来源、位置、权重
- 支持缓存机制 (`clear_search_cache`)

#### 向量索引 (`core::vector_index`)

`HnswVectorIndex` — 基于 HNSW 算法的向量索引：

- 维度：512 (CLIP embedding 维度)
- 持久化到 `app_data/vector_index/` 目录
- 支持插入、搜索、删除、统计

#### 其他 Core 模块

| 模块 | 关键结构/函数 | 说明 |
|------|---------------|------|
| `bk_tree` | `BKTree` | BK-Tree 数据结构，用于 pHash 近似去重 |
| `clip_embedder` | `ClipEmbedder` | CLIP 模型嵌入器 (ONNX) |
| `consistency_checker` | `ConsistencyChecker::check_all()` | 标签一致性校验 |
| `dedup` | 去重逻辑 | 基于 BK-Tree + pHash 的去重 |
| `face_detector` | `FaceDetector` | 人脸检测/识别 (ONNX) |
| `file_watcher` | `FileWatcherService` | 文件系统监控 (notify crate) |
| `image_classifier` | `ImageClassifier` | 图像分类 (ONNX) |
| `knowledge_graph` | `KnowledgeGraphEngine` | 知识图谱引擎 |
| `lm_studio` | `LMStudioClient` | LM Studio API 客户端 |
| `onnx_runtime` | `OnnxRuntimeManager` | ONNX Runtime 模型管理 |
| `xmp_service` | XMP 服务 | XMP 元数据读写 |

### 4.4 Models 层 — 数据模型

| 模型 | 说明 |
|------|------|
| `Image` | 图片主模型 (id, file_path, file_hash, ai_tags, ai_description, ai_status 等) |
| `ImageCategory` | 分类枚举 (风景/人物/物品/动物/建筑/文档/其他) |
| `Task` | 任务模型 (image_id, task_type, status, priority, retry_count) |

### 4.5 Utils 层 — 工具函数

| 模块 | 关键函数 | 说明 |
|------|----------|------|
| `crypto` | `encrypt_api_key()`, `decrypt_api_key()` | AES-256-GCM 加密 (⚠️ 固定 Nonce，待修复) |
| `error` | `AppError` 枚举, `AppResult<T>`, `init_logging()` | 统一错误类型 + 日志初始化 |
| `hash` | `calculate_sha256()` | SHA256 文件哈希 |

**AppError 枚举**：

```rust
pub enum AppError {
    Database(String),
    Validation(String),
    AI(String),
    IO(std::io::Error),
    // ...
}
```

---

## 5. 前端架构 (React)

### 5.1 组件体系

#### 布局组件 (`components/layout/`)

| 组件 | 职责 |
|------|------|
| `MainLayout` | 主布局框架，嵌套 Sidebar + TopBar + children |
| `Sidebar` | 侧边导航栏，管理活动页面样式 |
| `TopBar` | 顶部导航栏，标题 + 设置入口 |

#### 画廊组件 (`components/gallery/`)

| 组件 | 职责 |
|------|------|
| `ImageGrid` | 图片网格展示，虚拟滚动 (@tanstack/react-virtual) |
| `ImageCard` | 单张图片卡片，预览/标签/删除 |
| `ImageViewer` | 图片详情查看器，缩放/拖拽/切换 |
| `DropZone` | 拖拽上传区域 |
| `ImageFilter` | 图片过滤选项 (AI 状态/分类/日期/标签) |
| `ImportProgressBar` | 导入进度条 |
| `NarrativePrompt` | 叙事提示生成 |
| `SampleDataBanner` | 示例数据提示横幅 |

#### AI 组件 (`components/ai/`)

| 组件 | 职责 |
|------|------|
| `AIProgressPanel` | AI 处理进度面板 |
| `LMStudioGuide` | LM Studio 使用指南 |

#### 设置组件 (`components/settings/`)

| 组件 | 职责 |
|------|------|
| `SettingsPage` | 设置页面容器 |
| `AIConfig` | AI 推理配置 |
| `DisplayConfig` | 显示配置 |
| `StorageConfig` | 存储配置 |
| `PrivacyConfig` | 隐私配置 |
| `NotificationConfig` | 通知配置 |
| `LanguageSwitcher` | 语言切换 |
| `LogViewer` | 日志查看器 |
| `AboutPage` | 关于页面 |

#### 其他组件

| 组件 | 路径 | 职责 |
|------|------|------|
| `DedupManager` | `components/dedup/` | 去重管理 |
| `AccuracyChart` | `components/dashboard/` | 准确率图表 |
| `ErrorBoundary` | `components/common/` | 错误边界捕获 |

### 5.2 状态管理 (Zustand)

| Store | 状态字段 | 关键 Actions |
|-------|----------|-------------|
| `useImageStore` | images, currentPage, totalImages | 添加图片、更新状态、分页 |
| `useAIStore` | status, model, prompt | 设置 prompt、处理模型响应 |
| `useConfigStore` | UI 配置、图像过滤设置 | 更新配置项 |
| `useThemeStore` | theme | 切换深色/浅色主题 |
| `useDedupStore` | isProcessing, processedCount, totalCount | 启动/停止去重 |

### 5.3 路由系统

前端使用自定义状态路由系统 (`router/`)，而非 react-router：

- 通过 `router.navigate()` 实现页面跳转
- 页面状态驱动渲染：GalleryPage, AIPage, DedupPage, DashboardPage, SettingsPage

### 5.4 API 调用层

`lib/api.ts` — 封装所有 Tauri IPC 调用：

| 类别 | 方法 |
|------|------|
| 图片管理 | `importImages`, `getImages`, `getImageDetail`, `deleteImages` |
| AI 处理 | `startAIProcessing`, `pauseAIProcessing`, `resumeAIProcessing`, `getAIStatus`, `retryFailedAI` |
| 语义搜索 | `searchImages`, `queryAssociations` |
| 仪表盘 | `getLibraryStats`, `getAccuracyTrend` |
| 日志 | `getLogEntries`, `getLogStats`, `exportLogs`, `clearLogs` |

其他 lib 模块：

| 模块 | 说明 |
|------|------|
| `ai-integration` | AI 集成辅助 |
| `api-tester` | API 测试工具 |
| `errorMap` | 错误码映射 |
| `comic-generator` | 漫画生成器 |
| `infographic-generator` | 信息图生成器 |
| `slide-deck-generator` | 幻灯片生成器 |

### 5.5 国际化 (i18n)

- 框架：i18next + react-i18next
- 语言文件：`i18n/zh.json`, `i18n/en.json`
- 翻译类别：common, navigation, gallery, ai, settings, errors, dashboard, logs, narrative

---

## 6. 数据库设计 (SQLite)

### ER 关系概览

```
images ──1:N── image_tags ──N:1── tags
  │
  ├──1:N── search_index
  ├──1:N── task_queue
  ├──1:1── narratives ──N:N── semantic_edges
  ├──1:N── tag_corrections
  └──1:1── xmp_sidecars

kg_nodes ──N:N── kg_edges
kg_nodes ──N:1── kg_communities

settings (key-value)
calibration_samples / calibration_reports / calibration_curves
error_patterns
```

### 核心表结构

#### `images` — 图片主表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| file_path | TEXT UNIQUE | 文件绝对路径 |
| file_name | TEXT | 文件名 |
| file_size | INTEGER | 文件大小 (字节) |
| file_hash | TEXT | SHA256 哈希 (去重) |
| mime_type | TEXT | MIME 类型 |
| width / height | INTEGER | 图片尺寸 |
| thumbnail_path | TEXT | 缩略图路径 |
| phash | TEXT | 感知哈希 (去重) |
| exif_data | JSON | EXIF 元数据 |
| ai_status | TEXT | AI 状态: pending/processing/completed/failed/broken_link |
| ai_tags | JSON | AI 标签数组 |
| ai_description | TEXT | AI 描述 |
| ai_category | TEXT | AI 分类 |
| ai_confidence | REAL | AI 置信度 (0.0-1.0) |
| ai_tag_status | TEXT | 标签状态: verified/provisional/rejected |
| ai_provider | TEXT | AI 提供者 |
| ai_model | TEXT | AI 模型名 |
| ai_processed_at | DATETIME | AI 处理时间 |
| ai_error_message | TEXT | AI 错误信息 |
| ai_retry_count | INTEGER | 重试次数 |
| generation_source | TEXT | 生成来源 (默认 manual_import) |
| generation_metadata | JSON | 生成元数据 |
| source | TEXT | 来源 (默认 import) |
| created_at / updated_at | DATETIME | 时间戳 |

#### `search_index` — 倒排索引

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| term | TEXT | 分词词项 |
| image_id | INTEGER FK | 关联图片 |
| field | TEXT | 字段来源 (description/tags/category) |
| position | INTEGER | 词位置 |
| weight | REAL | 权重 (默认 1.0) |

#### `task_queue` — 任务队列

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| image_id | INTEGER FK | 关联图片 |
| task_type | TEXT | 任务类型 (ai_analysis) |
| status | TEXT | 状态: pending/processing/completed/failed |
| priority | INTEGER | 优先级 |
| retry_count | INTEGER | 重试次数 |
| max_retries | INTEGER | 最大重试 (默认 3) |

#### `settings` — 统一配置表

| 字段 | 类型 | 说明 |
|------|------|------|
| key | TEXT PK | 配置键 |
| value | TEXT | 配置值 |
| updated_at | DATETIME | 更新时间 |

**预设配置**：

| key | 默认值 | 说明 |
|-----|--------|------|
| lm_studio_url | http://localhost:1234 | LM Studio 地址 |
| ai_concurrency | 3 | AI 并发数 |
| ai_timeout_seconds | 60 | AI 超时 |
| ai_max_retries | 3 | 最大重试 |
| theme | system | 主题 |
| language | zh | 语言 |
| thumbnail_size | 300 | 缩略图尺寸 |
| inference_provider | lm_studio | 推理提供者 |
| inference_model | Qwen2.5-VL-7B-Instruct | 推理模型 |
| inference_api_key | (空) | API Key (加密存储) |
| inference_timeout | 60 | 推理超时 |

#### 知识图谱表

| 表 | 说明 |
|----|------|
| `kg_nodes` | 图谱节点 (id, node_type, label, properties_json, embedding_json, community_id, degree) |
| `kg_edges` | 图谱边 (id, source_id, target_id, edge_type, weight, properties_json) |
| `kg_communities` | 社区 (id, size, central_node_id, tags_json, density) |

---

## 7. 依赖关系

### Rust 依赖 (Cargo.toml)

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tauri` | 2 | 桌面应用框架 |
| `tauri-plugin-shell` | 2 | Shell 插件 |
| `tauri-plugin-dialog` | 2 | 对话框插件 |
| `serde` / `serde_json` | 1 | 序列化 |
| `rusqlite` | 0.31 | SQLite 驱动 (bundled + chrono) |
| `r2d2` / `r2d2_sqlite` | 0.8 / 0.24 | 连接池 |
| `tokio` | 1 | 异步运行时 (full features) |
| `reqwest` | 0.12 | HTTP 客户端 (json/stream/multipart) |
| `image` | 0.25 | 图像处理 (png/jpeg/webp/tiff/gif/ico/bmp) |
| `kamadak-exif` | 0.5 | EXIF 解析 |
| `jieba-rs` | 0.7 | 中文分词 |
| `thiserror` / `anyhow` | 1 | 错误处理 |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | 日志 |
| `chrono` | 0.4 | 时间处理 |
| `walkdir` | 2 | 目录遍历 |
| `sha2` / `hex` | 0.10 / 0.4 | 哈希计算 |
| `async-channel` | 2 | 异步通道 |
| `uuid` | 1 | UUID 生成 |
| `zip` | 2 | ZIP 压缩 (备份) |
| `aes-gcm` | 0.10 | AES-256-GCM 加密 |
| `base64` / `data-encoding` | 0.22 / 2 | Base64 编码 |
| `rand` | 0.9 | 随机数 |
| `xmpkit` / `xmp-writer` | 0.1 / 0.3 | XMP 元数据 |
| `notify` | 7 | 文件系统监控 |
| `ort` | 2.0.0-rc.12 | ONNX Runtime |
| `winapi` | 0.3 | Windows API (磁盘空间) |
| `regex` | 1 | 正则表达式 |

### 前端依赖 (package.json)

| 依赖 | 用途 |
|------|------|
| `react` / `react-dom` | UI 框架 |
| `@tauri-apps/api` | Tauri IPC 调用 |
| `zustand` | 状态管理 |
| `@tanstack/react-virtual` | 虚拟滚动 |
| `i18next` / `react-i18next` | 国际化 |
| `lucide-react` | 图标库 |
| `motion` (framer-motion) | 动画 |
| `react-dropzone` | 拖拽上传 |
| `clsx` | 类名合并 |

### 编译优化配置

```toml
[profile.dev]
opt-level = 1
incremental = false

[profile.dev.package."*"]
opt-level = 2          # 依赖项 O2，编译提速 40-60%

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"
```

链接器配置 (`.cargo/config.toml`)：

```toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld"    # 链接提速 2-5x
```

---

## 8. 构建与运行

### 开发模式

```bash
npm run tauri dev        # 启动 Tauri 开发服务器 (前端 HMR + Rust 热重载)
```

### 构建

```bash
npm run build            # 生产构建 (Rust release + Vite 打包)
```

### 测试

```bash
cd frontend && npm run test    # 前端测试 (Vitest)
cd src-tauri && cargo test     # Rust 测试
cd src-tauri && cargo check    # Rust 类型检查 (CI 用)
```

### 代码质量

```bash
cd src-tauri && cargo fmt      # Rust 格式化
npx tsc --noEmit               # TypeScript 类型检查
```

### Vite 构建配置

- 开发端口：1420
- 路径别名：`@` → `src/`
- Source map：启用
- 手动分块：vendor-react, vendor-ui, vendor-virtual

### Tauri 配置摘要

| 配置 | 值 |
|------|-----|
| 产品名 | ArcaneCodex |
| 标识符 | com.arcanecodex.app |
| 窗口尺寸 | 1280x800 |
| CSP | 严格 (仅允许 localhost:1234 等) |
| 打包格式 | NSIS (中文/英文语言选择器) |
| 协议 | asset 协议 + custom-protocol |

---

## 9. 安全机制

| 机制 | 实现 | 状态 |
|------|------|------|
| CSP 内容安全策略 | tauri.conf.json 严格配置 | ✅ |
| 上下文隔离 | Tauri 默认机制 | ✅ |
| API Key 加密 | AES-256-GCM | ⚠️ 固定 Nonce，待修复为随机 Nonce |
| 备份加密 | AES-256-GCM + 随机 Nonce | ✅ |
| 路径穿越防护 | `sanitize_path()` + `normalize_path()` + `canonicalize()` | ✅ |
| 敏感目录保护 | `is_sensitive_directory()` — 禁止导出到系统目录 | ✅ |
| 磁盘空间预检 | 导入/导出前检查可用空间 | ✅ |
| 文件魔术字节验证 | `validate_magic_bytes()` — 防止扩展名伪造 | ✅ |
| 日志脱敏 | — | ❌ 未实现 |

---

## 10. 关键数据流

### 图片导入流程

```
用户拖拽/选择文件
  │
  ▼
Frontend: invoke("import_images", {filePaths})
  │
  ▼
Commands::images::import_images
  ├─ 阶段1: 快速入库
  │   ├─ expand_paths() — 路径展开 (目录递归)
  │   ├─ validate_file() — 文件验证 (扩展名 + 魔术字节 + 大小)
  │   ├─ calculate_sha256() — 哈希计算
  │   ├─ is_duplicate() — 去重检查
  │   ├─ insert_image_record() — 插入数据库
  │   └─ emit("import-progress") — 进度事件
  │
  └─ 阶段2: 异步元数据
      ├─ spawn_blocking {
      │   ├─ ImageProcessor::generate_thumbnail() — WebP 缩略图
      │   ├─ ImageProcessor::calculate_phash() — 感知哈希
      │   └─ ImageProcessor::extract_exif() — EXIF 提取
      │ }
      ├─ update_image_metadata() — 写回元数据
      ├─ create_ai_task() — 创建 AI 任务
      └─ emit("import-metadata-progress") — 进度事件
```

### AI 分析流程

```
用户启动 AI 处理
  │
  ▼
Commands::ai::start_ai_processing
  │
  ▼
AITaskQueue::start() → spawn_workers() → submit_pending_tasks()
  │
  ▼
Worker::run() 循环
  ├─ 从 receiver 获取 AITask
  ├─ update_ai_status("processing")
  ├─ query_provider_config() → ProviderFactory::create()
  ├─ provider.analyze_image(file_path)
  │   ├─ encode_image_to_base64()
  │   ├─ build_prompt() — 构建 AI 提示词
  │   ├─ HTTP POST → /v1/chat/completions
  │   └─ parse_ai_response() — 解析 JSON 响应
  ├─ ConsistencyChecker::check_all() → tag_status
  ├─ update_ai_status_full() — 写入 AI 结果
  ├─ SearchIndexBuilder::build_for_image() — 构建搜索索引
  └─ emit("ai-progress") — 进度事件
```

### 语义搜索流程

```
用户输入查询
  │
  ▼
Commands::search::semantic_search
  ├─ jieba-rs 分词
  ├─ 倒排索引查询 (search_index 表)
  ├─ 权重排序
  └─ 返回匹配图片列表
```

### 向量搜索流程

```
用户触发向量搜索
  │
  ▼
Commands::ai_core::embed_image_clip → ClipEmbedding (512维)
  │
  ▼
Commands::ai_core::search_vectors(query, top_k, min_similarity)
  ├─ HnswVectorIndex::search() — HNSW 近似最近邻
  └─ 返回相似图片列表
```

---

> 文档生成时间: 2026-05-06 | 基于 Arcane Codex v1.0.0 源码分析
