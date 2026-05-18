# ArcaneCodex Roadmap

> Phase structure for v1.0.0-rc

## Context

All 10 phases are defined. Phases 1-8 are complete. Phase 9 is the current active phase (production readiness fixes) with remaining work broken out below. Phase 11 (Dead Code and Structure Cleanup) is planned.

## Phases

- [x] **Phase 1: Foundation** - Project scaffolding, database initialization, error handling, logging
- [x] **Phase 2: Core Features** - Image management, AI analysis, search, dedup, frontend UI, i18n, settings
- [x] **Phase 3: Quality Assurance** - E2E testing, UX fixes, unit tests, build verification
- [x] **Phase 4: Code Quality** - Lint fixes, unwrap removal, a11y, virtual scroll, type safety, i18n completion
- [x] **Phase 5: Pipeline Completion** - Core pipeline linkage, AI worker, search integration, export, narrative anchor
- [x] **Phase 6: Multi-Inference** - Abstract provider trait, 6 adapters, model auto-discovery, worker refactoring
- [x] **Phase 7: AI Accuracy** - Confidence calibration, tag grading, consistency checks, user feedback loop
- [x] **Phase 8: Production Features** - Batch operations, data visualization, settings management, performance
- [ ] **Phase 9: Production Readiness** - CI automation, security hardening, caching, compilation verification
- [ ] **Phase 10: Backend Optimization** - Fix critical bugs, N+1 queries, blocking calls, memory issues, and code quality
- [ ] **Phase 11: Dead Code and Structure Cleanup** - Remove dead code, add docs, split monolithic files

## Phase Details

### Phase 1: Foundation
**Goal**: Project scaffolding and infrastructure are in place for all subsequent development
**Depends on**: Nothing (first phase)
**Requirements**: REQ-SETUP-01
**Success Criteria** (what must be TRUE):
  1. Tauri 2.x project builds successfully with React 18 + TypeScript + Vite frontend
  2. SQLite database initializes with all required tables (images, tags, image_tags, search_index, task_queue, app_config)
  3. Migration system supports versioned schema upgrades with PRAGMA user_version tracking
  4. Unified error handling (AppError enum) covers all error paths
  5. Structured logging writes to `%APPDATA%\ArcaneCodex\logs\` with graded levels
**Plans**: Complete
**Status**: COMPLETE

### Phase 2: Core Features
**Goal**: Users can import images, view them in a gallery, get AI-generated tags, search by content, detect duplicates, and configure settings
**Depends on**: Phase 1
**Requirements**: REQ-IMG-01, REQ-AI-01, REQ-SEARCH-01, REQ-DEDUP-01, REQ-SETTINGS-01
**Success Criteria** (what must be TRUE):
  1. User can drag-and-drop images and see them appear in the gallery with thumbnails
  2. AI analysis runs in background worker queue with pause/resume/cancel, generating tags + description + category
  3. User can search images by Chinese/English keywords with relevance-ranked results
  4. User can scan for near-duplicate images by pHash similarity and batch-delete duplicates
  5. User can configure AI address, theme, language, and perform database backup/restore in settings
  6. i18n supports Chinese and English with seamless switching across all UI
**Plans**: Complete
**Status**: COMPLETE
**UI hint**: yes

### Phase 3: Quality Assurance
**Goal**: Application is stable, tested, and ready for quality validation through E2E flows and comprehensive test suites
**Depends on**: Phase 2
**Requirements**: None (infrastructure / quality phase)
**Success Criteria** (what must be TRUE):
  1. End-to-end user flow simulation covers import, browse, search, detail view, export, and dedup
  2. UX issues identified in simulation (25 items) are all fixed
  3. All hardcoded Chinese strings in 8 frontend components are migrated to i18n t() calls
  4. Rust unit tests pass (98 tests), frontend unit tests pass
  5. Frontend builds with zero TypeScript errors, Rust backend compiles with zero errors
**Plans**: Complete
**Status**: COMPLETE
**UI hint**: yes

### Phase 4: Code Quality
**Goal**: Codebase meets quality standards for lint, type safety, accessibility, and performance
**Depends on**: Phase 3
**Requirements**: None (infrastructure / quality phase)
**Success Criteria** (what must be TRUE):
  1. ESLint passes with zero errors (27 errors fixed), TypeScript compiles with zero errors
  2. All Rust production-code unwrap() calls are replaced with proper error handling
  3. All icon buttons have aria-label, forms have label associations, images have alt text
  4. ImageGrid virtual scroll renders correctly at all responsive breakpoints (2-5 columns)
  5. Theme persists across browser sessions via localStorage, theme+language settings are properly connected to backend
  6. ErrorBoundary catches React render errors with user-visible fallback UI
**Plans**: Complete
**Status**: COMPLETE
**UI hint**: yes

### Phase 5: Pipeline Completion
**Goal**: Core import-to-AI pipeline is fully connected end-to-end, export works, and narrative anchor system enables memory-style annotations
**Depends on**: Phase 4
**Requirements**: REQ-EXPORT-01, REQ-NARRATIVE-01
**Success Criteria** (what must be TRUE):
  1. Imported images automatically get thumbnail, pHash, EXIF extraction, and enter AI task queue
  2. AI worker loop processes queue: calls inference API, updates database, builds search index, emits progress events
  3. Failed AI tasks retry up to 3 times with exponential backoff; paused/resumed via broadcast channel
  4. User can export image metadata to JSON/CSV/XML format from single image or batch
  5. User can write narrative annotations about images with auto entity extraction (person/place/time/event)
  6. Semantic search falls back to narrative content when tag search returns no results
**Plans**: Complete
**Status**: COMPLETE
**UI hint**: yes

### Phase 6: Multi-Inference
**Goal**: Users can choose between 6 inference providers (local and cloud) with automatic model discovery
**Depends on**: Phase 5
**Requirements**: REQ-PROVIDER-01
**Success Criteria** (what must be TRUE):
  1. Application supports LM Studio, Ollama, Hermes (local) and Zhipu, OpenAI, OpenRouter (cloud) inference providers
  2. User can switch inference provider at runtime from settings; cloud providers require API key input
  3. Model auto-discovery scans local ports (1234, 11434, 18789) and reports available models
  4. API keys for cloud providers are never exposed in frontend memory
  5. AI worker uses ProviderFactory to dynamically create provider instances based on configuration
**Plans**: Complete
**Status**: COMPLETE
**UI hint**: yes

### Phase 7: AI Accuracy
**Goal**: AI-generated tags are calibrated, graded by confidence, and improved through user feedback
**Depends on**: Phase 6
**Requirements**: REQ-ACCURACY-01
**Success Criteria** (what must be TRUE):
  1. Model confidence values are calibrated per category (animals, landscapes, documents have different curves)
  2. AI tags are graded: verified (high confidence, participates in search), provisional (medium, marked for review), rejected (low, discarded)
  3. User corrections to tags are tracked in correction_history table
  4. Tag consistency checks detect contradictions (e.g. category=animal but tags contain "car")
**Plans**: Complete (with deferred items for v1.1: CLIP Python sidecar, prompt fine-tuning, error pattern feedback)
**Status**: COMPLETE
**UI hint**: yes

### Phase 8: Production Features
**Goal**: Users can perform batch operations, view library statistics, and manage the system through an admin interface
**Depends on**: Phase 7
**Requirements**: REQ-BATCH-01, REQ-STATS-01
**Success Criteria** (what must be TRUE):
  1. User can select multiple images for batch AI analysis, tag correction, and export
  2. Batch operations show progress with pause/resume/cancel controls
  3. Dashboard displays total images, category distribution, AI progress, and storage usage
  4. AI accuracy trend chart shows performance over time and by category
  5. Settings page manages all system configuration including backup/restore with version validation
**Plans**: Complete
**Status**: COMPLETE
**UI hint**: yes

### Phase 9: Production Readiness
**Goal**: Application is hardened for production: CI is automated, security vulnerabilities are fixed, performance is optimized, and all builds verify clean
**Depends on**: Phase 8
**Requirements**: REQ-PROD-01
**Success Criteria** (what must be TRUE):
  1. CI pipeline (GitHub Actions) runs `cargo test` and `npx vitest run` on every push
  2. CI runs `cargo audit` and `npm audit` to detect dependency vulnerabilities
  3. API keys are encrypted at rest using AES-256-GCM with machine-derived key
  4. Search queries under 5 minutes hit LRU cache instead of re-executing SQL
  5. Zip crate is upgraded from 0.6 to 2.0 with no breaking changes
  6. Both ends compile clean: `cargo check` zero errors, `tsc --noEmit` zero errors
**Plans**: See remaining tasks below
**Status**: IN PROGRESS (39 remaining)

#### Phase 9 Remaining Tasks

| Priority | Task | Status | Blocked | Notes |
|----------|------|--------|---------|-------|
| P0 | CI: Verify cargo test runs in GitHub Actions | Pending | Yes | Cannot trigger GitHub Actions locally |
| P0 | CI: Verify vitest runs in GitHub Actions | Pending | Yes | Cannot trigger GitHub Actions locally |
| P1 | CI: Verify cargo audit + npm audit in CI | Pending | Yes | Cannot trigger GitHub Actions locally |
| P2 | Compilation: Verify cargo test passes on Windows | Pending | Yes | STATUS_ENTRYPOINT_NOT_FOUND (DLL dependency) |
| P3 | Deferred: CLIP Python sidecar for cross-validation | Deferred to v1.1 | No | Phase 7 deferred item |
| P3 | Deferred: Prompt fine-tuning from correction samples | Deferred to v1.1 | No | Phase 7 deferred item |
| P3 | Deferred: Error pattern feedback loop | Deferred to v1.1 | No | Phase 7 deferred item |
| P3 | Deferred: error_patterns.rs full implementation | Deferred to v1.1 | No | Requires 50+ error pattern records |
| P3 | Deferred: tag_correction.rs full implementation | Deferred to v1.1 | No | Requires 100+ correction records |
| P3 | Deferred: SQLCipher database encryption evaluation | Deferred to v1.2 | No | Per ADR recommendation |

#### Phase 9 Completed Work

| Section | Items | Status |
|---------|-------|--------|
| 9.1 P0: get_available_disk_space graceful failure | 1 item | Done |
| 9.2 P1: API key encryption (AES-256-GCM) | 6 subtasks | Done |
| 9.2 P1: Unified config tables (v6 migration) | 4 subtasks | Done |
| 9.2 P1: Image model field completion | 3 subtasks | Done |
| 9.2 P1: Export path safety validation | 3 subtasks | Done |
| 9.2 P1: Database restore version validation | 3 subtasks | Done |
| 9.2 P1: LRU search cache | 5 subtasks | Done |
| 9.2 P1: CI audit steps configured in YAML | 2 subtasks | Done (verification blocked) |
| 9.2 P1: .env.example cleanup | 1 item | Done |
| 9.3 P2: Console cleanup + vite drop config | 2 subtasks | Done |
| 9.3 P2: Zip crate 0.6 to 2.0 upgrade | 3 subtasks | Done |
| 9.3 P2: TODO completion (HTTP mocks, tests) | 3 subtasks | Done |
| 9.3 P2: Frontend test act() warnings fix | 2 subtasks | Done |
| 9.3 P2: rust-version 1.70 to 1.75 | 1 item | Done |
| 9.4 Compilation: cargo check zero errors | 1 item | Done |
| 9.4 Compilation: tsc --noEmit zero errors | 1 item | Done |
| 9.4 Compilation: vitest 223 tests pass | 1 item | Done |

### Phase 10: Backend Optimization
**Goal**: Fix critical correctness bugs, performance anti-patterns, and code quality issues in the Rust backend discovered through deep code audit
**Depends on**: Phase 9
**Requirements**: REQ-PROD-01 (extended)
**Success Criteria** (what must be TRUE):
  1. Circuit breaker correctly tracks failure time and enforces reset timeout before half-open
  2. All N+1 SQL queries replaced with batched WHERE id IN (...) patterns
  3. All blocking I/O (image open, file ops) wrapped in spawn_blocking within async contexts
  4. ONNX inference uses read locks, enabling concurrent execution
  5. HNSW vector index implements actual HNSW algorithm or is renamed to brute-force
  6. AI queue dead query removed, config loading merged into single query
  7. Search cache has LRU eviction with size limit
  8. CSV export properly escapes all special characters
  9. Multi-statement config writes wrapped in transactions
  10. Image pipeline decodes each file once (shared DynamicImage)
  11. cargo check and cargo test (where possible) pass with zero regressions
**Plans**: TBD
**Status**: PLANNING

### Phase 11: Dead Code and Structure Cleanup
**Goal**: Codebase is leaner, large files are split into manageable modules, and public API surface is documented.
**Depends on**: Phase 10
**Requirements**: CLEAN-01, CLEAN-02, CLEAN-03, CLEAN-04, CLEAN-05
**Success Criteria** (what must be TRUE):
  1. All confirmed dead code removed -- zero `#[allow(dead_code)]` annotations in production code
  2. Orphaned functions (init_database, try_open_database, is_encrypted) deleted
  3. `cargo check` and `cargo clippy --lib -- -D warnings` pass
  4. `#![allow(missing_docs)]` removed from all 42 Rust files, doc comments added to pub items
  5. settings.rs split from 3231 lines into command handlers (~800 lines) + core/settings/{backup,config}
  6. images.rs split from 3005 lines into commands/images/{import,export,query,delete}
  7. All existing tests continue to pass
**Plans**: 3 plans
  - [ ] 03-01-PLAN.md -- Remove dead code (+ orphaned functions)
  - [ ] 03-02-PLAN.md -- Remove #![allow(missing_docs)] + add doc comments
  - [ ] 03-03-PLAN.md -- Split settings.rs + images.rs
**Status**: PLANNED

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | - | Complete | 2026-04 |
| 2. Core Features | - | Complete | 2026-04 |
| 3. Quality Assurance | - | Complete | 2026-04 |
| 4. Code Quality | - | Complete | 2026-04 |
| 5. Pipeline Completion | - | Complete | 2026-04 |
| 6. Multi-Inference | - | Complete | 2026-04 |
| 7. AI Accuracy | - | Complete | 2026-04/05 |
| 8. Production Features | - | Complete | 2026-05 |
| 9. Production Readiness | 5 remaining | In Progress | TBD |
| 10. Backend Optimization | TBD | Planning | TBD |
| 11. Dead Code and Structure Cleanup | 0/3 | PLANNED | TBD |
