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
          if (import.meta.env.DEV) {
            console.debug('[ImageStore] loadImages called:', { page, pageSize, hasFilters, filters })
          }
          const result = await getImages({
            page,
            page_size: pageSize,
            filters: hasFilters ? filters : undefined,
          })
          if (import.meta.env.DEV) {
            console.debug('[ImageStore] getImages result:', JSON.stringify(result)?.substring(0, 500))
          }
          if (result && result.images && Array.isArray(result.images)) {
            if (import.meta.env.DEV) {
              console.debug('[ImageStore] Setting images count:', result.images.length)
            }
            set({ images: result.images, loading: false })
          } else {
            console.warn('[ImageStore] Invalid result format:', result)
            set({ error: 'common.loadFailed', loading: false })
          }
        } catch (err: unknown) {
          console.error('[ImageStore] loadImages CATCH:', err)
          let message = 'common.unknownError'
          if (err === null || err === undefined) {
            message = 'null or undefined error'
          } else if (typeof err === 'string') {
            message = err
          } else if (typeof err === 'object') {
            const e = err as Record<string, unknown>
            if ('message' in e && typeof e.message === 'string') {
              try {
                const parsed = JSON.parse(e.message) as { code?: string; message?: string }
                message = parsed.message || parsed.code || e.message
              } catch {
                message = e.message
              }
            } else if ('code' in e && typeof e.code === 'string') {
              message = e.code
            } else {
              message = JSON.stringify(err)
            }
          } else if (err instanceof Error) {
            message = err.message
          } else {
            message = String(err)
          }
          console.error('[ImageStore] loadImages error message:', message)
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
