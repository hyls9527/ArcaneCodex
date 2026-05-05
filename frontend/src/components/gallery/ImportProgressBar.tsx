import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { CheckCircle, Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface ImportProgressPayload {
  current: number
  total: number
  current_file?: string
  status: 'processing' | 'success' | 'duplicate' | 'error'
}

interface ImportProgressProps {
  onComplete?: () => void
}

export function ImportProgressBar({ onComplete }: ImportProgressProps) {
  const { t } = useTranslation()
  const [progress, setProgress] = useState<ImportProgressPayload | null>(null)
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    let unlistenFn: (() => void) | null = null

    const setupListener = async () => {
      try {
        const unlisten = await listen<ImportProgressPayload>('import-progress', (event) => {
          const payload = event.payload
          const isCompleted = payload.status === 'success'
            || (payload.current > 0 && payload.current === payload.total && payload.status !== 'processing')

          setProgress({
            ...payload,
            status: isCompleted ? 'processing' : payload.status,
          })
          setVisible(true)

          if (isCompleted || (payload.current > 0 && payload.total > 0 && payload.current >= payload.total)) {
            setTimeout(() => {
              setVisible(false)
              setProgress(null)
              onComplete?.()
            }, 1500)
          }
        })
        unlistenFn = unlisten
      } catch {
        // Not in Tauri environment, ignore
      }
    }

    setupListener()

    return () => {
      unlistenFn?.()
    }
  }, [onComplete])

  if (!visible || !progress) return null

  const percent = progress.total > 0
    ? Math.round((progress.current / progress.total) * 100)
    : 0
  const isCompleted = progress.status === 'success'
    || (progress.current > 0 && progress.total > 0 && progress.current >= progress.total)

  return (
    <div className="fixed bottom-4 right-4 z-50 w-80 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg p-4 transition-all duration-300">
      <div className="flex items-center gap-2 mb-2">
        {isCompleted ? (
          <CheckCircle className="w-4 h-4 text-green-500 shrink-0" />
        ) : (
          <Loader2 className="w-4 h-4 text-primary-500 animate-spin shrink-0" />
        )}
        <span className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
          {isCompleted ? t('import.importComplete') : t('import.importingProgress', { current: progress.current, total: progress.total })}
        </span>
      </div>

      {!isCompleted && progress.current_file && (
        <p className="text-xs text-gray-500 dark:text-gray-400 mb-2 truncate">
          {progress.current_file}
        </p>
      )}

      <div className="w-full h-2 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-300 ${
            isCompleted
              ? 'bg-green-500'
              : 'bg-primary-500'
          }`}
          style={{ width: `${percent}%` }}
        />
      </div>

      <p className="text-xs text-gray-400 mt-1 text-right">
        {percent}%
      </p>
    </div>
  )
}
