import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Search, Trash2, Check, ArrowRight } from 'lucide-react'
import { cn } from '@/utils/cn'
import { motion, AnimatePresence } from 'framer-motion'
import type { DuplicateGroup } from '@/lib/api'

interface DedupManagerProps {
  groups?: DuplicateGroup[]
  onScan?: (threshold: number) => void
  onDelete?: (groupIds: string[]) => void
  isLoading?: boolean
}

export function DedupManager({
  groups = [],
  onScan,
  onDelete,
  isLoading = false,
}: DedupManagerProps) {
  const { t } = useTranslation()
  const [threshold, setThreshold] = useState(90)
  const [currentGroupIndex, setCurrentGroupIndex] = useState(0)
  const [selectedGroups, setSelectedGroups] = useState<Set<string>>(new Set())
  const [groupDecisions, setGroupDecisions] = useState<Record<string, number>>({})
  
  const currentGroup = groups[currentGroupIndex]
  
  // Auto-select the image with highest resolution as default recommendation
  const getRecommendedImageId = (group: DuplicateGroup): number => {
    if (group.images.length === 0) return 0
    
    return group.images.reduce((bestId, image) => {
      const bestImage = group.images.find(img => img.image_id === bestId)
      const bestPixels = (bestImage?.width ?? 0) * (bestImage?.height ?? 0)
      const currentPixels = (image.width ?? 0) * (image.height ?? 0)

      return currentPixels > bestPixels ? image.image_id : bestId
    }, group.images[0].image_id)
  }
  
  // Auto-select recommended on group change
  const handleGroupChange = (index: number) => {
    const group = groups[index]
    if (group && !groupDecisions[group.id]) {
      const recommendedId = getRecommendedImageId(group)
      setGroupDecisions(prev => ({
        ...prev,
        [group.id]: recommendedId,
      }))
    }
    setCurrentGroupIndex(index)
  }
  
  const handleKeepSelection = (groupId: string, imageId: number) => {
    setGroupDecisions(prev => ({
      ...prev,
      [groupId]: imageId,
    }))
  }
  
  const handleNext = () => {
    if (currentGroupIndex < groups.length - 1) {
      handleGroupChange(currentGroupIndex + 1)
    }
  }
  
  const handlePrevious = () => {
    if (currentGroupIndex > 0) {
      handleGroupChange(currentGroupIndex - 1)
    }
  }
  
  const handleToggleSelect = (groupId: string) => {
    setSelectedGroups(prev => {
      const next = new Set(prev)
      if (next.has(groupId)) {
        next.delete(groupId)
      } else {
        next.add(groupId)
      }
      return next
    })
  }
  
  const handleBatchDelete = () => {
    if (selectedGroups.size > 0) {
      onDelete?.(Array.from(selectedGroups))
      setSelectedGroups(new Set())
    }
  }
  
  if (groups.length === 0 && !isLoading) {
    return (
      <div className="p-8">
        <div className="max-w-md mx-auto text-center">
          <h2 className="text-2xl font-bold mb-4">{t('dedup.title')}</h2>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            {t('dedup.description')}
          </p>
          
          {/* Threshold Slider */}
          <div className="mb-6">
            <label className="block text-sm font-medium mb-2">
              {t('dedup.threshold')}: {threshold}%
            </label>
            <input
              type="range"
              min="70"
              max="99"
              value={threshold}
              onChange={(e) => setThreshold(Number(e.target.value))}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>{t('dedup.thresholdMin')}</span>
              <span>{t('dedup.thresholdMax')}</span>
            </div>
          </div>
          
          <button
            onClick={() => onScan?.(threshold)}
            className="btn-primary flex items-center gap-2 mx-auto"
          >
            <Search className="w-5 h-5" />
            {t('dedup.startScan')}
          </button>
        </div>
      </div>
    )
  }
  
  return (
    <div className="p-4">
      {/* Header Stats */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold">{t('dedup.results')}</h2>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-600 dark:text-gray-400">
            {t('dedup.foundGroups')} <span className="font-semibold text-red-600">{groups.length}</span> {t('dedup.groupCount')}
          </span>
          {selectedGroups.size > 0 && (
            <button
              onClick={handleBatchDelete}
              className="btn-primary bg-red-600 hover:bg-red-700 flex items-center gap-2"
            >
              <Trash2 className="w-4 h-4" />
              {t('dedup.deleteSelected')} ({selectedGroups.size})
            </button>
          )}
        </div>
      </div>
      
      <AnimatePresence mode="wait">
        {currentGroup && (
          <motion.div
            key={currentGroup.id}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            className="bg-white dark:bg-dark-100 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            {/* Group Header */}
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">
                  {t('dedup.groupProgress', { current: currentGroupIndex + 1, total: groups.length })}
                </span>
                <span className="px-2 py-0.5 bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400 rounded text-xs">
                  {t('dedup.similarity')}: {currentGroup.similarity}%
                </span>
              </div>
              
              <button
                onClick={() => handleToggleSelect(currentGroup.id)}
                className={cn(
                  'px-3 py-1 rounded text-sm border transition-colors',
                  selectedGroups.has(currentGroup.id)
                    ? 'bg-primary-50 border-primary-500 text-primary-600'
                    : 'border-gray-300 hover:border-gray-400',
                  'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100'
                )}
              >
                {selectedGroups.has(currentGroup.id) ? t('dedup.selected') : t('dedup.selectGroup')}
              </button>
            </div>
            
            {/* Comparison View */}
            <div className="grid grid-cols-2 gap-4 mb-4">
              {currentGroup.images.map((image) => {
                const isKeep = groupDecisions[currentGroup.id] === image.image_id
                const isRecommended = getRecommendedImageId(currentGroup) === image.image_id

                return (
                  <div
                    key={image.image_id}
                    onClick={() => handleKeepSelection(currentGroup.id, image.image_id)}
                    onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleKeepSelection(currentGroup.id, image.image_id); } }}
                    tabIndex={0}
                    role="button"
                    aria-label={t('dedup.selectKeep', { fileName: image.file_name })}
                    className={cn(
                      'relative rounded-lg overflow-hidden cursor-pointer transition-all',
                      'border-2',
                      isKeep
                        ? 'border-green-500 ring-2 ring-green-500/30'
                        : 'border-gray-200 dark:border-gray-700 hover:border-primary-400',
                      'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2'
                    )}
                  >
                    <img
                      src={image.file_path.startsWith('file:///') 
                        ? image.file_path.replace('file:///', '')
                        : image.file_path
                        || '/placeholder.png'}
                      alt={image.file_name}
                      className="w-full aspect-square object-cover"
                    />

                    {/* Keep Badge */}
                    {isKeep && (
                      <div className="absolute top-2 right-2 p-1 bg-green-500 rounded-full">
                        <Check className="w-4 h-4 text-white" />
                      </div>
                    )}

                    {/* Recommended Badge */}
                    {isRecommended && !isKeep && (
                      <div className="absolute top-2 left-2 px-2 py-1 bg-blue-500/90 text-white rounded text-xs font-medium">
                        {t('dedup.recommended')}
                      </div>
                    )}

                    {/* Image Info */}
                    <div className="p-2 bg-white dark:bg-dark-100">
                      <p className="text-xs truncate">{image.file_name}</p>
                      <div className="flex gap-2 text-xs text-gray-500 mt-1">
                        {image.width && image.height && (
                          <span>{image.width}x{image.height}</span>
                        )}
                        {image.file_size && (
                          <span>{(image.file_size / 1024).toFixed(1)} KB</span>
                        )}
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
            
            {/* Navigation */}
            <div className="flex items-center justify-between">
              <button
                onClick={handlePrevious}
                disabled={currentGroupIndex === 0}
                className="btn-secondary disabled:opacity-50"
              >
                {t('dedup.previousGroup')}
              </button>
              
              <div className="flex items-center gap-2">
                <button
                  onClick={() => {
                    if (groupDecisions[currentGroup.id]) {
                      handleNext()
                    }
                  }}
                  disabled={!groupDecisions[currentGroup.id]}
                  className="btn-primary disabled:opacity-50 flex items-center gap-2"
                >
                  {t('dedup.keepAndSkip')}
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
              
              <button
                onClick={handleNext}
                disabled={currentGroupIndex === groups.length - 1}
                className="btn-secondary disabled:opacity-50"
              >
                {t('dedup.nextGroup')}
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      
      {/* Loading State */}
      {isLoading && (
        <div className="flex items-center justify-center h-64">
          <div className="w-8 h-8 border-2 border-gray-300 border-t-primary-500 rounded-full animate-spin" />
          <span className="ml-3">{t('dedup.scanning')}</span>
        </div>
      )}
    </div>
  )
}
