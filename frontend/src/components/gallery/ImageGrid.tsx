import { useVirtualizer } from '@tanstack/react-virtual'
import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ImageCard } from './ImageCard'
import { cn } from '@/utils/cn'
import type { AppImage } from '@/types/image'

function getColumnCount(width: number): number {
  if (width >= 1280) return 5 // xl
  if (width >= 1024) return 4 // lg
  if (width >= 768) return 3 // md
  return 2 // sm
}

interface ImageGridProps {
  images: AppImage[]
  selectedIds?: number[]
  onImageClick?: (id: number) => void
  onToggleSelect?: (id: number) => void
  className?: string
  columnCount?: number
  gapSize?: number
}

export function ImageGrid({
  images,
  selectedIds = [],
  onImageClick,
  onToggleSelect,
  className,
}: ImageGridProps) {
  const { t } = useTranslation()
  const parentRef = useRef<HTMLDivElement>(null)
  const [columnCount, setColumnCount] = useState(5)
  
  const gapSize = 16 // gap-4 = 16px
  const cardHeight = 250
  const rowHeight = cardHeight + gapSize
  
  const rowCount = Math.ceil(images.length / columnCount)
  
  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => rowHeight,
    overscan: 5,
  })
  
  const handleResize = useCallback((entries: ResizeObserverEntry[]) => {
    for (const entry of entries) {
      const width = entry.contentRect.width
      setColumnCount(getColumnCount(width))
    }
  }, [])
  
  useEffect(() => {
    if (!parentRef.current) return
    
    const observer = new ResizeObserver(handleResize)
    observer.observe(parentRef.current)
    
    // Initial calculation
    const width = parentRef.current.clientWidth
    setColumnCount(getColumnCount(width))
    
    return () => observer.disconnect()
  }, [handleResize])
  
  if (images.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
        <p>{t('gallery.noImages')}</p>
      </div>
    )
  }
  
  return (
    <div
      ref={parentRef}
      className={cn('overflow-auto h-full scrollbar-thin', className)}
    >
      <div
        className="relative"
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const rowIndex = virtualRow.index
          const startIdx = rowIndex * columnCount
          const endIdx = Math.min(startIdx + columnCount, images.length)
          
          const cards: React.ReactNode[] = []
          for (let i = startIdx; i < endIdx; i++) {
            const image = images[i]
            if (!image) continue
            const colIndex = i - startIdx
            const left = colIndex * (100 / columnCount)
            const width = 100 / columnCount
            
            cards.push(
              <div
                key={image.id}
                className="absolute px-2"
                style={{
                  top: 0,
                  left: `${left}%`,
                  width: `${width}%`,
                  height: `${cardHeight}px`,
                }}
              >
                <ImageCard
                  id={image.id}
                  src={image.thumbnail_path || ''}
                  fileName={image.file_name}
                  tags={image.ai_tags
                    ? (typeof image.ai_tags === 'string'
                        ? JSON.parse(image.ai_tags)
                        : image.ai_tags)
                    : []}
                  aiStatus={image.ai_status}
                  isSelected={selectedIds.includes(image.id)}
                  onClick={onImageClick}
                  onToggleSelect={onToggleSelect}
                />
              </div>
            )
          }
          
          return (
            <div
              key={virtualRow.index}
              data-index={virtualRow.index}
              ref={virtualizer.measureElement}
              className="absolute left-0 right-0"
              style={{
                transform: `translateY(${virtualRow.start}px)`,
                height: `${rowHeight}px`,
              }}
            >
              {cards}
            </div>
          )
        })}
      </div>
    </div>
  )
}
