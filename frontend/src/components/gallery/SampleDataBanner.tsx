import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { Sparkles, X, Trash2, Loader2 } from 'lucide-react'
import { checkSampleData, clearSampleData } from '@/lib/api'

interface SampleDataBannerProps {
  onCleared: () => void
}

export function SampleDataBanner({ onCleared }: SampleDataBannerProps) {
  const { t } = useTranslation()
  const [visible, setVisible] = useState(false)
  const [clearing, setClearing] = useState(false)

  useEffect(() => {
    checkSampleData()
      .then(status => {
        if (status.has_sample_data) {
          setVisible(true)
        }
      })
      .catch(() => {})
  }, [])

  const handleClear = useCallback(async () => {
    setClearing(true)
    try {
      await clearSampleData()
      setVisible(false)
      onCleared()
    } catch {
      setClearing(false)
      return
    } finally {
      setClearing(false)
    }
  }, [onCleared, t])

  if (!visible) return null

  return (
    <div className="flex items-center gap-3 px-4 py-2.5 mb-4 rounded-lg bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800">
      <Sparkles className="w-4 h-4 text-primary-500 flex-shrink-0" />
      <span className="text-sm text-primary-700 dark:text-primary-300 flex-1">
        {t('sampleData.banner')}
      </span>
      <button
        onClick={handleClear}
        disabled={clearing}
        className="flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md bg-primary-600 hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed text-white transition-colors"
      >
        {clearing ? (
          <Loader2 className="w-3.5 h-3.5 animate-spin" />
        ) : (
          <Trash2 className="w-3.5 h-3.5" />
        )}
        {t('sampleData.clearButton')}
      </button>
      <button
        onClick={() => setVisible(false)}
        className="p-1 rounded hover:bg-primary-100 dark:hover:bg-primary-800/50 transition-colors"
      >
        <X className="w-3.5 h-3.5 text-primary-500" />
      </button>
    </div>
  )
}
