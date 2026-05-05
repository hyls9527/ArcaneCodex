import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, AlertCircle, Search, Link2, ImagePlus, CheckSquare, Trash2, X, Square, FolderOpen, FileImage, Database } from 'lucide-react'
import { ImageGrid } from '../components/gallery/ImageGrid'
import { ImageFilter } from '../components/gallery/ImageFilter'
import { DropZone } from '../components/gallery/DropZone'
import { LMStudioGuide } from '../components/ai/LMStudioGuide'
import { SampleDataBanner } from '../components/gallery/SampleDataBanner'
import { useImageStore } from '../stores/useImageStore'
import { useConfigStore } from '../stores/useConfigStore'
import { type AppImage } from '../types/image'
import {
  importImages,
  checkBrokenLinks,
  deleteImages,
  detectAiService,
  startAIProcessing,
  loadSampleData,
} from '../lib/api'
import { open as openDialog } from '@tauri-apps/plugin-dialog'

interface GalleryPageProps {
  images: AppImage[]
  loading: boolean
  error: string | null
  onLoadImages: () => Promise<void>
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
  onImageClick: (image: AppImage) => void
}

export function GalleryPage({
  images,
  loading,
  error,
  onLoadImages,
  addToast,
  onImageClick,
}: GalleryPageProps) {
  const { t } = useTranslation()
  const {
    filters,
    searchQuery,
    searchResults,
    searchLoading,
    hasSearched,
    selectedIds,
    toggleSelect,
    selectAll,
    deselectAll,
  } = useImageStore()
  const { aiServiceReady } = useConfigStore()

  const [selectionMode, setSelectionMode] = useState(false)
  const [deleting, setDeleting] = useState(false)
  const [clearingFailed, setClearingFailed] = useState(false)
  const [showAiGuide, setShowAiGuide] = useState(false)

  const selectedCount = selectedIds.length
  const failedCount = images.filter(img => img.ai_status === 'failed').length
  const hasFilters = !!(filters.ai_status || filters.category || filters.date_from || filters.date_to || (filters.tags && filters.tags.length > 0))

  useEffect(() => {
    if (hasFilters) {
      onLoadImages()
    }
  }, [filters.ai_status, filters.category, filters.date_from, filters.date_to, filters.tags, hasFilters, onLoadImages])

  const handleFilesSelected = useCallback(async (paths: string[]) => {
    if (paths.length === 0) return
    try {
      console.log('[Gallery] Importing paths:', paths)
      const result = await importImages(paths)
      console.log('[Gallery] importImages result:', result)
      await onLoadImages()

      if (result.success_count > 0) {
        addToast(t('gallery.importSuccess'), 'success')
      } else if (result.duplicate_count > 0) {
        addToast(t('gallery.importDuplicate', { count: result.duplicate_count }), 'info')
      }
      if (result.error_count > 0) {
        addToast(t('gallery.importPartialError', { count: result.error_count }), 'warning')
      }

      const hasSeenGuide = localStorage.getItem('has_seen_ai_guide')
      if (!hasSeenGuide) {
        const ok = await detectAiService()
        if (!ok) {
          setShowAiGuide(true)
        } else {
          localStorage.setItem('has_seen_ai_guide', 'true')
        }
      }

      if (aiServiceReady) {
        try {
          await startAIProcessing()
          addToast(t('gallery.aiAutoProcessing'), 'info')
        } catch {
          // Silently ignore — user can manually start from AI page
        }
      }
    } catch (e) {
      console.error('[Gallery] Import failed:', e)
      addToast(t('errors.importFailed'), 'error')
    }
  }, [onLoadImages, addToast, t, aiServiceReady])

  const handleImageClick = useCallback((id: number) => {
    if (selectionMode) {
      toggleSelect(id)
      return
    }
    const image = images.find(img => img.id === id)
    if (image) onImageClick(image)
  }, [images, onImageClick, selectionMode, toggleSelect])

  const handleToggleSelect = useCallback((id: number) => {
    toggleSelect(id)
  }, [toggleSelect])

  const handleBatchDelete = useCallback(async () => {
    if (selectedCount === 0) return
    if (!window.confirm(t('gallery.batchDeleteConfirm', { count: selectedCount }))) return
    setDeleting(true)
    try {
      await deleteImages(selectedIds)
      deselectAll()
      setSelectionMode(false)
      await onLoadImages()
      addToast(t('gallery.batchDeleteSuccess', { count: selectedCount }), 'success')
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    } finally {
      setDeleting(false)
    }
  }, [selectedCount, selectedIds, deselectAll, onLoadImages, addToast, t])

  const handleExitSelectionMode = useCallback(() => {
    setSelectionMode(false)
    deselectAll()
  }, [deselectAll])

  const handleClearFailedImages = useCallback(async () => {
    if (failedCount === 0) return
    setClearingFailed(true)
    try {
      const failedIds = images.filter(img => img.ai_status === 'failed').map(img => img.id)
      await deleteImages(failedIds)
      addToast(t('gallery.clearedFailedImages', { count: failedCount }), 'success')
      await onLoadImages()
    } catch (err) {
      console.error('[Gallery] clearFailed error:', err)
      addToast(t('errors.clearFailedFailed'), 'error')
    } finally {
      setClearingFailed(false)
    }
  }, [failedCount, images, deleteImages, onLoadImages, addToast, t])

  const handleSelectFiles = useCallback(async () => {
    try {
      const selected = await openDialog({
        multiple: true,
        filters: [{ name: 'Images', extensions: ['jpeg', 'jpg', 'png', 'webp', 'gif', 'bmp', 'tiff', 'tif', 'avif'] }],
        title: t('import.selectFiles'),
      })
      if (!selected) return
      const paths = Array.isArray(selected) ? selected : [selected]
      await handleFilesSelected(paths)
    } catch (e) {
      console.error('[Gallery] Import failed:', e)
      addToast(t('errors.importFailed'), 'error')
    }
  }, [handleFilesSelected, addToast, t])

  const handleSelectFolder = useCallback(async () => {
    try {
      const selected = await openDialog({
        directory: true,
        title: t('import.selectFolder'),
      })
      if (!selected) return
      await handleFilesSelected([selected as string])
    } catch {
      addToast(t('errors.importFailed'), 'error')
    }
  }, [handleFilesSelected, addToast, t])

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
        <p className="text-gray-500">{t('common.loading')}</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 text-red-600 dark:text-red-400">
        <AlertCircle className="w-8 h-8" />
        <p className="text-sm">{error}</p>
        <button onClick={onLoadImages} className="btn-primary mt-2">{t('common.retry')}</button>
      </div>
    )
  }

  return (
    <>
      {/* First-run AI service guide modal */}
      {showAiGuide && (
        <LMStudioGuide
          autoDetect
          onDismiss={() => setShowAiGuide(false)}
          onSkip={() => setShowAiGuide(false)}
        />
      )}

      <div className="mb-4">
        <DropZone onFilesSelected={handleFilesSelected} />
      </div>
      <SampleDataBanner onCleared={onLoadImages} />
      <div className="flex items-center gap-3 mb-4">
        <ImageFilter />

        {selectionMode ? (
          <div className="flex items-center gap-2 flex-1">
            <span className="text-sm font-medium text-primary-600 dark:text-primary-400">
              {t('gallery.selectedCount', { count: selectedCount })}
            </span>
            <button
              onClick={selectAll}
              className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
            >
              <CheckSquare className="w-3.5 h-3.5" />
              {t('gallery.selectAll')}
            </button>
            <button
              onClick={handleBatchDelete}
              disabled={deleting || selectedCount === 0}
              className="flex items-center gap-1 px-3 py-1.5 text-sm rounded-lg bg-red-500 hover:bg-red-600 disabled:opacity-50 disabled:cursor-not-allowed text-white transition-colors"
            >
              {deleting ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Trash2 className="w-4 h-4" />
              )}
              {t('gallery.batchDelete')}
            </button>
            <button
              onClick={handleExitSelectionMode}
              className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
            >
              <X className="w-3.5 h-3.5" />
              {t('gallery.cancelSelection')}
            </button>
          </div>
        ) : (
          <>
            <button
              onClick={() => setSelectionMode(true)}
              disabled={images.length === 0}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors disabled:opacity-50"
            >
              <Square className="w-4 h-4" />
              {t('gallery.selectMode')}
            </button>
            <button
              onClick={async () => {
                try {
                  const result = await checkBrokenLinks()
                  if (result.broken_count > 0) {
                    addToast(t('gallery.brokenLinksFound', { count: result.broken_count }), 'error')
                    await onLoadImages()
                  } else {
                    addToast(t('gallery.noBrokenLinks'), 'success')
                  }
                } catch {
                  addToast(t('errors.brokenLinksCheckFailed'), 'error')
                }
              }}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
            >
              <Link2 className="w-4 h-4" />
              {t('gallery.checkBrokenLinks')}
            </button>
            {failedCount > 0 && (
              <button
                onClick={handleClearFailedImages}
                disabled={clearingFailed}
                className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg bg-red-500 hover:bg-red-600 disabled:opacity-50 text-white transition-colors"
              >
                {clearingFailed ? <Loader2 className="w-4 h-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                {t('gallery.clearFailed', { count: failedCount })}
              </button>
            )}
          </>
        )}
      </div>

      {hasSearched && searchQuery.trim() && (
        <div className="mb-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-lg font-medium">{t('gallery.searchResults', { query: searchQuery })}</h3>
            <span className="text-sm text-gray-500">{searchResults.length} {t('gallery.resultsCount')}</span>
          </div>
          {searchLoading ? (
            <div className="flex items-center justify-center py-8 gap-2">
              <Loader2 className="w-5 h-5 animate-spin text-primary-500" />
              <span className="text-sm text-gray-500">{t('common.searching')}</span>
            </div>
          ) : searchResults.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-gray-500">
              <Search className="w-12 h-12 mb-2 opacity-50" />
              <p className="text-sm">{t('gallery.noResults', { query: searchQuery })}</p>
            </div>
          ) : (
            <div className="h-[calc(100%-200px)]">
              <ImageGrid
                images={searchResults.map(r => ({
                  id: r.image_id,
                  file_path: r.file_path,
                  thumbnail_path: r.thumbnail_path || '',
                  file_name: r.file_name || String(r.image_id),
                  ai_tags: r.tags ? JSON.stringify(r.tags) : undefined,
                  ai_status: 'completed' as const,
                  ai_category: r.category || '',
                  ai_description: r.description,
                  ai_confidence: r.ai_confidence,
                })) as AppImage[]}
                onImageClick={handleImageClick}
                selectedIds={selectedIds}
                onToggleSelect={handleToggleSelect}
              />
            </div>
          )}
        </div>
      )}

      {(!hasSearched || !searchQuery.trim()) && (
        <div className="h-[calc(100%-200px)]">
          {images.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full gap-6 px-8">
              <div className="w-24 h-24 rounded-full bg-gray-100 dark:bg-dark-200 flex items-center justify-center">
                <ImagePlus className="w-12 h-12 text-gray-300 dark:text-gray-600" />
              </div>
              <div className="text-center max-w-md">
                <h3 className="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-2">
                  {t('gallery.emptyTitle')}
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
                  {t('gallery.emptyDescription')}
                </p>
                <div className="flex items-center justify-center gap-3 mb-6">
                  <button
                    onClick={handleSelectFiles}
                    className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
                  >
                    <FileImage className="w-4 h-4" />
                    {t('import.selectFiles')}
                  </button>
                  <button
                    onClick={handleSelectFolder}
                    className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
                  >
                    <FolderOpen className="w-4 h-4" />
                    {t('import.selectFolder')}
                  </button>
                  <button
                    onClick={async () => {
                      try {
                        const count = await loadSampleData()
                        addToast(t('gallery.sampleDataLoaded', { count }), 'success')
                        await onLoadImages()
                      } catch (err) {
                        console.error('[Gallery] loadSampleData error:', err)
                        addToast(t('errors.loadSampleDataFailed'), 'error')
                      }
                    }}
                    className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg border border-primary-300 dark:border-primary-700 text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
                  >
                    <Database className="w-4 h-4" />
                    {t('gallery.loadSampleData')}
                  </button>
                </div>
                <div className="flex flex-col gap-3 text-left bg-gray-50 dark:bg-dark-200 rounded-lg p-4">
                  <div className="flex items-start gap-3">
                    <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mt-0.5">
                      <span className="text-xs font-bold text-primary-600 dark:text-primary-400">1</span>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {t('gallery.emptyStep1Title')}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {t('gallery.emptyStep1Desc')}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-start gap-3">
                    <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mt-0.5">
                      <span className="text-xs font-bold text-primary-600 dark:text-primary-400">2</span>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {t('gallery.emptyStep2Title')}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {t('gallery.emptyStep2Desc')}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-start gap-3">
                    <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mt-0.5">
                      <span className="text-xs font-bold text-primary-600 dark:text-primary-400">3</span>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {t('gallery.emptyStep3Title')}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {t('gallery.emptyStep3Desc')}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <ImageGrid images={images} onImageClick={handleImageClick} selectedIds={selectedIds} onToggleSelect={handleToggleSelect} />
          )}
        </div>
      )}
    </>
  )
}
