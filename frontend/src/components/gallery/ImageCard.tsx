import { useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/utils/cn'
import { motion } from 'motion/react'
import { FileImage } from 'lucide-react'

interface ImageCardProps {
  id: number
  src: string
  fileName: string
  aiDescription?: string
  tags?: string[]
  aiStatus?: 'pending' | 'processing' | 'completed' | 'failed' | string
  isSelected?: boolean
  isSample?: boolean
  onClick?: (id: number) => void
  onToggleSelect?: (id: number) => void
}

function generatePlaceholderGradient(fileName: string, tags: string[]): string {
  const hash = fileName.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  const tagHash = tags.join('').split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)

  const hue1 = (hash * 137) % 360
  const hue2 = (tagHash * 173) % 360
  const hue3 = ((hash + tagHash) * 97) % 360

  return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%), hsl(${hue2}, 65%, 55%), hsl(${hue3}, 75%, 50%))`
}

function SamplePlaceholder({ fileName, tags }: { fileName: string; tags: string[] }) {
  const gradient = useMemo(() => generatePlaceholderGradient(fileName, tags), [fileName, tags])
  const initials = fileName
    .split(/[_\-.]/)
    .map(w => w[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()

  return (
    <div
      className="absolute inset-0 flex flex-col items-center justify-center"
      style={{ background: gradient }}
    >
      <div className="w-16 h-16 rounded-full bg-white/20 flex items-center justify-center mb-2 backdrop-blur-sm">
        <span className="text-2xl font-bold text-white">{initials}</span>
      </div>
      <p className="text-xs text-white/80 text-center px-2 font-medium">{fileName}</p>
      {tags.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-2 justify-center px-2">
          {tags.slice(0, 3).map((tag, i) => (
            <span key={i} className="text-[10px] px-1.5 py-0.5 rounded-full bg-white/20 text-white">
              {tag}
            </span>
          ))}
        </div>
      )}
    </div>
  )
}

export function ImageCard({
  id,
  src,
  fileName,
  aiDescription,
  tags = [],
  aiStatus = 'pending',
  isSelected = false,
  isSample = false,
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
        'group relative w-full h-full rounded-lg overflow-hidden',
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
      {!imageLoaded && !imageError && !isSample && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-8 h-8 border-2 border-gray-300 border-t-primary-500 rounded-full animate-spin" />
        </div>
      )}

      {/* Sample Placeholder */}
      {isSample && <SamplePlaceholder fileName={fileName} tags={tags} />}

      {/* Broken Link Display */}
      {imageError && !isSample && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-gray-200 dark:bg-dark-200">
          <FileImage className="w-10 h-10 text-gray-400 mb-2" />
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center px-2">
            {t('gallery.fileDeleted')}
          </p>
        </div>
      )}

      {!isSample && (
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
      )}
      
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
