import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Play, Pause, RotateCcw, AlertCircle, CheckCircle, XCircle, Clock } from 'lucide-react'
import { cn } from '@/utils/cn'
import { motion, AnimatePresence } from 'motion/react'
import { listen } from '@tauri-apps/api/event'
import { type AIResult, getRecentAIResults, retrySingleAIResult } from '@/lib/api'

interface AITaskProgressEvent {
  image_id: number
  status: string
  message: string
  total: number
  current: number
}

interface AIStatus {
  status: 'idle' | 'processing' | 'paused' | 'completed' | 'failed'
  total: number
  completed: number
  failed: number
  retrying: number
  eta_seconds?: number
}

interface AIProgressPanelProps {
  status?: AIStatus
  isLoading?: boolean
  onStart?: () => void
  onPause?: () => void
  onResume?: () => void
  onCancel?: () => void
  onRetry?: () => void
}

export function AIProgressPanel({
  status = {
    status: 'idle',
    total: 0,
    completed: 0,
    failed: 0,
    retrying: 0,
  },
  onStart,
  onPause,
  onResume,
  onCancel,
  onRetry,
}: AIProgressPanelProps) {
  const { t } = useTranslation()
  const [showCancelConfirm, setShowCancelConfirm] = useState(false)
  const [results, setResults] = useState<AIResult[]>([])
  const [loadingResults, setLoadingResults] = useState(false)
  const [retryingId, setRetryingId] = useState<number | null>(null)
  const [liveStatus, setLiveStatus] = useState<AIStatus | null>(null)

  useEffect(() => {
    loadResults()
  }, [])

  // Listen for real-time AI progress events from backend
  useEffect(() => {
    const unlisten = listen<AITaskProgressEvent>('ai-progress', (event) => {
      const payload = event.payload
      setLiveStatus((prev) => ({
        status: payload.status as AIStatus['status'],
        total: payload.total,
        completed: payload.current,
        failed: payload.status === 'failed' ? (prev?.failed ?? 0) + 1 : prev?.failed ?? status.failed,
        retrying: payload.status === 'retrying' ? (prev?.retrying ?? 0) + 1 : payload.status === 'processing' ? Math.max(0, (prev?.retrying ?? 0) - 1) : prev?.retrying ?? status.retrying,
        eta_seconds: undefined,
      }))
      // Refresh results list when a task completes or fails
      if (payload.status === 'completed' || payload.status === 'failed') {
        loadResults()
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [status.failed, status.retrying])

  // Merge live status with prop status for display
  const displayStatus = liveStatus ?? status

  const loadResults = async () => {
    setLoadingResults(true)
    try {
      const data = await getRecentAIResults(50)
      setResults(data)
    } catch {
    } finally {
      setLoadingResults(false)
    }
  }

  const handleRetrySingle = async (imageId: number) => {
    setRetryingId(imageId)
    try {
      await retrySingleAIResult(imageId)
      await loadResults()
    } catch {
    } finally {
      setRetryingId(null)
    }
  }

  const parseTags = (tagsJson?: string): string[] => {
    if (!tagsJson) return []
    try {
      return JSON.parse(tagsJson)
    } catch {
      return []
    }
  }

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return ''
    try {
      const d = new Date(dateStr)
      return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
    } catch {
      return dateStr
    }
  }
  
  const progress = displayStatus.total > 0
    ? Math.round((displayStatus.completed / displayStatus.total) * 100)
    : 0

  const formatTime = (seconds?: number) => {
    if (!seconds) return t('ai.calculating')
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}${t('ai.minutes')} ${secs}${t('ai.seconds')}`
  }

  const statusText = {
    idle: t('ai.statusIdle'),
    processing: t('ai.statusProcessing'),
    paused: t('ai.statusPaused'),
    completed: t('ai.statusCompleted'),
    failed: t('ai.statusFailed'),
  }

  return (
    <div className="p-4 bg-white dark:bg-dark-100 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-lg font-semibold">{t('ai.progressTitle')}</h3>
        <span className={cn(
          'px-2 py-1 rounded text-xs',
          displayStatus.status === 'processing' && 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
          displayStatus.status === 'paused' && 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
          displayStatus.status === 'completed' && 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
          displayStatus.status === 'failed' && 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
        )}>
          {statusText[displayStatus.status]}
        </span>
      </div>

      {/* Progress Bar */}
      <div className="mb-4">
        <div className="h-2 bg-gray-200 dark:bg-dark-200 rounded-full overflow-hidden">
          <motion.div
            className={cn(
              'h-full transition-all duration-500',
              displayStatus.status === 'failed' ? 'bg-red-500' : 'bg-primary-500'
            )}
            initial={{ width: 0 }}
            animate={{ width: `${progress}%` }}
          />
        </div>
        <div className="flex justify-between mt-1 text-xs text-gray-500">
          <span>{progress}%</span>
          <span>{displayStatus.completed} / {displayStatus.total}</span>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-3 gap-3 mb-4">
        <div className="p-2 bg-green-50 dark:bg-green-900/20 rounded">
          <div className="text-2xl font-bold text-green-600">{displayStatus.completed}</div>
          <div className="text-xs text-gray-600">{t('ai.success')}</div>
        </div>

        <div className="p-2 bg-red-50 dark:bg-red-900/20 rounded">
          <div className="text-2xl font-bold text-red-600">{displayStatus.failed}</div>
          <div className="text-xs text-gray-600">{t('ai.failed')}</div>
        </div>

        <div className="p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded">
          <div className="text-2xl font-bold text-yellow-600">{displayStatus.retrying}</div>
          <div className="text-xs text-gray-600">{t('ai.retrying')}</div>
        </div>
      </div>

      {/* ETA */}
      {displayStatus.status === 'processing' && (
        <div className="mb-4 text-sm text-gray-600 dark:text-gray-400">
          {t('ai.eta')}: {formatTime(displayStatus.eta_seconds)}
        </div>
      )}

      {/* Controls */}
      <div className="flex gap-2">
        {displayStatus.status === 'idle' && (
          <button
            onClick={onStart}
            className="flex-1 btn-primary flex items-center justify-center gap-2"
          >
            <Play className="w-4 h-4" />
            {t('ai.startProcessing')}
          </button>
        )}
        
        {displayStatus.status === 'processing' && (
          <button
            onClick={onPause}
            className="flex-1 btn-secondary flex items-center justify-center gap-2"
          >
            <Pause className="w-4 h-4" />
            {t('ai.pause')}
          </button>
        )}

        {displayStatus.status === 'paused' && (
          <button
            onClick={onResume}
            className="flex-1 btn-primary flex items-center justify-center gap-2"
          >
            <RotateCcw className="w-4 h-4" />
            {t('ai.resume')}
          </button>
        )}

        {displayStatus.failed > 0 && (
          <button
            onClick={onRetry}
            className="btn-secondary flex items-center gap-2"
          >
            <AlertCircle className="w-4 h-4" />
            {t('ai.retryFailed')}
          </button>
        )}

        {displayStatus.status !== 'idle' && (
          <button
            onClick={() => setShowCancelConfirm(true)}
            className="btn-secondary text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
          >
            {t('ai.cancel')}
          </button>
        )}
      </div>
      
      {/* Cancel Confirmation */}
      {showCancelConfirm && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="mt-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded border border-yellow-200 dark:border-yellow-700"
        >
          <p className="text-sm text-yellow-800 dark:text-yellow-200 mb-2">
            {t('ai.cancelConfirm')}
          </p>
          <div className="flex gap-2">
            <button
              onClick={() => {
                onCancel?.()
                setShowCancelConfirm(false)
              }}
              className="btn-primary bg-red-600 hover:bg-red-700"
            >
              {t('ai.confirmCancel')}
            </button>
            <button
              onClick={() => setShowCancelConfirm(false)}
              className="btn-secondary"
            >
              {t('ai.back')}
            </button>
          </div>
        </motion.div>
      )}

      {/* Recent Results List */}
      <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <h4 className="text-sm font-medium mb-3">{t('ai.recentResults')}</h4>

        {loadingResults ? (
          <div className="flex items-center justify-center py-6">
            <Clock className="w-5 h-5 animate-spin text-gray-400" />
            <span className="ml-2 text-sm text-gray-500">{t('ai.loadingResults')}</span>
          </div>
        ) : results.length === 0 ? (
          <div className="text-center py-6 text-sm text-gray-500">
            {t('ai.noResults')}
          </div>
        ) : (
          <div className="space-y-2 max-h-80 overflow-y-auto">
            <AnimatePresence>
              {results.map((result) => {
                const isFailed = result.ai_status === 'failed'
                const tags = parseTags(result.ai_tags)

                return (
                  <motion.div
                    key={result.id}
                    initial={{ opacity: 0, y: -8 }}
                    animate={{ opacity: 1, y: 0 }}
                    className={cn(
                      'p-3 rounded border text-sm',
                      isFailed
                        ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
                        : 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
                    )}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          {isFailed ? (
                            <XCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
                          ) : (
                            <CheckCircle className="w-4 h-4 text-green-500 flex-shrink-0" />
                          )}
                          <span className="font-medium truncate">{result.file_name}</span>
                        </div>

                        {isFailed && result.ai_error_message && (
                          <p className="text-xs text-red-600 dark:text-red-400 mt-1 ml-6">
                            {result.ai_error_message}
                          </p>
                        )}

                        {!isFailed && result.ai_description && (
                          <p className="text-xs text-gray-600 dark:text-gray-400 mt-1 ml-6 line-clamp-2">
                            {result.ai_description}
                          </p>
                        )}

                        {!isFailed && tags.length > 0 && (
                          <div className="flex flex-wrap gap-1 mt-2 ml-6">
                            {tags.slice(0, 5).map((tag, idx) => (
                              <span
                                key={idx}
                                className="px-1.5 py-0.5 text-xs bg-white/60 dark:bg-dark-200 rounded"
                              >
                                {tag}
                              </span>
                            ))}
                          </div>
                        )}

                        {result.ai_processed_at && (
                          <div className="flex items-center gap-1 mt-1.5 ml-6 text-xs text-gray-400">
                            <Clock className="w-3 h-3" />
                            <span>{formatDate(result.ai_processed_at)}</span>
                          </div>
                        )}
                      </div>

                      {isFailed && (
                        <button
                          onClick={() => handleRetrySingle(result.id)}
                          disabled={retryingId === result.id}
                          className={cn(
                            'flex-shrink-0 p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors',
                            retryingId === result.id && 'opacity-50 cursor-not-allowed'
                          )}
                          title={t('ai.retry')}
                        >
                          <RotateCcw className={cn(
                            'w-4 h-4 text-red-500',
                            retryingId === result.id && 'animate-spin'
                          )} />
                        </button>
                      )}
                    </div>
                  </motion.div>
                )
              })}
            </AnimatePresence>
          </div>
        )}
      </div>
    </div>
  )
}
