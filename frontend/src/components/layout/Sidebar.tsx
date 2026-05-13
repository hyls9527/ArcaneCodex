import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { 
  Image as ImageIcon, 
  Settings, 
  Sparkles, 
  Copy,
  BarChart3,
  Network,
  ChevronLeft,
  ChevronRight
} from 'lucide-react'
import { cn } from '@/utils/cn'
import type { Page } from '@/types/image'
import { navigate } from '@/router/events'

interface SidebarProps {
  currentPage: Page
}

export function Sidebar({ currentPage }: SidebarProps) {
  const { t } = useTranslation()
  const [collapsed, setCollapsed] = useState(false)
  
  const navItems = [
    { id: 'gallery' as const, label: t('navigation.gallery'), icon: ImageIcon },
    { id: 'ai' as const, label: t('navigation.aiTagging'), icon: Sparkles },
    { id: 'dedup' as const, label: t('navigation.dedup'), icon: Copy },
    { id: 'knowledge_graph' as const, label: 'Knowledge Graph', icon: Network },
    { id: 'dashboard' as const, label: t('dashboard.title'), icon: BarChart3 },
    { id: 'settings' as const, label: t('navigation.settings'), icon: Settings },
  ]
  
  const handleNavigate = (page: Page) => {
    if (import.meta.env.DEV) {
      console.debug('[Sidebar] navigate to:', page)
    }
    try {
      navigate({ route: page, source: 'sidebar' })
      if (import.meta.env.DEV) {
        console.debug('[Sidebar] navigate() resolved')
      }
    } catch (err) {
      console.error('[Sidebar] navigate() threw:', err)
    }
  }
  
  return (
    <aside className={cn(
      'flex flex-col bg-white dark:bg-dark-100 border-r border-gray-200 dark:border-gray-700 transition-all duration-300',
      collapsed ? 'w-16' : 'w-64'
    )}>
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        {!collapsed && (
          <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
            Arcane Codex
          </h1>
        )}
        <button
          onClick={() => setCollapsed(!collapsed)}
          data-testid="sidebar-collapse"
          className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100"
          aria-label={collapsed ? t('navigation.expandSidebar') : t('navigation.collapseSidebar')}
        >
          {collapsed ? (
            <ChevronRight className="w-5 h-5" />
          ) : (
            <ChevronLeft className="w-5 h-5" />
          )}
        </button>
      </div>
      
      <nav className="flex-1 p-2 space-y-1">
        {navItems.map(({ id, label, icon: Icon }) => {
          const isActive = currentPage === id
          return (
            <button
              key={id}
              onClick={() => handleNavigate(id)}
              aria-label={label}
              data-testid={`nav-${id}`}
              className={cn(
                'group relative flex items-center w-full gap-3 px-3 py-2.5 rounded-xl transition-all duration-200',
                'hover:bg-gray-100 dark:hover:bg-gray-700',
                'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100',
                isActive && 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400 font-medium shadow-sm',
                collapsed && 'justify-center'
              )}
            >
              {/* Active indicator bar */}
              {isActive && (
                <span className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 rounded-r-full bg-primary-500" />
              )}
              <Icon className={cn(
                'w-5 h-5 flex-shrink-0 transition-transform duration-200',
                isActive && 'scale-110'
              )} />
              {!collapsed && <span>{label}</span>}
              {/* Collapsed tooltip */}
              {collapsed && (
                <span className="absolute left-full ml-2 px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-xs rounded-lg opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50 shadow-lg">
                  {label}
                </span>
              )}
            </button>
          )
        })}
      </nav>
    </aside>
  )
}
