/**
 * GalleryPage 页面测试
 * 覆盖: 初始渲染、搜索交互、过滤应用
 */

import React from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { GalleryPage } from '../GalleryPage'
import * as api from '@/lib/api'
import { useImageStore } from '@/stores/useImageStore'

// ===== Mocks =====
vi.mock('@/lib/api', () => ({
  importImages: vi.fn().mockResolvedValue({ success_count: 1, duplicate_count: 0, error_count: 0, image_ids: [1] }),
  checkBrokenLinks: vi.fn().mockResolvedValue({ broken_count: 0, broken_images: [] }),
  checkSampleData: vi.fn().mockResolvedValue({ has_sample_data: false }),
}))

vi.mock('@/components/gallery/ImageGrid', () => ({
  ImageGrid: ({ images, onImageClick }: { images?: Array<{ id: number; file_name: string }>; onImageClick?: (id: number) => void }) => (
    <div data-testid="image-grid">
      {images?.map((img: { id: number; file_name: string }) => (
        <div key={img.id} data-testid={`image-${img.id}`} onClick={() => onImageClick?.(img.id)}>
          {img.file_name}
        </div>
      ))}
    </div>
  ),
}))

vi.mock('@/components/gallery/ImageFilter', () => ({
  ImageFilter: () => <div data-testid="image-filter">ImageFilter</div>,
}))

vi.mock('@/components/gallery/DropZone', () => ({
  DropZone: ({ onFilesSelected }: { onFilesSelected?: (files: File[]) => void }) => (
    <div data-testid="dropzone">
      <button onClick={() => onFilesSelected?.([new File([''], 'test.jpg')])}>导入文件</button>
    </div>
  ),
}))

// IMPORTANT: `t` must be a stable reference since GalleryPage uses useCallback([t])
vi.mock('react-i18next', () => {
  const map: Record<string, string> = {
    'common.loading': '加载中...',
    'common.retry': '重试',
    'common.searching': '搜索中...',
    'gallery.importSuccess': '导入成功',
    'gallery.brokenLinksFound': '发现断链',
    'gallery.noBrokenLinks': '没有断链',
    'gallery.checkBrokenLinks': '检查断链',
    'gallery.searchResults': '搜索结果',
    'gallery.resultsCount': '条结果',
    'gallery.noResults': '没有结果',
    'errors.importFailed': '导入失败',
    'errors.brokenLinksCheckFailed': '断链检查失败',
  }
  const t = (key: string) => map[key] || key
  return {
    useTranslation: () => ({
      t,
      i18n: { language: 'zh' },
    }),
    initReactI18next: {
      type: '3rdParty',
      init: vi.fn(),
    },
  }
})

vi.mock('lucide-react', () => ({
  Loader2: () => <span data-testid="icon-loader" />,
  AlertCircle: () => <span data-testid="icon-alert" />,
  Search: () => <span data-testid="icon-search" />,
  Link2: () => <span data-testid="icon-link" />,
  Square: () => <span data-testid="icon-square" />,
  ImagePlus: () => <span data-testid="icon-image-plus" />,
  FileImage: () => <span data-testid="icon-file-image" />,
  FolderOpen: () => <span data-testid="icon-folder-open" />,
}))

// ===== Test Data =====
const mockImages = [
  { id: 1, file_path: '/a.jpg', file_name: 'sunset.jpg', thumbnail_path: '/t/a.webp', ai_status: 'completed', ai_tags: ['sunset'], ai_description: 'A sunset' },
  { id: 2, file_path: '/b.jpg', file_name: 'mountain.jpg', thumbnail_path: '/t/b.webp', ai_status: 'completed', ai_tags: ['nature'], ai_description: 'Mountain view' },
  { id: 3, file_path: '/c.jpg', file_name: 'beach.jpg', thumbnail_path: '/t/c.webp', ai_status: 'pending', ai_tags: [], ai_description: '' },
]

const defaultProps = {
  images: mockImages,
  loading: false,
  error: null,
  onLoadImages: vi.fn().mockResolvedValue(undefined),
  addToast: vi.fn(),
  onImageClick: vi.fn(),
}

describe('GalleryPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Reset search store state
    useImageStore.setState({
      searchQuery: '',
      searchResults: [],
      searchLoading: false,
      hasSearched: false,
    })
  })

  it('正常状态应显示 DropZone 和 ImageGrid', () => {
    render(<GalleryPage {...defaultProps} />)

    expect(screen.getByTestId('dropzone')).toBeInTheDocument()
    expect(screen.getByTestId('image-grid')).toBeInTheDocument()
    expect(screen.getByTestId('image-filter')).toBeInTheDocument()
  })

  it('加载中应显示加载状态', () => {
    render(<GalleryPage {...defaultProps} loading={true} images={[]} />)

    expect(screen.getByText('加载中...')).toBeInTheDocument()
    expect(screen.queryByTestId('image-grid')).not.toBeInTheDocument()
  })

  it('错误状态应显示错误信息', () => {
    render(<GalleryPage {...defaultProps} error="加载失败" images={[]} />)

    expect(screen.getByText('加载失败')).toBeInTheDocument()
    expect(screen.getByText('重试')).toBeInTheDocument()
  })

  it('错误状态点击重试应调用 onLoadImages', () => {
    const onLoadImages = vi.fn()
    render(<GalleryPage {...defaultProps} error="加载失败" images={[]} onLoadImages={onLoadImages} />)

    fireEvent.click(screen.getByText('重试'))
    expect(onLoadImages).toHaveBeenCalledTimes(1)
  })

  it('应该显示检查断链按钮', () => {
    render(<GalleryPage {...defaultProps} />)
    expect(screen.getByText('检查断链')).toBeInTheDocument()
  })

  it('点击检查断链按钮应调用 API', async () => {
    render(<GalleryPage {...defaultProps} />)

    fireEvent.click(screen.getByText('检查断链'))

    await vi.waitFor(() => {
      expect(vi.mocked(api.checkBrokenLinks)).toHaveBeenCalledTimes(1)
    })
  })

  it('ImageGrid 应接收正确的图片数据', () => {
    render(<GalleryPage {...defaultProps} />)

    expect(screen.getByTestId('image-1')).toBeInTheDocument()
    expect(screen.getByTestId('image-2')).toBeInTheDocument()
    expect(screen.getByTestId('image-3')).toBeInTheDocument()
    expect(screen.getByText('sunset.jpg')).toBeInTheDocument()
    expect(screen.getByText('mountain.jpg')).toBeInTheDocument()
    expect(screen.getByText('beach.jpg')).toBeInTheDocument()
  })

  it('点击图片应触发 onImageClick', () => {
    const onImageClick = vi.fn()
    render(<GalleryPage {...defaultProps} onImageClick={onImageClick} />)

    fireEvent.click(screen.getByTestId('image-1'))
    expect(onImageClick).toHaveBeenCalledWith(mockImages[0])
  })

  it('空图片列表时应正常渲染', () => {
    render(<GalleryPage {...defaultProps} images={[]} />)

    expect(screen.getByTestId('dropzone')).toBeInTheDocument()
    expect(screen.getByText('gallery.emptyTitle')).toBeInTheDocument()
  })

  it('有搜索结果时应显示搜索结果区域', () => {
    useImageStore.setState({
      searchQuery: 'sunset',
      searchResults: [
        { image_id: 1, file_path: '/a.jpg', file_name: 'sunset.jpg', thumbnail_path: '/t/a.webp', tags: ['sunset'], description: 'A sunset', score: 0.95 },
      ],
      searchLoading: false,
      hasSearched: true,
    })

    render(<GalleryPage {...defaultProps} />)

    expect(screen.getByText(/搜索结果/)).toBeInTheDocument()
  })

  it('搜索加载中应显示加载状态', () => {
    useImageStore.setState({
      searchQuery: 'test',
      searchResults: [],
      searchLoading: true,
      hasSearched: true,
    })

    render(<GalleryPage {...defaultProps} />)

    expect(screen.getByText('搜索中...')).toBeInTheDocument()
  })
})
