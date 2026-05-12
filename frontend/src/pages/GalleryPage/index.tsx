import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, AlertCircle } from 'lucide-react'
import { ImageGrid } from '../../components/gallery/ImageGrid'
import { DropZone } from '../../components/gallery/DropZone'
import { LMStudioGuide } from '../../components/ai/LMStudioGuide'
import { SampleDataBanner } from '../../components/gallery/SampleDataBanner'
import { useImageStore } from '../../stores/useImageStore'
import { type AppImage } from '../../types/image'
import { useGalleryActions } from './useGalleryActions'
import { GalleryActionBar } from './GalleryActionBar'
import { GalleryEmptyState } from './GalleryEmptyState'
import { GallerySearchResults } from './GallerySearchResults'

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

  const [selectionMode, setSelectionMode] = useState(false)
  const [showAiGuide, setShowAiGuide] = useState(false)

  const selectedCount = selectedIds.length
  const failedCount = images.filter(img => img.ai_status === 'failed').length
  const hasFilters = !!(filters.ai_status || filters.category || filters.date_from || filters.date_to || (filters.tags && filters.tags.length > 0))

  const {
    handleFilesSelected,
    handleBatchDelete,
    handleClearFailedImages,
    handleSelectFiles,
    handleSelectFolder,
    deleting,
    clearingFailed,
  } = useGalleryActions({
    images,
    onLoadImages,
    addToast,
    onShowAiGuide: useCallback(() => setShowAiGuide(true), []),
  })

  useEffect(() => {
    if (hasFilters) {
      onLoadImages()
    }
  }, [filters.ai_status, filters.category, filters.date_from, filters.date_to, filters.tags, hasFilters, onLoadImages])

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

  const handleExitSelectionMode = useCallback(() => {
    setSelectionMode(false)
    deselectAll()
  }, [deselectAll])

  const handleBrokenLinksChecked = useCallback(async (_brokenCount: number) => {
    await onLoadImages()
  }, [onLoadImages])

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

      <GalleryActionBar
        selectionMode={selectionMode}
        selectedCount={selectedCount}
        deleting={deleting}
        clearingFailed={clearingFailed}
        failedCount={failedCount}
        imageCount={images.length}
        onSelectAll={selectAll}
        onBatchDelete={handleBatchDelete}
        onExitSelectionMode={handleExitSelectionMode}
        onEnterSelectionMode={() => setSelectionMode(true)}
        onClearFailedImages={handleClearFailedImages}
        onBrokenLinksChecked={handleBrokenLinksChecked}
        addToast={addToast}
      />

      <GallerySearchResults
        searchQuery={searchQuery}
        searchResults={searchResults}
        searchLoading={searchLoading}
        hasSearched={hasSearched}
        onImageClick={handleImageClick}
        selectedIds={selectedIds}
        onToggleSelect={handleToggleSelect}
      />

      {(!hasSearched || !searchQuery.trim()) && (
        <div className="h-[calc(100%-200px)]">
          {images.length === 0 ? (
            <GalleryEmptyState
              onSelectFiles={handleSelectFiles}
              onSelectFolder={handleSelectFolder}
              onLoadImages={onLoadImages}
              addToast={addToast}
            />
          ) : (
            <ImageGrid images={images} onImageClick={handleImageClick} selectedIds={selectedIds} onToggleSelect={handleToggleSelect} />
          )}
        </div>
      )}
    </>
  )
}
