import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useDropzone, type FileRejection } from 'react-dropzone'
import { Upload, ImageOff } from 'lucide-react'
import { cn } from '@/utils/cn'

interface DropZoneProps {
  onFilesSelected: (files: File[]) => void
  className?: string
  accept?: Record<string, string[]>
  maxSize?: number
}

export function DropZone({ 
  onFilesSelected, 
  className,
  accept = { 'image/*': ['.jpeg', '.jpg', '.png', '.webp', '.gif', '.heic', '.heif'] },
  maxSize = 50 * 1024 * 1024
}: DropZoneProps) {
  const { t } = useTranslation()
  const [isDragging, setIsDragging] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const onDrop = useCallback((acceptedFiles: File[], rejectedFiles: FileRejection[]) => {
    if (rejectedFiles.length > 0) {
      const errors = rejectedFiles.map(r => r.errors[0]?.message).filter(Boolean)
      setError(errors.join(', '))
      return
    }
    
    setError(null)
    onFilesSelected(acceptedFiles)
  }, [onFilesSelected])
  
  const { getRootProps, getInputProps } = useDropzone({
    onDrop,
    accept,
    maxSize,
    onDragEnter: () => setIsDragging(true),
    onDragLeave: () => setIsDragging(false),
    onDropAccepted: () => setIsDragging(false),
    onDropRejected: () => setIsDragging(false),
  })
  
  return (
    <div className={cn('relative', className)}>
      <div
        {...getRootProps()}
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
      >
        <input {...getInputProps()} />
        
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
