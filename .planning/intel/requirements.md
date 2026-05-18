# Requirements (PRDs)

## Requirement: REQ-image-import

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/pre-requirements.md` + `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/01-requirements.md`
- **Description**: Image import and storage system. Support drag-and-drop import, file validation, duplicate detection (SHA256), thumbnail generation, and EXIF metadata extraction.
- **Acceptance Criteria**:
  - Drag-and-drop single/multiple images from file system
  - File validation: size <= 50MB, supported formats (JPEG, PNG, WebP, GIF, BMP, TIFF)
  - SHA256-based duplicate detection on import
  - Thumbnail generation: max 300x200px, WebP quality=80, stored at `%APPDATA%\ArcaneCodex\thumbnails\{image_id}.webp`
  - Index mode default (don't move original files, record path in database)
  - Optional archive mode: copy to `%APPDATA%\ArcaneCodex\images\`
  - Broken link detection: periodic scan for missing originals, mark `broken_link` status
  - Background async processing, thumbnail generation in `tokio::spawn_blocking`

## Requirement: REQ-ai-auto-tagging

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/01-requirements.md`
- **Description**: AI-powered automatic image tagging using local LLM (LM Studio). Generate tags, description, and category for each imported image.
- **Acceptance Criteria**:
  - Auto-create `task_queue` record (status=pending) on import
  - Rust background worker scans queue, calls LM Studio API at `localhost:1234/v1/chat/completions`
  - Semaphore-based concurrency control (default 3 concurrent workers)
  - Returns structured JSON: tags (5-10 keywords), description (1-2 sentences), category, confidence
  - On success: update `images` table (ai_status=completed, tags, description, category, confidence), build search index
  - On failure: retry up to 3 times with exponential backoff (2s->4s->8s)
  - LM Studio health check on startup, auto-reconnect every 30s
  - Pause/resume/cancel queue operations via broadcast channel
  - Chinese-priority prompt template, category system: nature/people/objects/animals/architecture/documents/other

## Requirement: REQ-semantic-search

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/01-requirements.md`
- **Description**: Semantic search engine using jieba-rs Chinese word segmentation and inverted index.
- **Acceptance Criteria**:
  - Chinese/English keyword input
  - jieba-rs word segmentation on both query and indexed content
  - Inverted index in `search_index` table (term, image_id, field, position)
  - Relevance scoring by matched term count
  - Filter combination: time range, category, tags
  - LRU cache for identical queries within 5-minute TTL
  - Narrative fallback: when tag search returns no results, fall back to narrative LIKE matching (relevance_score=0.5)

## Requirement: REQ-smart-dedup

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/01-requirements.md`
- **Description**: Perceptual hash (pHash) based near-duplicate image detection and management.
- **Acceptance Criteria**:
  - pHash calculation on import using mean hash algorithm (aHash)
  - Hamming distance comparison with configurable similarity threshold (70-99%, default 90%)
  - BK-Tree clustering (O(n log n)) for performance
  - Union-Find cluster aggregation
  - Side-by-side comparison view in UI
  - Retention policies: highest resolution, earliest import, or manual selection
  - Batch delete with cascading cleanup (search_index, thumbnails, images)

## Requirement: REQ-settings-system

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/01-requirements.md`
- **Description**: System settings page with AI config, display settings, storage management.
- **Acceptance Criteria**:
  - AI configuration: LM Studio address, concurrency (1-10), timeout, test connection
  - Display settings: theme (light/dark/system), language (zh-CN/en-US), thumbnail size
  - Storage: data directory path, database backup/restore via zip
  - About page: version, license, open source links
  - Settings persisted to `settings` table (migrated from `app_config` in v6 migration)

## Requirement: REQ-narrative-anchor

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/04-ralph-tasks.md` (Phase 5.7)
- **Description**: Narrative anchor system for memory-style image annotation. Users write short narratives about images; entities are auto-extracted for associative search.
- **Acceptance Criteria**:
  - Database tables: narratives, semantic_edges (v3 migration)
  - Write narrative via Tauri command with auto entity extraction (person/place/time/event)
  - Entity tagging: person=blue, location=green, time=orange, event=purple
  - Narrative display as cards in ImageViewer
  - Semantic search fallback: LIKE match on narrative content
  - Conversation-style prompts with rotating placeholders (5 each for zh/en)
  - i18n support for narrative UI

## Requirement: REQ-multi-inference-provider

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/04-ralph-tasks.md` (Phase 6)
- **Description**: Multi-inference source support with abstract provider trait and model auto-discovery.
- **Acceptance Criteria**:
  - `InferenceProvider` trait with LM Studio, Ollama, Hermes, Zhipu, OpenAI, OpenRouter adapters
  - ProviderFactory for dynamic creation
  - ModelDiscoveryService scans local ports for available services
  - Runtime switching of inference provider via settings
  - Database v4 migration for multi-provider config
  - Frontend inference source selector with test connection button

## Requirement: REQ-ai-accuracy

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/04-ralph-tasks.md` (Phase 7)
- **Description**: AI tagging accuracy improvement via confidence calibration, independent verification, and tag status grading.
- **Acceptance Criteria**:
  - Confidence calibration module using model confidence -> real accuracy curves
  - Per-category calibration (animals, landscapes, documents have different curves)
  - ECE (Expected Calibration Error) calculation
  - CLIP zero-shot interface for cross-validation (Python sidecar/FFI)
  - Tag consistency checks: category vs tags contradiction detection
  - Three-tier tag status: verified, provisional, rejected
  - Verified tags participate in search; provisional and rejected do not
  - User correction tracking via correction_history table
  - Error pattern library for known failure modes
