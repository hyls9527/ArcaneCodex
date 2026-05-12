import { useTranslation } from 'react-i18next'
import { NarrativePrompt } from '../NarrativePrompt'
import type { Narrative } from '@/lib/api'

interface ImageBottomBarProps {
  image: {
    id: number
    file_name: string
    width?: number
    height?: number
    ai_tags?: string[]
    ai_description?: string
    ai_category?: string
  }
  narratives: Narrative[]
  onWriteNarrative: (imageId: number, content: string) => Promise<void>
}

export function ImageBottomBar({
  image,
  narratives,
  onWriteNarrative,
}: ImageBottomBarProps) {
  const { t } = useTranslation()

  return (
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
          onWriteNarrative={onWriteNarrative}
        />
      </div>
    </div>
  )
}
