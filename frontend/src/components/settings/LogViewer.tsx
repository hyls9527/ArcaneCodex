import { useEffect, useState, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { FileText, Download, Trash2, RefreshCw, ChevronDown, ChevronUp } from 'lucide-react'
import { getLogEntries, getLogStats, exportLogs, clearLogs } from '@/lib/api'
import type { LogEntry, LogFileStats } from '@/lib/api'

const levelColors: Record<string, string> = {
  INFO: 'text-blue-500 bg-blue-50 dark:bg-blue-900/20',
  WARN: 'text-yellow-500 bg-yellow-50 dark:bg-yellow-900/20',
  ERROR: 'text-red-500 bg-red-50 dark:bg-red-900/20',
  DEBUG: 'text-gray-500 bg-gray-50 dark:bg-gray-900/20',
  UNKNOWN: 'text-gray-400',
}

export function LogViewer() {
  const { t } = useTranslation()
  const [entries, setEntries] = useState<LogEntry[]>([])
  const [stats, setStats] = useState<LogFileStats | null>(null)
  const [loading, setLoading] = useState(false)
  const [levelFilter, setLevelFilter] = useState<string>('ALL')
  const [hasMore, setHasMore] = useState(false)
  const [expandedId, setExpandedId] = useState<number | null>(null)
  const logRef = useRef<HTMLDivElement>(null)
  const offsetRef = useRef(0)

  const loadLogs = useCallback(async (loadMore = false) => {
    setLoading(true)
    try {
      const filter = levelFilter === 'ALL' ? undefined : levelFilter
      const currentOffset = loadMore ? offsetRef.current : 0
      const result = await getLogEntries(200, currentOffset, filter)
      setEntries(prev => loadMore ? [...prev, ...result.entries] : result.entries)
      setHasMore(result.has_more)
      offsetRef.current = currentOffset + result.entries.length
    } catch {
      setEntries([])
      setHasMore(false)
    } finally {
      setLoading(false)
    }
  }, [levelFilter])

  const loadStats = async () => {
    try {
      const result = await getLogStats()
      setStats(result)
    } catch {
      setStats(null)
    }
  }

  useEffect(() => {
    offsetRef.current = 0
    loadLogs(false)
    loadStats()
  }, [levelFilter, loadLogs])

  const handleExport = async () => {
    const filter = levelFilter === 'ALL' ? undefined : levelFilter
    const path = `logs_export_${new Date().toISOString().slice(0, 10)}.log`
    try {
      await exportLogs(path, filter)
      alert(t('logs.exportSuccess'))
    } catch {
      alert(t('logs.exportFailed'))
    }
  }

  const handleClear = async () => {
    try {
      await clearLogs()
      offsetRef.current = 0
      loadLogs(false)
      loadStats()
      alert(t('logs.clearSuccess'))
    } catch {
      alert(t('logs.clearFailed'))
    }
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
  }

  const levels = ['ALL', 'INFO', 'WARN', 'ERROR', 'DEBUG']

  return (
    <div className="space-y-4">
      {/* 控制栏 */}
      <div className="flex flex-wrap items-center gap-3">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
          <FileText className="w-5 h-5 text-gray-500" />
          {t('logs.title')}
        </h2>

        <div className="flex gap-1 ml-auto">
          {levels.map(level => (
            <button
              key={level}
              onClick={() => setLevelFilter(level)}
              className={`px-3 py-1 text-xs rounded-lg transition-colors ${
                levelFilter === level
                  ? 'bg-primary-500 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
              }`}
            >
              {level}
            </button>
          ))}
        </div>

        <button
          onClick={() => { offsetRef.current = 0; loadLogs(false) }}
          disabled={loading}
          className="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </button>

        <button
          onClick={handleExport}
          className="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600"
        >
          <Download className="w-4 h-4" />
        </button>

        <button
          onClick={handleClear}
          className="p-2 rounded-lg bg-red-50 dark:bg-red-900/20 text-red-500 hover:bg-red-100 dark:hover:bg-red-900/40"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      {/* 统计信息 */}
      {stats && (
        <div className="flex gap-4 text-xs text-gray-500 dark:text-gray-400">
          <span>{t('logs.path')}: {stats.path}</span>
          <span>{t('logs.size')}: {formatBytes(stats.size_bytes)}</span>
          <span>{t('logs.totalLines')}: {stats.line_count.toLocaleString()}</span>
        </div>
      )}

      {/* 日志列表 */}
      <div
        ref={logRef}
        className="bg-gray-950 text-gray-100 rounded-xl overflow-hidden font-mono text-sm max-h-96 overflow-y-auto"
      >
        {entries.length === 0 ? (
          <div className="flex items-center justify-center h-48 text-gray-500">
            {t('logs.noLogs')}
          </div>
        ) : (
          <div className="divide-y divide-gray-800">
            {entries.map((entry, idx) => {
              const colorClass = levelColors[entry.level] || levelColors.UNKNOWN
              const isExpanded = expandedId === idx
              return (
                <div
                  key={idx}
                  className={`p-3 hover:bg-gray-900 cursor-pointer transition-colors ${isExpanded ? 'bg-gray-900' : ''}`}
                  onClick={() => setExpandedId(isExpanded ? null : idx)}
                >
                  <div className="flex items-start gap-3">
                    <span className={`px-2 py-0.5 rounded text-xs font-semibold ${colorClass}`}>
                      {entry.level}
                    </span>
                    <span className="text-gray-500 text-xs whitespace-nowrap">
                      {entry.timestamp}
                    </span>
                    <span className="text-gray-400 text-xs truncate flex-1">
                      {entry.message}
                    </span>
                    {entry.message.length > 100 && (
                      isExpanded ? <ChevronUp className="w-4 h-4 text-gray-500 flex-shrink-0" /> : <ChevronDown className="w-4 h-4 text-gray-500 flex-shrink-0" />
                    )}
                  </div>
                  {isExpanded && (
                    <div className="mt-2 pl-20 text-xs text-gray-400 space-y-1">
                      <div>{t('logs.target')}: {entry.target}</div>
                      <div className="whitespace-pre-wrap break-all">{entry.message}</div>
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* 加载更多 */}
      {hasMore && (
        <div className="flex justify-center">
          <button
            onClick={() => loadLogs(true)}
            disabled={loading}
            className="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50"
          >
            {loading ? t('logs.loading') : t('logs.loadMore')}
          </button>
        </div>
      )}
    </div>
  )
}
