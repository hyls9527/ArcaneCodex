import { useTranslation } from 'react-i18next'
import { ZoomIn, ZoomOut, RotateCw, Download, Trash2, Info } from 'lucide-react'
import { cn } from '@/utils/cn'

interface ImageToolbarProps {
  scale: number
  showInfoPanel: boolean
  onZoomIn: (step?: number) => void
  onZoomOut: (step?: number) => void
  onReset: () => void
  onToggleInfoPanel: () => void
  onExport?: (id: number) => void
  onDelete?: (id: number) => void
  imageId: number
}

export function ImageToolbar({
  scale,
  showInfoPanel,
  onZoomIn,
  onZoomOut,
  onReset,
  onToggleInfoPanel,
  onExport,
  onDelete,
  imageId,
}: ImageToolbarProps) {
  const { t } = useTranslation()

  return (
    <div className="absolute top-4 left-4 flex items-center gap-2">
      <button
        onClick={() => onZoomOut()}
        className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
        aria-label={t('imageViewer.zoomOut')}
      >
        <ZoomOut className="w-5 h-5" />
      </button>

      <span className="text-white px-2">{Math.round(scale * 100)}%</span>

      <button
        onClick={() => onZoomIn()}
        className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
        aria-label={t('imageViewer.zoomIn')}
      >
        <ZoomIn className="w-5 h-5" />
      </button>

      <button
        onClick={onReset}
        className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
        aria-label={t('imageViewer.resetZoom')}
      >
        <RotateCw className="w-5 h-5" />
      </button>

      <button
        onClick={onToggleInfoPanel}
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
          onClick={() => onExport(imageId)}
          className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-black/80"
          aria-label={t('imageViewer.export')}
        >
          <Download className="w-5 h-5" />
        </button>
      )}

      {onDelete && (
        <button
          onClick={() => onDelete(imageId)}
          className="p-2 rounded-full bg-red-500/20 hover:bg-red-500/40 text-red-400 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 focus:ring-offset-black/80"
          aria-label={t('imageViewer.delete')}
        >
          <Trash2 className="w-5 h-5" />
        </button>
      )}
    </div>
  )
}
