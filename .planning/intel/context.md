# Context (DOCs)

## Topic: AI Integration Implementation Tasks

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/04-ai-integration-tasks.md`
- **Notes**: Implementation task list for the layered adaptive AI integration architecture defined in ai-integration-architecture.md. Covers 7 tasks: architecture doc (done), frontend router (`ai-router.ts`), Tauri proxy (`ai_proxy.rs`), keyvault module (`keyvault.rs`), secure string type (`secure_string.rs`), model config UI (`AIProviderConfig.tsx`), and security tests. Execution order: keyvault -> secure_string -> proxy -> router -> config UI -> tests. Dependencies: tauri-plugin-stronghold, reqwest with stream, zeroize.

## Topic: Governance Roadmap

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/governance-roadmap.md`
- **Notes**: 4-phase governance roadmap for ArcaneCodex. Phase 0 (emergency, ~2h): fix panic! in inference.rs, disable production sourcemap, pin CI action versions + permissions, fix double JSON.parse of ai_tags. Phase 1 (short-term, ~15h): split 3 giant components (KnowledgeGraphView 533 lines, ImageViewer 461 lines, GalleryPage 441 lines), remove CSP unsafe-inline, create .env.example, clean notify crate features, DRY stores. Phase 2 (medium, ~20h): upgrade key derivation to PBKDF2 (600K iterations), add database transactions, clean test unwraps, evaluate SQLite encryption, document unsafe code. Phase 3 (long-term, 1-3 months): architecture evolution assessment, observability (structured logging, metrics, error tracking), performance benchmarking, i18n completion. Total estimated: ~37h (Phase 0-2) + Phase 3 TBD. Current scores: Rust 4.2/5, Frontend 3.8/5, Security 3.5/5. 10 positive practices recorded must not be reverted. Generated 2026-05-12, status pending approval.

## Topic: Development Task Status

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/04-ralph-tasks.md`
- **Notes**: Complete task tracking across 10 phases. Phase 1 (project setup): Tauri 2.x scaffolding, React 18 setup, Rust dependencies, SQLite initialization with migration system. Phase 2 (core features): image import pipeline, thumbnail generation, pHash/dedup, EXIF extraction, AI analysis, task queue, semantic search, frontend UI components, i18n. Phase 3 (QA): E2E testing, UX fixes (25 issues identified and fixed), unit tests (98 Rust, frontend tests), release preparation. Phase 4 (code quality): ESLint fixes (27 errors), hardcoded Chinese fixes (15+ strings), Rust unwrap() removals, virtual scroll optimization, responsive grid, type safety, a11y. Phase 5 (core pipeline): import image flow linkage, AI worker loop, AITaskQueue restructuring, frontend-backend search bridge, API contract alignment, export module, narrative anchor (Phase 5.7). Phase 6 (multi-inference): provider trait, 6 adapters, model auto-discovery, Worker refactoring, database v4 migration. Phase 7 (AI accuracy): calibration module, CLIP verification, consistency checks, tag status grading (verified/provisional/rejected), user feedback loop. Phase 8 (production-ready): batch operations, data visualization, admin backend, performance optimization. Phase 9 (production fixes): CI automation, disk space fix, API key encryption, unified config tables, cached searches, security auditing. Phase 10 (release prep): version bump to 1.0.0-rc, CHANGELOG, final frontend verification. Overall: 500/539 tasks [x] completed, 1 blocked (cargo test on Windows STATUS_ENTRYPOINT_NOT_FOUND).

## Topic: Test Plan

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/05-test-plan.md`
- **Notes**: Comprehensive test plan covering 14 sections. Key findings: All automatable tests passing. Blocked tests require LM Studio runtime or frontend integration environment. Image import E2E blocked on frontend integration. HEIC/HEIF blocked on crate support. Notable defect: theme and language settings NOT persisted (theme buttons don't call updateField, language switcher doesn't write to config system). 223 frontend tests passing (via --pool=forks). 5 test categories with coverage percentages tracked. Phase 10 adds release validation checklist. Dependency: Rust cargo test blocked on Windows STATUS_ENTRYPOINT_NOT_FOUND (DLL dependency issue).

## Topic: Learnings

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/06-learnings.md`
- **Notes**: Agent-facing reference. Key conventions: Tauri Command must use `#[tauri::command]` macro, async functions return `Result<T, AppError>`, React functional components + TypeScript, Zustand for state. Gotchas: Tauri 2.x uses `app.windows` not `tauri.windows`, rusqlite bundled auto-compiles SQLite, thumbnail generation must use `tokio::spawn_blocking`, LM Studio API timeout is 60s, Windows paths use backslash. Commands: `cargo tauri dev` / `cargo tauri build` / `cd frontend && pnpm dev` / `cargo test` / `pnpm test`.

## Topic: Project State

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/v1.0.0-rc/RALPH_STATE.md`
- **Notes**: Current state snapshot (2026-05-03). Phase 9/10 complete, Phase 10 fully complete. 500/539 tasks done (~92.8%). Blocking issue: cargo test STATUS_ENTRYPOINT_NOT_FOUND on Windows. GitHub repository created: github.com/hyls9527/ArcaneCodex. Phase 6-7 results: multi-inference support (6 providers), model auto-discovery, confidence calibration, CLIP verification interface, tag grading system (verified/provisional/rejected). Phase 8 results: API key encryption, LRU search cache, Zip Slip protection, console cleanup. Phase 9 results: zip 0.6->2.0, rust-version 1.70->1.75, LTO config. Phase 10 results: version 1.0.0-rc, CHANGELOG.md, final verification. Remaining TODOs: resolve cargo test issue, implement CLIP Python sidecar (7.2.1), prompt fine-tuning (7.4.1), error pattern feedback (7.4.2).
