/**
 * Store 单元测试
 * 覆盖: useImageStore, useAIStore, useConfigStore, useThemeStore, useDedupStore
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('useImageStore', () => {
  beforeEach(async () => {
    const { useImageStore } = await import('../useImageStore')
    useImageStore.setState({
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
    })
  })

  it('初始状态应该正确', async () => {
    const { useImageStore } = await import('../useImageStore')
    const state = useImageStore.getState()

    expect(state.images).toEqual([])
    expect(state.selectedIds).toEqual([])
    expect(state.filters).toEqual({})
    expect(state.page).toBe(1)
    expect(state.pageSize).toBe(50)
    expect(state.total).toBe(0)
    expect(state.loading).toBe(false)
    expect(state.error).toBeNull()
    expect(state.searchQuery).toBe('')
    expect(state.searchResults).toEqual([])
    expect(state.searchLoading).toBe(false)
    expect(state.hasSearched).toBe(false)
  })

  it('setImages 应该替换图片列表', async () => {
    const { useImageStore } = await import('../useImageStore')
    const mockImages = [
      { id: 1, file_path: '/a.jpg', file_name: 'a.jpg', thumbnail_path: '/t/a.webp', ai_status: 'completed' },
      { id: 2, file_path: '/b.jpg', file_name: 'b.jpg', thumbnail_path: '/t/b.webp', ai_status: 'pending' },
    ]

    useImageStore.getState().setImages(mockImages)
    expect(useImageStore.getState().images).toHaveLength(2)
    expect(useImageStore.getState().images[0].id).toBe(1)
  })

  it('addImages 应该将新图片添加到列表前面', async () => {
    const { useImageStore } = await import('../useImageStore')
    const existing = [
      { id: 1, file_path: '/a.jpg', file_name: 'a.jpg', thumbnail_path: '/t/a.webp', ai_status: 'completed' },
    ]
    const newImages = [
      { id: 2, file_path: '/b.jpg', file_name: 'b.jpg', thumbnail_path: '/t/b.webp', ai_status: 'pending' },
    ]

    useImageStore.setState({ images: existing })
    useImageStore.getState().addImages(newImages)

    const { images } = useImageStore.getState()
    expect(images).toHaveLength(2)
    expect(images[0].id).toBe(2)
    expect(images[1].id).toBe(1)
  })

  it('removeImages 应该移除指定 id 的图片和选中状态', async () => {
    const { useImageStore } = await import('../useImageStore')
    const images = [
      { id: 1, file_path: '/a.jpg', file_name: 'a.jpg', thumbnail_path: '/t/a.webp', ai_status: 'completed' },
      { id: 2, file_path: '/b.jpg', file_name: 'b.jpg', thumbnail_path: '/t/b.webp', ai_status: 'completed' },
      { id: 3, file_path: '/c.jpg', file_name: 'c.jpg', thumbnail_path: '/t/c.webp', ai_status: 'completed' },
    ]

    useImageStore.setState({ images, selectedIds: [1, 2] })
    useImageStore.getState().removeImages([1, 3])

    const state = useImageStore.getState()
    expect(state.images).toHaveLength(1)
    expect(state.images[0].id).toBe(2)
    expect(state.selectedIds.includes(1)).toBe(false)
    expect(state.selectedIds.includes(2)).toBe(true)
  })

  it('setSelectedIds 应该设置选中的 ID 数组', async () => {
    const { useImageStore } = await import('../useImageStore')
    useImageStore.getState().setSelectedIds([10, 20])
    expect(useImageStore.getState().selectedIds).toEqual([10, 20])
  })

  it('toggleSelect 应该切换选中/取消选中', async () => {
    const { useImageStore } = await import('../useImageStore')

    useImageStore.setState({ selectedIds: [] })
    useImageStore.getState().toggleSelect(5)
    expect(useImageStore.getState().selectedIds.includes(5)).toBe(true)

    useImageStore.getState().toggleSelect(5)
    expect(useImageStore.getState().selectedIds.includes(5)).toBe(false)
  })

  it('selectAll 应该选中所有图片的 id', async () => {
    const { useImageStore } = await import('../useImageStore')
    const images = [
      { id: 1, file_path: '/a.jpg', file_name: 'a.jpg', thumbnail_path: '/t/a.webp', ai_status: 'completed' },
      { id: 2, file_path: '/b.jpg', file_name: 'b.jpg', thumbnail_path: '/t/b.webp', ai_status: 'completed' },
    ]

    useImageStore.setState({ images, selectedIds: [] })
    useImageStore.getState().selectAll()

    const { selectedIds } = useImageStore.getState()
    expect(selectedIds).toHaveLength(2)
    expect(selectedIds.includes(1)).toBe(true)
    expect(selectedIds.includes(2)).toBe(true)
  })

  it('deselectAll 应该清空所有选中', async () => {
    const { useImageStore } = await import('../useImageStore')
    useImageStore.setState({ selectedIds: [1, 2, 3] })
    useImageStore.getState().deselectAll()
    expect(useImageStore.getState().selectedIds).toHaveLength(0)
  })

  it('setFilters 应该合并过滤器并重置页码', async () => {
    const { useImageStore } = await import('../useImageStore')
    useImageStore.setState({ page: 5, filters: { category: 'nature' } })
    useImageStore.getState().setFilters({ ai_status: 'completed' })

    const state = useImageStore.getState()
    expect(state.filters.category).toBe('nature')
    expect(state.filters.ai_status).toBe('completed')
    expect(state.page).toBe(1)
  })

  it('setPage 和 setPageSize 应该正确更新', async () => {
    const { useImageStore } = await import('../useImageStore')

    useImageStore.getState().setPage(3)
    expect(useImageStore.getState().page).toBe(3)

    useImageStore.getState().setPageSize(100)
    expect(useImageStore.getState().pageSize).toBe(100)
    expect(useImageStore.getState().page).toBe(1)
  })

  it('setTotal 应该更新 total', async () => {
    const { useImageStore } = await import('../useImageStore')
    useImageStore.getState().setTotal(500)
    expect(useImageStore.getState().total).toBe(500)
  })

  it('setLoading 和 setError 应该正确更新', async () => {
    const { useImageStore } = await import('../useImageStore')

    useImageStore.getState().setLoading(true)
    expect(useImageStore.getState().loading).toBe(true)

    useImageStore.getState().setError('some error')
    expect(useImageStore.getState().error).toBe('some error')
  })

  it('setSearchQuery / setSearchResults / setSearchLoading / setHasSearched 应该正确更新', async () => {
    const { useImageStore } = await import('../useImageStore')
    const store = useImageStore.getState()

    store.setSearchQuery('sunset')
    expect(useImageStore.getState().searchQuery).toBe('sunset')

    store.setSearchResults([{ image_id: 1, file_path: '', file_name: '', match_count: 0, relevance_score: 0.9 }])
    expect(useImageStore.getState().searchResults).toHaveLength(1)

    store.setSearchLoading(true)
    expect(useImageStore.getState().searchLoading).toBe(true)

    store.setHasSearched(true)
    expect(useImageStore.getState().hasSearched).toBe(true)
  })

  it('clearSearch 应该重置所有搜索相关状态', async () => {
    const { useImageStore } = await import('../useImageStore')
    useImageStore.setState({
      searchQuery: 'test',
      searchResults: [{ image_id: 1, file_path: '', file_name: '', match_count: 0, relevance_score: 0 }],
      searchLoading: true,
      hasSearched: true,
    })
    useImageStore.getState().clearSearch()

    const state = useImageStore.getState()
    expect(state.searchQuery).toBe('')
    expect(state.searchResults).toEqual([])
    expect(state.searchLoading).toBe(false)
    expect(state.hasSearched).toBe(false)
  })
})

describe('useAIStore', () => {
  beforeEach(async () => {
    const { useAIStore } = await import('../useAIStore')
    useAIStore.getState().reset()
  })

  it('初始状态应该正确', async () => {
    const { useAIStore } = await import('../useAIStore')
    const state = useAIStore.getState()

    expect(state.status).toBe('idle')
    expect(state.total).toBe(0)
    expect(state.completed).toBe(0)
    expect(state.failed).toBe(0)
    expect(state.retrying).toBe(0)
    expect(state.eta_seconds).toBeUndefined()
  })

  it('setStatus 应该更新 AI 状态', async () => {
    const { useAIStore } = await import('../useAIStore')
    useAIStore.getState().setStatus('processing')
    expect(useAIStore.getState().status).toBe('processing')
  })

  it('setTotal / setCompleted / setFailed / setRetrying 应该正确更新', async () => {
    const { useAIStore } = await import('../useAIStore')
    const store = useAIStore.getState()

    store.setTotal(100)
    expect(useAIStore.getState().total).toBe(100)

    store.setCompleted(50)
    expect(useAIStore.getState().completed).toBe(50)

    store.setFailed(3)
    expect(useAIStore.getState().failed).toBe(3)

    store.setRetrying(2)
    expect(useAIStore.getState().retrying).toBe(2)
  })

  it('setETA 应该更新预计时间', async () => {
    const { useAIStore } = await import('../useAIStore')

    useAIStore.getState().setETA(120)
    expect(useAIStore.getState().eta_seconds).toBe(120)

    useAIStore.getState().setETA(undefined)
    expect(useAIStore.getState().eta_seconds).toBeUndefined()
  })

  it('updateStatus 应该批量更新部分状态', async () => {
    const { useAIStore } = await import('../useAIStore')
    useAIStore.getState().updateStatus({
      status: 'processing',
      total: 200,
      completed: 100,
    })

    const state = useAIStore.getState()
    expect(state.status).toBe('processing')
    expect(state.total).toBe(200)
    expect(state.completed).toBe(100)
  })

  it('reset 应该恢复所有状态到初始值', async () => {
    const { useAIStore } = await import('../useAIStore')
    useAIStore.setState({
      status: 'processing',
      total: 100,
      completed: 50,
      failed: 5,
      retrying: 2,
      eta_seconds: 60,
    })

    useAIStore.getState().reset()

    const state = useAIStore.getState()
    expect(state.status).toBe('idle')
    expect(state.total).toBe(0)
    expect(state.completed).toBe(0)
    expect(state.failed).toBe(0)
    expect(state.retrying).toBe(0)
    expect(state.eta_seconds).toBeUndefined()
  })

  it('所有状态值应正确切换', async () => {
    const { useAIStore } = await import('../useAIStore')
    const validStatuses = ['idle', 'processing', 'paused', 'completed', 'failed'] as const

    for (const s of validStatuses) {
      useAIStore.getState().setStatus(s)
      expect(useAIStore.getState().status).toBe(s)
    }
  })
})

describe('useConfigStore', () => {
  beforeEach(async () => {
    const { useConfigStore } = await import('../useConfigStore')
    useConfigStore.setState({
      lmStudioUrl: 'http://localhost:1234',
      aiConcurrency: 3,
      aiTimeout: 60,
      thumbnailSize: 300,
      theme: 'system',
      language: 'zh',
      notificationEnabled: true,
      notificationAiComplete: true,
      notificationDedupComplete: true,
      privacySendAnalytics: false,
      privacyShareData: false,
      isLoaded: false,
      pendingChanges: {},
    })
  })

  it('初始状态应该有正确的默认值', async () => {
    const { useConfigStore } = await import('../useConfigStore')
    const state = useConfigStore.getState()

    expect(state.lmStudioUrl).toBe('http://localhost:1234')
    expect(state.aiConcurrency).toBe(3)
    expect(state.aiTimeout).toBe(60)
    expect(state.thumbnailSize).toBe(300)
    expect(state.theme).toBe('system')
    expect(state.language).toBe('zh')
    expect(state.notificationEnabled).toBe(true)
    expect(state.notificationAiComplete).toBe(true)
    expect(state.notificationDedupComplete).toBe(true)
    expect(state.privacySendAnalytics).toBe(false)
    expect(state.privacyShareData).toBe(false)
    expect(state.isLoaded).toBe(false)
    expect(state.pendingChanges).toEqual({})
  })

  it('updateField 应该将变更加入 pendingChanges', async () => {
    const { useConfigStore } = await import('../useConfigStore')
    useConfigStore.setState({ pendingChanges: {} })

    useConfigStore.getState().updateField('theme', 'dark')
    expect(useConfigStore.getState().pendingChanges.theme).toBe('dark')

    useConfigStore.getState().updateField('language', 'en')
    expect(useConfigStore.getState().pendingChanges.language).toBe('en')
  })

  it('hasPendingChanges 应该正确反映是否有未保存的变更', async () => {
    const { useConfigStore } = await import('../useConfigStore')
    useConfigStore.setState({ pendingChanges: {} })

    expect(useConfigStore.getState().hasPendingChanges()).toBe(false)

    useConfigStore.getState().updateField('theme', 'dark')
    expect(useConfigStore.getState().hasPendingChanges()).toBe(true)
  })

  it('loadConfigs 应该从 API 加载配置并设置 isLoaded', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue([
      { key: 'theme', value: 'dark' },
      { key: 'language', value: 'en' },
      { key: 'ai_concurrency', value: '5' },
      { key: 'notification_enabled', value: 'false' },
    ])

    const { useConfigStore } = await import('../useConfigStore')
    await useConfigStore.getState().loadConfigs()

    const state = useConfigStore.getState()
    expect(state.isLoaded).toBe(true)
    expect(state.theme).toBe('dark')
    expect(state.language).toBe('en')
    expect(state.aiConcurrency).toBe(5)
    expect(state.notificationEnabled).toBe(false)
    expect(state.pendingChanges).toEqual({})
  })

  it('loadConfigs 失败时应设置 isLoaded 为 true 并保留默认值', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockRejectedValue(new Error('Network error'))

    const { useConfigStore } = await import('../useConfigStore')
    await useConfigStore.getState().loadConfigs()

    expect(useConfigStore.getState().isLoaded).toBe(true)
    expect(useConfigStore.getState().theme).toBe('system')
  })

  it('saveConfigs 应该保存变更并清除 pendingChanges', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue(undefined)

    const { useConfigStore } = await import('../useConfigStore')
    useConfigStore.setState({ pendingChanges: {} })

    useConfigStore.getState().updateField('theme', 'dark')
    useConfigStore.getState().updateField('ai_concurrency', '8')

    await useConfigStore.getState().saveConfigs()

    const state = useConfigStore.getState()
    expect(state.pendingChanges).toEqual({})
    expect(state.theme).toBe('dark')
    expect(state.aiConcurrency).toBe(8)
  })

  it('saveConfigs 在无变更时不应调用 API', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockClear()

    const { useConfigStore } = await import('../useConfigStore')
    useConfigStore.setState({ pendingChanges: {} })

    await useConfigStore.getState().saveConfigs()

    expect(mockInvoke).not.toHaveBeenCalled()
  })
})

describe('useThemeStore', () => {
  beforeEach(() => {
    document.documentElement.className = ''
  })

  it('应该有 applyTheme 方法', async () => {
    const { useThemeStore } = await import('../useThemeStore')
    expect(typeof useThemeStore.getState().applyTheme).toBe('function')
  })

  it('applyTheme("dark") 应该添加 dark 类并移除 light', async () => {
    const { useThemeStore } = await import('../useThemeStore')
    useThemeStore.getState().applyTheme('dark')

    expect(document.documentElement.classList.contains('dark')).toBe(true)
    expect(document.documentElement.classList.contains('light')).toBe(false)
  })

  it('applyTheme("light") 应该添加 light 类并移除 dark', async () => {
    const { useThemeStore } = await import('../useThemeStore')
    document.documentElement.classList.add('dark')
    useThemeStore.getState().applyTheme('light')

    expect(document.documentElement.classList.contains('light')).toBe(true)
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })

  it('applyTheme("system") 应该根据 prefers-color-scheme 设置', async () => {
    const { useThemeStore } = await import('../useThemeStore')

    useThemeStore.getState().applyTheme('system')
    expect(document.documentElement.classList.contains('light')).toBe(true)
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })
})

describe('useDedupStore', () => {
  beforeEach(async () => {
    const { useDedupStore } = await import('../useDedupStore')
    useDedupStore.getState().reset()
  })

  it('初始状态应该正确', async () => {
    const { useDedupStore } = await import('../useDedupStore')
    const state = useDedupStore.getState()

    expect(state.groups).toEqual([])
    expect(state.loading).toBe(false)
    expect(state.threshold).toBe(95)
  })

  it('setGroups 应该设置重复组列表', async () => {
    const { useDedupStore } = await import('../useDedupStore')
    const mockGroups = [
      {
        id: 'group-1',
        images: [
          { image_id: 1, file_path: '/a.jpg', file_name: 'a.jpg', file_size: 1000, phash: 'abc', distance: 0 },
          { image_id: 2, file_path: '/b.jpg', file_name: 'b.jpg', file_size: 1000, phash: 'abc', distance: 2 },
        ],
        image_ids: [1, 2],
        similarity: 98,
      },
    ]

    useDedupStore.getState().setGroups(mockGroups)
    expect(useDedupStore.getState().groups).toHaveLength(1)
    expect(useDedupStore.getState().groups[0].id).toBe('group-1')
  })

  it('setLoading 应该更新加载状态', async () => {
    const { useDedupStore } = await import('../useDedupStore')
    useDedupStore.getState().setLoading(true)
    expect(useDedupStore.getState().loading).toBe(true)
  })

  it('setThreshold 应该更新阈值', async () => {
    const { useDedupStore } = await import('../useDedupStore')
    useDedupStore.getState().setThreshold(85)
    expect(useDedupStore.getState().threshold).toBe(85)
  })

  it('removeGroups 应该按 id 移除指定组', async () => {
    const { useDedupStore } = await import('../useDedupStore')
    useDedupStore.setState({
      groups: [
        { id: 'g1', images: [], image_ids: [], similarity: 95 },
        { id: 'g2', images: [], image_ids: [], similarity: 90 },
        { id: 'g3', images: [], image_ids: [], similarity: 85 },
      ],
    })

    useDedupStore.getState().removeGroups(['g1', 'g3'])
    expect(useDedupStore.getState().groups).toHaveLength(1)
    expect(useDedupStore.getState().groups[0].id).toBe('g2')
  })

  it('reset 应该恢复所有状态到初始值', async () => {
    const { useDedupStore } = await import('../useDedupStore')
    useDedupStore.setState({
      groups: [{ id: 'g1', images: [], image_ids: [], similarity: 95 }],
      loading: true,
      threshold: 80,
    })

    useDedupStore.getState().reset()

    const state = useDedupStore.getState()
    expect(state.groups).toEqual([])
    expect(state.loading).toBe(false)
    expect(state.threshold).toBe(95)
  })
})
