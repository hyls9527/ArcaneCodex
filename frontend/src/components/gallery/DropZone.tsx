import { useState, useCallback, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Upload, ImageOff } from 'lucide-react'
import { cn } from '@/utils/cn'

interface DropZoneProps {
  onFilesSelected: (paths: string[]) => void
  className?: string
  maxSize?: number
}

const SUPPORTED_EXTENSIONS = new Set([
  '.jpeg', '.jpg', '.png', '.webp', '.gif', '.bmp', '.tiff', '.tif', '.avif'
])

function isSupportedImage(path: string): boolean {
  const ext = '.' + path.split('.').pop()?.toLowerCase()
  return SUPPORTED_EXTENSIONS.has(ext)
}

function isTauriAvailable(): boolean {
  return typeof window !== 'undefined' && !!(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
}

export function DropZone({ 
  onFilesSelected, 
  className,
  maxSize: _maxSize = 50 * 1024 * 1024
}: DropZoneProps) {
  const { t } = useTranslation()
  const [isDragging, setIsDragging] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const dropZoneRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!isTauriAvailable()) return

    let cancelled = false
    import('@tauri-apps/api/webview').then(({ getCurrentWebview }) => {
      if (cancelled) return
      const unlisten = getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === 'over') {
          setIsDragging(true)
        } else if (event.payload.type === 'leave') {
          setIsDragging(false)
        } else if (event.payload.type === 'drop') {
          setIsDragging(false)
          const paths = event.payload.paths.filter(isSupportedImage)
          if (paths.length === 0) {
            setError(t('gallery.noSupportedFiles', '没有找到支持的图片格式文件'))
            return
          }
          setError(null)
          onFilesSelected(paths)
        }
      })

      return () => {
        unlisten.then(fn => fn())
      }
    })

    return () => {
      cancelled = true
    }
  }, [onFilesSelected, t])

  const handleClick = useCallback(() => {
    setError(null)
  }, [])

  return (
    <div className={cn('relative', className)}>
      <div
        ref={dropZoneRef}
        className={cn(
          'flex flex-col items-center justify-center p-8',
          'border-2 border-dashed rounded-xl',
          'transition-all duration-200 cursor-pointer',
          'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2',
          isDragging
            ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
            : 'border-gray-300 dark:border-gray-600 hover:border-primary-400',
          error && 'border-red-500 bg-red-50 dark:bg-red-900/20'
        )}
        role="button"
        tabIndex={0}
        aria-label={t('gallery.dropzoneLabel')}
        onClick={handleClick}
      >
        {error ? (
          <>
            <ImageOff className="w-12 h-12 text-red-500 mb-3" />
            <p className="text-red-600 dark:text-red-400 text-sm">{error}</p>
          </>
        ) : (
          <>
            <Upload className={cn(
              'w-12 h-12 mb-3 transition-colors',
              isDragging ? 'text-primary-500' : 'text-gray-400'
            )} />
            <p className="text-gray-600 dark:text-gray-300 mb-1">
              {t('gallery.dropzoneText')} <span className="text-primary-600">{t('gallery.dropzoneClick')}</span>
            </p>
            <p className="text-xs text-gray-400">
              {t('gallery.dropzoneFormats')}
            </p>
          </>
        )}
      </div>
    </div>
  )
}
