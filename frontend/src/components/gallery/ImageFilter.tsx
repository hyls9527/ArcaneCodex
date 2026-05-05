import { useState, useEffect, useMemo, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { X, Filter, Calendar, Tag, Hash } from 'lucide-react'
import { useImageStore } from '@/stores/useImageStore'
import { cn } from '@/utils/cn'

export function ImageFilter() {
  const { t } = useTranslation()
  const { filters, setFilters, images } = useImageStore()
  const [isOpen, setIsOpen] = useState(false)
  const [localFilters, setLocalFilters] = useState(filters)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!isOpen) return
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [isOpen])

  // Sync store filters to local state
  useEffect(() => {
    setLocalFilters(filters)
  }, [filters])

  // Get unique categories from images
  const categories = useMemo(() => {
    const cats = new Set<string>()
    images.forEach(img => {
      if (img.ai_category) cats.add(img.ai_category)
    })
    return Array.from(cats).sort()
  }, [images])

  // Get unique tags from images
  const allTags = useMemo(() => {
    const tags = new Set<string>()
    images.forEach(img => {
      if (img.ai_tags) {
        if (Array.isArray(img.ai_tags)) {
          img.ai_tags.forEach(tag => tags.add(tag))
        } else if (typeof img.ai_tags === 'string') {
          try {
            const parsed = JSON.parse(img.ai_tags)
            if (Array.isArray(parsed)) {
              parsed.forEach(tag => tags.add(tag))
            }
          } catch {
            // If not JSON, treat as single tag
            tags.add(img.ai_tags as string)
          }
        }
      }
    })
    return Array.from(tags).sort()
  }, [images])

  const hasActiveFilters = filters.ai_status || filters.category || filters.date_from || filters.date_to || (filters.tags && filters.tags.length > 0)

  const handleApply = () => {
    setFilters(localFilters)
    setIsOpen(false)
  }

  const handleClearAll = () => {
    setFilters({})
    setLocalFilters({})
    setIsOpen(false)
  }

  const handleTagToggle = (tag: string) => {
    const currentTags = localFilters.tags || []
    const newTags = currentTags.includes(tag)
      ? currentTags.filter(t => t !== tag)
      : [...currentTags, tag]
    setLocalFilters({ ...localFilters, tags: newTags })
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          'flex items-center gap-2 px-3 py-2 rounded-lg border transition-colors text-sm',
          hasActiveFilters
            ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
            : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800'
        )}
        aria-label={t('filter.title')}
      >
        <Filter className="w-4 h-4" />
        {t('filter.title')}
        {hasActiveFilters && (
          <span className="w-2 h-2 bg-primary-500 rounded-full" />
        )}
      </button>

      {isOpen && (
        <div className="absolute left-0 top-full mt-2 w-72 bg-white dark:bg-gray-900 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-40 p-4 max-h-[80vh] overflow-y-auto">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold">{t('filter.title')}</h3>
            <div className="flex gap-2">
              {hasActiveFilters && (
                <button
                  onClick={handleClearAll}
                  className="text-xs text-red-500 hover:text-red-600"
                >
                  {t('filter.clearAll')}
                </button>
              )}
              <button onClick={() => setIsOpen(false)} aria-label={t('common.close')}>
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* AI Status Filter */}
          <div className="mb-4">
            <label className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-1">
              <Hash className="w-3 h-3" />
              {t('filter.status')}
            </label>
            <select
              value={localFilters.ai_status || ''}
              onChange={(e) => setLocalFilters({ ...localFilters, ai_status: e.target.value || undefined })}
              className="w-full px-3 py-2 text-sm border border-gray-200 dark:border-gray-700 rounded-lg bg-transparent"
            >
              <option value="">{t('filter.allStatus')}</option>
              <option value="pending">{t('filter.pending')}</option>
              <option value="processing">{t('filter.processing')}</option>
              <option value="completed">{t('filter.completed')}</option>
              <option value="failed">{t('filter.failed')}</option>
            </select>
          </div>

          {/* Category Filter */}
          {categories.length > 0 && (
            <div className="mb-4">
              <label className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-1">
                <Tag className="w-3 h-3" />
                {t('filter.category')}
              </label>
              <select
                value={localFilters.category || ''}
                onChange={(e) => setLocalFilters({ ...localFilters, category: e.target.value || undefined })}
                className="w-full px-3 py-2 text-sm border border-gray-200 dark:border-gray-700 rounded-lg bg-transparent"
              >
                <option value="">{t('filter.allCategories')}</option>
                {categories.map(cat => (
                  <option key={cat} value={cat}>{cat}</option>
                ))}
              </select>
            </div>
          )}

          {/* Date Range Filter */}
          <div className="mb-4">
            <label className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-1">
              <Calendar className="w-3 h-3" />
              {t('filter.dateRange')}
            </label>
            <div className="grid grid-cols-2 gap-2">
              <input
                type="date"
                value={localFilters.date_from || ''}
                onChange={(e) => setLocalFilters({ ...localFilters, date_from: e.target.value || undefined })}
                className="px-2 py-1.5 text-sm border border-gray-200 dark:border-gray-700 rounded-lg bg-transparent"
                placeholder={t('filter.from')}
              />
              <input
                type="date"
                value={localFilters.date_to || ''}
                onChange={(e) => setLocalFilters({ ...localFilters, date_to: e.target.value || undefined })}
                className="px-2 py-1.5 text-sm border border-gray-200 dark:border-gray-700 rounded-lg bg-transparent"
                placeholder={t('filter.to')}
              />
            </div>
          </div>

          {/* Tag Filter */}
          {allTags.length > 0 && (
            <div className="mb-4">
              <label className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-1">
                <Tag className="w-3 h-3" />
                {t('filter.tags')}
              </label>
              <div className="flex flex-wrap gap-1 max-h-24 overflow-y-auto">
                {allTags.map(tag => (
                  <button
                    key={tag}
                    onClick={() => handleTagToggle(tag)}
                    className={cn(
                      'px-2 py-1 text-xs rounded-full border transition-colors',
                      localFilters.tags?.includes(tag)
                        ? 'bg-primary-100 dark:bg-primary-900/30 border-primary-300 dark:border-primary-700 text-primary-600 dark:text-primary-400'
                        : 'border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800'
                    )}
                  >
                    {tag}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Apply Button */}
          <button
            onClick={handleApply}
            className="w-full btn-primary text-sm"
          >
            {t('filter.apply')}
          </button>
        </div>
      )}
    </div>
  )
}
