import { useEffect, useState, useCallback, Suspense, lazy } from 'react'
import { useTranslation } from 'react-i18next'
import { ErrorBoundary } from './components/common/ErrorBoundary'
import { TopBar } from './components/layout/TopBar'
import { Sidebar } from './components/layout/Sidebar'
import { MainLayout } from './components/layout/MainLayout'
import { useThemeStore } from './stores/useThemeStore'
import { useConfigStore } from './stores/useConfigStore'
import type { Theme } from './stores/useThemeStore'
import { useImageStore } from './stores/useImageStore'
import { ImageViewer } from './components/gallery/ImageViewer'
import { ImportProgressBar } from './components/gallery/ImportProgressBar'
import i18n from './i18n'
import { save, open as openDialog } from '@tauri-apps/plugin-dialog'
import { autoConfigureInference } from './lib/api'

// Lazy-loaded page components (code splitting)
const GalleryPage = lazy(() => import('./pages/GalleryPage').then(m => ({ default: m.GalleryPage })))
const AIPage = lazy(() => import('./pages/AIPage').then(m => ({ default: m.AIPage })))
const DedupPage = lazy(() => import('./pages/DedupPage').then(m => ({ default: m.DedupPage })))
const DashboardPage = lazy(() => import('./pages/DashboardPage').then(m => ({ default: m.DashboardPage })))
const SettingsPage = lazy(() => import('./components/settings/SettingsPage').then(m => ({ default: m.SettingsPage })))
import {
  deleteImages,
  exportData,
  retrySingleAIResult,
  archiveImage,
  safeExport,
} from './lib/api'
import type { AppImage, Toast } from './types/image'
import { useStateRouter } from './router/state-router'
import { navigate } from './router/events'

function App() {
  const { current: currentPage } = useStateRouter('gallery')
  const { theme } = useConfigStore()
  const { applyTheme } = useThemeStore()
  const { loadConfigs, scanAiServices, setAiServiceReady } = useConfigStore()
  const { images, loading, error, searchQuery, loadImages, setSearchQuery } = useImageStore()
  const [toasts, setToasts] = useState<Toast[]>([])
  const [viewingImage, setViewingImage] = useState<AppImage | null>(null)
  const { t } = useTranslation()

  const addToast = useCallback((message: string, type: 'error' | 'success' | 'info' = 'info') => {
    const id = Date.now()
    setToasts(prev => [...prev, { id, message, type }])
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id))
    }, 4000)
  }, [])

  useEffect(() => {
    loadConfigs().then((configs) => {
      if (configs?.language) {
        i18n.changeLanguage(configs.language)
      }
    })
  }, [loadConfigs])

  // Startup: scan local AI services and auto-configure if models are loaded
  useEffect(() => {
    scanAiServices().then((models) => {
      const onlineModel = models.find(m => m.is_online)
      if (onlineModel) {
        // Auto-configure the first loaded model, or first online model as fallback
        autoConfigureInference().then((result) => {
          if (result) {
            setAiServiceReady(true)
          }
        })
      }
    })
  }, [scanAiServices, setAiServiceReady])

  useEffect(() => {
    loadImages()
  }, [loadImages])

  useEffect(() => {
    applyTheme(theme as Theme)
  }, [theme, applyTheme])

  const handleImageClick = useCallback((image: AppImage) => {
    setViewingImage(image)
  }, [])

  const handleViewerClose = useCallback(() => {
    setViewingImage(null)
  }, [])

  const handleViewerDelete = useCallback(async (id: number) => {
    try {
      await deleteImages([id])
      setViewingImage(null)
      await loadImages()
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    }
  }, [loadImages, addToast, t])

  const handleViewerExport = useCallback(async (id: number) => {
    try {
      const defaultName = `ArcaneCodex_Export_${id}.json`
      const outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (!outputPath) return

      const result = await exportData({
        format: 'json',
        output_path: outputPath,
        image_ids: [id],
      })
      addToast(t('gallery.exportSuccess', { count: result.exported_count }), 'success')
    } catch {
      addToast(t('errors.exportFailed'), 'error')
    }
  }, [addToast, t])

  const handleViewerReAnalyze = useCallback(async (id: number) => {
    try {
      await retrySingleAIResult(id)
      addToast(t('imageViewer.reAnalyzeStarted'), 'success')
    } catch {
      addToast(t('errors.reAnalyzeFailed'), 'error')
    }
  }, [addToast, t])

  const handleViewerArchive = useCallback(async (id: number) => {
    try {
      const result = await archiveImage(id)
      if (result.archived) {
        addToast(t('imageViewer.archiveSuccess', { path: result.dest_path }), 'success')
      }
    } catch (err) {
      addToast(`${t('errors.archiveFailed')}: ${err instanceof Error ? err.message : t('common.unknownError')}`, 'error')
    }
  }, [addToast, t])

  const handleViewerSafeExport = useCallback(async (id: number) => {
    try {
      const destDir = await openDialog({
        directory: true,
        title: t('imageViewer.safeExport'),
      })
      if (!destDir) return

      const result = await safeExport([id], destDir as string)
      if (result.exported_count > 0) {
        addToast(t('imageViewer.safeExportSuccess', { count: result.exported_count, dir: destDir }), 'success')
      } else {
        addToast(t('imageViewer.safeExportNoFiles'), 'info')
      }
    } catch (err) {
      addToast(`${t('errors.safeExportFailed')}: ${err instanceof Error ? err.message : t('common.unknownError')}`, 'error')
    }
  }, [addToast, t])

  const handleViewerTagClick = useCallback((tag: string) => {
    setViewingImage(null)
    setSearchQuery(tag)
    navigate({ route: 'gallery', source: 'action' })
  }, [setSearchQuery])

  const handleSearch = useCallback((query: string) => {
    setSearchQuery(query)
    navigate({ route: 'gallery', source: 'action' })
  }, [setSearchQuery])

  return (
    <ErrorBoundary>
      <div className="h-screen w-screen overflow-hidden bg-background text-foreground">
        <MainLayout>
          <Sidebar currentPage={currentPage} />
          <div className="flex flex-col flex-1">
            <TopBar onSearch={handleSearch} searchQuery={searchQuery} />
            <main className="flex-1 overflow-auto p-4">
              <Suspense fallback={
                <div className="flex items-center justify-center h-full">
                  <div className="flex flex-col items-center gap-3">
                    <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
                    <span className="text-sm text-muted-foreground">{t('common.loading')}</span>
                  </div>
                </div>
              }>
                {currentPage === 'gallery' && (
                  <GalleryPage
                    images={images}
                    loading={loading}
                    error={error}
                    onLoadImages={loadImages}
                    addToast={addToast}
                    onImageClick={handleImageClick}
                  />
                )}
                {currentPage === 'settings' && <SettingsPage />}
                {currentPage === 'ai' && <AIPage addToast={addToast} />}
                {currentPage === 'dedup' && (
                  <DedupPage addToast={addToast} onImagesChanged={loadImages} />
                )}
                {currentPage === 'dashboard' && <DashboardPage />}
              </Suspense>
            </main>
          </div>
        </MainLayout>
      </div>

      {viewingImage && (
        <ImageViewer
          image={{
            id: viewingImage.id,
            file_path: viewingImage.file_path,
            file_name: viewingImage.file_name,
            width: viewingImage.width,
            height: viewingImage.height,
            file_size: viewingImage.file_size,
            ai_tags: viewingImage.ai_tags
              ? (Array.isArray(viewingImage.ai_tags)
                  ? viewingImage.ai_tags
                  : typeof viewingImage.ai_tags === 'string'
                    ? JSON.parse(viewingImage.ai_tags)
                    : [])
              : undefined,
            ai_description: viewingImage.ai_description,
            ai_category: viewingImage.ai_category,
            exif_data: viewingImage.exif_data
              ? (typeof viewingImage.exif_data === 'string'
                  ? JSON.parse(viewingImage.exif_data)
                  : viewingImage.exif_data)
              : undefined,
          }}
          onClose={handleViewerClose}
          onDelete={handleViewerDelete}
          onExport={handleViewerExport}
          onReAnalyze={handleViewerReAnalyze}
          onArchive={handleViewerArchive}
          onSafeExport={handleViewerSafeExport}
          onTagClick={handleViewerTagClick}
        />
      )}

      <ImportProgressBar onComplete={() => loadImages()} />

      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
        {toasts.map(toast => (
          <div
            key={toast.id}
            className={`px-4 py-3 rounded-lg shadow-lg text-white text-sm max-w-xs animate-slide-in ${
              toast.type === 'error' ? 'bg-red-500' :
              toast.type === 'success' ? 'bg-green-500' :
              'bg-blue-500'
            }`}
          >
            {toast.message}
          </div>
        ))}
      </div>
    </ErrorBoundary>
  )
}

export default App
