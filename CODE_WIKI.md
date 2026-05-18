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
11. [CLI 工具](#11-cli-工具)
12. [CI/CD 流水线](#12-cicd-流水线)

---

## 1. 项目概览

**Arcane Codex** 是一款本地优先的 AI 图像知识管理系统。核心理念：**你的照片，就是记忆**。

### 核心能力

| 能力 | 说明 |
|------|------|
| 图片管理 | 批量导入、拖拽上传、缩略图生成 (WebP 300×200)、EXIF 提取、虚拟滚动 (5000+ 流畅) |
| AI 打标 | 6 种后端 (LM Studio/Ollama/Hermes/智谱/OpenAI/OpenRouter)、多模态分析、任务队列 (并发 3，指数退避重试 3 次) |
| ONNX 推理 | 图像分类 (MobileNetV3)、人脸检测/识别 (RetinaFace+ArcFace)、CLIP 向量嵌入 (ViT-B/32) |
| 智能去重 | BK-Tree + pHash 算法 + UnionFind 聚类，相似度阈值 70-99% |
| 语义搜索 | jieba-rs 分词 + 倒排索引 + 向量搜索 (HNSW) |
| 知识图谱 | 节点/边/社区管理、语义关联、路径查找 (BFS) |
| 置信度校准 | ECE/MCE 计算、校准曲线、校准样本管理 |
| 用户反馈 | 标签修正记录、错误模式识别、一致性校验 |
| 国际化 | 中文/英文切换 (i18next) |
| 安全导出 | 路径穿越防护、敏感目录保护、磁盘空间预检 |
| XMP 互操作 | XMP 元数据读写、Sidecar 文件管理 |
| 文件监控 | 目录递归监控、变更事件广播 |
| 熔断保护 | CircuitBreaker 保护 AI 推理调用 |

### 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 框架 | Tauri | 2.x |
| 后端 | Rust | 2021 Edition (MSRV 1.75) |
| 前端 | React + TypeScript + Tailwind CSS | React 18, TS 5.5, Tailwind 3.4 |
| 数据库 | SQLite (rusqlite + r2d2 连接池) | rusqlite 0.31 |
| 状态管理 | Zustand | 4.5 |
| AI 推理 | ONNX Runtime (ort) | 2.0.0-rc.12 |
| 构建 | Vite | 5.4 |
| 测试 | Vitest + @testing-library/react | Vitest 2.0 |

---

## 2. 项目结构

```
ArcaneCodex\
├── src-tauri/                    # Rust Tauri 2.0 后端
│   ├── src/
│   │   ├── main.rs               # 应用入口、命令注册、服务初始化
│   │   ├── lib.rs                # 库入口 (pub mod commands/core/models/utils)
│   │   ├── commands/             # Tauri 命令层 (18 个模块)
│   │   │   ├── mod.rs
│   │   │   ├── ai.rs             # AI 处理队列控制
│   │   │   ├── ai_core.rs        # ONNX 推理 (分类/人脸/CLIP/向量)
│   │   │   ├── batch_ops.rs      # 批量操作 (打标/导出/日志/统计)
│   │   │   ├── calibration.rs    # 置信度校准
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
│   │   ├── core/                 # 业务逻辑层 (21 个模块)
│   │   │   ├── mod.rs
│   │   │   ├── ai_queue.rs       # AI 任务队列 + Worker
│   │   │   ├── bk_tree.rs        # BK-Tree 去重数据结构
│   │   │   ├── calibration.rs    # 置信度校准引擎
│   │   │   ├── circuit_breaker.rs # 熔断器
│   │   │   ├── clip_embedder.rs  # CLIP 向量嵌入
│   │   │   ├── consistency_checker.rs # 一致性校验
│   │   │   ├── db.rs             # 数据库连接 + 迁移 (v1-v8)
│   │   │   ├── dedup.rs          # 去重逻辑 (UnionFind 聚类)
│   │   │   ├── face_detector.rs  # 人脸检测/识别
│   │   │   ├── file_watcher.rs   # 文件系统监控 (notify crate)
│   │   │   ├── image.rs          # 图像处理 (缩略图/pHash/EXIF)
│   │   │   ├── image_classifier.rs # 图像分类 (ImageNet 1000 类)
│   │   │   ├── inference.rs      # AI 推理引擎 (多 Provider)
│   │   │   ├── knowledge_graph.rs # 知识图谱引擎
│   │   │   ├── lm_studio.rs      # LM Studio 客户端 (OpenAI 兼容)
│   │   │   ├── onnx_runtime.rs   # ONNX 推理运行时管理
│   │   │   ├── search_index.rs   # 倒排索引 (jieba-rs 分词)
│   │   │   ├── vector_index.rs   # HNSW 向量索引
│   │   │   └── xmp_service.rs    # XMP 元数据服务
│   │   ├── models/               # 数据模型 (3 个模块)
│   │   │   ├── mod.rs
│   │   │   ├── category.rs       # ImageCategory 枚举
│   │   │   ├── image.rs          # Image 模型 (28 字段)
│   │   │   └── task.rs           # Task 模型
│   │   └── utils/                # 工具函数 (4 个模块)
│   │       ├── mod.rs
│   │       ├── crypto.rs         # AES-256-GCM 加密 (v1/v2/v3 格式)
│   │       ├── error.rs          # AppError 统一错误类型 + 日志初始化
│   │       ├── hash.rs           # SHA256 文件哈希
│   │       └── log_sanitizer.rs  # 日志脱敏 (API Key/路径/SQL)
│   ├── Cargo.toml                # Rust 依赖配置
│   ├── tauri.conf.json           # Tauri 应用配置
│   └── build.rs                  # Tauri 构建脚本
├── frontend/                     # React 前端
│   ├── src/
│   │   ├── components/
│   │   │   ├── layout/           # MainLayout, Sidebar, TopBar
│   │   │   ├── gallery/          # ImageGrid, ImageCard, ImageViewer, DropZone, ImageFilter, NarrativePrompt, ImportProgressBar, SampleDataBanner
│   │   │   │   ├── ImageViewer/  # 子组件: ImageBottomBar, ImageInfoPanel, ImageToolbar, SampleViewerPlaceholder, useImageZoom
│   │   │   │   └── KnowledgeGraphView/ # 子组件: GraphCanvas, GraphHeader, GraphSidebar, useForceSimulation
│   │   │   ├── ai/              # AIProgressPanel, LMStudioGuide
│   │   │   ├── dedup/           # DedupManager
│   │   │   ├── settings/        # SettingsPage, AIConfig, DisplayConfig, StorageConfig, PrivacyConfig, LanguageSwitcher, LogViewer, NotificationConfig, AboutPage
│   │   │   ├── dashboard/       # AccuracyChart
│   │   │   └── common/          # ErrorBoundary
│   │   ├── pages/               # GalleryPage (含子组件), AIPage, DedupPage, DashboardPage
│   │   ├── stores/              # useImageStore, useAIStore, useConfigStore, useThemeStore, useDedupStore
│   │   ├── hooks/               # useAIActions, useDedupActions
│   │   ├── router/              # 状态路由系统 (state-router + events)
│   │   ├── lib/                 # api, errorMap, motion
│   │   ├── i18n/                # zh.json, en.json, index.ts
│   │   ├── types/               # image.ts
│   │   └── utils/               # cn.ts, assetUrl.ts
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
├── arcane-codex-cli/             # Python CLI 工具 (ac 命令)
│   ├── arcanecodex.py           # CLI 主程序 (click 框架)
│   └── setup.py                 # 包配置
├── ralph-engine/                 # Ralph Protocol MCP Server
│   ├── server.py                # MCP Server 实现
│   └── pyproject.toml           # Python 项目配置
├── .github/workflows/           # CI/CD 配置
│   ├── ci.yml                   # 持续集成
│   └── release.yml              # 发布流水线
├── package.json                  # 根目录构建脚本
└── .cargo/config.toml            # Rust 编译器配置 (rust-lld)
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
| **Utils** | 基础设施工具，错误类型、加密、哈希、日志脱敏 | 无状态，纯函数 |

### 前后端通信

前端通过 `@tauri-apps/api` 的 `invoke` 函数调用后端 Tauri 命令，后端通过 `app.emit()` 向前端推送事件：

```
Frontend (invoke) ──► Tauri IPC ──► Commands ──► Core ──► Database
Frontend (listen) ◄── Tauri Event ◄── app.emit() ◄── Worker/AI Queue
```

### 应用初始化流程

[main.rs](src-tauri/src/main.rs) 中的 `setup` 闭包按顺序初始化所有服务：

```
1. 创建 broadcast channel (文件变更事件)
2. 注册 MonitorState (FileWatcherService)
3. 初始化 Database + run_migrations()
4. 种子数据检查 (seed_if_empty)
5. 注册 Database 到 Tauri managed state
6. 创建 AITaskQueue (并发默认 3)
7. 创建 models/ 和 vector_index/ 目录
8. 初始化 OnnxRuntimeManager
9. 创建 ImageClassifier, FaceDetector, ClipEmbedder
10. 创建 HnswVectorIndex (维度 512)
11. 注册 AppState 到 managed state
12. 创建 KnowledgeGraphEngine
13. 注册 KgState 到 managed state
```

---

## 4. 后端架构 (Rust / Tauri)

### 4.1 模块总览

| 模块路径 | 文件数 | 职责 |
|----------|--------|------|
| `commands/` | 18 | Tauri 命令层，暴露给前端的 API |
| `core/` | 21 | 业务逻辑层，核心引擎 |
| `models/` | 3 | 数据模型定义 |
| `utils/` | 4 | 工具函数 (含日志脱敏) |

### 4.2 Commands 层 — Tauri 命令

所有命令通过 `#[tauri::command]` 宏标注，在 [main.rs](src-tauri/src/main.rs) 中统一注册（共 50+ 个命令）。

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
    缩略图生成 (WebP 300×200)
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
| `classify_image` | `(state, image_path, top_n) -> ClassificationResult` | 图像分类 (MobileNetV3) |
| `detect_faces` | `(state, image_path, confidence) -> Vec<FaceDetection>` | 人脸检测 (RetinaFace) |
| `extract_face_embedding` | `(state, image_path, bbox) -> FaceEmbedding` | 人脸嵌入提取 (ArcFace) |
| `register_face` | `(state, image_path, bbox) -> String` | 注册人脸 |
| `recognize_face` | `(state, image_path, bbox, threshold) -> Option<FaceMatch>` | 人脸识别 |
| `get_registered_face_count` | `(state) -> usize` | 已注册人脸数 |
| `embed_image_clip` | `(state, image_path) -> ClipEmbedding` | CLIP 向量嵌入 (ViT-B/32) |
| `insert_vector` | `(state, entry) -> ()` | 插入向量 |
| `search_vectors` | `(state, query, top_k, min_similarity) -> Vec<SearchResult>` | 向量搜索 (HNSW) |
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

| ModelType | 默认模型文件 | 输入尺寸 | 输出维度 |
|-----------|-------------|----------|----------|
| ImageClassification | mobilenetv3_large.onnx | [1,3,224,224] | 1000 |
| FaceDetection | retinaface_resnet50.onnx | [1,3,640,640] | 42000 |
| FaceRecognition | arcface_r100.onnx | [1,3,112,112] | 512 |
| ClipEmbedding | clip_vit_b32.onnx | [1,3,224,224] | 512 |

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
| `kg_find_path` | 寻找路径 (BFS) |
| `kg_search_nodes` | 搜索节点 |
| `kg_clear` | 清空图谱 |
| `kg_load_from_db` | 从数据库加载 |
| `kg_save_to_db` | 保存到数据库 |

#### 置信度校准 (`commands::calibration`)

| 命令 | 说明 |
|------|------|
| `record_calibration_sample` | 记录校准样本 |
| `calculate_and_save_calibration` | 计算并保存校准报告 |
| `get_latest_calibration_report` | 获取最新校准报告 |
| `get_calibration_curve_data` | 获取校准曲线数据 |

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

**类型别名**：

```rust
pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type PooledConn = r2d2::PooledConnection<SqliteConnectionManager>;
```

**关键方法**：

| 方法 | 签名 | 说明 |
|------|------|------|
| `new` | `(app_handle: &AppHandle) -> Result<Self>` | 从 Tauri AppHandle 创建数据库 |
| `new_from_path` | `(path: &str) -> Result<Self>` | 从路径创建 (测试用) |
| `open_connection` | `(&self) -> Result<PooledConn>` | 从连接池获取连接 |
| `run_migrations` | `(&self) -> Result<()>` | 执行数据库迁移 (v1-v8) |

**PRAGMA 配置**：`journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5000`

**迁移版本**：

| 版本 | 方法名 | 内容 |
|------|--------|------|
| v1 | `apply_v1_initial_schema` | 初始 schema (images, tags, image_tags, search_index, task_queue, app_config) |
| v2 | `apply_v2_comfyui_generation` | ComfyUI 生成支持 (generation_source, generation_metadata, generation_workflow_id) |
| v3 | `apply_v3_narrative_anchor` | 叙事锚点 (narratives, semantic_edges) |
| v4 | `apply_v4_multi_provider` | 多 Provider 支持 (ai_provider, settings 表) |
| v5 | `apply_v5_ai_tag_status` | AI 标签状态分级 (ai_tag_status, calibration_*, tag_corrections, error_patterns) |
| v6 | `apply_v6_unify_config` | 统一配置表 (settings 替代 app_config) |
| v7 | `apply_v7_xmp_sidecars` | XMP Sidecar 支持 (xmp_sidecars) |
| v8 | `apply_v8_knowledge_graph` | 知识图谱持久化 (kg_nodes, kg_edges, kg_communities) |

#### AI 任务队列 (`core::ai_queue`)

```rust
pub struct AITaskQueue {
    sender: mpsc::Sender<AITask>,
    receiver: Arc<TokioMutex<Option<mpsc::Receiver<AITask>>>>,
    command_sender: mpsc::Sender<QueueCommand>,
    command_receiver: Arc<TokioMutex<Option<mpsc::Receiver<QueueCommand>>>>,
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

**辅助结构体**：

```rust
pub struct AITask {
    pub image_id: i64,
    pub file_path: String,
    pub retry_count: u32,
}

pub struct QueueStatus {
    pub is_running: bool,
    pub is_paused: bool,
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub processed_tasks: usize,
    pub failed_tasks: usize,
    pub concurrency: usize,
}

pub struct AIProgressEvent {
    pub image_id: i64,
    pub status: String,
    pub message: String,
    pub total: usize,
    pub current: usize,
}

pub enum QueueCommand { Pause, Resume, Cancel }
```

**常量**：`QUEUE_CAPACITY = 1000`, `DEFAULT_CONCURRENCY = 3`, `MAX_RETRIES = 3`

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
pub struct AIResult {
    pub tags: Vec<String>,
    pub description: String,
    pub category: String,
    pub confidence: f64,
    pub raw_response: String,
    pub provider: String,
    pub model: String,
}

pub enum InferenceProviderType {
    LMStudio, Ollama, Hermes, OpenAI, OpenRouter,
}

pub struct ProviderConfig {
    pub provider_type: InferenceProviderType,
    pub base_url: String,        // 默认 "http://127.0.0.1:1234"
    pub model: String,           // 默认 "Qwen2.5-VL-7B-Instruct"
    pub api_key: Option<String>,
    pub timeout_secs: u64,       // 默认 60
}
```

**InferenceProvider trait**：

```rust
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult>;
    async fn health_check(&self) -> AppResult<Vec<String>>;
    async fn check_vision_capability(&self) -> AppResult<bool>;
    fn clone_box(&self) -> Box<dyn InferenceProvider>;
}
```

**Provider 架构**：

```
InferenceProvider (trait)
├── OpenAICompatibleAdapter  ← LM Studio / Ollama / Hermes (共享 LMStudioClient)
└── OpenAIClient             ← OpenAI / OpenRouter (独立实现)
```

**ProviderFactory::create()** 路由逻辑：
- `LMStudio | Ollama | Hermes` → 创建 `LMStudioClient` + 包装为 `OpenAICompatibleAdapter`
- `OpenAI | OpenRouter` → 创建 `OpenAIClient` (需要 API Key)

#### LM Studio 客户端 (`core::lm_studio`)

```rust
pub struct LMStudioConfig {
    pub base_url: String,       // 默认 "http://127.0.0.1:1234"
    pub model: String,          // 默认 "Qwen2.5-VL-7B-Instruct"
    pub timeout_secs: u64,      // 默认 60
}

pub struct LMStudioClient {
    client: Client,             // reqwest HTTP 客户端
    pub config: LMStudioConfig,
}
```

**关键方法**：

| 方法 | 说明 |
|------|------|
| `new(config)` | 创建客户端 (5s 连接超时) |
| `health_check()` | GET /v1/models 获取模型列表 |
| `analyze_image(path)` | POST /v1/chat/completions 多模态分析 |
| `list_models()` | 列出可用模型 |

**模型发现服务** (`ModelDiscoveryService`)：扫描 LM Studio (1234)、Ollama (11434)、Hermes (18789) 三个本地服务，检测可用模型和加载状态。

#### 图像处理 (`core::image`)

`ImageProcessor` 提供三个核心方法：

| 方法 | 说明 |
|------|------|
| `generate_thumbnail(src, dest)` | 生成 WebP 缩略图 (300×200) |
| `calculate_phash(path)` | 计算感知哈希 (32×32 缩放 → 灰度 → DCT → 64 位哈希) |
| `extract_exif(path)` | 提取 EXIF 元数据 |

**pHash 算法**：缩放到 32×32 → 灰度转换 → 计算 DCT → 取低频分量 → 生成 64 位哈希

#### 搜索索引 (`core::search_index`)

`SearchIndexBuilder` 使用 jieba-rs 中文分词构建倒排索引：

- 全局 Jieba 实例 (`OnceLock<Jieba>`)
- 搜索缓存 (TTL 300 秒)
- 停用词过滤 (中英文)
- 对 AI 描述、标签、分类进行分词
- 每个词项记录 image_id、字段来源、位置、权重

**关键方法**：

| 方法 | 说明 |
|------|------|
| `build_for_image(db, image_id)` | 为单张图片构建索引 |
| `search(db, query, filters, limit, offset)` | 执行搜索 |
| `clear_search_cache()` | 清除搜索缓存 |

**SearchResult** 结构体：

```rust
pub struct SearchResult {
    pub image_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub thumbnail_path: Option<String>,
    pub ai_description: Option<String>,
    pub ai_tags: Option<String>,
    pub ai_category: Option<String>,
    pub ai_confidence: Option<f64>,
    pub match_count: usize,
    pub relevance_score: f64,
}
```

#### BK-Tree (`core::bk_tree`)

```rust
pub struct BkTree<T> {
    root: Option<BkNode<T>>,
}

struct BkNode<T> {
    item: T,
    indices: Vec<usize>,
    children: HashMap<u32, BkNode<T>>,
}
```

**关键方法**：

| 方法 | 签名 | 说明 |
|------|------|------|
| `insert` | `(&mut self, item: T, index: usize, distance_fn: F)` | 插入元素 |
| `search` | `(&self, query: &T, threshold: u32, distance_fn: F) -> Vec<(u32, usize)>` | 近似搜索 |

- 距离为 0 时合并到同一节点的 indices
- 支持自定义距离函数 (通常使用 Hamming 距离)

#### 去重逻辑 (`core::dedup`)

```rust
pub struct DuplicateGroup {
    pub images: Vec<DuplicateImage>,
    pub similarity: f64,
}

pub struct DuplicateImage {
    pub image_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub phash: String,
    pub distance: u32,
}

pub struct ScanResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_scanned: usize,
    pub total_duplicates: usize,
}
```

**去重算法**：
1. 从数据库加载所有图片的 pHash
2. 构建 BK-Tree
3. 对每个 pHash 执行近似搜索
4. 使用 UnionFind (并查集) 聚类相似图片
5. 计算相似度百分比 → 分组返回

**辅助函数**：`similarity_to_hamming(similarity_percent) -> u32` — 将相似度百分比转换为 Hamming 距离阈值

#### ONNX Runtime (`core::onnx_runtime`)

```rust
pub struct OnnxRuntimeManager {
    // 管理 ONNX 模型会话
}

pub enum ModelType {
    ImageClassification,  // mobilenetv3_large.onnx
    FaceDetection,        // retinaface_resnet50.onnx
    FaceRecognition,      // arcface_r100.onnx
    ClipEmbedding,        // clip_vit_b32.onnx
}

pub struct ModelConfig {
    pub model_type: ModelType,
    pub name: String,
    pub path: PathBuf,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
    pub is_loaded: bool,
}

pub struct InferenceResult {
    pub outputs: HashMap<String, Vec<f32>>,
    pub inference_time_ms: u64,
    pub model_type: String,
}
```

**OnnxError** 枚举：`SessionCreate | ModelNotFound | InvalidInput | InferenceFailed | Io`

#### CLIP 嵌入器 (`core::clip_embedder`)

```rust
pub struct ClipEmbedder {
    runtime_manager: Arc<OnnxRuntimeManager>,
}

pub struct ClipEmbedding {
    pub image_id: String,
    pub embedding: Vec<f32>,       // 512 维
    pub image_path: PathBuf,
    pub created_at: DateTime<Utc>,
}
```

**CLIP 预处理参数**：
- Mean: [0.48145466, 0.4578275, 0.40821073]
- Std: [0.26862954, 0.26130258, 0.27577711]

**ClipError** 枚举：`ModelNotLoaded | ImageProcessingFailed | InferenceFailed | InvalidEmbeddingDimension`

#### 人脸检测器 (`core::face_detector`)

```rust
pub struct FaceDetector {
    runtime_manager: Arc<OnnxRuntimeManager>,
}

pub struct FaceDetection {
    pub bbox: BoundingBox,
    pub landmarks: Vec<Landmark>,
    pub confidence: f32,
}

pub struct FaceEmbedding {
    pub face_id: String,
    pub embedding: Vec<f32>,       // 512 维
    pub image_path: PathBuf,
    pub detection: FaceDetection,
    pub created_at: DateTime<Utc>,
}

pub struct FaceMatch {
    pub face_id: String,
    pub similarity: f32,
    pub face_embedding: FaceEmbedding,
}
```

**FaceError** 枚举：`ModelNotLoaded | ImageProcessingFailed | InferenceFailed | NoFacesDetected | InvalidInput`

#### 向量索引 (`core::vector_index`)

```rust
pub struct HnswVectorIndex {
    // HNSW 算法实现
}

pub struct VectorEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
    pub image_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct SearchResult {
    pub id: String,
    pub similarity: f32,
    pub entry: VectorEntry,
}

pub struct IndexStats {
    pub total_vectors: usize,
    pub dimension: usize,          // 512
    pub index_size_bytes: u64,
    pub last_updated: DateTime<Utc>,
}
```

**VectorIndexError** 枚举：`IndexNotInitialized | DimensionMismatch | EmptyVector | InsertFailed | SearchFailed | SerializationError | DeserializationError | Io`

- 维度：512 (CLIP embedding 维度)
- 持久化到 `app_data/vector_index/` 目录

#### 知识图谱引擎 (`core::knowledge_graph`)

```rust
pub struct KnowledgeGraphEngine {
    db: Arc<Database>,
    clip_embedder: Arc<ClipEmbedder>,
    vector_index: Arc<HnswVectorIndex>,
}

pub enum NodeType { Image, Entity, Tag, Concept }

pub enum EdgeType {
    SemanticSimilarity, TagOverlap, TemporalProximity,
    LocationProximity, FaceMatch, Custom,
}

pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub community_id: Option<u32>,
    pub degree: u32,
}

pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f64,
    pub properties: serde_json::Value,
}

pub struct GraphCommunity {
    pub id: u32,
    pub size: usize,
    pub central_node_id: Option<String>,
    pub tags: Vec<String>,
    pub density: f64,
}
```

**内部数据结构**：
- `AdjacencyMap` — `Arc<RwLock<HashMap<String, HashSet<(String, String)>>>>`
- `NodeMap` — `Arc<RwLock<HashMap<String, GraphNode>>>`
- `EdgeMap` — `Arc<RwLock<HashMap<String, GraphEdge>>>`
- `CommunityList` — `Arc<RwLock<Vec<GraphCommunity>>>`

#### 熔断器 (`core::circuit_breaker`)

```rust
pub struct CircuitBreaker {
    state: AtomicU8,                    // Closed=0, Open=1, HalfOpen=2
    failure_count: AtomicU64,
    last_failure_ms: AtomicU64,
    half_open_permits: AtomicU64,
    half_open_success_threshold: u64,   // 默认 2
    half_open_success_count: AtomicU64,
    failure_threshold: u64,
    reset_timeout_ms: u64,
}
```

**状态机**：

```
Closed ──(失败数 ≥ threshold)──► Open
   ▲                                │
   │                                │ (超时后)
   │                                ▼
   └──(成功数 ≥ success_threshold)── HalfOpen
```

**关键方法**：

| 方法 | 说明 |
|------|------|
| `new(failure_threshold, reset_timeout_ms)` | 创建熔断器 |
| `allow_request()` | 判断是否允许请求 |
| `record_success()` | 记录成功 |
| `record_failure()` | 记录失败 |
| `current_state()` | 获取当前状态 |

#### 置信度校准 (`core::calibration`)

```rust
pub struct CalibrationEngine {
    db: Arc<Database>,
}

pub struct CalibrationSample {
    pub id: Option<i64>,
    pub image_id: i64,
    pub predicted_category: String,
    pub raw_confidence: f64,
    pub is_correct: bool,
    pub annotated_at: Option<String>,
}

pub struct CalibrationReport {
    pub id: Option<i64>,
    pub ece: f64,              // Expected Calibration Error
    pub mce: f64,              // Maximum Calibration Error
    pub total_samples: i64,
    pub n_bins: usize,         // 默认 10
    pub computed_at: String,
}

pub struct CalibrationCurvePoint {
    pub bin_index: usize,
    pub confidence_avg: f64,
    pub accuracy: f64,
    pub sample_count: usize,
}
```

#### 一致性校验 (`core::consistency_checker`)

```rust
pub struct ConsistencyChecker;
```

**关键方法**：

| 方法 | 说明 |
|------|------|
| `check_category_vs_tags(category, tags)` | 检查分类与标签一致性 |
| `check_category_vs_description(category, description)` | 检查分类与描述一致性 |

#### 文件监控 (`core::file_watcher`)

```rust
pub struct FileWatcherService {
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    tx: broadcast::Sender<FileChangeEvent>,
}

pub struct FileChangeEvent {
    pub event_type: String,
    pub file_path: String,
    pub timestamp: u64,
}
```

**关键方法**：

| 方法 | 说明 |
|------|------|
| `new(tx)` | 创建监控服务 |
| `watch_directory(dir)` | 开始监控目录 (递归) |
| `stop_watching()` | 停止监控 |

#### XMP 元数据服务 (`core::xmp_service`)

```rust
pub struct XmpMetadata {
    pub creator: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub subject: Vec<String>,
    pub keywords: Vec<String>,
    pub rating: Option<i32>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
}

pub struct XmpService;
```

**关键方法**：

| 方法 | 说明 |
|------|------|
| `read_xmp_from_file(path)` | 从文件读取 XMP 元数据 |
| `write_xmp_to_file(path, metadata)` | 写入 XMP 元数据 |
| `generate_sidecar(image_path, metadata)` | 生成 Sidecar 文件 |

#### 图像分类器 (`core::image_classifier`)

```rust
pub struct ImageClassifier {
    runtime_manager: Arc<OnnxRuntimeManager>,
}
```

- 使用 MobileNetV3 模型
- 内置 ImageNet 1000 类别标签
- 输入预处理：缩放到 224×224，归一化

### 4.4 Models 层 — 数据模型

#### Image 模型 (`models::image`)

```rust
pub struct Image {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: Option<String>,
    pub mime_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnail_path: Option<String>,
    pub phash: Option<String>,
    pub exif_data: Option<serde_json::Value>,
    pub ai_status: String,                // pending/processing/completed/failed/broken_link
    pub ai_tags: Option<serde_json::Value>,
    pub ai_description: Option<String>,
    pub ai_category: Option<String>,
    pub ai_confidence: Option<f64>,
    pub ai_model: Option<String>,
    pub ai_processed_at: Option<String>,
    pub ai_error_message: Option<String>,
    pub ai_retry_count: i32,
    pub source: String,                   // 默认 "import"
    pub generation_source: Option<String>,
    pub generation_metadata: Option<serde_json::Value>,
    pub generation_workflow_id: Option<String>,
    pub ai_provider: Option<String>,
    pub ai_tag_status: Option<String>,    // verified/provisional/rejected
    pub created_at: String,
    pub updated_at: String,
}
```

**关键方法**：`from_row(row: &rusqlite::Row) -> Result<Self>` — 从数据库行构造 Image，自动解析 JSON 字段

#### ImageCategory 枚举 (`models::category`)

```rust
pub enum ImageCategory {
    Landscape,    // 风景
    Person,       // 人物
    Object,       // 物品
    Animal,       // 动物
    Architecture, // 建筑
    Document,     // 文档
    Other,        // 其他
}
```

- 实现了 `FromStr` — 支持中英文解析 ("风景" → Landscape, "landscape" → Landscape)
- `as_str()` — 返回中文名称

#### Task 模型 (`models::task`)

```rust
pub struct Task {
    pub id: i64,
    pub image_id: i64,
    pub task_type: String,
    pub status: String,
    pub priority: i32,
    pub retry_count: i32,
    pub max_retries: i32,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}
```

### 4.5 Utils 层 — 工具函数

#### 加密模块 (`utils::crypto`)

**加密格式演进**：

| 版本 | 格式 | 密钥派生 | Nonce | 状态 |
|------|------|----------|-------|------|
| v1 | `enc:v1:` + base64 | 固定密钥 | 固定 | ❌ 已废弃，拒绝解密 |
| v2 | `enc:v2:` + base64(nonce+ciphertext) | SHA256(machine_id) | 随机 | ✅ 向后兼容 |
| v3 | `enc:v3:` + base64(salt+nonce+ciphertext) | PBKDF2-HMAC-SHA256 (600k 迭代) | 随机 | ✅ 当前使用 |

**关键函数**：

| 函数 | 说明 |
|------|------|
| `encrypt_api_key(plaintext)` | 加密 API Key (v3 格式) |
| `decrypt_api_key(ciphertext)` | 解密 API Key (支持 v2/v3，拒绝 v1) |
| `is_encrypted(value)` | 判断是否为加密格式 |
| `derive_key_v2()` | SHA256 密钥派生 (v2 兼容) |
| `derive_key_v3(salt)` | PBKDF2 密钥派生 (v3) |
| `generate_salt()` | 生成 16 字节随机 salt |

**安全参数**：PBKDF2 迭代 600,000 次 (OWASP 2023 推荐)，Salt 16 字节，Nonce 12 字节

#### 错误处理 (`utils::error`)

```rust
pub enum AppError {
    Database { code: String, message: String, source: rusqlite::Error },
    IoError { code: String, message: String, source: std::io::Error },
    ValidationError { code: String, message: String },
    AI { code: String, message: String },
    Http { code: String, message: String, source: reqwest::Error },
    Config { code: String, message: String },
}

pub type AppResult<T> = Result<T, AppError>;
```

**错误码体系**：

| 错误码 | 类型 | 说明 |
|--------|------|------|
| DB_001 | Database | 数据库错误 |
| IO_001 | IoError | IO 错误 |
| VAL_001 | Validation | 验证错误 |
| NF_001 | Validation | 资源不存在 |
| AUTH_001 | Validation | 认证失败 |
| AI_001 | AI | AI 推理错误 |
| HTTP_001 | Http | HTTP 请求错误 |
| CFG_001 / INT_001 | Config | 配置/内部错误 |

**关键函数**：

| 函数 | 说明 |
|------|------|
| `sanitize_error(msg)` | 错误消息脱敏 (替换路径为 [PATH]，SQL 为 [QUERY]) |
| `init_logging()` | 初始化日志系统 (委托给 log_sanitizer) |

**序列化**：`AppError` 实现了 `Serialize`，序列化为 `{ code, message }` 格式，message 经过脱敏处理

#### 哈希模块 (`utils::hash`)

| 函数 | 签名 | 说明 |
|------|------|------|
| `calculate_sha256` | `(file_path: &Path) -> io::Result<String>` | 计算文件 SHA256 哈希 |

#### 日志脱敏 (`utils::log_sanitizer`)

**脱敏规则**：

| 规则 | 正则匹配 | 替换为 |
|------|----------|--------|
| API Key 前缀 | `sk-`, `Bearer `, `api_key=`, `token=` 等 | 保留前缀 + `****` |
| 加密密文 | `enc:v1/v2:` + base64 | 保留前缀 + `****` |
| Windows 路径 | `C:\Users\...` | `[REDACTED_PATH]` |
| Unix 路径 | `/home/...`, `/Users/...` | `[REDACTED_PATH]` |
| URL 敏感参数 | `?token=`, `?api_key=` 等 | `param=[REDACTED]` |

**关键函数**：

| 函数 | 说明 |
|------|------|
| `sanitize_log_message(msg)` | 日志消息脱敏 (主入口) |
| `redact_api_key(key)` | API Key 脱敏 |
| `redact_path(path)` | 路径脱敏 |
| `redact_url(url)` | URL 脱敏 |
| `init_sanitized_logging()` | 初始化脱敏日志系统 |

**日志系统**：
- 日志目录：`%APPDATA%\ArcaneCodex\logs\app.log`
- 使用自定义 `SanitizedEventFormatter` 格式化器
- 所有日志消息经过 `sanitize_log_message()` 脱敏
- 环境变量控制日志级别 (默认 INFO)

**SanitizedDisplay** 泛型包装器：对任何 `Display` 类型自动脱敏

---

## 5. 前端架构 (React)

### 5.1 组件体系

#### 布局组件 (`components/layout/`)

| 组件 | 职责 |
|------|------|
| `MainLayout` | 主布局框架，嵌套 Sidebar + TopBar + children |
| `Sidebar` | 侧边导航栏，管理活动页面样式，6 个导航项 |
| `TopBar` | 顶部导航栏，搜索框 + 设置入口 |

#### 画廊组件 (`components/gallery/`)

| 组件 | 职责 |
|------|------|
| `ImageGrid` | 图片网格展示，虚拟滚动 (@tanstack/react-virtual) |
| `ImageCard` | 单张图片卡片，预览/标签/删除 |
| `ImageViewer` | 图片详情查看器 (含子组件) |
| `DropZone` | 拖拽上传区域 (react-dropzone) |
| `ImageFilter` | 图片过滤选项 (AI 状态/分类/日期/标签) |
| `ImportProgressBar` | 导入进度条 |
| `NarrativePrompt` | 叙事提示生成 |
| `SampleDataBanner` | 示例数据提示横幅 |

**ImageViewer 子组件**：

| 子组件 | 职责 |
|--------|------|
| `ImageBottomBar` | 底部操作栏 |
| `ImageInfoPanel` | 图片信息面板 (EXIF/AI 标签) |
| `ImageToolbar` | 工具栏 (缩放/导出/删除) |
| `SampleViewerPlaceholder` | 示例查看占位 |
| `useImageZoom` | 缩放 Hook |

**KnowledgeGraphView 子组件**：

| 子组件 | 职责 |
|--------|------|
| `GraphCanvas` | 图谱画布 (SVG) |
| `GraphHeader` | 图谱头部 |
| `GraphSidebar` | 图谱侧边栏 |
| `useForceSimulation` | 力导向布局 Hook |

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
| `LogViewer` | 日志查看器 |
| `AboutPage` | 关于页面 |

#### 其他组件

| 组件 | 路径 | 职责 |
|------|------|------|
| `DedupManager` | `components/dedup/` | 去重管理 |
| `AccuracyChart` | `components/dashboard/` | 准确率图表 |
| `ErrorBoundary` | `components/common/` | 错误边界捕获 |

#### 页面组件 (`pages/`)

| 页面 | 路由 | 子组件 |
|------|------|--------|
| `GalleryPage` | gallery | GalleryActionBar, GalleryEmptyState, GallerySearchResults, useGalleryActions |
| `AIPage` | ai | AIProgressPanel, LMStudioGuide |
| `DedupPage` | dedup | DedupManager |
| `DashboardPage` | dashboard | AccuracyChart |
| `SettingsPage` | settings | 各 Config 组件 |
| `KnowledgeGraphView` | knowledge_graph | GraphCanvas, GraphHeader, GraphSidebar |

### 5.2 状态管理 (Zustand)

#### useImageStore

```typescript
interface ImageState {
  images: AppImage[]
  selectedIds: number[]
  filters: { ai_status?, date_from?, date_to?, category?, tags? }
  page: number
  pageSize: number
  total: number
  loading: boolean
  error: string | null
  searchQuery: string
  searchResults: SearchResult[]
  searchLoading: boolean
  hasSearched: boolean
}
```

**关键 Actions**：`setImages`, `toggleSelect`, `selectAll`, `deselectAll`, `setFilters`, `loadImages`, `clearSearch`

**持久化**：`filters` 和 `pageSize` 通过 `zustand/persist` 持久化到 localStorage

#### useAIStore

```typescript
interface AIState {
  status: 'idle' | 'processing' | 'paused' | 'completed' | 'failed'
  total: number
  completed: number
  failed: number
  retrying: number
  eta_seconds?: number
}
```

**关键 Actions**：`setStatus`, `updateStatus`, `reset`

#### useConfigStore

```typescript
interface ConfigState {
  lmStudioUrl: string           // 默认 "http://localhost:1234"
  aiConcurrency: number         // 默认 3
  aiTimeout: number             // 默认 60
  thumbnailSize: number         // 默认 300
  theme: string                 // 默认 "system"
  language: string              // 默认 "zh"
  notificationEnabled: boolean  // 默认 true
  notificationAiComplete: boolean
  notificationDedupComplete: boolean
  privacySendAnalytics: boolean // 默认 false
  privacyShareData: boolean     // 默认 false
  isLoaded: boolean
  loadError: string | null
  pendingChanges: Partial<Record<ConfigKey, string>>
  discoveredModels: DiscoveredModel[]
  aiServiceReady: boolean
  aiServiceScanning: boolean
}
```

**CONFIG_KEYS 常量**：`LM_STUDIO_URL`, `AI_CONCURRENCY`, `AI_TIMEOUT`, `THUMBNAIL_SIZE`, `THEME`, `LANGUAGE`, `NOTIFICATION_ENABLED`, `NOTIFICATION_AI_COMPLETE`, `NOTIFICATION_DEDUP_COMPLETE`, `PRIVACY_SEND_ANALYTICS`, `PRIVACY_SHARE_DATA`

**关键 Actions**：`loadConfigs`, `updateField`, `saveConfigs` (带回滚), `hasPendingChanges`, `scanAiServices`, `setAiServiceReady`

**saveConfigs 回滚机制**：逐个设置配置，失败时回滚已应用的配置

#### useDedupStore

```typescript
interface DedupState {
  groups: DuplicateGroup[]
  loading: boolean
  threshold: number    // 默认 95
}
```

**关键 Actions**：`setGroups`, `setLoading`, `setThreshold`, `removeGroups`, `reset`

#### useThemeStore

```typescript
type Theme = 'light' | 'dark' | 'system'

interface ThemeStore {
  applyTheme: (theme: Theme) => void
}
```

- 通过 `document.documentElement.classList` 切换 `dark`/`light` 类名
- 监听系统主题变化 (`prefers-color-scheme`)

### 5.3 路由系统

前端使用自定义状态路由系统 (`router/`)，而非 react-router：

**路由类型**：

```typescript
type AppRoute = 'gallery' | 'settings' | 'ai' | 'dedup' | 'dashboard' | 'knowledge_graph'
type NavigationSource = 'sidebar' | 'action' | 'tauri-event' | 'keyboard' | 'system'
```

**事件系统** (`router/events.ts`)：

| 事件 | 说明 |
|------|------|
| `app:route-change` | 路由变更 |
| `app:route-back` | 后退 |
| `app:route-forward` | 前进 |

**导航函数**：`navigate(payload)`, `goBack()`, `goForward()`

**双通道事件**：同时支持 Tauri 事件和 Web CustomEvent，确保在 Tauri 窗口和浏览器中都能工作

**状态路由** (`router/state-router.ts`)：

```typescript
interface RouterState {
  current: AppRoute
  history: AppRoute[]
  index: number
  params: Record<string, string>
  initialized: boolean
}
```

**路由守卫**：`VALID_TRANSITIONS` 定义合法路由跳转，阻止非法跳转

**历史持久化**：路由历史存储在 localStorage (`tesr-history`)，最多 100 条

**useStateRouter Hook**：返回 `{ current, params, initialized, canGoBack, canGoForward, history, goBack, goForward }`

### 5.4 API 调用层

`lib/api.ts` — 封装所有 Tauri IPC 调用 (1043 行)，共 50+ 个 API 函数：

| 类别 | 方法 | 说明 |
|------|------|------|
| 图片管理 | `importImages`, `getImages`, `getImageDetail`, `deleteImages` | 图片 CRUD |
| AI 处理 | `startAIProcessing`, `pauseAIProcessing`, `resumeAIProcessing`, `getAIStatus`, `retryFailedAI`, `retrySingleAIResult` | AI 队列控制 |
| 语义搜索 | `searchImages` | 语义搜索 |
| 去重 | `scanDuplicates`, `deleteDuplicates` | 去重操作 |
| 设置 | `getConfig`, `setConfig`, `getAllConfigs`, `setConfigs` (带回滚) | 配置管理 |
| 备份 | `backupDatabase`, `restoreDatabase` | 数据库备份/恢复 |
| AI 服务检测 | `discoverAvailableModels`, `detectAiService`, `autoConfigureInference` | AI 服务自动发现 |
| 数据导出 | `exportData` | JSON/CSV 导出 |
| 断链检测 | `checkBrokenLinks` | 检查断链 |
| 归档 | `archiveImage` | 图片归档 |
| 安全导出 | `safeExport` | 安全导出图片文件 |
| 叙事锚点 | `writeNarrative`, `getNarratives`, `queryAssociations` | 叙事管理 |
| 仪表盘 | `getLibraryStats`, `getAccuracyTrend` | 统计数据 |
| 日志 | `getLogEntries`, `getLogStats`, `exportLogs`, `clearLogs` | 日志管理 |
| 示例数据 | `checkSampleData`, `clearSampleData`, `loadSampleData` | 示例数据 |
| XMP | `readXmpMetadata`, `writeXmpMetadata`, `generateXmpSidecar`, `exportAsXmp` | XMP 元数据 |
| 文件监控 | `startFileMonitor`, `stopFileMonitor`, `getMonitorStatus` | 文件监控 |
| ONNX 模型 | `getAIModelStatus`, `loadAIModel`, `unloadAIModel` | 模型管理 |
| 图像分类 | `classifyImage` | MobileNetV3 分类 |
| 人脸 | `detectFaces`, `extractFaceEmbedding`, `registerFace`, `recognizeFace`, `getRegisteredFaceCount` | 人脸检测/识别 |
| CLIP | `embedImageWithClip` | CLIP 嵌入 |
| 向量搜索 | `insertVector`, `searchVectors`, `deleteVector`, `getVectorIndexStats` | HNSW 向量搜索 |
| 知识图谱 | `kgBuildGraph`, `kgGetStats`, `kgGetAllNodes`, `kgGetAllEdges`, `kgGetCommunities`, `kgGetCommunityNodes`, `kgGetNeighbors`, `kgFindPath`, `kgSearchNodes`, `kgClear`, `kgLoadFromDb`, `kgSaveToDb` | 知识图谱 |
| 标签修正 | `recordTagCorrection`, `getTagCorrectionHistory`, `getAllTagCorrections` | 标签修正 |
| 错误模式 | `recordErrorPattern`, `getErrorPatterns`, `checkErrorPatternExists`, `deleteErrorPattern`, `getHighFrequencyErrorPatterns` | 错误模式 |
| 批量操作 | `startBatchAITag`, `getBatchAIStatus`, `pauseBatchAITask`, `resumeBatchAITask`, `cancelBatchAITask`, `batchTagCorrection`, `batchExport` | 批量操作 |

**Tauri 上下文检测**：`invoke` 函数检查 `window.__TAURI_INTERNALS__` 是否存在，不存在时抛出错误

**其他 lib 模块**：

| 模块 | 说明 |
|------|------|
| `errorMap` | 错误码映射 |
| `motion` | motion (framer-motion) 动画配置 |

### 5.5 国际化 (i18n)

- 框架：i18next + react-i18next
- 语言文件：`i18n/zh.json`, `i18n/en.json`
- 默认语言：zh，回退语言：en
- 翻译类别：common, navigation, gallery, ai, settings, errors, dashboard, logs, narrative
- 切换方式：`useConfigStore.language` 变更时调用 `i18n.changeLanguage()`

### 5.6 TypeScript 类型定义 (`types/image.ts`)

```typescript
type AIStatusEnum = 'pending' | 'processing' | 'completed' | 'failed'
type Page = 'gallery' | 'settings' | 'ai' | 'dedup' | 'dashboard' | 'knowledge_graph'

interface Toast {
  id: number
  message: string
  type: 'error' | 'success' | 'info'
}
```

`AppImage` 类型从 `lib/api.ts` 重新导出，确保类型一致性

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
  ├──1:1── xmp_sidecars
  └──1:N── calibration_samples

kg_nodes ──N:N── kg_edges
kg_nodes ──N:1── kg_communities

settings (key-value)
calibration_reports / calibration_curves
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
| generation_workflow_id | TEXT | 生成工作流 ID |
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
| ai_timeout | 60 | AI 超时 |
| theme | system | 主题 |
| language | zh | 语言 |
| thumbnail_size | 300 | 缩略图尺寸 |
| inference_provider | lm_studio | 推理提供者 |
| inference_model | Qwen2.5-VL-7B-Instruct | 推理模型 |
| inference_api_key | (空) | API Key (加密存储) |

#### 知识图谱表

| 表 | 说明 |
|----|------|
| `kg_nodes` | 图谱节点 (id, node_type, label, properties_json, embedding_json, community_id, degree) |
| `kg_edges` | 图谱边 (id, source_id, target_id, edge_type, weight, properties_json) |
| `kg_communities` | 社区 (id, size, central_node_id, tags_json, density) |

#### 校准相关表

| 表 | 说明 |
|----|------|
| `calibration_samples` | 校准样本 (image_id, predicted_category, raw_confidence, is_correct) |
| `calibration_reports` | 校准报告 (ece, mce, total_samples, n_bins) |
| `calibration_curves` | 校准曲线 (bin_index, confidence_avg, accuracy, sample_count) |

---

## 7. 依赖关系

### Rust 依赖 (Cargo.toml)

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tauri` | 2 | 桌面应用框架 (protocol-asset feature) |
| `tauri-plugin-shell` | 2 | Shell 插件 |
| `tauri-plugin-dialog` | 2 | 对话框插件 |
| `serde` / `serde_json` | 1 | 序列化 (derive feature) |
| `rusqlite` | 0.31 | SQLite 驱动 (bundled + chrono features) |
| `r2d2` / `r2d2_sqlite` | 0.8 / 0.24 | 连接池 |
| `tokio` | 1 | 异步运行时 (full features) |
| `reqwest` | 0.12 | HTTP 客户端 (json/stream/multipart features) |
| `image` | 0.25 | 图像处理 (png/jpeg/webp/tiff/gif/ico/bmp features) |
| `kamadak-exif` | 0.5 | EXIF 解析 |
| `jieba-rs` | 0.7 | 中文分词 |
| `thiserror` / `anyhow` | 1 | 错误处理 |
| `regex` | 1 | 正则表达式 |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | 日志 (env-filter/json/local-time features) |
| `chrono` | 0.4 | 时间处理 (serde feature) |
| `walkdir` | 2 | 目录遍历 |
| `sha2` / `hex` | 0.10 / 0.4 | 哈希计算 |
| `async-channel` | 2 | 异步通道 |
| `async-trait` | 0.1 | 异步 trait |
| `uuid` | 1 | UUID 生成 (v4/serde features) |
| `zip` | 2 | ZIP 压缩 (备份) |
| `data-encoding` | 2 | Base64 编码 |
| `aes-gcm` | 0.10 | AES-256-GCM 加密 |
| `base64` | 0.22 | Base64 编解码 |
| `whoami` | 1 | 系统信息 (密钥派生) |
| `rand` | 0.9 | 随机数生成 |
| `pbkdf2` | 0.12 | PBKDF2 密钥派生 (hmac feature) |
| `hmac` | 0.12 | HMAC |
| `xmpkit` | 0.1 | XMP 元数据读取 |
| `xmp-writer` | 0.3 | XMP 元数据写入 |
| `notify` | 7 | 文件系统监控 (macos_kqueue feature) |
| `ort` | 2.0.0-rc.12 | ONNX Runtime (download-binaries/copy-dylibs features) |
| `winapi` | 0.3 | Windows API (磁盘空间，仅 Windows) |
| `libc` | 0.2 | POSIX API (仅 Unix) |

**开发依赖**：`tempfile` (3), `mockito` (1)

### 前端依赖 (package.json)

#### 运行时依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `react` / `react-dom` | ^18.3.1 | UI 框架 |
| `@tauri-apps/api` | ^2.0.0 | Tauri IPC 调用 |
| `@tauri-apps/plugin-dialog` | ^2.7.0 | Tauri 对话框 |
| `@tauri-apps/plugin-shell` | ^2.3.5 | Tauri Shell |
| `zustand` | ^4.5.4 | 状态管理 |
| `@tanstack/react-virtual` | ^3.10.4 | 虚拟滚动 |
| `i18next` | ^23.14.0 | 国际化 |
| `react-i18next` | ^15.0.1 | React 国际化绑定 |
| `lucide-react` | ^0.438.0 | 图标库 |
| `motion` | ^11.18.2 | 动画 (framer-motion 新包名) |
| `react-dropzone` | ^14.2.3 | 拖拽上传 |
| `clsx` | ^2.1.1 | 类名合并 |

#### 开发依赖

| 依赖 | 用途 |
|------|------|
| `vite` / `@vitejs/plugin-react` | 构建工具 |
| `typescript` | 类型检查 |
| `tailwindcss` / `postcss` / `autoprefixer` | CSS 框架 |
| `vitest` / `@testing-library/react` / `@testing-library/jest-dom` | 测试 |
| `eslint` / `typescript-eslint` / `eslint-plugin-react-hooks` | 代码检查 |
| `@playwright/test` / `playwright` | E2E 测试 |
| `jsdom` | DOM 模拟 |
| `tsx` | TypeScript 执行 |

### 模块间依赖关系

```
commands/ ──依赖──► core/
                    models/
                    utils/

core/ ──依赖──► models/
                utils/
                (Tauri AppHandle 仅 ai_queue 使用)

models/ ──依赖──► (无内部依赖，纯数据结构)

utils/ ──依赖──► (无内部依赖，纯工具)

core/ 内部依赖:
  ai_queue ──► inference, db, search_index, consistency_checker, circuit_breaker, crypto
  inference ──► lm_studio
  knowledge_graph ──► db, clip_embedder, vector_index
  clip_embedder ──► onnx_runtime
  face_detector ──► onnx_runtime
  image_classifier ──► onnx_runtime
  dedup ──► bk_tree, db
  search_index ──► db
```

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
cd src-tauri && cargo fmt          # Rust 格式化
cd src-tauri && cargo clippy       # Rust Lint
cd frontend && npm run lint        # ESLint
npx tsc --noEmit                   # TypeScript 类型检查
```

### Vite 构建配置

- 开发端口：1420
- 路径别名：`@` → `src/`
- Source map：`VITE_SOURCEMAP=true` 时启用（生产默认关闭）
- 手动分块：vendor-react, vendor-state, vendor-ui, vendor-i18n, vendor-virtual, vendor-dropzone

### Tauri 配置摘要

| 配置 | 值 |
|------|-----|
| 产品名 | ArcaneCodex |
| 标识符 | com.arcanecodex.app |
| 窗口尺寸 | 1280×800 |
| CSP | 严格 (仅允许 localhost:1234 等本地 AI 服务) |
| 打包格式 | NSIS (中文/英文语言选择器) |
| 协议 | asset 协议 + custom-protocol |
| Asset 协议范围 | $APPDATA, $APPLOCALDATA, $DOWNLOAD, $DOCUMENT, $PICTURE |
| 安全特性 | freezePrototype, 严格权限列表 |

---

## 9. 安全机制

| 机制 | 实现 | 状态 |
|------|------|------|
| CSP 内容安全策略 | tauri.conf.json 严格配置 | ✅ |
| 上下文隔离 | Tauri 默认机制 | ✅ |
| API Key 加密 (v3) | AES-256-GCM + PBKDF2 (600k 迭代) + 随机 Salt + 随机 Nonce | ✅ |
| API Key 加密 (v2) | AES-256-GCM + SHA256 派生 + 随机 Nonce | ✅ 向后兼容 |
| API Key 加密 (v1) | 固定 Nonce | ❌ 已废弃，拒绝解密 |
| 备份加密 | AES-256-GCM + 随机 Nonce | ✅ |
| 路径穿越防护 | `sanitize_path()` + `normalize_path()` + `canonicalize()` | ✅ |
| 敏感目录保护 | `is_sensitive_directory()` — 禁止导出到系统目录 | ✅ |
| 磁盘空间预检 | 导入/导出前检查可用空间 | ✅ |
| 文件魔术字节验证 | `validate_magic_bytes()` — 防止扩展名伪造 | ✅ |
| 日志脱敏 | `log_sanitizer` — API Key/路径/SQL/URL 参数自动脱敏 | ✅ |
| 错误消息脱敏 | `sanitize_error()` — 替换路径和 SQL | ✅ |
| 熔断保护 | `CircuitBreaker` — 保护 AI 推理调用 | ✅ |

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
  ├─ 停用词过滤
  ├─ 倒排索引查询 (search_index 表)
  ├─ 权重排序 (relevance_score)
  └─ 返回匹配图片列表
```

### 向量搜索流程

```
用户触发向量搜索
  │
  ▼
Commands::ai_core::embed_image_clip → ClipEmbedding (512 维)
  │
  ▼
Commands::ai_core::search_vectors(query, top_k, min_similarity)
  ├─ HnswVectorIndex::search() — HNSW 近似最近邻
  └─ 返回相似图片列表
```

### 去重扫描流程

```
用户触发去重扫描
  │
  ▼
Commands::dedup::scan_duplicates
  ├─ 从数据库加载所有图片 pHash
  ├─ 构建 BK-Tree
  ├─ 对每个 pHash 执行近似搜索
  ├─ UnionFind 聚类相似图片
  ├─ 计算相似度百分比
  └─ 返回 DuplicateGroup 列表
```

---

## 11. CLI 工具

**位置**：`arcane-codex-cli/arcanecodex.py`
**框架**：Python Click
**命令前缀**：`ac`

### 命令组

| 命令组 | 子命令 | 说明 |
|--------|--------|------|
| `ac image` | `import`, `list`, `delete` | 图片管理 |
| `ac ai` | `start`, `pause`, `resume`, `status`, `retry` | AI 处理 |
| `ac search` | `query`, `rebuild-index` | 语义搜索 |
| `ac dedup` | `scan`, `delete` | 智能去重 |
| `ac system` | `db-info`, `db-backup`, `db-restore`, `config`, `health` | 系统管理 |

### 核心命令示例

```bash
ac image import --path ./photos --recursive     # 递归导入
ac image list --filter "category:风景" --format json  # 过滤列表
ac ai start --concurrency 3                     # 启动 AI (并发 3)
ac search --query "日落 海滩" --limit 20        # 语义搜索
ac dedup scan --threshold 90                    # 去重扫描
ac system db-backup --output ./backup.zip       # 数据库备份
```

### 设计原则

- **Agent-Native**：所有命令支持 `--format json` 输出
- **结构化输出**：JSON 格式包含 `command`, `status`, `data`, `meta` (执行时间/时间戳)
- **错误处理**：结构化错误码 + 建议

---

## 12. CI/CD 流水线

### CI 流水线 (`.github/workflows/ci.yml`)

**触发条件**：push/PR 到 master/main 分支

```
lint-rust (ubuntu) ──┐
                      ├──► build (windows)
lint-frontend (ubuntu)┤
                      │
test-frontend (ubuntu)┘
```

| Job | 运行环境 | 步骤 |
|-----|----------|------|
| lint-rust | ubuntu-latest | checkout → 安装 Tauri 依赖 → Rust 工具链 → cargo fmt --check → cargo clippy -D warnings |
| lint-frontend | ubuntu-latest | checkout → Node 20 → npm ci → ESLint → tsc --noEmit |
| test-frontend | ubuntu-latest | checkout → Node 20 → npm ci → vitest run --pool=forks |
| build | windows-latest | checkout → Node 20 → npm ci → Rust 工具链 → 安装 Tauri CLI → npm run build → npm run tauri build → 上传 artifact |

### 发布流水线 (`.github/workflows/release.yml`)

- 基于 Git tag 触发
- 构建 Windows MSI + EXE 安装包
- 上传到 GitHub Release

---

> 文档生成时间: 2026-05-13 | 基于 Arcane Codex v1.0.0 源码分析
