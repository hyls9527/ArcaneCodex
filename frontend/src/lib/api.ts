// Tauri API Integration Layer
// This module provides a clean interface to communicate with the Rust backend

import { invoke } from '@tauri-apps/api/core'

// ===== Image Management =====

export interface ImportResult {
  success_count: number
  duplicate_count: number
  error_count: number
  image_ids: number[]
}

export async function importImages(filePaths: string[]): Promise<ImportResult> {
  return invoke<ImportResult>('import_images', { filePaths })
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
  return invoke<ImageListResponse>('get_images', {
    page: query.page,
    page_size: query.page_size,
    filters: query.filters,
  })
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
