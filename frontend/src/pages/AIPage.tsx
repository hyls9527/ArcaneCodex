import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { Brain, ImagePlus, ArrowRight } from 'lucide-react'
import { AIProgressPanel } from '../components/ai/AIProgressPanel'
import { useAIStore } from '../stores/useAIStore'
import { useImageStore } from '../stores/useImageStore'
import {
  getAIStatus,
  startAIProcessing,
  pauseAIProcessing,
  resumeAIProcessing,
  retryFailedAI,
} from '../lib/api'
import { navigate } from '../router/events'

interface AIPageProps {
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
}

export function AIPage({ addToast }: AIPageProps) {
  const { t } = useTranslation()
  const { status, total, completed, failed, retrying, updateStatus } = useAIStore()
  const { images } = useImageStore()
  const [loading, setLoading] = useState(false)

  const loadStatus = useCallback(async () => {
    try {
      const s = await getAIStatus()
      updateStatus(s)
    } catch {
      addToast(t('errors.loadAIStatusFailed'), 'error')
    }
  }, [updateStatus, addToast, t])

  const handleStart = useCallback(async () => {
    setLoading(true)
    try {
      await startAIProcessing()
      await loadStatus()
      addToast(t('ai.processStarted'), 'success')
    } catch {
      addToast(t('errors.aiStartFailed'), 'error')
    } finally {
      setLoading(false)
    }
  }, [loadStatus, addToast, t])

  const handlePause = useCallback(async () => {
    try {
      await pauseAIProcessing()
      await loadStatus()
      addToast(t('ai.processPaused'), 'info')
    } catch {
      addToast(t('errors.aiPauseFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  const handleResume = useCallback(async () => {
    try {
      await resumeAIProcessing()
      await loadStatus()
      addToast(t('ai.processResumed'), 'info')
    } catch {
      addToast(t('errors.aiResumeFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  const handleRetry = useCallback(async () => {
    try {
      await retryFailedAI()
      await loadStatus()
      addToast(t('ai.retryQueued'), 'success')
    } catch {
      addToast(t('errors.aiRetryFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  const handleCancel = useCallback(async () => {
    try {
      await pauseAIProcessing()
      await loadStatus()
      addToast(t('ai.processPaused'), 'info')
    } catch {
      addToast(t('errors.aiPauseFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  return (
    <div className="max-w-2xl mx-auto">
      {total === 0 && images.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 gap-6">
          <div className="w-20 h-20 rounded-full bg-primary-50 dark:bg-primary-900/20 flex items-center justify-center">
            <Brain className="w-10 h-10 text-primary-400 dark:text-primary-500" />
          </div>
          <div className="text-center max-w-sm">
            <h3 className="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-2">
              {t('ai.emptyTitle')}
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
              {t('ai.emptyDescription')}
            </p>
            <button
              onClick={() => navigate({ route: 'gallery', source: 'action' })}
              className="inline-flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
            >
              <ImagePlus className="w-4 h-4" />
              {t('ai.goToGallery')}
              <ArrowRight className="w-4 h-4" />
            </button>
          </div>
        </div>
      ) : (
        <AIProgressPanel
          status={{ status, total, completed, failed, retrying }}
          isLoading={loading}
          onStart={handleStart}
          onPause={handlePause}
          onResume={handleResume}
          onCancel={handleCancel}
          onRetry={handleRetry}
        />
      )}
    </div>
  )
}
