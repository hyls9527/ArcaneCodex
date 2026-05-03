import { useState, useCallback, useRef, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { X, ZoomIn, ZoomOut, RotateCw, Download, Trash2, Camera, Clock, MapPin, Tag, Archive, RefreshCw, Info } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from '@/utils/cn'
import { NarrativePrompt } from './NarrativePrompt'
import { getNarratives, writeNarrative } from '@/lib/api'
import type { Narrative } from '@/lib/api'

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
  const [scale, setScale] = useState(1)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [isDragging, setIsDragging] = useState(false)
  const [showInfoPanel, setShowInfoPanel] = useState(false)
  const [narratives, setNarratives] = useState<Narrative[]>([])
  const dragStart = useRef({ x: 0, y: 0 })
  
  const formatFileSize = (bytes?: number): string => {
    if (!bytes) return ''
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }
  
  const parseExifData = (exifData?: Record<string, string | number | undefined>) => {
    if (!exifData || Object.keys(exifData).length === 0) return null
    try {
      if (typeof exifData === 'string') {
        return JSON.parse(exifData)
      }
      return exifData
    } catch {
      return null
    }
  }
  
  const exifParsed = parseExifData(image.exif_data)

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
  
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    setScale(prev => Math.max(0.1, Math.min(5, prev + delta)))
  }, [])
  
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (scale > 1) {
      setIsDragging(true)
      dragStart.current = { x: e.clientX - position.x, y: e.clientY - position.y }
    }
  }, [scale, position])
  
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isDragging) {
      setPosition({
        x: e.clientX - dragStart.current.x,
        y: e.clientY - dragStart.current.y,
      })
    }
  }, [isDragging])
  
  const handleMouseUp = useCallback(() => {
    setIsDragging(false)
  }, [])
  
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          onClose()
          break
        case 'ArrowLeft':
        case 'ArrowDown':
          setScale(prev => Math.max(0.1, prev - 0.1))
          break
        case 'ArrowUp':
        case 'ArrowRight':
          setScale(prev => Math.min(5, prev + 0.1))
          break
        case '0':
          setScale(1)
          setPosition({ x: 0, y: 0 })
          break
        case 'i':
          setShowInfoPanel(prev => !prev)
          break
      }
    }
    
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onClose])
  
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
        <div
          className="flex-1 flex items-center justify-center overflow-hidden"
          onWheel={handleWheel}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onClick={(e) => e.stopPropagation()}
        >
          <motion.img
            src={image.file_path}
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
        
        {/* Info Panel (Slide-in from right) */}
        <AnimatePresence>
          {showInfoPanel && (
            <motion.div
              initial={{ x: '100%' }}
              animate={{ x: 0 }}
              exit={{ x: '100%' }}
              transition={{ type: 'spring', damping: 25, stiffness: 200 }}
              className="absolute right-0 top-0 bottom-0 w-96 bg-gray-900/95 backdrop-blur-sm text-white overflow-y-auto z-40 p-6"
              onClick={(e) => e.stopPropagation()}
            >
              <h3 className="text-lg font-semibold mb-4">{t('imageViewer.imageInfo')}</h3>
              
              {/* File Info */}
              <div className="mb-4">
                <h4 className="text-sm font-medium text-gray-400 mb-2">{t('imageViewer.fileInfo')}</h4>
                <div className="space-y-1 text-sm">
                  <p>{image.file_name}</p>
                  {image.width && image.height && (
                    <p>{image.width} x {image.height}</p>
                  )}
                  {image.file_size && (
                    <p>{formatFileSize(image.file_size)}</p>
                  )}
                </div>
              </div>
              
              {/* AI Description */}
              {image.ai_description && (
                <div className="mb-4">
                  <h4 className="text-sm font-medium text-gray-400 mb-2">{t('imageViewer.aiDescription')}</h4>
                  <p className="text-sm text-gray-200">{image.ai_description}</p>
                </div>
              )}
              
              {/* AI Tags - Clickable */}
              {image.ai_tags && image.ai_tags.length > 0 && (
                <div className="mb-4">
                  <h4 className="text-sm font-medium text-gray-400 mb-2 flex items-center gap-1">
                    <Tag className="w-4 h-4" />
                    {t('imageViewer.aiTags')}
                  </h4>
                  <div className="flex flex-wrap gap-2">
                    {image.ai_tags.map((tag, i) => (
                      <button
                        key={i}
                        onClick={() => onTagClick?.(tag)}
                        className="px-2 py-1 bg-white/20 hover:bg-white/30 rounded-full text-xs cursor-pointer transition-colors"
                      >
                        {tag}
                      </button>
                    ))}
                  </div>
                </div>
              )}
              
              {/* EXIF Metadata */}
              {exifParsed && Object.keys(exifParsed).length > 0 && (
                <div className="mb-4">
                  <h4 className="text-sm font-medium text-gray-400 mb-2 flex items-center gap-1">
                    <Camera className="w-4 h-4" />
                    {t('imageViewer.exifData')}
                  </h4>
                  <div className="space-y-1 text-xs">
                    {exifParsed.DateTimeOriginal && (
                      <p className="flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        {exifParsed.DateTimeOriginal}
                      </p>
                    )}
                    {exifParsed.Make && exifParsed.Model && (
                      <p>{exifParsed.Make} {exifParsed.Model}</p>
                    )}
                    {exifParsed.GPSLatitude && exifParsed.GPSLongitude && (
                      <p className="flex items-center gap-1">
                        <MapPin className="w-3 h-3" />
                        {exifParsed.GPSLatitude}, {exifParsed.GPSLongitude}
                      </p>
                    )}
                    {Object.entries(exifParsed)
                      .filter(([key]) => !['DateTimeOriginal', 'Make', 'Model', 'GPSLatitude', 'GPSLongitude'].includes(key))
                      .map(([key, value]) => (
                        <p key={key} className="text-gray-400">{key}: {String(value)}</p>
                      ))}
                  </div>
                </div>
              )}
              
              {/* Actions */}
              <div className="border-t border-white/10 pt-4 mt-4">
                <h4 className="text-sm font-medium text-gray-400 mb-2">{t('imageViewer.actions')}</h4>
                <div className="space-y-2">
                  {onReAnalyze && (
                    <button
                      onClick={() => onReAnalyze(image.id)}
                      className="w-full flex items-center gap-2 px-3 py-2 bg-blue-500/20 hover:bg-blue-500/30 rounded-lg text-sm transition-colors"
                    >
                      <RefreshCw className="w-4 h-4" />
                      {t('imageViewer.reAnalyze')}
                    </button>
                  )}
                  {onArchive && (
                    <button
                      onClick={() => onArchive(image.id)}
                      className="w-full flex items-center gap-2 px-3 py-2 bg-green-500/20 hover:bg-green-500/30 rounded-lg text-sm transition-colors"
                    >
                      <Archive className="w-4 h-4" />
                      {t('imageViewer.archive')}
                    </button>
                  )}
                  {onSafeExport && (
                    <button
                      onClick={() => onSafeExport(image.id)}
                      className="w-full flex items-center gap-2 px-3 py-2 bg-purple-500/20 hover:bg-purple-500/30 rounded-lg text-sm transition-colors"
                    >
                      <Download className="w-4 h-4" />
                      {t('imageViewer.safeExport')}
                    </button>
                  )}
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
        
        {/* Bottom Info Bar */}
        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-6">
          <div className="max-w-4xl mx-auto text-white">
            <h2 className="text-xl font-semibold mb-2">{image.file_name}</h2>
            
            {/* AI Description */}
            {image.ai_description && (
              <p className="text-gray-200 mb-3">{image.ai_description}</p>
            )}
            
            {/* AI Tags */}
            {image.ai_tags && image.ai_tags.length > 0 && (
              <div className="flex flex-wrap gap-2 mb-3">
                {image.ai_tags.map((tag, i) => (
                  <span
                    key={i}
                    className="px-3 py-1 bg-white/20 rounded-full text-sm"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            )}
            
            {/* Metadata */}
            <div className="flex items-center gap-4 text-sm text-gray-300">
              {image.width && image.height && (
                <span>{image.width} x {image.height}</span>
              )}
              {image.ai_category && (
                <span>{t('imageViewer.category')}: {image.ai_category}</span>
              )}
            </div>

            <NarrativePrompt
              imageId={image.id}
              narratives={narratives}
              onWriteNarrative={handleWriteNarrative}
            />
          </div>
        </div>
        
        {/* Toolbar */}
        <div className="absolute top-4 left-4 flex items-center gap-2">
          <button
            onClick={() => setScale(prev => Math.max(0.1, prev - 0.2))}
            className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
            aria-label={t('imageViewer.zoomOut')}
          >
            <ZoomOut className="w-5 h-5" />
          </button>
          
          <span className="text-white px-2">{Math.round(scale * 100)}%</span>
          
          <button
            onClick={() => setScale(prev => Math.min(5, prev + 0.2))}
            className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
            aria-label={t('imageViewer.zoomIn')}
          >
            <ZoomIn className="w-5 h-5" />
          </button>
          
          <button
            onClick={() => {
              setScale(1)
              setPosition({ x: 0, y: 0 })
            }}
            className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
            aria-label={t('imageViewer.resetZoom')}
          >
            <RotateCw className="w-5 h-5" />
          </button>
          
          <button
            onClick={() => setShowInfoPanel(prev => !prev)}
            className={cn(
              "p-2 rounded-full focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80 transition-colors",
              showInfoPanel ? "bg-blue-500/40 text-blue-300" : "bg-white/10 hover:bg-white/20 text-white"
            )}
            aria-label={t('imageViewer.toggleInfo')}
          >
            <Info className="w-5 h-5" />
          </button>
          
          <div className="w-px h-6 bg-white/20 mx-2" />
          
          {onExport && (
            <button
              onClick={() => onExport(image.id)}
              className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
              aria-label={t('imageViewer.export')}
            >
              <Download className="w-5 h-5" />
            </button>
          )}
          
          {onDelete && (
            <button
              onClick={() => onDelete(image.id)}
              className="p-2 rounded-full bg-red-500/20 hover:bg-red-500/40 text-red-400 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 focus:ring-offset-black/80"
              aria-label={t('imageViewer.delete')}
            >
              <Trash2 className="w-5 h-5" />
            </button>
          )}
        </div>
      </motion.div>
    </AnimatePresence>
  )
}
