import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { navigate } from '@/router/events'
import { HardDrive, BarChart3, Tag, Sparkles, RefreshCw, AlertCircle, CheckCircle, Clock, ImagePlus, Settings, ArrowRight } from 'lucide-react'
import { getLibraryStats, getAccuracyTrend } from '@/lib/api'
import type { LibraryStats, AccuracyTrend } from '@/lib/api'
import { AccuracyChart } from '@/components/dashboard/AccuracyChart'

export function DashboardPage() {
  const { t } = useTranslation()
  const [stats, setStats] = useState<LibraryStats | null>(null)
  const [trendData, setTrendData] = useState<AccuracyTrend | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadStats = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [statsData, trendDataResult] = await Promise.all([
        getLibraryStats(),
        getAccuracyTrend(30),
      ])
      setStats(statsData)
      setTrendData(trendDataResult)
    } catch {
      setError(t('dashboard.loadFailed'))
    } finally {
      setLoading(false)
    }
  }, [t])

  useEffect(() => {
    loadStats()
  }, [loadStats])

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
  }

  const maxCategoryCount = Math.max(1, ...(stats?.category_distribution?.map(([, c]) => c) ?? []))
  const maxTagCount = Math.max(1, ...(stats?.tag_cloud?.map(([, c]) => c) ?? []))

  if (loading && !stats) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <RefreshCw className="w-8 h-8 animate-spin text-primary-500" />
          <span className="text-gray-500">{t('dashboard.loading')}</span>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4 text-red-500">
          <AlertCircle className="w-8 h-8" />
          <span>{error}</span>
          <button
            onClick={loadStats}
            className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
          >
            {t('dashboard.refresh')}
          </button>
        </div>
      </div>
    )
  }

  if (!stats) return null

  const isEmpty = stats.total_images === 0

  if (isEmpty) {
    return (
      <div className="p-6 space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            {t('dashboard.title')}
          </h1>
        </div>

        <div className="flex flex-col items-center justify-center py-16 gap-8">
          <div className="w-28 h-28 rounded-full bg-gradient-to-br from-primary-100 to-primary-50 dark:from-primary-900/30 dark:to-primary-900/10 flex items-center justify-center">
            <ImagePlus className="w-14 h-14 text-primary-400 dark:text-primary-500" />
          </div>

          <div className="text-center max-w-lg">
            <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-3">
              {t('dashboard.emptyTitle')}
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-8">
              {t('dashboard.emptyDescription')}
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 w-full max-w-3xl">
            <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-700 transition-all cursor-pointer group"
                 onClick={() => navigate({ route: 'gallery' })}>
              <div className="flex items-center gap-3 mb-3">
                <div className="w-10 h-10 rounded-lg bg-blue-50 dark:bg-blue-900/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                  <ImagePlus className="w-5 h-5 text-blue-500" />
                </div>
                <h3 className="font-semibold text-gray-800 dark:text-gray-200">
                  {t('dashboard.emptyCard1Title')}
                </h3>
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                {t('dashboard.emptyCard1Desc')}
              </p>
              <div className="flex items-center gap-1 text-xs text-primary-600 dark:text-primary-400 font-medium group-hover:gap-2 transition-all">
                {t('dashboard.getStarted')}
                <ArrowRight className="w-3.5 h-3.5" />
              </div>
            </div>

            <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700 hover:border-purple-300 dark:hover:border-purple-700 transition-all cursor-pointer group"
                 onClick={() => navigate({ route: 'settings' })}>
              <div className="flex items-center gap-3 mb-3">
                <div className="w-10 h-10 rounded-lg bg-purple-50 dark:bg-purple-900/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                  <Settings className="w-5 h-5 text-purple-500" />
                </div>
                <h3 className="font-semibold text-gray-800 dark:text-gray-200">
                  {t('dashboard.emptyCard2Title')}
                </h3>
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                {t('dashboard.emptyCard2Desc')}
              </p>
              <div className="flex items-center gap-1 text-xs text-primary-600 dark:text-primary-400 font-medium group-hover:gap-2 transition-all">
                {t('dashboard.configureNow')}
                <ArrowRight className="w-3.5 h-3.5" />
              </div>
            </div>

            <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700 hover:border-green-300 dark:hover:border-green-700 transition-all cursor-pointer group"
                 onClick={() => navigate({ route: 'ai' })}>
              <div className="flex items-center gap-3 mb-3">
                <div className="w-10 h-10 rounded-lg bg-green-50 dark:bg-green-900/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                  <Sparkles className="w-5 h-5 text-green-500" />
                </div>
                <h3 className="font-semibold text-gray-800 dark:text-gray-200">
                  {t('dashboard.emptyCard3Title')}
                </h3>
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                {t('dashboard.emptyCard3Desc')}
              </p>
              <div className="flex items-center gap-1 text-xs text-primary-600 dark:text-primary-400 font-medium group-hover:gap-2 transition-all">
                {t('dashboard.learnMore')}
                <ArrowRight className="w-3.5 h-3.5" />
              </div>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          {t('dashboard.title')}
        </h1>
        <button
          onClick={loadStats}
          className="flex items-center gap-2 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          {t('dashboard.refresh')}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* 图片总数 */}
        <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
          <div className="flex items-center gap-3 mb-4">
            <BarChart3 className="w-6 h-6 text-primary-500" />
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              {t('dashboard.totalImages')}
            </h2>
          </div>
          <p className="text-4xl font-bold text-gray-900 dark:text-white">
            {stats.total_images.toLocaleString()}
          </p>
        </div>

        {/* 存储使用情况 */}
        <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
          <div className="flex items-center gap-3 mb-4">
            <HardDrive className="w-6 h-6 text-blue-500" />
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              {t('dashboard.storageUsage')}
            </h2>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">{t('dashboard.totalSize')}</span>
              <span className="font-medium">{formatBytes(stats.storage_usage.total_size_bytes)}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">{t('dashboard.averageSize')}</span>
              <span className="font-medium">{formatBytes(stats.storage_usage.average_image_size)}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">{t('dashboard.largestSize')}</span>
              <span className="font-medium">{formatBytes(stats.storage_usage.largest_image_size)}</span>
            </div>
          </div>
        </div>

        {/* AI 打标进度 */}
        <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
          <div className="flex items-center gap-3 mb-4">
            <Sparkles className="w-6 h-6 text-purple-500" />
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              {t('dashboard.aiProgress')}
            </h2>
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2 text-gray-500">
                <Clock className="w-3 h-3" />
                {t('dashboard.pending')}
              </span>
              <span className="font-medium">{stats.ai_progress.pending}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2 text-gray-500">
                <RefreshCw className="w-3 h-3 animate-spin" />
                {t('dashboard.processing')}
              </span>
              <span className="font-medium">{stats.ai_progress.processing}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2 text-green-500">
                <CheckCircle className="w-3 h-3" />
                {t('dashboard.completed')}
              </span>
              <span className="font-medium text-green-600">{stats.ai_progress.completed}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2 text-red-500">
                <AlertCircle className="w-3 h-3" />
                {t('dashboard.failed')}
              </span>
              <span className="font-medium text-red-600">{stats.ai_progress.failed}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-500">{t('dashboard.verified')}</span>
              <span className="font-medium">{stats.ai_progress.verified}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-500">{t('dashboard.provisional')}</span>
              <span className="font-medium">{stats.ai_progress.provisional}</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-500">{t('dashboard.rejected')}</span>
              <span className="font-medium">{stats.ai_progress.rejected}</span>
            </div>
          </div>
        </div>
      </div>

      {/* 分类分布 */}
      <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          {t('dashboard.categoryDistribution')}
        </h2>
        <div className="space-y-3">
          {stats.category_distribution.map(([category, count]) => (
            <div key={category} className="flex items-center gap-4">
              <span className="w-24 text-sm text-gray-600 dark:text-gray-300 truncate">
                {category || t('dashboard.uncategorized')}
              </span>
              <div className="flex-1 bg-gray-200 dark:bg-gray-700 rounded-full h-6 overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-primary-500 to-primary-400 rounded-full transition-all duration-500 flex items-center justify-end pr-2"
                  style={{ width: `${Math.max((count / maxCategoryCount) * 100, 8)}%` }}
                >
                  <span className="text-xs text-white font-medium">{count}</span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 标签词云 */}
      <div className="bg-white dark:bg-dark-100 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-3 mb-4">
          <Tag className="w-6 h-6 text-green-500" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            {t('dashboard.tagCloud')}
          </h2>
        </div>
        <div className="flex flex-wrap gap-3">
          {stats.tag_cloud.map(([tag, count]) => {
            const size = 0.75 + (count / maxTagCount) * 1.25
            const opacity = 0.5 + (count / maxTagCount) * 0.5
            return (
              <span
                key={tag}
                className="inline-block px-3 py-1 bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400 rounded-full transition-all hover:scale-105 cursor-default"
                style={{ fontSize: `${size}rem`, opacity }}
                title={`${tag}: ${count}`}
              >
                {tag}
              </span>
            )
          })}
        </div>
      </div>

      {/* AI 准确率趋势图 */}
      <AccuracyChart initialData={trendData} />
    </div>
  )
}
