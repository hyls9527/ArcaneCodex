import { useTranslation } from 'react-i18next'
import { Loader2, CheckSquare, Trash2, X, Square, Link2 } from 'lucide-react'
import { ImageFilter } from '../../components/gallery/ImageFilter'
import { checkBrokenLinks } from '../../lib/api'

interface GalleryActionBarProps {
  selectionMode: boolean
  selectedCount: number
  deleting: boolean
  clearingFailed: boolean
  failedCount: number
  imageCount: number
  onSelectAll: () => void
  onBatchDelete: () => void
  onExitSelectionMode: () => void
  onEnterSelectionMode: () => void
  onClearFailedImages: () => void
  onBrokenLinksChecked: (brokenCount: number) => Promise<void>
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
}

export function GalleryActionBar({
  selectionMode,
  selectedCount,
  deleting,
  clearingFailed,
  failedCount,
  imageCount,
  onSelectAll,
  onBatchDelete,
  onExitSelectionMode,
  onEnterSelectionMode,
  onClearFailedImages,
  onBrokenLinksChecked,
  addToast,
}: GalleryActionBarProps) {
  const { t } = useTranslation()

  const handleCheckBrokenLinks = async () => {
    try {
      const result = await checkBrokenLinks()
      if (result.broken_count > 0) {
        addToast(t('gallery.brokenLinksFound', { count: result.broken_count }), 'error')
        await onBrokenLinksChecked(result.broken_count)
      } else {
        addToast(t('gallery.noBrokenLinks'), 'success')
      }
    } catch {
      addToast(t('errors.brokenLinksCheckFailed'), 'error')
    }
  }

  return (
    <div className="flex items-center gap-3 mb-4">
      <ImageFilter />

      {selectionMode ? (
        <div className="flex items-center gap-2 flex-1">
          <span className="text-sm font-medium text-primary-600 dark:text-primary-400">
            {t('gallery.selectedCount', { count: selectedCount })}
          </span>
          <button
            onClick={onSelectAll}
            className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
          >
            <CheckSquare className="w-3.5 h-3.5" />
            {t('gallery.selectAll')}
          </button>
          <button
            onClick={onBatchDelete}
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
            onClick={onExitSelectionMode}
            className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
          >
            <X className="w-3.5 h-3.5" />
            {t('gallery.cancelSelection')}
          </button>
        </div>
      ) : (
        <>
          <button
            onClick={onEnterSelectionMode}
            disabled={imageCount === 0}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors disabled:opacity-50"
          >
            <Square className="w-4 h-4" />
            {t('gallery.selectMode')}
          </button>
          <button
            onClick={handleCheckBrokenLinks}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
          >
            <Link2 className="w-4 h-4" />
            {t('gallery.checkBrokenLinks')}
          </button>
          {failedCount > 0 && (
            <button
              onClick={onClearFailedImages}
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
  )
}
