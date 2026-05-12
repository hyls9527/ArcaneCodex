import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { X } from 'lucide-react'
import { motion, AnimatePresence } from 'motion/react'
import { cn } from '@/utils/cn'
import { toAssetUrl } from '@/utils/assetUrl'
import { getNarratives, writeNarrative } from '@/lib/api'
import type { Narrative } from '@/lib/api'
import { useImageZoom } from './useImageZoom'
import { ImageInfoPanel } from './ImageInfoPanel'
import { ImageBottomBar } from './ImageBottomBar'
import { ImageToolbar } from './ImageToolbar'
import { SampleViewerPlaceholder } from './SampleViewerPlaceholder'

interface ImageViewerProps {
  image: {
    id: number
    file_path: string
    file_name: string
    width?: number
    height?: number
    file_size?: number
    ai_tags?: string[]
    ai_description?: string
    ai_category?: string
    exif_data?: Record<string, string | number | undefined>
  }
  onClose: () => void
  onDelete?: (id: number) => void
  onExport?: (id: number) => void
  onArchive?: (id: number) => void
  onReAnalyze?: (id: number) => void
  onSafeExport?: (id: number) => void
  onTagClick?: (tag: string) => void
}

export function ImageViewer({
  image,
  onClose,
  onDelete,
  onExport,
  onArchive,
  onReAnalyze,
  onSafeExport,
  onTagClick,
}: ImageViewerProps) {
  const { t } = useTranslation()
  const [showInfoPanel, setShowInfoPanel] = useState(false)
  const [narratives, setNarratives] = useState<Narrative[]>([])

  const isSample = image.file_path?.includes('/sample/') || image.file_name?.includes('sample')

  const toggleInfoPanel = () => setShowInfoPanel(prev => !prev)

  const { scale, position, isDragging, handlers, zoomIn, zoomOut, reset } = useImageZoom({
    onClose,
    onToggleInfoPanel: toggleInfoPanel,
  })

  useEffect(() => {
    getNarratives(image.id).then(setNarratives).catch(() => setNarratives([]))
  }, [image.id])

  const handleWriteNarrative = async (imageId: number, content: string) => {
    try {
      const result = await writeNarrative(imageId, content)
      setNarratives(prev => [result, ...prev])
    } catch {
    }
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 bg-black/90 flex"
        onClick={onClose}
      >
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-2 rounded-full bg-white/10 hover:bg-white/20 text-white z-50 focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
          aria-label={t('imageViewer.close')}
        >
          <X className="w-6 h-6" />
        </button>

        {/* Image Container */}
        {isSample ? (
          <SampleViewerPlaceholder fileName={image.file_name} tags={image.ai_tags || []} />
        ) : (
          <div
            className="flex-1 flex items-center justify-center overflow-hidden"
            onWheel={handlers.onWheel}
            onMouseDown={handlers.onMouseDown}
            onMouseMove={handlers.onMouseMove}
            onMouseUp={handlers.onMouseUp}
            onMouseLeave={handlers.onMouseUp}
            onClick={(e) => e.stopPropagation()}
          >
            <motion.img
              src={toAssetUrl(image.file_path)}
              alt={image.ai_description || image.file_name}
              className={cn(
                'max-w-full max-h-full object-contain transition-transform',
                isDragging && 'cursor-grabbing'
              )}
              style={{
                transform: `translate(${position.x}px, ${position.y}px) scale(${scale})`,
              }}
            />
          </div>
        )}

        {/* Info Panel */}
        <ImageInfoPanel
          image={image}
          show={showInfoPanel}
          onTagClick={onTagClick}
          onReAnalyze={onReAnalyze}
          onArchive={onArchive}
          onSafeExport={onSafeExport}
        />

        {/* Bottom Info Bar */}
        <ImageBottomBar
          image={image}
          narratives={narratives}
          onWriteNarrative={handleWriteNarrative}
        />

        {/* Toolbar */}
        <ImageToolbar
          scale={scale}
          showInfoPanel={showInfoPanel}
          onZoomIn={zoomIn}
          onZoomOut={zoomOut}
          onReset={reset}
          onToggleInfoPanel={toggleInfoPanel}
          onExport={onExport}
          onDelete={onDelete}
          imageId={image.id}
        />
      </motion.div>
    </AnimatePresence>
  )
}
