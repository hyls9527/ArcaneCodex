import { useTranslation } from 'react-i18next'
import { ImagePlus, Sparkles, FileImage, FolderOpen, Database } from 'lucide-react'
import { loadSampleData } from '../../lib/api'

interface GalleryEmptyStateProps {
  onSelectFiles: () => void
  onSelectFolder: () => void
  onLoadImages: () => Promise<void>
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
}

export function GalleryEmptyState({
  onSelectFiles,
  onSelectFolder,
  onLoadImages,
  addToast,
}: GalleryEmptyStateProps) {
  const { t } = useTranslation()

  const handleLoadSampleData = async () => {
    try {
      const count = await loadSampleData()
      addToast(t('gallery.sampleDataLoaded', { count }), 'success')
      await onLoadImages()
    } catch (err) {
      console.error('[Gallery] loadSampleData error:', err)
      addToast(t('errors.loadSampleDataFailed'), 'error')
    }
  }

  return (
    <div className="flex flex-col items-center justify-center h-full gap-6 px-8">
      <div className="relative">
        <div className="w-28 h-28 rounded-2xl bg-gradient-to-br from-primary-100 to-primary-200 dark:from-primary-900/30 dark:to-primary-800/20 flex items-center justify-center shadow-lg">
          <ImagePlus className="w-14 h-14 text-primary-500 dark:text-primary-400" />
        </div>
        <div className="absolute -bottom-2 -right-2 w-10 h-10 rounded-xl bg-white dark:bg-dark-100 shadow-md flex items-center justify-center">
          <Sparkles className="w-5 h-5 text-amber-500" />
        </div>
      </div>
      <div className="text-center max-w-md">
        <h3 className="text-xl font-bold text-gray-800 dark:text-gray-100 mb-2">
          {t('gallery.emptyTitle')}
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
          {t('gallery.emptyDescription')}
        </p>
        <div className="flex items-center justify-center gap-3 mb-6 flex-wrap">
          <button
            onClick={onSelectFiles}
            className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-xl bg-primary-600 hover:bg-primary-700 text-white transition-all shadow-sm hover:shadow-md active:scale-95"
          >
            <FileImage className="w-4 h-4" />
            {t('import.selectFiles')}
          </button>
          <button
            onClick={onSelectFolder}
            className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-xl border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-all active:scale-95"
          >
            <FolderOpen className="w-4 h-4" />
            {t('import.selectFolder')}
          </button>
          <button
            onClick={handleLoadSampleData}
            className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-xl border border-primary-300 dark:border-primary-700 text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-all active:scale-95"
          >
            <Database className="w-4 h-4" />
            {t('gallery.loadSampleData')}
          </button>
        </div>
        <div className="flex flex-col gap-3 text-left bg-gray-50 dark:bg-dark-200/50 rounded-xl p-4 border border-gray-100 dark:border-gray-700/50">
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 w-7 h-7 rounded-full bg-primary-500 text-white flex items-center justify-center text-xs font-bold mt-0.5 shadow-sm">
              1
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
            <div className="flex-shrink-0 w-7 h-7 rounded-full bg-primary-500 text-white flex items-center justify-center text-xs font-bold mt-0.5 shadow-sm">
              2
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
            <div className="flex-shrink-0 w-7 h-7 rounded-full bg-primary-500 text-white flex items-center justify-center text-xs font-bold mt-0.5 shadow-sm">
              3
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
  )
}
