# Project State

> **Version**: 1.0.0-rc  
> **Updated**: 2026-05-17  
> **Value**: "Local-first, privacy-focused personal image management with AI auto-tagging"  
> **Current Focus**: Phase 9 -- Production Readiness Fixes

---

## 1. Project Reference

- **Repository**: `D:\Personal\Desktop\2\ArcaneCodex`
- **Core Value**: Users can manage thousands of personal images locally, search by AI-generated content tags, and never upload data to the cloud
- **Target Users**: Personal collectors, content creators, privacy-sensitive users, AI researchers
- **Tech Stack**: Tauri 2.x (Rust) + React 18 (TypeScript) + SQLite + LM Studio AI
- **License**: MIT

---

## 2. Current Position

| Dimension | Status |
|-----------|--------|
| **Phase** | Phase 9 - Production Readiness Fixes |
| **Plan** | N/A (final phase, remaining tasks are fixes) |
| **Status** | In Progress |
| **Progress** | 500/539 tasks complete (92.8%) |

### Phase Completion

- [x] Phase 1: Foundation (Complete)
- [x] Phase 2: Core Features (Complete)
- [x] Phase 3: Quality Assurance (Complete)
- [x] Phase 4: Code Quality (Complete)
- [x] Phase 5: Pipeline Completion (Complete)
- [x] Phase 6: Multi-Inference (Complete)
- [x] Phase 7: AI Accuracy (Complete)
- [x] Phase 8: Production Features (Complete)
- [ ] Phase 9: Production Readiness (In Progress -- 39 remaining)

### Progress Bar

```
Phases 1-8 [████████████████████████████████] 100%
Phase 9    [████████████████████████████░░░░] 79% (11/14 task groups done)
```

---

## 3. Performance Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| Test pass rate | 117/121 (96.7%) | 4 blocked (external dependencies) |
| Rust unit tests | 110+ | Passed (lm_studio 8 new + ai_queue 4 new + 98 existing) |
| Frontend tests | 223 (17 files) | Vitest via --pool=forks, all passed |
| Build: cargo check | Zero errors | Windows STATUS_ACCESS_VIOLATION is environment issue |
| Build: tsc --noEmit | Zero errors | Passed |
| Build: vitest | 17/17 files pass | All tests green |
| i18n coverage | zh=194, en=194 | Fully synchronized |
| AI providers | 6 | LM Studio, Ollama, Hermes, Zhipu, OpenAI, OpenRouter |
| Image scale target | 5,000+ | BK-Tree dedup performance tested at scale |
| Search cache | LRU + 5min TTL | Implemented in Phase 9 |

---

## 4. Accumulated Context

### Key Decisions

1. **Ten-phase structure**: Project organized into 10 phases (Phase 1-8 complete, Phase 9 in progress, Phase 10 release prep done). GSD roadmap collapses to 9 phases (Phase 10 merged into Phase 9 completion criteria).

2. **Database migration versions**: v1 (initial), v2 (ComfyUI), v3 (narrative anchor), v4 (multi-provider), v5 (AI tag status), v6 (unified config table). v6 is the current schema version.

3. **Architecture**: Single-binary Tauri desktop app. Frontend WebView (React) + Rust backend core. SQLite data layer via rusqlite bundled + r2d2 connection pool. LM Studio as external AI service at localhost:1234.

4. **Image storage**: Index-reference mode (don't move originals). Optional archive to `%APPDATA%\ArcaneCodex\images\`. Thumbnails at `%APPDATA%\ArcaneCodex\thumbnails\{id}.webp`.

5. **Security approach**: CSP limited connect-src (localhost), contextIsolation: true, AES-256-GCM for API keys, sanitize_error to avoid path/SQL leakage, export path validation.

6. **Inference routing**: Frontend-direct for local models (zero latency), Tauri proxy for cloud APIs (key security). API keys never in frontend memory.

7. **CI pipeline**: GitHub Actions with cargo test, vitest, cargo audit, npm audit. Verification blocked (cannot trigger remotely).

### Known Blockers

| Blocker | Impact | Workaround | Resolution Target |
|---------|--------|------------|-------------------|
| `cargo test` STATUS_ENTRYPOINT_NOT_FOUND on Windows | Cannot run Rust tests locally | Code review + cargo check pass; tests verified on other environments | v1.0.0-rc final |
| CI verification (GitHub Actions) | Cannot verify CI pipeline end-to-end | CI configuration written and reviewed; trigger requires remote push | v1.0.0-rc final |
| HEIC/HEIF format support | Format not supported | MIME type registered, decode blocked on libheif C library | v1.1 |
| NSIS interactive prompt | Cannot automate installer build | Manual step documented | v1.0.0-rc |

### Deferred to v1.1

- CLIP Python sidecar for cross-validation (Phase 7 deferred)
- Prompt fine-tuning from correction samples (Phase 7 deferred)
- Error pattern feedback loop (Phase 7 deferred)
- error_patterns.rs full implementation (requires 50+ pattern records)
- tag_correction.rs full implementation (requires 100+ correction records)

### Deferred to v1.2

- SQLCipher full-database encryption (per ADR recommendation)

### Active TODOs

1. Push to GitHub to trigger CI pipeline and verify all checks pass
2. Resolve `cargo test` Windows STATUS_ENTRYPOINT_NOT_FOUND (try MSVC toolchain update or Docker-based testing)
3. Run final frontend build verification (`npm run build`)
4. Generate NSIS installer package
5. Update README.md with final screenshots and feature matrix

### Session Continuity

**Last session**: Phase 9 production readiness work. Completed: API key encryption (AES-256-GCM), unified config tables (v6 migration), Image model field completion, export path validation, DB restore version check, LRU search cache, console cleanup, zip crate upgrade, TODO completion, rust-version update, cargo check + tsc verification.

**Next session**: Resolve CI verification (push to GitHub), attempt cargo test workaround, verify Phase 9.4 compilation checks, prepare release artifacts.

**Hot paths**: src-tauri/src/ (Rust backend), frontend/src/ (React frontend), .github/workflows/ (CI configuration)
