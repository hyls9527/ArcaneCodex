import { useTranslation } from 'react-i18next'
import { Loader2, Search } from 'lucide-react'
import { ImageGrid } from '../../components/gallery/ImageGrid'
import { type SearchResult } from '../../lib/api'

interface GallerySearchResultsProps {
  searchQuery: string
  searchResults: SearchResult[]
  searchLoading: boolean
  hasSearched: boolean
  onImageClick: (id: number) => void
  selectedIds: number[]
  onToggleSelect: (id: number) => void
}

export function GallerySearchResults({
  searchQuery,
  searchResults,
  searchLoading,
  hasSearched,
  onImageClick,
  selectedIds,
  onToggleSelect,
}: GallerySearchResultsProps) {
  const { t } = useTranslation()

  if (!hasSearched || !searchQuery.trim()) return null

  return (
    <div className="mb-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-lg font-medium">{t('gallery.searchResults', { query: searchQuery })}</h3>
        <span className="text-sm text-gray-500">{searchResults.length} {t('gallery.resultsCount')}</span>
      </div>
      {searchLoading ? (
        <div className="flex items-center justify-center py-8 gap-2">
          <Loader2 className="w-5 h-5 animate-spin text-primary-500" />
          <span className="text-sm text-gray-500">{t('common.searching')}</span>
        </div>
      ) : searchResults.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-8 text-gray-500">
          <Search className="w-12 h-12 mb-2 opacity-50" />
          <p className="text-sm">{t('gallery.noResults', { query: searchQuery })}</p>
        </div>
      ) : (
        <div className="h-[calc(100%-200px)]">
          <ImageGrid
            images={searchResults.map(r => ({
              id: r.image_id,
              file_path: r.file_path,
              thumbnail_path: r.thumbnail_path || '',
              file_name: r.file_name || String(r.image_id),
              ai_tags: r.tags ? JSON.stringify(r.tags) : undefined,
              ai_status: 'completed' as const,
              ai_category: r.category || '',
              ai_description: r.description,
              ai_confidence: r.ai_confidence,
            }))}
            onImageClick={onImageClick}
            selectedIds={selectedIds}
            onToggleSelect={onToggleSelect}
          />
        </div>
      )}
    </div>
  )
}
