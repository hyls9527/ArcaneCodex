# ArcaneCodex

> Your Photos, Your Memories -- 你的照片，就是记忆

## Core Value Proposition

A local-first, privacy-focused desktop application for personal image management. Uses local AI (LM Studio) to automatically tag, describe, and categorize images so users can search their photo library by content rather than by filename or folder. All data stays on the user's machine -- no cloud uploads, no telemetry, no third-party servers.

## Target Users

- **Personal collectors**: Organizing travel photos, design assets, screenshot collections
- **Content creators**: Managing mood boards, reference image libraries, design inspiration
- **Privacy-sensitive users**: Local-only storage, fully offline operation, rejects cloud upload
- **AI researchers**: Testing multimodal models, batch processing image data

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Frontend | React 18 + TypeScript + Vite | UI framework, type safety, fast builds |
| Styling | Tailwind CSS 3.4 + Framer Motion 11 | Utility-first CSS, animation |
| State | Zustand 4.5 | Lightweight state management |
| Backend | Rust + Tauri 2.x | Desktop shell, native APIs, performance |
| Database | SQLite via rusqlite 0.31 (bundled) | Zero-config, embedded, WAL mode |
| AI | LM Studio / Ollama / OpenAI-compatible API | Image analysis via multimodal LLM |
| Search | jieba-rs Chinese segmentation + inverted index | Semantic keyword search |
| Testing | Vitest (frontend), cargo test (Rust) | Unit and integration testing |
| CI/CD | GitHub Actions | Automated build, test, audit |

## Team

- **Visionary / Product Owner**: User (single developer)
- **Builder**: Claude (AI assistant, GSD workflow)
- **Scope**: Solo developer + Claude workflow. No teams, no stakeholders, no sprints.

## Constraints

- Windows 10+ only (Build 17763)
- Local-first: no cloud dependencies, no telemetry, no user data upload
- AI requires external service (LM Studio / Ollama) running locally
- All new code must live in this repository; the original `arcane-codex-src` project is read-only reference
- SQLite embedded database with rusqlite bundled compilation
- 1-2 sentence max for AI descriptions, 5-10 tags per image
- Thumbnail storage: `%APPDATA%\ArcaneCodex\thumbnails\{image_id}.webp`

## Current Status

- **Version**: v1.0.0-rc
- **Overall**: 92.8% complete (500/539 tasks)
- **Phases 1-8**: COMPLETE
- **Phase 9**: In progress (production readiness fixes)
- **Remaining**: 39 tasks (2 P0, 8 P1, 10 P2, 4 compilation verification, remaining Phase 9 items)
- **Test pass rate**: 117/121 (96.7%)
- **Known blockers**: `cargo test` STATUS_ENTRYPOINT_NOT_FOUND on Windows (DLL dependency); CI verification blocked (cannot trigger GitHub Actions locally)
