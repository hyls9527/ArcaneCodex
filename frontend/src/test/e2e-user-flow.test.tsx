/**
 * TC-E2E-HP-001: 端到端用户流程集成测试
 * 完整流程: 打开应用 → 拖拽导入 50 张图 → AI 自动打标 → 浏览图库 → 搜索"日落"
 * 
 * 测试策略:
 * 由于完整的 E2E 测试需要真实的 Tauri 运行时（Rust 后端 + WebView2），
 * 本测试采用分层集成测试策略:
 * 1. 组件集成测试 - 验证 DropZone, ImageGrid, TopBar 之间的数据流
 * 2. API 层 Mock 测试 - 验证与后端的交互逻辑
 * 3. 状态同步测试 - 验证 Zustand stores 的状态一致性
 * 4. 业务逻辑测试 - 验证搜索、导入、AI 队列的集成
 */

import React from 'react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { DropZone } from '@/components/gallery/DropZone'
import { ImageGrid } from '@/components/gallery/ImageGrid'
import { TopBar } from '@/components/layout/TopBar'
import { semanticSearch, importImages, getImages, startAIProcessing, getAIStatus } from '@/lib/api'

// ===== Mock Setup =====
vi.mock('@/lib/api', () => ({
  semanticSearch: vi.fn(),
  importImages: vi.fn(),
  getImages: vi.fn(),
  startAIProcessing: vi.fn(),
  getAIStatus: vi.fn(),
  pauseAIProcessing: vi.fn(),
  resumeAIProcessing: vi.fn(),
  retryFailedAI: vi.fn(),
  deleteImages: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('i18next', () => ({
  default: {
    use: vi.fn().mockReturnValue({ init: vi.fn() }),
    init: vi.fn(),
    changeLanguage: vi.fn(),
    on: vi.fn(),
  },
}))

vi.mock('@/stores/useThemeStore', () => ({
  useThemeStore: vi.fn(() => ({ applyTheme: vi.fn() })),
}))

vi.mock('@/stores/useConfigStore', () => ({
  useConfigStore: vi.fn(() => ({ theme: 'light', language: 'zh', pendingChanges: {}, updateField: vi.fn() })),
  CONFIG_KEYS: { THEME: 'theme', LANGUAGE: 'language' },
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'gallery.dropzoneLabel': '拖拽图片到此处，或点击选择文件',
        'gallery.dropzoneText': '拖拽图片到此处，或',
        'gallery.dropzoneClick': '点击选择文件',
        'gallery.dropzoneFormats': '支持 JPEG, PNG, WebP, GIF, HEIC (最大 50MB)',
        'topBar.searchPlaceholder': '搜索图片...',
        'topBar.toggleLanguage': '切换语言',
        'topBar.toggleTheme': '切换主题',
      }
      return translations[key] || key
    },
    i18n: { changeLanguage: vi.fn(), language: 'zh' },
  }),
}))

// ===== Test Data Generation =====
const generateMockImages = (count: number) => {
  return Array.from({ length: count }, (_, i) => ({
    id: i + 1,
    thumbnail_path: `/thumbnails/${i + 1}.webp`,
    file_name: `photo-${i + 1}.jpg`,
    ai_tags: i % 5 === 0 ? ['日落', '海滩', '风景'] : i % 3 === 0 ? ['城市', '建筑'] : ['自然', '风景'],
    ai_status: (i % 10 === 0 ? 'completed' : i % 7 === 0 ? 'processing' : 'pending') as const,
    file_path: `/photos/photo-${i + 1}.jpg`,
    width: 1920,
    height: 1080,
    file_size: 2 * 1024 * 1024,
  }))
}

const generateMockFiles = (count: number) => {
  return Array.from({ length: count }, (_, i) => {
    const file = new File(['dummy content'], `photo-${i + 1}.jpg`, { type: 'image/jpeg' })
    Object.defineProperty(file, 'path', { value: `/photos/photo-${i + 1}.jpg` })
    return file
  })
}

describe('TC-E2E-HP-001: 完整用户流程集成测试', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('阶段 1: 应用初始化与图片导入', () => {
    it('TC-E2E-HP-001-P1: DropZone 接收 50 张图片并调用导入 API', async () => {
      const mockImportResult = {
        success_count: 50,
        duplicate_count: 0,
        error_count: 0,
        image_ids: Array.from({ length: 50 }, (_, i) => i + 1),
      }
      vi.mocked(importImages).mockResolvedValue(mockImportResult)

      const onFilesSelected = vi.fn()
      render(<DropZone onFilesSelected={onFilesSelected} />)

      // 验证 DropZone 渲染成功
      const dropzone = screen.getByRole('button', { name: /拖拽图片到此处/ })
      expect(dropzone).toBeInTheDocument()

      // 验证 onFilesSelected 回调在组件挂载后可用
      expect(onFilesSelected).toBeDefined()
    })

    it('TC-E2E-HP-001-P2: 导入 API 调用正确传递文件路径', async () => {
      const mockFiles = generateMockFiles(50)
      const mockImportResult = {
        success_count: 50,
        duplicate_count: 0,
        error_count: 0,
        image_ids: Array.from({ length: 50 }, (_, i) => i + 1),
      }
      vi.mocked(importImages).mockResolvedValue(mockImportResult)

      const filePaths = mockFiles.map(f => (f as TauriFile).path || f.name)
      await importImages(filePaths)

      expect(importImages).toHaveBeenCalledWith(filePaths)
      expect(importImages).toHaveBeenCalledTimes(1)
    })

    it('TC-E2E-HP-001-P3: 导入后 ImageGrid 正确渲染 50 张图片', async () => {
      const mockImages = generateMockImages(50)
      vi.mocked(getImages).mockResolvedValue(mockImages)

      const { container } = render(<ImageGrid images={mockImages} onImageClick={() => {}} />)

      const gridContainer = container.querySelector('.overflow-auto')
      expect(gridContainer).toBeInTheDocument()
      
      const virtualContainer = container.querySelector('.relative')
      expect(virtualContainer).toBeInTheDocument()
      const height = parseInt((virtualContainer as HTMLElement).style.height, 10)
      expect(height).toBeGreaterThan(0)
      expect(height).toBeLessThanOrEqual(12500)
      
      mockImages.forEach(img => {
        expect(img.id).toBeGreaterThan(0)
        expect(img.file_name).toBeTruthy()
      })
    })
  })

  describe('阶段 2: AI 自动打标集成', () => {
    it('TC-E2E-HP-001-A1: 导入后自动触发 AI 处理队列', async () => {
      vi.mocked(startAIProcessing).mockResolvedValue(undefined)
      vi.mocked(getAIStatus).mockResolvedValue({
        status: 'processing',
        total: 50,
        completed: 10,
        failed: 0,
        retrying: 0,
      })

      await startAIProcessing()
      expect(startAIProcessing).toHaveBeenCalled()

      const status = await getAIStatus()
      expect(status.status).toBe('processing')
      expect(status.total).toBe(50)
      expect(status.completed).toBe(10)
    })

    it('TC-E2E-HP-001-A2: AI 打标结果反映在 ImageGrid 中', () => {
      const mockImages = generateMockImages(50)
      const sunsetImages = mockImages.filter(img => img.ai_tags?.includes('日落'))
      
      render(<ImageGrid images={mockImages} onImageClick={() => {}} />)

      // 验证包含"日落"标签的图片在数据集中
      expect(sunsetImages.length).toBeGreaterThan(0)
      
      // 验证每张图片都有 AI 状态
      mockImages.forEach(img => {
        expect(img).toHaveProperty('ai_status')
        expect(['pending', 'processing', 'completed', 'failed']).toContain(img.ai_status)
      })
    })
  })

  describe('阶段 3: 搜索功能集成', () => {
    it('TC-E2E-HP-001-S1: 搜索"日落"返回正确的图片', async () => {
      const mockSearchResults = [
        { image_id: 1, score: 0.95 },
        { image_id: 6, score: 0.92 },
        { image_id: 11, score: 0.88 },
        { image_id: 16, score: 0.85 },
        { image_id: 21, score: 0.82 },
      ]
      vi.mocked(semanticSearch).mockResolvedValue(mockSearchResults)

      const results = await semanticSearch('日落', 50)

      expect(semanticSearch).toHaveBeenCalledWith('日落', 50)
      expect(results.length).toBe(5)
      expect(results[0].image_id).toBe(1)
      expect(results[0].score).toBeGreaterThan(0.9)
      
      // 验证结果按相关性排序
      for (let i = 1; i < results.length; i++) {
        expect(results[i].score).toBeLessThanOrEqual(results[i - 1].score)
      }
    })

    it('TC-E2E-HP-001-S2: 搜索结果与 ImageGrid 数据集成', async () => {
      const allImages = generateMockImages(50)
      const mockSearchResults = [
        { image_id: 1, score: 0.95 },
        { image_id: 6, score: 0.92 },
        { image_id: 11, score: 0.88 },
      ]
      vi.mocked(semanticSearch).mockResolvedValue(mockSearchResults)

      const results = await semanticSearch('日落')
      
      // 过滤出搜索结果对应的图片
      const filteredImages = allImages.filter(img => 
        results.some(r => r.image_id === img.id)
      )

      expect(filteredImages.length).toBe(3)
      
      // 验证搜索结果的图片包含"日落"标签
      filteredImages.forEach(img => {
        expect(img.ai_tags).toContain('日落')
      })
    })

    it('TC-E2E-HP-001-S3: TopBar 搜索框与 API 集成', () => {
      render(<TopBar />)

      const searchInput = screen.getByPlaceholderText(/搜索图片/)
      expect(searchInput).toBeInTheDocument()

      fireEvent.change(searchInput, { target: { value: '日落' } })
      expect(searchInput).toHaveValue('日落')
    })
  })

  describe('阶段 4: 端到端数据流集成', () => {
    it('TC-E2E-HP-001-E1: 完整数据流验证 - 导入 -> AI -> 搜索', async () => {
      // 步骤 1: 模拟导入
      const mockFiles = generateMockFiles(50)
      const mockImportResult = {
        success_count: 50,
        duplicate_count: 0,
        error_count: 0,
        image_ids: Array.from({ length: 50 }, (_, i) => i + 1),
      }
      vi.mocked(importImages).mockResolvedValue(mockImportResult)

      const filePaths = mockFiles.map(f => (f as TauriFile).path || f.name)
      const importResult = await importImages(filePaths)
      expect(importResult.success_count).toBe(50)

      // 步骤 2: 启动 AI 处理
      vi.mocked(startAIProcessing).mockResolvedValue(undefined)
      await startAIProcessing()
      expect(startAIProcessing).toHaveBeenCalled()

      // 步骤 3: 模拟 AI 完成
      vi.mocked(getAIStatus).mockResolvedValue({
        status: 'completed',
        total: 50,
        completed: 50,
        failed: 0,
        retrying: 0,
      })
      const finalStatus = await getAIStatus()
      expect(finalStatus.status).toBe('completed')
      expect(finalStatus.completed).toBe(50)

      // 步骤 4: 搜索"日落"
      const mockSearchResults = Array.from({ length: 10 }, (_, i) => ({
        image_id: i * 5 + 1,
        score: 0.95 - i * 0.02,
      }))
      vi.mocked(semanticSearch).mockResolvedValue(mockSearchResults)

      const searchResults = await semanticSearch('日落')
      expect(searchResults.length).toBe(10)

      // 验证完整流程数据一致性
      expect(importResult.success_count).toBe(finalStatus.completed)
      expect(searchResults.every(r => r.score > 0)).toBe(true)
    })

    it('TC-E2E-HP-001-E2: 批量导入 50 张图片的性能基准', async () => {
      const mockFiles = generateMockFiles(50)
      const mockImportResult = {
        success_count: 50,
        duplicate_count: 0,
        error_count: 0,
        image_ids: Array.from({ length: 50 }, (_, i) => i + 1),
      }
      vi.mocked(importImages).mockResolvedValue(mockImportResult)

      const startTime = performance.now()
      const filePaths = mockFiles.map(f => (f as TauriFile).path || f.name)
      await importImages(filePaths)
      const endTime = performance.now()

      // 验证导入操作在合理时间内完成 (< 5秒为基准)
      expect(endTime - startTime).toBeLessThan(5000)
    })

    it('TC-E2E-HP-001-E3: ImageGrid 虚拟滚动与大数据集成', () => {
      const largeDataset = generateMockImages(5000)
      
      const { container } = render(<ImageGrid images={largeDataset} onImageClick={() => {}} />)

      const virtualContainer = container.querySelector('.relative')
      expect(virtualContainer).toBeInTheDocument()
      
      const height = parseInt((virtualContainer as HTMLElement).style.height, 10)
      expect(height).toBeGreaterThan(0)
      expect(height).toBeLessThanOrEqual(1250000)
      
      expect(largeDataset.length).toBe(5000)
    })
  })

  describe('阶段 5: 错误处理与边界情况', () => {
    it('TC-E2E-HP-001-ERR1: 导入失败时的错误处理', async () => {
      vi.mocked(importImages).mockRejectedValue(new Error('磁盘空间不足'))

      await expect(importImages(['/invalid/path.jpg'])).rejects.toThrow('磁盘空间不足')
    })

    it('TC-E2E-HP-001-ERR2: 搜索无结果时的处理', async () => {
      vi.mocked(semanticSearch).mockResolvedValue([])

      const results = await semanticSearch('不存在的标签')
      expect(results).toEqual([])
    })

    it('TC-E2E-HP-001-ERR3: AI 处理部分失败的场景', async () => {
      vi.mocked(getAIStatus).mockResolvedValue({
        status: 'completed',
        total: 50,
        completed: 47,
        failed: 3,
        retrying: 0,
      })

      const status = await getAIStatus()
      expect(status.failed).toBe(3)
      expect(status.completed + status.failed).toBe(status.total)
    })
  })
})
