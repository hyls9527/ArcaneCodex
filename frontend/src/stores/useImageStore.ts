import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { type AppImage } from '../types/image'
import { type SearchResult, getImages } from '../lib/api'

export interface ImageState {
  images: AppImage[]
  selectedIds: number[]
  filters: {
    ai_status?: string
    date_from?: string
    date_to?: string
    category?: string
    tags?: string[]
  }
  page: number
  pageSize: number
  total: number
  loading: boolean
  error: string | null
  searchQuery: string
  searchResults: SearchResult[]
  searchLoading: boolean
  hasSearched: boolean

  setImages: (images: AppImage[]) => void
  setSelectedIds: (ids: number[]) => void
  toggleSelect: (id: number) => void
  selectAll: () => void
  deselectAll: () => void
  setFilters: (filters: Partial<ImageState['filters']>) => void
  setPage: (page: number) => void
  setPageSize: (pageSize: number) => void
  setTotal: (total: number) => void
  addImages: (images: AppImage[]) => void
  removeImages: (ids: number[]) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  setSearchQuery: (query: string) => void
  setSearchResults: (results: SearchResult[]) => void
  setSearchLoading: (loading: boolean) => void
  setHasSearched: (searched: boolean) => void
  clearSearch: () => void
  loadImages: () => Promise<void>
}

export const useImageStore = create<ImageState>()(
  persist(
    (set, get) => ({
      images: [],
      selectedIds: [],
      filters: {},
      page: 1,
      pageSize: 50,
      total: 0,
      loading: false,
      error: null,
      searchQuery: '',
      searchResults: [],
      searchLoading: false,
      hasSearched: false,

      setImages: (images) => set({ images }),
      setSelectedIds: (selectedIds) => set({ selectedIds }),
      toggleSelect: (id) => set((state) => ({
        selectedIds: state.selectedIds.includes(id)
          ? state.selectedIds.filter(i => i !== id)
          : [...state.selectedIds, id]
      })),
      selectAll: () => set((state) => ({
        selectedIds: state.images.map(img => img.id)
      })),
      deselectAll: () => set({ selectedIds: [] }),
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters },
        page: 1,
      })),
      setPage: (page) => set({ page }),
      setPageSize: (pageSize) => set({ pageSize, page: 1 }),
      setTotal: (total) => set({ total }),
      addImages: (images) => set((state) => ({
        images: [...images, ...state.images]
      })),
      removeImages: (ids) => set((state) => ({
        images: state.images.filter(img => !ids.includes(img.id)),
        selectedIds: state.selectedIds.filter(id => !ids.includes(id))
      })),
      setLoading: (loading) => set({ loading }),
      setError: (error) => set({ error }),
      setSearchQuery: (searchQuery) => set({ searchQuery }),
      setSearchResults: (searchResults) => set({ searchResults }),
      setSearchLoading: (searchLoading) => set({ searchLoading }),
      setHasSearched: (hasSearched) => set({ hasSearched }),
      clearSearch: () => set({ searchQuery: '', searchResults: [], searchLoading: false, hasSearched: false }),
      loadImages: async () => {
        const { filters, page, pageSize } = get()
        const hasFilters = filters.ai_status || filters.category || filters.date_from || filters.date_to || (filters.tags && filters.tags.length > 0)
        try {
          set({ loading: true, error: null })
          const result = await getImages({
            page,
            page_size: pageSize,
            filters: hasFilters ? filters : undefined,
          })
          if (result && result.images && Array.isArray(result.images)) {
            set({ images: result.images, loading: false })
          } else {
            set({ error: 'common.loadFailed', loading: false })
          }
        } catch (err) {
          // Tauri invoke errors are objects like { code: '...', message: '...' }
          let message = 'common.unknownError'
          if (err && typeof err === 'object') {
            if ('message' in err && typeof err.message === 'string') {
              message = err.message
            } else if ('code' in err && typeof err.code === 'string') {
              message = err.code
            } else {
              message = JSON.stringify(err)
            }
          } else if (err instanceof Error) {
            message = err.message
          }
          set({ error: `errors.loadImagesFailed: ${message}`, loading: false })
        }
      },
    }),
    {
      name: 'image-store',
      partialize: (state) => ({
        filters: state.filters,
        pageSize: state.pageSize,
      }),
    }
  )
)
