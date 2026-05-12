import { useTranslation } from 'react-i18next'
import { Camera, Clock, MapPin, Tag, Archive, RefreshCw, Download } from 'lucide-react'
import { motion, AnimatePresence } from 'motion/react'
import { formatFileSize, parseExifData } from './utils'

interface ImageInfoPanelProps {
  image: {
    id: number
    file_name: string
    width?: number
    height?: number
    file_size?: number
    ai_tags?: string[]
    ai_description?: string
    ai_category?: string
    exif_data?: Record<string, string | number | undefined>
  }
  show: boolean
  onTagClick?: (tag: string) => void
  onReAnalyze?: (id: number) => void
  onArchive?: (id: number) => void
  onSafeExport?: (id: number) => void
}

export function ImageInfoPanel({
  image,
  show,
  onTagClick,
  onReAnalyze,
  onArchive,
  onSafeExport,
}: ImageInfoPanelProps) {
  const { t } = useTranslation()
  const exifParsed = parseExifData(image.exif_data)

  return (
    <AnimatePresence>
      {show && (
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
  )
}
