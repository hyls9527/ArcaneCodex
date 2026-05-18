# Conflict Detection Report

Generated: 2026-05-17
Mode: new
Precedence: ADR > SPEC > PRD > DOC

---

## BLOCKERS (0)

No unresolved blockers detected.

All 12 documents have confidence >= medium. No UNKNOWN or low-confidence documents found.
No LOCKED ADRs found in the ingest set (only 1 ADR, sqlcipher-decision.md, with locked: false).
No cycle-dependent semantic blockers identified (see INFO section for cycle analysis).

---

## WARNINGS (0)

No competing acceptance variants detected.

The two PRDs (pre-requirements.md and v1.0.0-rc/01-requirements.md) define requirements at
different levels of specificity and for different stages of the project. The pre-requirements
document is an early exploration (2026-05-04) with high-level capability desires, while the
v1.0.0-rc PRD is the formal product requirements for the implemented system. Their differences
on technology choices are resolved by SPEC-level documents (see auto-resolved section).

---

## INFO (N)

### [INFO] Cross-reference graph cycles detected (benign)

The cross-reference graph planning documents contains mutual references between closely related documents. These are benign relationship indicators (design doc + implementation plan, task list + status update) and do not create semantic resolution loops.

Cycles detected:

1. ai-integration-architecture.md (SPEC) <-> 04-ai-integration-tasks.md (DOC)
   - SPEC defines the AI integration architecture; DOC provides implementation tasks referencing the SPEC.
   - Both documents are independently interpretable; the cross-ref indicates they belong together.

2. 04-ralph-tasks.md (DOC) <-> RALPH_STATE.md (DOC)
   - Task list references the state document for progress tracking; state document references the task list for detailed counts.
   - Information flows one direction (tasks update state); the bidirectional reference is organizational.

3. 04-ralph-tasks.md (DOC) <-> 05-test-plan.md (DOC)
   - Task list references test plan for verification; test plan references task list for task IDs.
   - Standard project management cross-reference pattern.

4. RALPH_STATE.md (DOC) <-> 05-test-plan.md (DOC)
   - State references test plan for blocked status; test plan references state for progress context.

Resolution: None required. All cycles are benign mutual references between task tracking and planning documents that do not affect semantic interpretation. Synthesis proceeds normally.

---

### [INFO] Auto-resolved: pre-requirements.md PRD conflicts with SPEC (02-architecture.md)

The early pre-requirements document (PRD type) proposed technology choices that differ from the implemented system as documented in the v1.0.0-rc SPEC (02-architecture.md). Per precedence rules (SPEC > PRD), the SPEC wins in all cases.

Source: docs/planning/pre-requirements.md (PRD)
Winner: docs/planning/v1.0.0-rc/02-architecture.md (SPEC)

Conflicts resolved:

1. **Database ORM**
   - pre-requirements: sqlx (PRD, lower precedence)
   - 02-architecture: rusqlite 0.31 bundled (SPEC, higher precedence)
   - Resolution: rusqlite wins. Confirmed by v1.0.0-rc PRD and actual Cargo.toml dependencies.

2. **AI inference engine**
   - pre-requirements: ONNX Runtime or TensorFlow Lite for local model execution (PRD)
   - 02-architecture: LM Studio HTTP API at localhost:1234/v1 with Qwen2.5-VL-7B-Instruct (SPEC)
   - Resolution: LM Studio wins. Confirmed by v1.0.0-rc PRD and Phase 6 multi-provider implementation.

3. **Search engine**
   - pre-requirements: Tantivy or SQLite FTS5 for full-text search (PRD)
   - 02-architecture: jieba-rs Chinese word segmentation + search_index inverted index table (SPEC)
   - Resolution: jieba-rs + inverted index wins. Confirmed by v1.0.0-rc PRD and implementation.

4. **Image storage strategy**
   - pre-requirements: move/copy to `{用户文档}/ArcaneGallery/` with optional date-based reorganization (PRD)
   - v1.0.0-rc PRD + 02-architecture: index reference mode (don't move originals), optional archive to `%APPDATA%\ArcaneCodex\images\` (SPEC)
   - Resolution: index reference mode wins. Confirmed by actual architecture and Phase 5 implementation.

5. **Platform targeting**
   - pre-requirements: cross-platform (Windows/macOS/Linux) (PRD)
   - v1.0.0-rc PRD: Windows 10+ only, targeting Build 17763 (PRD)
   - Resolution: Windows 10+ wins. The v1.0.0-rc PRD is the authoritative requirements document for the implemented system. The SPEC does not explicitly constrain platform, but the implementation targets Windows.

6. **AI model stack**
   - pre-requirements: MobileNetV3/EfficientNet-Lite/CLIP/InsightFace for local image recognition (PRD)
   - v1.0.0-rc PRD + SPEC: Qwen2.5-VL-7B-Instruct via LM Studio for multimodal analysis (SPEC)
   - Resolution: LM Studio + Qwen2.5-VL-7B-Instruct wins. Confirmed by architecture SPEC and Phase 6 multi-provider trait.

7. **User scale assumptions**
   - pre-requirements: 5,000-50,000 images, mid-advanced technical users (PRD)
   - v1.0.0-rc PRD: 5,000+ images, personal collectors/content creators/privacy-sensitive users (PRD)
   - Resolution: v1.0.0-rc PRD user profile wins as authoritative source for implemented system.

Note: The pre-requirements document remains valuable as a historical artifact describing the project's initial vision. Its content is preserved in the intel/context.md for reference but its technology decisions have been superseded.

---

### [INFO] Auto-resolved: governance-roadmap.md enforcement task superseded by ADR

Source: docs/planning/governance-roadmap.md (DOC)
Winner: docs/planning/sqlcipher-decision.md (ADR)

- governance-roadmap.md P2-04 calls for "SQLite encryption evaluation" as a future task (DOC, lower precedence)
- sqlcipher-decision.md (ADR, higher precedence) has already performed the evaluation and produced a recommendation (Plan B now, Plan A as roadmap target)
- Resolution: The ADR supersedes the roadmap task. The evaluation is complete; the governance roadmap should be updated to reflect that P2-04 is done.

---

### [INFO] Auto-resolved: governance-roadmap.md P1-04 CSP unsafe-inline removal vs SPEC CSP analysis

Source: docs/planning/governance-roadmap.md (DOC)
Winner: docs/planning/phase1-component-split-blueprint.md (SPEC)

- governance-roadmap P1-04: "Remove CSP unsafe-inline" with goal of `style-src 'self'` (DOC)
- phase1-component-split-blueprint CSP Analysis section (SPEC): "Not recommended to remove at current stage" due to 46 motion animation inline styles and virtual scroll absolute positioning (SPEC, higher precedence)
- Resolution: SPEC's analysis wins. The implementation ultimately set CSP to null (4.20.1) then limited `connect-src` (5.5.1), effectively side-stepping the `style-src unsafe-inline` issue. Recommended: revisit CSP if/when motion library is replaced with CSS animations.

---

### [INFO] ADS - AI pre-requirements vs AI integration architecture on model providers

Source: docs/planning/pre-requirements.md (PRD)
Source: docs/planning/ai-integration-architecture.md (SPEC)
Winner: ai-integration-architecture.md (SPEC)

- pre-requirements: Local-only AI, ONNX Runtime/TensorFlow Lite for inference (PRD)
- ai-integration-architecture: Hybrid approach supporting both local (Ollama/LM Studio frontend-direct) and cloud (Zhipu/OpenRouter via Tauri proxy) (SPEC, higher precedence)
- Resolution: Hybrid approach wins. Confirmed by Phase 6 multi-inference provider implementation with 6 provider adapters.

---

### [INFO] Auto-resolved: sqlcipher-decision.md (ADR) cross-references point to code files, not blocking

The ADR's cross_refs point to `file:///e:/ArcaneCodex/` paths which reference the source code directory (Cargo.toml, db.rs, crypto.rs). These are code-level relationships, not planning document dependencies, and do not affect synthesis.

---

### [INFO] Consistency confirmed across docs on key architectural decisions

The following areas show no conflicts across all 12 documents:

- **Tauri 2.x + React 18 + SQLite stack**: Consistent across all PRDs, SPECs, and DOCs
- **LM Studio as primary AI backend**: Consistent in v1.0.0-rc PRD, both SPECs, and task documentation
- **Phase numbering and completion tracking**: 04-ralph-tasks.md, 05-test-plan.md, and RALPH_STATE.md all agree on phase structure and completion status
- **v1.0.0-rc version target**: All Phase 10 docs agree on version 1.0.0-rc
- **Narrative anchor system**: Described in Phase 5.7 tasks, tested in 05-test-plan.md Section 13, consistent
