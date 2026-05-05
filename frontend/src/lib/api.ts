// Tauri API Integration Layer
// This module provides a clean interface to communicate with the Rust backend

import { invoke as tauriInvoke } from '@tauri-apps/api/core'

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window !== 'undefined' && (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__) {
    return tauriInvoke<T>(cmd, args)
  }
  console.warn('[api] invoke called outside Tauri context:', cmd, args)
  throw new Error('Tauri not available. Please use the desktop app window, not browser.')
}

// ===== Image Management =====

export interface ImportError {
  file_path: string
  reason: string
}

export interface ImportResult {
  success_count: number
  duplicate_count: number
  error_count: number
  image_ids: number[]
  errors: ImportError[]
}

export async function importImages(filePaths: string[]): Promise<ImportResult> {
  console.log('[api] importImages called with:', filePaths)
  const result = await invoke<ImportResult>('import_images', { filePaths })
  console.log('[api] importImages result:', result)
  // 如果有错误，抛出包含详细信息的错误
  if (result.error_count > 0) {
    const errorDetails = result.errors.map(e => `${e.file_path}: ${e.reason}`).join('\n')
    console.error('[api] Import errors:', errorDetails)
    // 如果全部失败，抛出错误
    if (result.success_count === 0 && result.duplicate_count === 0) {
      throw new Error(`导入失败:\n${errorDetails}`)
    }
  }
  return result
}

export interface ImageQuery {
  page: number
  page_size: number
  filters?: {
    ai_status?: string
    date_from?: string
    date_to?: string
    category?: string
    tags?: string[]
  }
}

export interface AppImage {
  id: number
  file_path: string
  file_name: string
  thumbnail_path?: string
  ai_status: string
  ai_tags?: string
  ai_description?: string
  ai_category?: string
  ai_confidence?: number
  ai_tag_status?: string
  created_at?: string
  width?: number
  height?: number
  file_size?: number
  exif_data?: string
  file_hash?: string
  phash?: string
  ai_provider?: string
  ai_processed_at?: string
  ai_error_message?: string
  generation_source?: string | null
  generation_metadata?: string | null
}

export interface ImageListResponse {
  images: AppImage[]
  total: number
  page: number
  page_size: number
}

export async function getImages(query: ImageQuery): Promise<ImageListResponse> {
  const result = await invoke<ImageListResponse>('get_images', {
    page: query.page ?? 1,
    pageSize: query.page_size ?? 50,
    filters: query.filters ?? null,
  })
  return result
}

export async function getImageDetail(id: number): Promise<AppImage> {
  return invoke<AppImage>('get_image_detail', { id })
}

export async function deleteImages(ids: number[]): Promise<number> {
  return invoke<number>('delete_images', { ids })
}

// ===== AI Processing =====

export interface AIStatus {
  status: 'idle' | 'processing' | 'paused' | 'completed' | 'failed'
  total: number
  completed: number
  failed: number
  retrying: number
  eta_seconds?: number
}

export async function startAIProcessing(): Promise<void> {
  return invoke('start_ai_processing')
}

export async function pauseAIProcessing(): Promise<void> {
  return invoke('pause_ai_processing')
}

export async function resumeAIProcessing(): Promise<void> {
  return invoke('resume_ai_processing')
}

export async function getAIStatus(): Promise<AIStatus> {
  return invoke<AIStatus>('get_ai_status')
}

export async function retryFailedAI(): Promise<number> {
  return invoke<number>('retry_failed_ai')
}

export async function retrySingleAIResult(imageId: number): Promise<void> {
  return invoke('retry_failed_ai', { imageId })
}

export interface AIResult {
  id: number
  file_name: string
  ai_status: string
  ai_tags?: string
  ai_description?: string
  ai_category?: string
  ai_error_message?: string
  ai_processed_at?: string
}

export async function getRecentAIResults(limit: number = 20): Promise<AIResult[]> {
  return invoke<AIResult[]>('get_recent_ai_results', { limit })
}

// ===== Semantic Search =====

export interface SearchResult {
  image_id: number
  file_path: string
  file_name: string
  thumbnail_path?: string
  ai_description?: string
  ai_tags?: string
  ai_category?: string
  ai_confidence?: number
  match_count: number
  relevance_score: number
  score: number
  tags?: string[]
  description?: string
  category?: string
}

export interface SearchFilters {
  category?: string
  tags?: string[]
  start_date?: string
  end_date?: string
  page?: number
  page_size?: number
}

export interface SearchResponse {
  results: SearchResult[]
  total: number
  page: number
  page_size: number
}

export async function searchImages(
  query: string,
  filters: SearchFilters = {}
): Promise<SearchResult[]> {
  const request = {
    query,
    category: filters.category || null,
    tags: filters.tags || null,
    start_date: filters.start_date || null,
    end_date: filters.end_date || null,
    page: filters.page ?? 0,
    page_size: filters.page_size ?? 20,
  }

  const response = await invoke<SearchResponse>('semantic_search', { request })

  return (response.results || []).map(r => ({
    ...r,
    score: r.relevance_score || 0,
    tags: r.ai_tags ? (() => { try { return JSON.parse(r.ai_tags) } catch { return [] } })() : [],
    description: r.ai_description || '',
    category: r.ai_category || '',
  }))
}

// ===== Deduplication =====

export interface BackendDuplicateImage {
  image_id: number
  file_path: string
  file_name: string
  file_size: number
  width?: number
  height?: number
  phash: string
  distance: number
}

export interface BackendDuplicateGroup {
  images: BackendDuplicateImage[]
  similarity: number
}

export interface DedupScanResult {
  groups: BackendDuplicateGroup[]
  total_scanned: number
  total_duplicates: number
  threshold: number
}

export interface DeleteResult {
  deleted_count: number
  kept_count: number
  freed_space_bytes: number
  dry_run: boolean
}

// Legacy UI-facing type for DedupManager component
export interface DuplicateGroup {
  id: string
  images: BackendDuplicateImage[]
  image_ids: number[]
  similarity: number
}

export function mapBackendGroupsToUI(groups: BackendDuplicateGroup[]): DuplicateGroup[] {
  return groups.map((g, idx) => ({
    id: String(idx),
    images: g.images,
    image_ids: g.images.map(img => img.image_id),
    similarity: g.similarity,
  }))
}

export async function scanDuplicates(threshold: number = 90): Promise<DuplicateGroup[]> {
  const result = await invoke<DedupScanResult>('scan_duplicates', { request: { similarityPercent: threshold } })
  return mapBackendGroupsToUI(result.groups)
}

export async function deleteDuplicates(groups: BackendDuplicateGroup[], policy: string): Promise<DeleteResult> {
  return invoke<DeleteResult>('delete_duplicates', {
    request: {
      groups,
      policy,
      dry_run: false,
    },
  })
}

// ===== Settings =====

export interface AppConfig {
  key: string
  value: string
}

export async function getConfig(key: string): Promise<AppConfig | null> {
  return invoke<AppConfig | null>('get_config', { key })
}

export async function getAllConfigs(): Promise<AppConfig[]> {
  return invoke<AppConfig[]>('get_all_configs')
}

export async function setConfig(key: string, value: string): Promise<void> {
  return invoke('set_config', { key, value })
}

// Batch set configs with rollback on failure
export async function setConfigs(entries: [string, string][]): Promise<void> {
  // Save old values for rollback
  const oldValues: [string, string][] = []
  for (const [key] of entries) {
    try {
      const existing = await invoke<AppConfig | null>('get_config', { key })
      oldValues.push([key, existing?.value ?? ''])
    } catch {
      oldValues.push([key, ''])
    }
  }

  // Apply new values one by one
  const applied: [string, string][] = []
  for (const [key, value] of entries) {
    try {
      await invoke('set_config', { key, value })
      applied.push([key, value])
    } catch (err) {
      // Rollback already-applied values
      for (const [rollbackKey, rollbackValue] of applied.reverse()) {
        try {
          await invoke('set_config', { key: rollbackKey, value: rollbackValue })
        } catch {
        }
      }
      throw new Error(`Failed to set config "${key}": ${err instanceof Error ? err.message : String(err)}. Rolled back ${applied.length} changes.`)
    }
  }
}

export async function backupDatabase(outputPath: string): Promise<string> {
  return invoke<string>('backup_database', { outputPath })
}

export async function restoreDatabase(backupPath: string): Promise<void> {
  return invoke('restore_database', { backupPath })
}

export async function testLmStudioConnection(url: string): Promise<boolean> {
  return invoke<boolean>('test_lm_studio_connection', { url })
}

// ===== AI Service Detection (First-run Guide) =====

export interface DiscoveredModel {
  provider: string
  provider_name: string
  base_url: string
  model_id: string
  model_name: string | null
  port: number
  is_online: boolean
  /** Whether the model is currently loaded into VRAM and ready for inference */
  loaded: boolean
}

/**
 * Detect if any AI inference service is available locally.
 * Checks LM Studio (1234), Ollama (11434), and Hermes (18789) via backend scan.
 * Returns the list of discovered models, or empty array if none found.
 */
export async function discoverAvailableModels(): Promise<DiscoveredModel[]> {
  return invoke<DiscoveredModel[]>('discover_available_models')
}

/**
 * Quick check if any local AI service is reachable.
 * Uses backend discover_available_models command for reliable detection.
 */
export async function detectAiService(): Promise<boolean> {
  try {
    const models = await discoverAvailableModels()
    return models.some(m => m.is_online)
  } catch {
    return false
  }
}

/**
 * Auto-configure inference provider from discovered models.
 * Picks the first loaded model, or the first online model as fallback.
 * Returns the configured provider info, or null if no model was found.
 */
export async function autoConfigureInference(): Promise<{ provider: string; model: string; baseUrl: string } | null> {
  try {
    const models = await discoverAvailableModels()
    // Prefer loaded models (ready for inference)
    const loaded = models.find(m => m.loaded && m.is_online)
    const target = loaded || models.find(m => m.is_online)
    if (!target) return null

    await invoke('set_inference_provider', {
      provider: target.provider,
      model: target.model_id,
      apiKey: null,
    })

    return { provider: target.provider, model: target.model_id, baseUrl: target.base_url }
  } catch {
    return null
  }
}

// ===== Data Export =====

export interface ExportRequest {
  format: 'json' | 'csv'
  output_path: string
  image_ids?: number[]
}

export interface ExportResult {
  exported_count: number
  output_file: string
  format: string
}

export async function exportData(request: ExportRequest): Promise<ExportResult> {
  return invoke<ExportResult>('export_data', { request })
}

// ===== Broken Link Detection =====

export interface BrokenLinkInfo {
  id: number
  file_path: string
  file_name: string
}

export interface CheckBrokenLinksResult {
  broken_count: number
  broken_images: BrokenLinkInfo[]
}

export async function checkBrokenLinks(): Promise<CheckBrokenLinksResult> {
  return invoke<CheckBrokenLinksResult>('check_broken_links')
}

// ===== Image Archive =====

export interface ArchiveImageResult {
  archived: boolean
  dest_path: string
}

export async function archiveImage(id: number): Promise<ArchiveImageResult> {
  return invoke<ArchiveImageResult>('archive_image', { id })
}

// ===== Safe Export =====

export interface SafeExportError {
  id: number
  reason: string
}

export interface SafeExportResult {
  exported_count: number
  errors: SafeExportError[]
}

export async function safeExport(imageIds: number[], destDir: string): Promise<SafeExportResult> {
  return invoke<SafeExportResult>('safe_export', { imageIds, destDir })
}

// ===== Narrative Anchor =====

export interface Narrative {
  id: number
  image_id: number
  content: string
  entities_json: string
}

export interface AssociationResult {
  image_id: number
  file_path: string
  file_name: string
  thumbnail_path: string | null
  narrative_content: string
  match_type: string
  relevance: number
}

export async function writeNarrative(imageId: number, content: string): Promise<Narrative> {
  return invoke<Narrative>('write_narrative', { imageId, content })
}

export async function getNarratives(imageId: number): Promise<Narrative[]> {
  return invoke<Narrative[]>('get_narratives', { imageId })
}

export async function queryAssociations(query: string, limit?: number): Promise<AssociationResult[]> {
  return invoke<AssociationResult[]>('query_associations', { query, limit: limit ?? 20 })
}

// ===== Dashboard Statistics =====

export interface AIProgressStats {
  pending: number
  processing: number
  completed: number
  failed: number
  verified: number
  provisional: number
  rejected: number
}

export interface StorageStats {
  total_size_bytes: number
  average_image_size: number
  largest_image_size: number
}

export interface LibraryStats {
  total_images: number
  category_distribution: [string, number][]
  ai_progress: AIProgressStats
  storage_usage: StorageStats
  tag_cloud: [string, number][]
}

export async function getLibraryStats(): Promise<LibraryStats> {
  return invoke<LibraryStats>('get_library_stats')
}

export interface AccuracyDataPoint {
  date: string
  total: number
  correct: number
  accuracy: number
}

export interface CategoryAccuracy {
  category: string
  total: number
  verified: number
  provisional: number
  rejected: number
  average_confidence: number
}

export interface CalibrationComparison {
  before_ece: number
  after_ece: number
  improvement_percent: number
}

export interface AccuracyTrend {
  daily_data: AccuracyDataPoint[]
  category_accuracy: CategoryAccuracy[]
  calibration_comparison: CalibrationComparison | null
}

export async function getAccuracyTrend(days: number = 30): Promise<AccuracyTrend> {
  return invoke<AccuracyTrend>('get_accuracy_trend', { days })
}

// ===== Log Management =====

export interface LogEntry {
  timestamp: string
  level: string
  target: string
  message: string
}

export interface LogFileStats {
  path: string
  size_bytes: number
  line_count: number
  exists: boolean
}

export interface LogResponse {
  entries: LogEntry[]
  total_lines: number
  has_more: boolean
}

export async function getLogEntries(
  maxLines?: number,
  offset?: number,
  levelFilter?: string
): Promise<LogResponse> {
  return invoke<LogResponse>('get_log_entries', { maxLines: maxLines ?? 200, offset: offset ?? 0, levelFilter })
}

export async function getLogStats(): Promise<LogFileStats> {
  return invoke<LogFileStats>('get_log_stats')
}

export async function exportLogs(exportPath: string, levelFilter?: string): Promise<number> {
  return invoke<number>('export_logs', { exportPath, levelFilter })
}

export async function clearLogs(): Promise<number> {
  return invoke<number>('clear_logs')
}

// ===== Sample Data =====

export interface SampleDataStatus {
  has_sample_data: boolean
  sample_count: number
}

export async function checkSampleData(): Promise<SampleDataStatus> {
  return invoke<SampleDataStatus>('check_sample_data')
}

export async function clearSampleData(): Promise<number> {
  return invoke<number>('clear_sample_data')
}

export async function loadSampleData(): Promise<number> {
  return invoke<number>('load_sample_data')
}

// === XMP Metadata API ===

export interface XmpMetadata {
  creator?: string
  title?: string
  description?: string
  subject?: string[]
  keywords?: string[]
  rating?: number
}

export async function readXmpMetadata(filePath: string): Promise<XmpMetadata | null> {
  try {
    return invoke<XmpMetadata | null>('read_xmp_metadata', { filePath })
  } catch {
    return null
  }
}

export async function writeXmpMetadata(filePath: string, metadata: XmpMetadata): Promise<void> {
  return invoke('write_xmp_metadata', { filePath, metadata })
}

export async function generateXmpSidecar(imagePath: string, metadata: XmpMetadata): Promise<string> {
  return invoke<string>('generate_xmp_sidecar', { imagePath, metadata })
}

export async function exportAsXmp(imageIds: number[]): Promise<string[]> {
  return invoke<string[]>('export_as_xmp', { imageIds })
}

// === File Monitor API ===

export interface MonitorStatus {
  is_running: boolean
  watched_directories: number
}

export async function startFileMonitor(directory: string): Promise<void> {
  return invoke('start_file_monitor', { directory })
}

export async function stopFileMonitor(): Promise<void> {
  return invoke('stop_file_monitor')
}

export async function getMonitorStatus(): Promise<MonitorStatus> {
  return invoke<MonitorStatus>('get_monitor_status')
}

// ===== ONNX Runtime & AI Models =====

export interface ModelStatus {
  model_type: string
  name: string
  is_loaded: boolean
  path: string
}

export interface ModelLoadResult {
  success: boolean
  model: ModelStatus | null
  error?: string
}

export async function getAIModelStatus(): Promise<ModelStatus[]> {
  return invoke<ModelStatus[]>('get_ai_model_status')
}

export async function loadAIModel(modelType: string, customPath?: string): Promise<ModelLoadResult> {
  return invoke<ModelLoadResult>('load_ai_model', { modelType, customPath })
}

export async function unloadAIModel(modelType: string): Promise<boolean> {
  return invoke<boolean>('unload_ai_model', { modelType })
}

// ===== Image Classification (MobileNet) =====

export interface ClassPrediction {
  class_name: string
  confidence: number
}

export interface ClassificationResult {
  label: string
  confidence: number
  top_n_predictions: ClassPrediction[]
}

export async function classifyImage(imagePath: string, topN?: number): Promise<ClassificationResult> {
  return invoke<ClassificationResult>('classify_image', { imagePath, topN: topN ?? 5 })
}

// ===== Face Detection & Recognition =====

export interface BoundingBox {
  x: number
  y: number
  width: number
  height: number
}

export interface Landmark {
  x: number
  y: number
}

export interface FaceDetection {
  bbox: BoundingBox
  landmarks: Landmark[]
  confidence: number
}

export interface FaceEmbeddingInfo {
  face_id: string
  embedding: number[]
  image_path: string
  detection: FaceDetection
}

export interface FaceMatch {
  face_id: string
  similarity: number
  face_embedding: FaceEmbeddingInfo
}

export async function detectFaces(imagePath: string, confidenceThreshold?: number): Promise<FaceDetection[]> {
  return invoke<FaceDetection[]>('detect_faces', { imagePath, confidenceThreshold: confidenceThreshold ?? 0.5 })
}

export async function extractFaceEmbedding(imagePath: string, bbox: BoundingBox): Promise<FaceEmbeddingInfo> {
  return invoke<FaceEmbeddingInfo>('extract_face_embedding', { imagePath, bbox })
}

export async function registerFace(imagePath: string, bbox: BoundingBox): Promise<string> {
  return invoke<string>('register_face', { imagePath, bbox })
}

export async function recognizeFace(imagePath: string, bbox: BoundingBox, threshold?: number): Promise<FaceMatch | null> {
  return invoke<FaceMatch | null>('recognize_face', { imagePath, bbox, threshold: threshold ?? 0.6 })
}

export async function getRegisteredFaceCount(): Promise<number> {
  return invoke<number>('get_registered_face_count')
}

// ===== CLIP Embeddings =====

export interface ClipEmbeddingInfo {
  image_id: string
  embedding: number[]
  image_path: string
}

export async function embedImageWithClip(imagePath: string): Promise<ClipEmbeddingInfo> {
  return invoke<ClipEmbeddingInfo>('embed_image_clip', { imagePath })
}

// ===== Vector Search (HNSW) =====

export interface VectorEntry {
  id: string
  embedding: number[]
  metadata?: Record<string, unknown>
  image_path?: string
}

export interface SearchResultItem {
  id: string
  similarity: number
  entry: VectorEntry
}

export interface IndexStats {
  total_vectors: number
  dimension: number
  index_size_bytes: number
}

export async function insertVector(entry: VectorEntry): Promise<void> {
  return invoke('insert_vector', { entry })
}

export async function searchVectors(query: number[], topK?: number, minSimilarity?: number): Promise<SearchResultItem[]> {
  return invoke<SearchResultItem[]>('search_vectors', { query, topK: topK ?? 10, minSimilarity: minSimilarity ?? 0.3 })
}

export async function deleteVector(id: string): Promise<boolean> {
  return invoke<boolean>('delete_vector', { id })
}

export async function getVectorIndexStats(): Promise<IndexStats> {
  return invoke<IndexStats>('get_vector_index_stats')
}

// ===== Knowledge Graph =====

export interface KgNode {
  id: string
  node_type: 'image' | 'entity' | 'tag' | 'concept'
  label: string
  properties: Record<string, unknown>
  embedding?: number[]
  community_id?: number | null
  degree: number
}

export interface KgEdge {
  id: string
  source_id: string
  target_id: string
  edge_type: string
  weight: number
  properties: Record<string, unknown>
}

export interface KgCommunity {
  id: number
  size: number
  central_node_id?: string | null
  tags: string[]
  density: number
}

export interface KgStats {
  total_nodes: number
  total_edges: number
  node_types: Record<string, number>
  edge_types: Record<string, number>
  communities: number
  avg_degree: number
  density: number
}

export interface KgNeighbor {
  node: KgNode
  edge: KgEdge
  distance: number
}

export interface KgPath {
  nodes: KgNode[]
  edges: KgEdge[]
  total_weight: number
  length: number
}

export interface KgBuildResult {
  success: boolean
  nodes_added: number
  error?: string | null
}

export async function kgBuildGraph(): Promise<KgBuildResult> {
  return invoke<KgBuildResult>('kg_build_graph')
}

export async function kgGetStats(): Promise<KgStats> {
  return invoke<KgStats>('kg_get_stats')
}

export async function kgGetAllNodes(): Promise<KgNode[]> {
  return invoke<KgNode[]>('kg_get_all_nodes')
}

export async function kgGetAllEdges(): Promise<KgEdge[]> {
  return invoke<KgEdge[]>('kg_get_all_edges')
}

export async function kgGetCommunities(): Promise<KgCommunity[]> {
  return invoke<KgCommunity[]>('kg_get_communities')
}

export async function kgGetCommunityNodes(communityId: number): Promise<KgNode[]> {
  return invoke<KgNode[]>('kg_get_community_nodes', { communityId })
}

export async function kgGetNeighbors(
  nodeId: string,
  edgeTypes?: string[],
  limit?: number
): Promise<KgNeighbor[]> {
  return invoke<KgNeighbor[]>('kg_get_neighbors', { nodeId, edgeTypes, limit })
}

export async function kgFindPath(sourceId: string, targetId: string): Promise<KgPath | null> {
  return invoke<KgPath | null>('kg_find_path', { sourceId, targetId })
}

export async function kgSearchNodes(query: string, limit?: number): Promise<KgNode[]> {
  return invoke<KgNode[]>('kg_search_nodes', { query, limit: limit ?? 20 })
}

export async function kgClear(): Promise<void> {
  return invoke('kg_clear')
}

export async function kgLoadFromDb(): Promise<KgBuildResult> {
  return invoke<KgBuildResult>('kg_load_from_db')
}

export async function kgSaveToDb(): Promise<KgBuildResult> {
  return invoke<KgBuildResult>('kg_save_to_db')
}

// ===== Tag Correction (死命令接入) =====

export interface TagCorrectionRequest {
  image_id: number
  old_tags: string[]
  new_tags: string[]
}

export interface TagCorrectionRecord {
  id: number
  image_id: number
  old_tags: string[]
  new_tags: string[]
  corrected_at: string
}

export async function recordTagCorrection(request: TagCorrectionRequest): Promise<number> {
  return invoke<number>('record_tag_correction', { request })
}

export async function getTagCorrectionHistory(imageId: number): Promise<TagCorrectionRecord[]> {
  return invoke<TagCorrectionRecord[]>('get_tag_correction_history', { imageId })
}

export async function getAllTagCorrections(limit?: number, offset?: number): Promise<TagCorrectionRecord[]> {
  return invoke<TagCorrectionRecord[]>('get_all_tag_corrections', { limit, offset })
}

// ===== Error Patterns (死命令接入) =====

export interface ErrorPattern {
  id: number
  pattern_name: string
  pattern_description: string | null
  occurrence_count: number
  first_seen: string
  last_seen: string
}

export interface RecordErrorPatternRequest {
  pattern_name: string
  pattern_description?: string
}

export async function recordErrorPattern(request: RecordErrorPatternRequest): Promise<number> {
  return invoke<number>('record_error_pattern', { request })
}

export async function getErrorPatterns(limit?: number, minOccurrences?: number): Promise<ErrorPattern[]> {
  return invoke<ErrorPattern[]>('get_error_patterns', { limit, minOccurrences })
}

export async function checkErrorPatternExists(patternName: string): Promise<ErrorPattern | null> {
  return invoke<ErrorPattern | null>('check_error_pattern_exists', { patternName })
}

export async function deleteErrorPattern(patternId: number): Promise<void> {
  return invoke('delete_error_pattern', { patternId })
}

export async function getHighFrequencyErrorPatterns(minCount?: number): Promise<ErrorPattern[]> {
  return invoke<ErrorPattern[]>('get_high_frequency_error_patterns', { minCount })
}

// ===== Batch Operations (死命令接入) =====

export interface BatchAITagRequest {
  image_ids: number[]
}

export interface BatchTaskStatus {
  total: number
  completed: number
  failed: number
  in_progress: number
}

export async function startBatchAITag(imageIds: number[]): Promise<number> {
  return invoke<number>('start_batch_ai_tag', { request: { image_ids: imageIds } })
}

export async function getBatchAIStatus(): Promise<BatchTaskStatus> {
  return invoke<BatchTaskStatus>('get_batch_ai_status')
}

export async function pauseBatchAITask(): Promise<void> {
  return invoke('pause_batch_ai_task')
}

export async function resumeBatchAITask(): Promise<void> {
  return invoke('resume_batch_ai_task')
}

export async function cancelBatchAITask(): Promise<number> {
  return invoke<number>('cancel_batch_ai_task')
}

export type TagOperation = 'Add' | 'Remove' | 'Replace'

export interface BatchTagCorrectionRequest {
  image_ids: number[]
  tags: string[]
  operation: TagOperation
}

export async function batchTagCorrection(request: BatchTagCorrectionRequest): Promise<number> {
  return invoke<number>('batch_tag_correction', { request })
}

export interface BatchExportRequest {
  image_ids: number[]
  format: 'json' | 'csv' | 'xmp'
  output_path: string
}

export async function batchExport(request: BatchExportRequest): Promise<ExportResult> {
  return invoke<ExportResult>('batch_export', { request })
}
