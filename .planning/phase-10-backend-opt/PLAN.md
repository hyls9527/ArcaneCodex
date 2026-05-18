# Phase 10: Backend Optimization — PLAN

> Based on deep code audit 2026-05-18. All findings verified with file:line references.

## Scope

Fix 11 critical/high-impact issues + selected medium issues in the Rust backend. No frontend changes. No new features.

---

## Task List

### Wave 1: Critical Correctness (P0) — 3 tasks

#### Task 1.1: Fix Circuit Breaker Timekeeping (Lock-Free)
**File**: `src-tauri/src/core/circuit_breaker.rs`
**Lines**: 45-48, 89
**Problem**: Uses `Instant::now().elapsed().as_millis()` which returns ~0ms every call. The reset timeout check always passes immediately — circuit breaker never stays open.
**Fix**: Keep `AtomicU64` lock-free design. Use `OnceLock<Instant>` for process start time. Store `duration_since(START).as_millis()` as monotonic millis in the atomic. `allow_request()` computes `current_ms.saturating_sub(last_failure_ms) >= reset_timeout_ms`. This preserves lock-free concurrency while fixing time semantics.
**Verification**: Add `test_circuit_stays_open_during_timeout` and update `test_half_open_allows_request`.

#### Task 1.2: Remove Dead Query in AI Queue
**File**: `src-tauri/src/core/ai_queue.rs`
**Lines**: 583-587
**Problem**: `conn.query_row("SELECT key, value FROM settings WHERE key IN (...)", [], |_| { Ok(()) }).ok()` — executes query, discards result, then re-queries each key individually.
**Fix**: Delete the dead query. Merge the 3 subsequent individual queries (provider, model, api_key) into a single `SELECT key, value FROM settings WHERE key IN (?, ?, ?)` with proper row mapping.
**Verification**: `cargo build` compiles, AI queue processing still works.

#### Task 1.3: Fix HNSW Misnomer — Rename to BruteForceVectorIndex
**File**: `src-tauri/src/core/vector_index.rs`
**Lines**: 1-312 (full file)
**Problem**: `HnswVectorIndex` does zero HNSW — it's a `HashMap<usize, VectorEntry>` with O(n) brute-force linear scan. The name is actively misleading.
**Fix**: Rename struct to `BruteForceVectorIndex`. Update all references in `commands/ai_core.rs` and `main.rs`. Add a doc comment noting the current algorithm and that HNSW is a future optimization.
**Verification**: `cargo build` compiles, all existing vector tests pass.

---

### Wave 2: Performance Anti-Patterns (P1) — 5 tasks

#### Task 2.1: Eliminate N+1 Queries
**Files affected**:
- `commands/batch_ops.rs:38-55` — `start_batch_ai_tag`: loop of `SELECT ... WHERE id = ?1`
- `commands/batch_ops.rs:246-298` — `batch_export`: same pattern
- `commands/images.rs:1244-1321` — `delete_images`: per-ID SELECT + per-ID DELETE
- `commands/images.rs:1326-1383` — `check_broken_links`: per-ID UPDATE in loop
- `commands/xmp.rs:47-75` — `export_as_xmp`: per-ID query in loop

**Fix**: Replace each loop with single batch query using chunked `WHERE id IN (...)`:
- Chunk size: 500 (safe below SQLite's 999 param limit)
- Use `ids.chunks(500)` to split, execute one batched query per chunk
- `DELETE FROM images WHERE id IN (...)` with chunked binding
- `UPDATE images SET ... WHERE id IN (...)` for broken links
- Use dynamic placeholder generation (`ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")`)
**Verification**: `cargo test` for affected modules. Verify batch ops with 1000+ IDs don't hit SQLite variable limit.

#### Task 2.2: Merge Image Decode in Import Pipeline (Two-Step)
**File**: `src-tauri/src/core/image.rs`, `commands/images.rs:961-964`
**Problem**: Each imported image is decoded 3 times: once for thumbnail, once for pHash, once for EXIF/dimensions. For a 50MB JPEG that's 150MB of raw pixel data decoded.
**Fix — Step A**: Add `generate_thumbnail_from_image(&DynamicImage)` as a new method that accepts pre-opened image. Keep old `generate_thumbnail(path)` unchanged.
**Fix — Step B**: In `import_images` Phase 2: open image once in `spawn_blocking` → share `DynamicImage` with new thumbnail method + `calculate_phash` (refactored to accept `&DynamicImage`). Extract EXIF from file bytes directly via `exif::Reader::new()`. Fallback dimensions from `DynamicImage` if EXIF fails.
**Verification**: `cargo test core::image` passes after each step. `git bisect` stays clean — Step A and B are separate commits.

#### Task 2.3: Fix ONNX Inference Lock Contention
**File**: `src-tauri/src/core/onnx_runtime.rs:210-256`
**Problem**: `sessions.write().await` serializes all concurrent inference. Only reads happen — should be `read().await`.
**Fix**: Change `sessions.write().await` to `sessions.read().await` in `run_inference`. Verify no writers are concurrent (warmup holds write lock, inference holds read lock — this is correct).
**Verification**: `cargo build` compiles. No deadlock in concurrent usage.

#### Task 2.4: Add LRU Eviction to Search Cache
**File**: `src-tauri/src/core/search_index.rs:10-24, 36-58`
**Problem**: Global `Mutex<HashMap<u64, (Instant, Vec<SearchResult>)>>` grows unbounded. No eviction.
**Fix**: Add LRU eviction to the existing `HashMap`-based cache. On insert: evict entries older than TTL (5 min) and enforce max 1000 entries. The `std::sync::Mutex` is kept — cache access is <1μs, well below tokio scheduling granularity, so the blocking concern is theoretical.
**Verification**: `cargo test core::search_index` passes. No memory leak in long-running sessions.

#### Task 2.5: Stream Database Backup Instead of Loading to Memory
**File**: `src-tauri/src/commands/settings.rs:328-460, 462-605`
**Problem**: `std::fs::read_to_end()` loads entire DB file into memory. For 1GB+ databases, OOM risk.
**Fix**: Use `std::fs::File::open()` + `std::io::copy(&mut file, &mut zip_writer)` to stream the file into the ZIP archive without loading entirely into memory. Same for WAL and SHM files.
**Verification**: `cargo test commands::settings::test_backup_roundtrip` passes. No regression.

---

### Wave 3: Correctness & Quality (P1/P2) — 5 tasks

#### Task 3.1: Wrap Config Writes in Transactions
**File**: `src-tauri/src/commands/inference_settings.rs:67-111`
**Problem**: 3 separate INSERT/REPLACE queries without transaction. Second failure → inconsistent config state.
**Fix**: Wrap all 3 writes in `conn.transaction()`.
**Verification**: `cargo test commands::inference_settings` passes.

#### Task 3.2: Fix CSV Escaping
**Files**: `src-tauri/src/commands/export.rs:213-215`, `src-tauri/src/commands/batch_ops.rs:304-333`
**Problem**: Only escapes `"` characters. Commas, newlines, and other special chars produce malformed CSV.
**Fix**: Create a shared `escape_csv_field(value: &str) -> String` function that:
1. Checks if field contains `"`, `,`, `\n`, or `\r`
2. If yes: wraps in `"` and doubles internal `"`
3. If no: returns as-is
Use it in both files.
**Verification**: Add test with comma-containing and newline-containing field values.

#### Task 3.3: Deduplicate cosine_similarity (4 copies)
**Files**: `src-tauri/src/core/vector_index.rs:296`, `src-tauri/src/core/knowledge_graph.rs:1103`, `src-tauri/src/core/clip_embedder.rs:161`, `src-tauri/src/core/face_detector.rs:360`
**Problem**: Four byte-for-byte identical implementations of `cosine_similarity(a: &[f32], b: &[f32]) -> f32`.
**Fix**: Move canonical version to `core/vector_index.rs`. Import from there in knowledge_graph.rs, clip_embedder.rs, face_detector.rs. Remove all 3 duplicates.
**Verification**: `cargo build` compiles. All 4 callers still work.

#### Task 3.4: Fix phash_to_u64 Panic on Invalid Length
**File**: `src-tauri/src/core/dedup.rs:283-293`
**Problem**: Assumes phash hex string is exactly 16 chars. Shorter/longer strings produce incorrect result or panic on shift overflow.
**Fix**: Return `Result<u64, AppError>` or handle edge cases gracefully (pad/truncate with warning). At minimum, use `u64::from_str_radix` on the full string instead of manual nibble shifting.
**Verification**: `cargo test core::dedup` passes. Invalid phash values don't panic.

#### Task 3.5: Reduce run_migrations Connection Thrashing
**File**: `src-tauri/src/core/db.rs:66-147`
**Problem**: Opens 9+ connections for 8 migration checks — opens→checks→drops→repeats per version.
**Fix**: Open one connection, check all versions sequentially, apply needed migrations, close once.
**Verification**: `cargo test core::db` passes. Fresh database initializes correctly.

#### Task 3.6: Rename Misleading detect_communities
**File**: `src-tauri/src/core/knowledge_graph.rs:492-598`
**Problem**: `detect_communities()` implements BFS connected components, not actual community detection (no Louvain/modularity/Girvan-Newman). Name implies sophisticated functionality that doesn't exist.
**Fix**: Rename to `find_connected_components()`. Rename related types: `Community` → `ConnectedComponent`, `GraphCommunity` → `GraphComponent`. Update all references in `commands/knowledge_graph.rs` and struct fields.
**Verification**: `cargo build` compiles. Knowledge graph still builds and saves correctly.

---

### Wave 4: Compilation & Regression Tests — 3 tasks

#### Task 4.1: Add Regression Test — Circuit Breaker
**New test**: `test_circuit_stays_open_during_timeout` in `core/circuit_breaker.rs`
**Validates**: After recording a failure, subsequent `allow_request()` calls return false until `reset_timeout_ms` elapses.
**Technique**: Set `failure_threshold=1`, `reset_timeout_ms=100`. Record failure → assert blocked. Sleep 150ms → assert half-open allowed.

#### Task 4.2: Add Regression Test — CSV Escaping
**New test**: `test_escape_csv_field_with_commas_and_newlines` in `commands/export.rs`
**Validates**: Fields containing commas, double-quotes, and newlines are properly escaped.

#### Task 4.3: Run cargo check + clippy + test
**Commands**: 
- `cargo check` — zero errors
- `cargo clippy -- -D warnings` — zero warnings  
- `cargo test` — all tests pass (except known Windows STATUS_ENTRYPOINT_NOT_FOUND if present)
**Verification**: Exit code 0 for all three commands.

---

## Dependency Graph

```
Wave 1 (no deps):
  1.1 Circuit Breaker ─┐
  1.2 Dead Query       ├── Parallel, independent
  1.3 HNSW Rename     ─┘

Wave 2 (2.2 before 2.1 — both touch images.rs; others parallel):
  2.2 Image Decode      ─┐
  2.3 ONNX Lock         ├── Parallel
  2.4 Search Cache LRU  │
  2.5 Backup Stream     ─┘
  2.1 N+1 Queries       ─── After 2.2 (shares images.rs)

Wave 3 (no deps between tasks):
  3.1 Config Tx         ─┐
  3.2 CSV Escape        │
  3.3 Dedup cosine_sim  ├── Parallel, independent
  3.4 phash fix         │
  3.5 Migration conn    │
  3.6 KG Rename         ─┘

Wave 4 (depends on Waves 1-3):
  4.1 Regression: circuit breaker timeout
  4.2 Regression: CSV escaping
  4.3 cargo check + clippy + test
```

## Files Changed (Estimate)

| File | Lines Changed | Tasks |
|------|--------------|-------|
| `core/circuit_breaker.rs` | ~20 | 1.1 |
| `core/ai_queue.rs` | ~15 | 1.2 |
| `core/vector_index.rs` | ~20 | 1.3 |
| `commands/ai_core.rs` | ~5 | 1.3 |
| `main.rs` | ~5 | 1.3 |
| `commands/batch_ops.rs` | ~60 | 2.1, 3.2 |
| `commands/images.rs` | ~50 | 2.1, 2.2 |
| `commands/xmp.rs` | ~10 | 2.1 |
| `core/image.rs` | ~30 | 2.2 |
| `core/onnx_runtime.rs` | ~3 | 2.3 |
| `core/search_index.rs` | ~30 | 2.4 |
| `commands/settings.rs` | ~40 | 2.5 |
| `commands/inference_settings.rs` | ~10 | 3.1 |
| `commands/export.rs` | ~15 | 3.2 |
| `core/knowledge_graph.rs` | ~5 | 3.3 |
| `core/dedup.rs` | ~10 | 3.4 |
| `core/db.rs` | ~40 | 3.5 |
| **Total** | **~368 lines** | 15 tasks across 17 files |

## Rollback Strategy

Every task is isolated to a single file (except 3.3 which touches 2). Each commit is atomic. If any task breaks `cargo check`, revert that commit.
