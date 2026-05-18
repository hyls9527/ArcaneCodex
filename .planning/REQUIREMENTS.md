# Requirements

> Generated: 2026-05-17
> Version: v1.0.0-rc
> Status: Implemented (Phase 9 remaining)

## v1 Requirements

### REQ-SETUP-01: Project Scaffolding and Infrastructure

- **Status**: Delivered (Phase 1)
- **Description**: Tauri 2.x project scaffolding, React 18 + TypeScript + Vite frontend, Rust dependency configuration, SQLite database initialization with migration system, unified error handling, and structured logging.
- **Acceptance Criteria**:
  - Tauri 2.x + React 18 + Vite project initializes and builds
  - Rust Cargo.toml configured with all core dependencies
  - SQLite database initializes with `images`, `tags`, `image_tags`, `search_index`, `task_queue`, `app_config` tables
  - Migration system supports versioned schema upgrades (v1-v6)
  - Unified `AppError` enum with `thiserror` for all error types
  - Structured logging via `tracing` + `tracing-subscriber` to file

### REQ-IMG-01: Image Import and Storage Pipeline

- **Status**: Delivered (Phase 2)
- **Description**: Drag-and-drop image import with file validation, SHA256 duplicate detection, thumbnail generation (WebP 300x200 quality=80), pHash calculation, EXIF metadata extraction, and index-reference storage mode.
- **Acceptance Criteria**:
  - Drag-and-drop single/multiple images and folders
  - File validation: size <= 50MB, supported formats (JPEG, PNG, WebP, GIF, BMP, TIFF)
  - SHA256-based duplicate detection on import
  - Thumbnail generation: max 300x200px, WebP quality=80, stored at `%APPDATA%\ArcaneCodex\thumbnails\{id}.webp`
  - Index-reference mode (don't move originals, record path in database)
  - Optional archive mode: copy to `%APPDATA%\ArcaneCodex\images\`
  - Broken link detection: periodic scan for missing originals, mark `broken_link` status
  - Background async processing via `tokio::spawn_blocking`

### REQ-AI-01: Automatic AI Tagging via LM Studio

- **Status**: Delivered (Phase 2 core, Phase 5 pipeline completed)
- **Description**: AI-powered automatic image tagging using local LLM backend. Generates tags, description, and category for each imported image via configurable inference provider.
- **Acceptance Criteria**:
  - Auto-create `task_queue` record (status=pending) on import
  - Background worker scans queue, calls inference API (LM Studio default)
  - Semaphore-based concurrency control (default 3 concurrent workers)
  - Returns structured JSON: tags (5-10 keywords), description (1-2 sentences), category, confidence
  - On success: update images table, build search index
  - On failure: retry up to 3 times with exponential backoff (2s -> 4s -> 8s)
  - Inference service health check on startup, auto-reconnect every 30s
  - Pause/resume/cancel queue operations
  - Chinese-priority prompt template

### REQ-SEARCH-01: Chinese Semantic Search

- **Status**: Delivered (Phase 2, Phase 5 caching)
- **Description**: Semantic search engine using jieba-rs Chinese word segmentation and inverted index table, with LRU caching for repeated queries.
- **Acceptance Criteria**:
  - Chinese/English keyword input with 300ms debounce
  - jieba-rs word segmentation on query and indexed content
  - Inverted index in `search_index` table (term, image_id, field, position)
  - Relevance scoring by matched term count
  - Filter combination: time range, category, tags
  - LRU cache with 5-minute TTL for identical queries
  - Narrative fallback when tag search returns no results

### REQ-DEDUP-01: Near-Duplicate Detection

- **Status**: Delivered (Phase 2, Phase 5 optimization)
- **Description**: Perceptual hash (pHash) based near-duplicate image detection with BK-Tree clustering and side-by-side comparison UI.
- **Acceptance Criteria**:
  - pHash calculation on import using mean hash algorithm
  - Hamming distance comparison with configurable threshold (70-99%, default 90%)
  - BK-Tree clustering (O(n log n)) for performance at 5000+ images
  - Union-Find cluster aggregation
  - Side-by-side comparison view in UI
  - Retention policies: highest resolution, earliest import, or manual selection
  - Batch delete with cascading cleanup

### REQ-SETTINGS-01: Configuration and Preferences

- **Status**: Delivered (Phase 2)
- **Description**: System settings page with AI configuration, display settings, storage management, and about page.
- **Acceptance Criteria**:
  - AI configuration: inference address, concurrency (1-10), timeout, test connection
  - Display settings: theme (light/dark/system), language (zh-CN/en-US), thumbnail size
  - Storage: data directory path, database backup/restore via zip with version validation
  - About page: version, license, open source links
  - Settings persisted to unified `settings` table (v6 migration from `app_config`)

### REQ-NARRATIVE-01: Narrative Anchor System

- **Status**: Delivered (Phase 5.7)
- **Description**: Memory-style narrative annotations. Users write short narratives about images; entities are auto-extracted for associative search.
- **Acceptance Criteria**:
  - Database tables: `narratives`, `semantic_edges` (v3 migration)
  - Write narrative via Tauri command with auto entity extraction (person/place/time/event)
  - Entity tagging color coding: person=blue, location=green, time=orange, event=purple
  - Narrative display as cards in ImageViewer
  - Semantic search fallback: LIKE match on narrative content (relevance_score=0.5)
  - Conversation-style prompts with rotating placeholders (5 each for zh/en)
  - i18n support for narrative UI (15 keys per locale)

### REQ-PROVIDER-01: Multi-Inference Provider Support

- **Status**: Delivered (Phase 6)
- **Description**: Abstract inference provider trait with 6 adapters (LM Studio, Ollama, Hermes, Zhipu, OpenAI, OpenRouter) and model auto-discovery.
- **Acceptance Criteria**:
  - `InferenceProvider` trait with `analyze_image` / `health_check` interface
  - LM Studio, Ollama, Hermes (local), Zhipu, OpenAI, OpenRouter (cloud) adapters
  - `ProviderFactory::create(config)` dynamic provider instantiation
  - `ModelDiscoveryService` scans local ports for available services
  - Runtime switching of inference provider via settings
  - Database v4 migration for multi-provider config
  - Frontend inference source selector with test connection and auto-discover buttons

### REQ-ACCURACY-01: AI Tagging Accuracy Improvement

- **Status**: Delivered (Phase 7, with deferred items)
- **Description**: Confidence calibration, independent verification, and tag status grading for AI-generated tags.
- **Acceptance Criteria**:
  - Confidence calibration module using model confidence -> real accuracy curves per category
  - ECE (Expected Calibration Error) calculation
  - CLIP zero-shot interface for cross-validation (Python sidecar -- deferred to v1.1)
  - Three-tier tag status: `verified`, `provisional`, `rejected`
  - Verified tags participate in search; provisional and rejected do not
  - User correction tracking via `correction_history` table
  - Error pattern library for known failure modes (deferred to v1.1)
  - Prompt fine-tuning from correction samples (deferred to v1.1)

### REQ-EXPORT-01: Data Export

- **Status**: Delivered (Phase 5.3)
- **Description**: Export image metadata to JSON/CSV/XML format with path safety validation.
- **Acceptance Criteria**:
  - Export metadata to JSON, CSV, XML formats
  - Export from single image view or batch selection
  - Secure path validation (`validate_export_path`) prevents export to system directories
  - Export progress indication
  - 8 unit tests verifying export functionality

### REQ-BATCH-01: Batch Operations

- **Status**: Delivered (Phase 8.1)
- **Description**: Batch AI tagging, batch tag correction, and batch export operations.
- **Acceptance Criteria**:
  - Select multiple images for batch AI analysis
  - Batch task progress display with pause/resume/cancel
  - Multi-select batch tag add/remove/replace
  - Batch export images + metadata with format selection
  - Unit tests for all batch operations

### REQ-STATS-01: Data Visualization Dashboard

- **Status**: Delivered (Phase 8.2)
- **Description**: Image library statistics dashboard and AI accuracy trend visualization.
- **Acceptance Criteria**:
  - Dashboard: total images, category distribution, tag word cloud
  - AI tagging progress statistics
  - Storage usage display
  - AI accuracy trend chart by time and category
  - i18n translation keys for all dashboard UI

### REQ-PROD-01: Production Readiness

- **Status**: In progress (Phase 9)
- **Description**: Production hardening including CI automation, security fixes, performance optimization, and compilation verification.
- **Acceptance Criteria**:
  - CI pipeline runs tests (cargo test, vitest) on every push
  - `get_available_disk_space` fails gracefully instead of returning `u64::MAX`
  - API keys encrypted at rest (AES-256-GCM)
  - Unified `settings` table (v6 migration removes `app_config`)
  - Image model includes all fields (`ai_provider`, `ai_tag_status`, etc.)
  - Export path validation prevents system directory traversal
  - Database restore validates version compatibility
  - LRU search cache with 5-minute TTL and proper invalidation
  - CI security audit steps (cargo audit, npm audit)
  - Zip crate upgraded from 0.6 to 2.0
  - Double ends build verification (cargo check, tsc --noEmit)
  - cargo test passes on Windows (blocked: STATUS_ENTRYPOINT_NOT_FOUND)

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REQ-SETUP-01 | Phase 1 | Delivered |
| REQ-IMG-01 | Phase 2 | Delivered |
| REQ-AI-01 | Phase 2 | Delivered |
| REQ-SEARCH-01 | Phase 2 | Delivered |
| REQ-DEDUP-01 | Phase 2 | Delivered |
| REQ-SETTINGS-01 | Phase 2 | Delivered |
| REQ-EXPORT-01 | Phase 5 | Delivered |
| REQ-NARRATIVE-01 | Phase 5 | Delivered |
| REQ-PROVIDER-01 | Phase 6 | Delivered |
| REQ-ACCURACY-01 | Phase 7 | Delivered |
| REQ-BATCH-01 | Phase 8 | Delivered |
| REQ-STATS-01 | Phase 8 | Delivered |
| REQ-PROD-01 | Phase 9 | In Progress |
