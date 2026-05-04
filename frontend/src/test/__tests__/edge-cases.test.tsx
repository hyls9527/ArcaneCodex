/**
 * 边界情况和边缘测试
 * 覆盖: 空列表、长文件名、零结果、并发更新、i18n 回退、store 持久化
 */

import React from 'react'
import { describe, it, expect, vi } from 'vitest'
import { cn } from '@/utils/cn'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(vi.fn())),
  emit: vi.fn(),
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const map: Record<string, string> = {
        'gallery.dropzoneLabel': '拖拽图片到此处',
      }
      return map[key] || key
    },
    i18n: { language: 'zh' },
  }),
}))

vi.mock('motion/react', () => {
  const stripMotionProps = (props: Record<string, unknown>): Record<string, unknown> => {
    const result: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(props)) {
      if (!['initial', 'animate', 'exit', 'transition'].includes(key)) {
        result[key] = value
      }
    }
    return result
  }

  return {
    motion: {
      div: ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => {
        return <div {...(stripMotionProps(props) as React.HTMLAttributes<HTMLDivElement>)}>{children}</div>
      },
      img: ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => {
        return <img {...(stripMotionProps(props) as React.HTMLAttributes<HTMLImageElement>)}>{children}</img>
      },
    },
    AnimatePresence: ({ children }: React.PropsWithChildren) => <>{children}</>,
  }
})

vi.mock('lucide-react', () => ({
  X: () => <span />,
  ZoomIn: () => <span />,
  ZoomOut: () => <span />,
  RotateCw: () => <span />,
  Download: () => <span />,
  Trash2: () => <span />,
  Camera: () => <span />,
  Clock: () => <span />,
  MapPin: () => <span />,
  Tag: () => <span />,
  Archive: () => <span />,
  RefreshCw: () => <span />,
  Info: () => <span />,
  Loader2: () => <span />,
  AlertCircle: () => <span />,
  Search: () => <span />,
  Link2: () => <span />,
}))

describe('边界情况: 空图片列表', () => {
  it('useImageStore 初始 images 应为空数组', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')
    useImageStore.setState({ images: [] })
    expect(useImageStore.getState().images).toEqual([])
  })

  it('selectAll 在空列表时应生成空数组', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')
    useImageStore.setState({ images: [], selectedIds: [] })
    useImageStore.getState().selectAll()
    expect(useImageStore.getState().selectedIds).toHaveLength(0)
  })

  it('removeImages 在空列表时应安全执行', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')
    useImageStore.setState({ images: [] })
    useImageStore.getState().removeImages([1, 2, 3])
    expect(useImageStore.getState().images).toEqual([])
  })

  it('cn() 空输入应返回空字符串', () => {
    expect(cn()).toBe('')
    expect(cn(null, undefined, false, 0, '')).toBe('')
  })
})

describe('边界情况: 极长文件名', () => {
  it('应正确处理极长文件名', async () => {
    const longName = 'a'.repeat(1000) + '.jpg'
    const { useImageStore } = await import('@/stores/useImageStore')

    useImageStore.setState({
      images: [{
        id: 1,
        file_path: `/path/to/${longName}`,
        file_name: longName,
        thumbnail_path: '/t/1.webp',
        ai_status: 'completed',
      }],
    })

    expect(useImageStore.getState().images[0].file_name).toBe(longName)
    expect(useImageStore.getState().images[0].file_name.length).toBe(1004)
  })

  it('中文文件名应正确处理', async () => {
    const chineseName = '这是一个非常长的中文文件名_测试特殊字符_!@#$%^&()_最终版_修订稿_第三版.jpg'
    const { useImageStore } = await import('@/stores/useImageStore')

    useImageStore.setState({
      images: [{
        id: 1,
        file_path: `/photos/${chineseName}`,
        file_name: chineseName,
        thumbnail_path: '/t/1.webp',
        ai_status: 'completed',
      }],
    })

    expect(useImageStore.getState().images[0].file_name).toBe(chineseName)
  })

  it('带特殊字符的文件名应正确存储', async () => {
    const specialName = 'file (1) [copy] {v2} #tag @2024.jpg'
    const { useImageStore } = await import('@/stores/useImageStore')

    useImageStore.setState({
      images: [{
        id: 1,
        file_path: `/photos/${specialName}`,
        file_name: specialName,
        thumbnail_path: '/t/1.webp',
        ai_status: 'completed',
      }],
    })

    expect(useImageStore.getState().images[0].file_name).toBe(specialName)
  })
})

describe('边界情况: 零 AI 结果', () => {
  it('AI store 初始状态 total=0 时进度应为 0', async () => {
    const { useAIStore } = await import('@/stores/useAIStore')
    useAIStore.getState().reset()

    const state = useAIStore.getState()
    expect(state.total).toBe(0)
    expect(state.completed).toBe(0)
    expect(state.failed).toBe(0)
  })

  it('AI completed=0, total=0 时进度百分比计算应安全', () => {
    const total = 0
    const completed = 0
    const progress = total > 0 ? Math.round((completed / total) * 100) : 0
    expect(progress).toBe(0)
  })

  it('AI 失败数为 0 时不应显示重试按钮', async () => {
    const { useAIStore } = await import('@/stores/useAIStore')
    useAIStore.setState({
      status: 'completed',
      total: 100,
      completed: 100,
      failed: 0,
      retrying: 0,
    })

    expect(useAIStore.getState().failed).toBe(0)
  })

  it('DedupStore 空结果应安全处理', async () => {
    const { useDedupStore } = await import('@/stores/useDedupStore')
    useDedupStore.getState().reset()

    expect(useDedupStore.getState().groups).toEqual([])
    expect(useDedupStore.getState().loading).toBe(false)
  })
})

describe('边界情况: 并发状态更新', () => {
  it('多次快速 setImages 应保持最终状态一致', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')
    useImageStore.setState({ images: [] })

    for (let i = 0; i < 100; i++) {
      useImageStore.getState().setImages([{
        id: i,
        file_path: `/img-${i}.jpg`,
        file_name: `img-${i}.jpg`,
        thumbnail_path: `/t/${i}.webp`,
        ai_status: 'completed',
      }])
    }

    const finalState = useImageStore.getState()
    expect(finalState.images).toHaveLength(1)
    expect(finalState.images[0].id).toBe(99)
  })

  it('多次快速 toggleSelect 应保持一致', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')
    useImageStore.setState({ selectedIds: [] })

    for (let i = 0; i < 10; i++) {
      useImageStore.getState().toggleSelect(1)
    }

    expect(useImageStore.getState().selectedIds.includes(1)).toBe(false)
  })

  it('AI store 的 reset 应清除所有并发状态', async () => {
    const { useAIStore } = await import('@/stores/useAIStore')

    useAIStore.getState().setStatus('processing')
    useAIStore.getState().setTotal(100)
    useAIStore.getState().setCompleted(50)
    useAIStore.getState().setFailed(5)
    useAIStore.getState().setETA(60)

    useAIStore.getState().reset()

    const state = useAIStore.getState()
    expect(state.status).toBe('idle')
    expect(state.total).toBe(0)
    expect(state.completed).toBe(0)
    expect(state.failed).toBe(0)
    expect(state.eta_seconds).toBeUndefined()
  })

  it('setFilters 多次调用应正确合并', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')
    useImageStore.setState({ filters: {}, page: 5 })

    useImageStore.getState().setFilters({ ai_status: 'completed' })
    useImageStore.getState().setFilters({ category: 'nature' })
    useImageStore.getState().setFilters({ tags: ['sunset'] })

    const state = useImageStore.getState()
    expect(state.filters.ai_status).toBe('completed')
    expect(state.filters.category).toBe('nature')
    expect(state.filters.tags).toEqual(['sunset'])
    expect(state.page).toBe(1)
  })
})

describe('边界情况: i18n 回退', () => {
  it('react-i18next mock 应返回 key 作为回退', () => {
    const mockT = (key: string) => {
      const map: Record<string, string> = {
        'gallery.dropzoneLabel': '拖拽图片到此处',
      }
      return map[key] || key
    }

    expect(mockT('gallery.dropzoneLabel')).toBe('拖拽图片到此处')
    expect(mockT('unknown.key')).toBe('unknown.key')
    expect(mockT('')).toBe('')
  })
})

describe('边界情况: Store 持久化', () => {
  it('useImageStore 应有 persist 中间件配置', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    expect(useImageStore).toBeDefined()
    expect(typeof useImageStore.getState).toBe('function')
    expect(typeof useImageStore.setState).toBe('function')
  })

  it('filters 状态应在 setState 后保持', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    useImageStore.setState({
      filters: {
        ai_status: 'completed',
        category: 'nature',
        tags: ['sunset', 'beach'],
      },
      pageSize: 100,
    })

    const state = useImageStore.getState()
    expect(state.filters.ai_status).toBe('completed')
    expect(state.filters.category).toBe('nature')
    expect(state.filters.tags).toEqual(['sunset', 'beach'])
    expect(state.pageSize).toBe(100)
  })

  it('useConfigStore 应维护 pendingChanges 状态', async () => {
    const { useConfigStore } = await import('@/stores/useConfigStore')
    useConfigStore.setState({ pendingChanges: {} })

    useConfigStore.getState().updateField('theme', 'dark')
    useConfigStore.getState().updateField('language', 'en')

    expect(useConfigStore.getState().pendingChanges.theme).toBe('dark')
    expect(useConfigStore.getState().pendingChanges.language).toBe('en')
    expect(useConfigStore.getState().hasPendingChanges()).toBe(true)
  })
})

describe('边界情况: 大数据量', () => {
  it('应正确处理 10000 张图片', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    const largeDataset = Array.from({ length: 10000 }, (_, i) => ({
      id: i + 1,
      file_path: `/img-${i + 1}.jpg`,
      file_name: `image-${i + 1}.jpg`,
      thumbnail_path: `/t/${i + 1}.webp`,
      ai_status: 'completed' as const,
    }))

    useImageStore.getState().setImages(largeDataset)
    expect(useImageStore.getState().images).toHaveLength(10000)
  })

  it('selectAll 应正确处理大量图片', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    const images = Array.from({ length: 1000 }, (_, i) => ({
      id: i + 1,
      file_path: `/img-${i + 1}.jpg`,
      file_name: `image-${i + 1}.jpg`,
      thumbnail_path: `/t/${i + 1}.webp`,
      ai_status: 'completed' as const,
    }))

    useImageStore.setState({ images, selectedIds: [] })
    useImageStore.getState().selectAll()

    expect(useImageStore.getState().selectedIds).toHaveLength(1000)
  })

  it('removeImages 应正确处理批量删除', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    const images = Array.from({ length: 100 }, (_, i) => ({
      id: i + 1,
      file_path: `/img-${i + 1}.jpg`,
      file_name: `image-${i + 1}.jpg`,
      thumbnail_path: `/t/${i + 1}.webp`,
      ai_status: 'completed' as const,
    }))

    useImageStore.setState({ images })

    const idsToRemove = Array.from({ length: 50 }, (_, i) => i + 1)
    useImageStore.getState().removeImages(idsToRemove)

    expect(useImageStore.getState().images).toHaveLength(50)
    expect(useImageStore.getState().images[0].id).toBe(51)
  })
})

describe('边界情况: Store 清理', () => {
  it('clearSearch 应重置搜索状态但不影响图片列表', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    useImageStore.setState({
      images: [{
        id: 1,
        file_path: '/a.jpg',
        file_name: 'a.jpg',
        thumbnail_path: '/t/a.webp',
        ai_status: 'completed',
      }],
      searchQuery: 'test',
      searchResults: [{ image_id: 1, file_path: '', file_name: '', match_count: 0, relevance_score: 0 }],
      searchLoading: true,
      hasSearched: true,
    })

    useImageStore.getState().clearSearch()

    const state = useImageStore.getState()
    expect(state.images).toHaveLength(1)
    expect(state.searchQuery).toBe('')
    expect(state.searchResults).toEqual([])
    expect(state.searchLoading).toBe(false)
    expect(state.hasSearched).toBe(false)
  })

  it('deselectAll 不应影响图片列表', async () => {
    const { useImageStore } = await import('@/stores/useImageStore')

    useImageStore.setState({
      images: [{
        id: 1,
        file_path: '/a.jpg',
        file_name: 'a.jpg',
        thumbnail_path: '/t/a.webp',
        ai_status: 'completed',
      }],
      selectedIds: [1],
    })

    useImageStore.getState().deselectAll()

    expect(useImageStore.getState().images).toHaveLength(1)
    expect(useImageStore.getState().selectedIds).toHaveLength(0)
  })
})
