# Constraints (SPECs)

## Constraint: System Architecture (02-architecture)

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/02-architecture.md`
- **Type**: schema, api-contract
- **Content**:
  - **Tech Stack**: React 18 + TypeScript + Tailwind CSS 3.4 + Framer Motion 11 + Zustand 4.5 + React Query 5 + @tanstack/react-virtual 3.5 + i18next 23 + Lucide React 0.378 + Radix UI
  - **Rust Backend**: Tauri 2.x + tokio 1 + rusqlite 0.31 bundled + image 0.25 + img_hash 0.4 + reqwest 0.12 + jieba-rs 0.7 + async-channel 2.2
  - **AI**: LM Studio (localhost:1234/v1), Qwen2.5-VL-7B-Instruct, OpenAI Chat Completions API compatible
  - **QA**: cargo test + Vitest + React Testing Library + Playwright + Tauri NSIS
  - **Database Schema**: 6 tables (images, tags, image_tags, search_index, task_queue, app_config)
  - **API**: 12 Tauri commands across 5 modules (image management, AI analysis, search, dedup, export)
  - **Architecture**: Single-binary Tauri desktop app, frontend WebView + Rust backend core, SQLite data layer, LM Studio as external AI service
  - **Directory Structure**: `src-tauri/` (Rust commands, core, models, utils), `frontend/` (React components, stores, hooks, i18n)
  - **Process Flows**: Image import (validate -> hash -> thumbnail -> phash -> EXIF -> DB insert -> task_queue), AI tagging (queue -> LM Studio -> update DB -> search_index), semantic search (query -> jieba -> inverted index join)
  - **Constraints**: Single binary, no cloud dependencies, targeting Windows 10+ (1809+), offline-first

## Constraint: AI Integration Architecture (ai-integration-architecture)

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/ai-integration-architecture.md`
- **Type**: api-contract, nfr
- **Content**:
  - **Core Decision**: Not "frontend or backend" but "auto-select optimal path based on target service characteristics"
  - **Local models** (Ollama/LM Studio): Frontend-direct, no API key needed, zero latency
  - **Cloud APIs** (Zhipu/OpenRouter): Tauri proxy, key security, unified management
  - **Key Management**: tauri-plugin-stronghold or keytar-rs, decrypt-on-demand, clear-after-use
  - **Security**: API key never appears in frontend code/memory, Tauri process holds key only during request, system keychain is sole long-term storage
  - **Dependencies**: tauri-plugin-stronghold 2.0, reqwest 0.12 with stream, tokio 1 full
  - **Components**: Frontend AI router (`ai-router.ts`), Tauri proxy (`ai_proxy.rs`), keyvault module (`keyvault.rs`), secure string (`secure_string.rs`)
  - **Risk Mitigation**: Memory dump prevention, git pre-commit hooks, HTTPS enforcement, localhost-only proxy
  - **Acceptance Criteria**: Local TTFT < 100ms, zero API key in frontend, seamless local/cloud switching, unit tests verifying key non-leakage

## Constraint: Phase 1 Frontend Component Split (phase1-component-split-blueprint)

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/phase1-component-split-blueprint.md`
- **Type**: api-contract, nfr
- **Content**:
  - **ImageViewer Split** (P0, ~3h): Split 461-line component into index.tsx, useImageZoom.ts hook, ImageInfoPanel, ImageBottomBar, ImageToolbar, SampleViewerPlaceholder, utils.ts
  - **useImageZoom Hook API**: scale, position, isDragging, handlers (onWheel/onMouseDown/onMouseMove/onMouseUp), zoomIn/zoomOut/reset
  - **KnowledgeGraphView Split** (P1, ~4h): Split 533-line component into index.tsx, useForceSimulation.ts, GraphCanvas, GraphSidebar, GraphHeader, types.ts, constants.ts
  - **GalleryPage Split** (P2, ~3h): Split 441-line component into index.tsx, GalleryActionBar, GalleryEmptyState, GallerySearchResults, useGalleryActions.ts
  - **useGalleryActions API**: handleFilesSelected, handleBatchDelete, handleClearFailedImages, handleSelectFiles, handleSelectFolder, deleting, clearingFailed
  - **CSP Analysis**: 46 motion inline styles, 12 explicit inline styles. Not recommended to remove `unsafe-inline` while motion is in use. 1 low-cost improvement available.
  - **Framing**: keyvault interface, AI router interface, component interface contracts
  - **Execution Plan**: 3 rounds, total ~10h, each round requires tsc --noEmit + lint + vitest (223 tests) + manual verification
