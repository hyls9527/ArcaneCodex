import React from 'react'
import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { MainLayout } from './MainLayout'
import { Sidebar } from './Sidebar'
import { TopBar } from './TopBar'

vi.mock('@/stores/useThemeStore', () => ({
  useThemeStore: vi.fn(() => ({ applyTheme: vi.fn() })),
}))

vi.mock('@/stores/useConfigStore', () => ({
  useConfigStore: vi.fn(() => ({ 
    theme: 'light', 
    language: 'zh', 
    pendingChanges: {},
    updateField: vi.fn() 
  })),
  CONFIG_KEYS: { THEME: 'theme', LANGUAGE: 'language' },
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'navigation.gallery': '图库',
        'navigation.aiTagging': 'AI 打标',
        'navigation.dedup': '去重',
        'navigation.settings': '设置',
        'navigation.expandSidebar': '展开侧边栏',
        'navigation.collapseSidebar': '折叠侧边栏',
        'dashboard.title': '数据仪表盘',
        'topBar.searchPlaceholder': '搜索图片...',
        'topBar.toggleLanguage': '切换语言',
        'topBar.toggleTheme': '切换主题',
      }
      return translations[key] || key
    },
    i18n: { changeLanguage: vi.fn(), language: 'zh' },
  }),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(vi.fn())),
  emit: vi.fn(() => Promise.resolve()),
}))

vi.mock('@/router/events', () => ({
  navigate: vi.fn(),
}))

describe('布局组件单元测试', () => {
  describe('MainLayout', () => {
    it('应该正确渲染子组件', () => {
      render(
        <MainLayout>
          <div data-testid="child-content">Test Content</div>
        </MainLayout>
      )
      expect(screen.getByTestId('child-content')).toBeInTheDocument()
    })

    it('应该应用正确的 CSS 类', () => {
      const { container } = render(<MainLayout>Content</MainLayout>)
      const root = container.firstChild as HTMLElement
      expect(root).toHaveClass('flex', 'h-screen', 'w-screen')
    })

    it('应该支持自定义 className', () => {
      const { container } = render(<MainLayout className="custom-class">Content</MainLayout>)
      const root = container.firstChild as HTMLElement
      expect(root).toHaveClass('custom-class')
    })
  })

  describe('Sidebar', () => {
    it('应该渲染所有导航项', () => {
      render(<Sidebar currentPage="gallery" />)
      expect(screen.getByText('图库')).toBeInTheDocument()
      expect(screen.getByText('AI 打标')).toBeInTheDocument()
      expect(screen.getByText('去重')).toBeInTheDocument()
      expect(screen.getByText('设置')).toBeInTheDocument()
    })

    it('应该高亮当前页面', () => {
      render(<Sidebar currentPage="ai" />)
      const aiButton = screen.getByText('AI 打标').closest('button')
      expect(aiButton).toHaveClass('bg-primary-50')
    })

    it('应该支持折叠功能', async () => {
      render(<Sidebar currentPage="gallery" />)
      expect(screen.getByText('图库')).toBeInTheDocument()
      
      const collapseButton = screen.getByRole('button', { name: '折叠侧边栏' })
      await fireEvent.click(collapseButton)
      
      await waitFor(() => {
        expect(screen.queryByText('图库')).not.toBeInTheDocument()
      })
    })
  })

  describe('TopBar', () => {
    it('应该渲染搜索输入框', () => {
      render(<TopBar />)
      expect(screen.getByPlaceholderText('搜索图片...')).toBeInTheDocument()
    })

    it('应该渲染主题切换按钮', () => {
      render(<TopBar />)
      const themeButton = screen.getByRole('button', { name: '切换主题' })
      expect(themeButton).toBeInTheDocument()
    })

    it('应该在搜索框输入时更新值', () => {
      render(<TopBar />)
      const input = screen.getByPlaceholderText('搜索图片...') as HTMLInputElement
      input.value = 'test query'
      expect(input.value).toBe('test query')
    })
  })
})
