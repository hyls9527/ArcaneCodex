import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { TrendingUp, BarChart3, Shield } from 'lucide-react'
import { getAccuracyTrend } from '@/lib/api'
import type { AccuracyTrend } from '@/lib/api'

interface AccuracyChartProps {
  initialData?: AccuracyTrend | null
}

export function AccuracyChart({ initialData }: AccuracyChartProps) {
  const { t } = useTranslation()
  const [data, setData] = useState<AccuracyTrend | null>(initialData ?? null)
  const [loading, setLoading] = useState(false)
  const [days, setDays] = useState(30)

  const loadData = async (d: number) => {
    setLoading(true)
    try {
      const result = await getAccuracyTrend(d)
      setData(result)
    } catch {
      setData(null)
    } finally {
      setLoading(false)
    }
  }

  const handleDaysChange = (d: number) => {
    setDays(d)
    loadData(d)
  }

  if (loading) {
    return (
      <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
        <div className="flex items-center justify-center h-48">
          <TrendingUp className="w-6 h-6 animate-spin text-primary-500" />
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* 控制栏 */}
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
          <TrendingUp className="w-5 h-5 text-purple-500" />
          {t('dashboard.aiAccuracyTrend')}
        </h2>
        <div className="flex gap-2">
          {[7, 14, 30, 90].map(d => (
            <button
              key={d}
              onClick={() => handleDaysChange(d)}
              className={`px-3 py-1 text-sm rounded-lg transition-colors ${
                days === d
                  ? 'bg-primary-500 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
              }`}
            >
              {t('dashboard.lastDays', { days: d })}
            </button>
          ))}
        </div>
      </div>

      {/* 趋势图 */}
      <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
        {data && data.daily_data.length > 0 ? (
          <div className="space-y-4">
            <div className="flex items-end gap-1 h-48">
              {data.daily_data.map((point, idx) => {
                const height = Math.max(point.accuracy * 100, 4)
                return (
                  <div
                    key={idx}
                    className="flex-1 flex flex-col items-center gap-1 group"
                  >
                    <div
                      className="w-full bg-gradient-to-t from-primary-500 to-primary-400 rounded-t transition-all hover:from-primary-600 hover:to-primary-500 relative"
                      style={{ height: `${height}%` }}
                    >
                      <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
                        {(point.accuracy * 100).toFixed(1)}% ({point.correct}/{point.total})
                      </div>
                    </div>
                    {data.daily_data.length <= 14 && (
                      <span className="text-xs text-gray-400 truncate w-full text-center">
                        {point.date.slice(5)}
                      </span>
                    )}
                  </div>
                )
              })}
            </div>
            <div className="flex justify-between text-xs text-gray-400">
              <span>{data.daily_data[0]?.date}</span>
              <span>{data.daily_data[data.daily_data.length - 1]?.date}</span>
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-center h-48 text-gray-400">
            {t('dashboard.noAccuracyData')}
          </div>
        )}
      </div>

      {/* 按分类准确率 */}
      {data && data.category_accuracy.length > 0 && (
        <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
          <h3 className="text-md font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <BarChart3 className="w-4 h-4 text-blue-500" />
            {t('dashboard.accuracyByCategory')}
          </h3>
          <div className="space-y-3">
            {data.category_accuracy.map(cat => {
              const accuracy = cat.total > 0 ? cat.verified / cat.total : 0
              const width = Math.max(accuracy * 100, 4)
              return (
                <div key={cat.category} className="space-y-1">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-300">{cat.category}</span>
                    <span className="font-medium">{(accuracy * 100).toFixed(1)}%</span>
                  </div>
                  <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5 overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-blue-500 to-blue-400 rounded-full transition-all duration-500"
                      style={{ width: `${width}%` }}
                    />
                  </div>
                  <div className="flex gap-4 text-xs text-gray-400">
                    <span>{t('dashboard.verified')}: {cat.verified}</span>
                    <span>{t('dashboard.provisional')}: {cat.provisional}</span>
                    <span>{t('dashboard.rejected')}: {cat.rejected}</span>
                    <span>{t('dashboard.confidence')}: {(cat.average_confidence * 100).toFixed(1)}%</span>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* 校准对比 */}
      {data?.calibration_comparison && (
        <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
          <h3 className="text-md font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <Shield className="w-4 h-4 text-green-500" />
            {t('dashboard.calibrationComparison')}
          </h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="text-center">
              <p className="text-sm text-gray-500">{t('dashboard.beforeCalibration')}</p>
              <p className="text-2xl font-bold text-red-500">
                {(data.calibration_comparison.before_ece * 100).toFixed(1)}%
              </p>
              <p className="text-xs text-gray-400">ECE</p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">{t('dashboard.afterCalibration')}</p>
              <p className="text-2xl font-bold text-green-500">
                {(data.calibration_comparison.after_ece * 100).toFixed(1)}%
              </p>
              <p className="text-xs text-gray-400">ECE</p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">{t('dashboard.improvement')}</p>
              <p className="text-2xl font-bold text-primary-500">
                +{data.calibration_comparison.improvement_percent.toFixed(1)}%
              </p>
              <p className="text-xs text-gray-400">{t('dashboard.accuracyImprovement')}</p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
