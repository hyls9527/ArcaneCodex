import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { Copy, ImagePlus, ArrowRight } from 'lucide-react'
import { DedupManager } from '../components/dedup/DedupManager'
import { useDedupStore } from '../stores/useDedupStore'
import { useImageStore } from '../stores/useImageStore'
import {
  scanDuplicates,
  deleteDuplicates,
  type BackendDuplicateGroup,
} from '../lib/api'
import { navigate } from '../router/events'

interface DedupPageProps {
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
  onImagesChanged: () => void
}

export function DedupPage({ addToast, onImagesChanged }: DedupPageProps) {
  const { t } = useTranslation()
  const { groups, loading, setGroups, setLoading, removeGroups } = useDedupStore()
  const { images } = useImageStore()

  const handleScan = useCallback(async (threshold: number) => {
    setLoading(true)
    try {
      const result = await scanDuplicates(threshold)
      setGroups(result)
      addToast(t('dedup.scanComplete', { count: result.length }), 'success')
    } catch {
      addToast(t('errors.scanFailed'), 'error')
    } finally {
      setLoading(false)
    }
  }, [setGroups, setLoading, addToast, t])

  const handleDelete = useCallback(async (groupIds: string[]) => {
    try {
      const groupsToDelete = groups.filter(g => groupIds.includes(g.id))
      const backendGroups: BackendDuplicateGroup[] = groupsToDelete.map(g => ({
        images: g.images,
        similarity: g.similarity,
      }))
      if (backendGroups.length > 0) {
        await deleteDuplicates(backendGroups, 'keep_highest_resolution')
        onImagesChanged()
        removeGroups(groupIds)
        addToast(t('dedup.deleteSuccess', { count: groupsToDelete.reduce((sum, g) => sum + g.images.length - 1, 0) }), 'success')
      }
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    }
  }, [groups, removeGroups, onImagesChanged, addToast, t])

  return (
    <>
      {images.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 gap-6">
          <div className="w-20 h-20 rounded-full bg-orange-50 dark:bg-orange-900/20 flex items-center justify-center">
            <Copy className="w-10 h-10 text-orange-400 dark:text-orange-500" />
          </div>
          <div className="text-center max-w-sm">
            <h3 className="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-2">
              {t('dedup.emptyTitle')}
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
              {t('dedup.emptyDescription')}
            </p>
            <button
              onClick={() => navigate({ route: 'gallery', source: 'action' })}
              className="inline-flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
            >
              <ImagePlus className="w-4 h-4" />
              {t('dedup.goToGallery')}
              <ArrowRight className="w-4 h-4" />
            </button>
          </div>
        </div>
      ) : (
        <DedupManager
          groups={groups}
          isLoading={loading}
          onScan={handleScan}
          onDelete={handleDelete}
        />
      )}
    </>
  )
}
