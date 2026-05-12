import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useImageStore } from '../../stores/useImageStore'
import { useConfigStore } from '../../stores/useConfigStore'
import {
  importImages,
  deleteImages,
  detectAiService,
  startAIProcessing,
} from '../../lib/api'
import { open as openDialog } from '@tauri-apps/plugin-dialog'

interface UseGalleryActionsParams {
  images: { id: number; ai_status: string }[]
  onLoadImages: () => Promise<void>
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
  onShowAiGuide: () => void
}

export interface UseGalleryActionsReturn {
  handleFilesSelected: (paths: string[]) => Promise<void>
  handleBatchDelete: () => Promise<void>
  handleClearFailedImages: () => Promise<void>
  handleSelectFiles: () => Promise<void>
  handleSelectFolder: () => Promise<void>
  deleting: boolean
  clearingFailed: boolean
}

export function useGalleryActions({
  images,
  onLoadImages,
  addToast,
  onShowAiGuide,
}: UseGalleryActionsParams): UseGalleryActionsReturn {
  const { t } = useTranslation()
  const { selectedIds, deselectAll } = useImageStore()
  const { aiServiceReady } = useConfigStore()

  const [deleting, setDeleting] = useState(false)
  const [clearingFailed, setClearingFailed] = useState(false)

  const selectedCount = selectedIds.length
  const failedCount = images.filter(img => img.ai_status === 'failed').length

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
        addToast(t('gallery.importPartialError', { count: result.error_count }), 'error')
      }

      const hasSeenGuide = localStorage.getItem('has_seen_ai_guide')
      if (!hasSeenGuide) {
        const ok = await detectAiService()
        if (!ok) {
          onShowAiGuide()
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
  }, [onLoadImages, addToast, t, aiServiceReady, onShowAiGuide])

  const handleBatchDelete = useCallback(async () => {
    if (selectedCount === 0) return
    if (!window.confirm(t('gallery.batchDeleteConfirm', { count: selectedCount }))) return
    setDeleting(true)
    try {
      await deleteImages(selectedIds)
      deselectAll()
      await onLoadImages()
      addToast(t('gallery.batchDeleteSuccess', { count: selectedCount }), 'success')
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    } finally {
      setDeleting(false)
    }
  }, [selectedCount, selectedIds, deselectAll, onLoadImages, addToast, t])

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
  }, [failedCount, images, onLoadImages, addToast, t])

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

  return {
    handleFilesSelected,
    handleBatchDelete,
    handleClearFailedImages,
    handleSelectFiles,
    handleSelectFolder,
    deleting,
    clearingFailed,
  }
}
