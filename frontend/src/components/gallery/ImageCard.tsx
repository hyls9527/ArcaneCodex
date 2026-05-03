import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/utils/cn'
import { motion } from 'framer-motion'
import { FileImage } from 'lucide-react'

interface ImageCardProps {
  id: number
  src: string
  fileName: string
  aiDescription?: string
  tags?: string[]
  aiStatus?: 'pending' | 'processing' | 'completed' | 'failed' | string
  isSelected?: boolean
  onClick?: (id: number) => void
  onToggleSelect?: (id: number) => void
}

export function ImageCard({
  id,
  src,
  fileName,
  aiDescription,
  tags = [],
  aiStatus = 'pending',
  isSelected = false,
  onClick,
  onToggleSelect,
}: ImageCardProps) {
  const { t } = useTranslation()
  const [isHovered, setIsHovered] = useState(false)
  const [imageLoaded, setImageLoaded] = useState(false)
  const [imageError, setImageError] = useState(false)
  
  const statusColors = {
    pending: 'bg-gray-400',
    processing: 'bg-blue-500 animate-pulse',
    completed: 'bg-green-500',
    failed: 'bg-red-500',
  }
  
  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      className={cn(
        'group relative aspect-square rounded-lg overflow-hidden',
        'bg-gray-100 dark:bg-dark-100',
        'cursor-pointer',
        'ring-2 ring-transparent',
        isSelected && 'ring-primary-500',
        'hover:shadow-lg transition-shadow',
        'focus-within:ring-2 focus-within:ring-primary-500'
      )}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onClick={() => onClick?.(id)}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick?.(id); } }}
      tabIndex={0}
      role="button"
      aria-label={t('imageCard.viewImage', { fileName })}
    >
      {/* Loading Spinner */}
      {!imageLoaded && !imageError && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-8 h-8 border-2 border-gray-300 border-t-primary-500 rounded-full animate-spin" />
        </div>
      )}
      
      {/* Broken Link Display */}
      {imageError && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-200 dark:bg-dark-200">
          <FileImage className="w-10 h-10 text-gray-400 mb-2" />
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center px-2">
            {t('gallery.fileDeleted')}
          </p>
        </div>
      )}
      
      <img
        src={src}
        alt={aiDescription || fileName}
        loading="lazy"
        onLoad={() => {
          setImageLoaded(true)
          setImageError(false)
        }}
        onError={() => {
          setImageError(true)
          setImageLoaded(false)
        }}
        className={cn(
          'w-full h-full object-cover transition-opacity duration-300',
          (!imageLoaded || imageError) && 'opacity-0'
        )}
      />
      
      {/* Hover Overlay */}
      {isHovered && !imageError && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="absolute inset-0 bg-gradient-to-t from-black/60 to-transparent"
        >
          <div className="absolute bottom-0 left-0 right-0 p-2">
            <p className="text-white text-xs truncate">{fileName}</p>
            
            {/* Tags */}
            {tags.length > 0 && (
              <div className="flex flex-wrap gap-1 mt-1">
                {tags.slice(0, 3).map((tag, i) => (
                  <span
                    key={i}
                    className="px-1.5 py-0.5 bg-white/20 rounded text-white text-[10px]"
                  >
                    {tag}
                  </span>
                ))}
                {tags.length > 3 && (
                  <span className="text-white/80 text-[10px]">+{tags.length - 3}</span>
                )}
              </div>
            )}
          </div>
          
          {/* Selection Checkbox */}
          <button
            onClick={(e) => {
              e.stopPropagation()
              onToggleSelect?.(id)
            }}
            className={cn(
              'absolute top-2 left-2 w-5 h-5 rounded border-2',
              'transition-colors',
              isSelected
                ? 'bg-primary-500 border-primary-500'
                : 'border-white/50 hover:border-white',
              'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-1 focus:ring-offset-black/60'
            )}
            aria-label={isSelected ? t('imageCard.deselect') : t('imageCard.select')}
          >
            {isSelected && (
              <svg className="w-full h-full text-white" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
              </svg>
            )}
          </button>
        </motion.div>
      )}
      
      {/* AI Status Indicator */}
      <div className={cn(
        'absolute top-2 right-2 w-2.5 h-2.5 rounded-full',
        statusColors[aiStatus]
      )} />
      
      {/* Selection Overlay */}
      {isSelected && (
        <div className="absolute inset-0 bg-primary-500/10 pointer-events-none" />
      )}
    </motion.div>
  )
}
