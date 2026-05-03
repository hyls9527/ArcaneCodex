import { Search, Sun, Moon, Monitor, Languages } from 'lucide-react'
import { useState, useEffect, useRef } from 'react'
import { useThemeStore } from '@/stores/useThemeStore'
import type { Theme } from '@/stores/useThemeStore'
import { useTranslation } from 'react-i18next'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'
import { cn } from '@/utils/cn'

interface TopBarProps {
  onSearch?: (query: string) => void
  searchQuery?: string
}

function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value)

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay)
    return () => clearTimeout(timer)
  }, [value, delay])

  return debouncedValue
}

export function TopBar({ onSearch, searchQuery: externalQuery }: TopBarProps) {
  const [searchQuery, setSearchQuery] = useState(externalQuery ?? '')
  const [showLangMenu, setShowLangMenu] = useState(false)
  const { theme, pendingChanges, updateField } = useConfigStore()
  const { applyTheme } = useThemeStore()
  const { i18n, t } = useTranslation()
  const { language } = useConfigStore()

  const debouncedQuery = useDebounce(searchQuery, 300)
  const prevDebouncedQuery = useRef(debouncedQuery)

  useEffect(() => {
    if (externalQuery !== undefined) {
      setSearchQuery(externalQuery)
    }
  }, [externalQuery])

  useEffect(() => {
    if (prevDebouncedQuery.current !== debouncedQuery && onSearch) {
      prevDebouncedQuery.current = debouncedQuery
      onSearch(debouncedQuery)
    }
  }, [debouncedQuery, onSearch])

  // Sync i18n with persisted language on mount
  useEffect(() => {
    if (language) {
      i18n.changeLanguage(language)
    }
  }, [language, i18n])

  const cycleTheme = () => {
    const themes: Theme[] = ['light', 'dark', 'system']
    const rawTheme = (pendingChanges[CONFIG_KEYS.THEME] as string) || theme
    const currentTheme: Theme = (themes.includes(rawTheme as Theme) ? rawTheme : theme) as Theme
    const currentIndex = themes.indexOf(currentTheme)
    const nextTheme = themes[(currentIndex + 1) % themes.length]
    applyTheme(nextTheme)
    updateField(CONFIG_KEYS.THEME, nextTheme)
  }

  const getThemeIcon = () => {
    const currentTheme = (pendingChanges[CONFIG_KEYS.THEME] as Theme) || theme
    switch (currentTheme) {
      case 'light': return <Sun className="w-5 h-5" />
      case 'dark': return <Moon className="w-5 h-5" />
      case 'system': return <Monitor className="w-5 h-5" />
    }
  }

  const switchLanguage = (lng: string) => {
    i18n.changeLanguage(lng)
    setShowLangMenu(false)
    updateField(CONFIG_KEYS.LANGUAGE, lng)
  }
  
  return (
    <header className="flex items-center gap-4 p-3 bg-white dark:bg-dark-100 border-b border-gray-200 dark:border-gray-700">
      <div className="flex-1 relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          type="text"
          placeholder={t('topBar.searchPlaceholder')}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2 rounded-lg bg-gray-100 dark:bg-dark-200 
                     border-none focus:ring-2 focus:ring-primary-500 outline-none"
        />
      </div>
      
      {/* Language Switcher */}
      <div className="relative">
        <button
          onClick={() => setShowLangMenu(!showLangMenu)}
          data-testid="lang-toggle"
          className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100"
          aria-label={t('topBar.toggleLanguage')}
        >
          <Languages className="w-5 h-5" />
        </button>
        
        {showLangMenu && (
          <div className="absolute right-0 top-full mt-2 bg-white dark:bg-dark-100 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50 min-w-[120px]">
            <button
              onClick={() => switchLanguage('zh')}
              data-testid="lang-zh"
              className={cn(
                'w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors',
                'focus:outline-none focus:bg-gray-100 dark:focus:bg-gray-700',
                language === 'zh' && 'text-primary-600 dark:text-primary-400 font-medium'
              )}
            >
              中文
            </button>
            <button
              onClick={() => switchLanguage('en')}
              data-testid="lang-en"
              className={cn(
                'w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors',
                'focus:outline-none focus:bg-gray-100 dark:focus:bg-gray-700',
                language === 'en' && 'text-primary-600 dark:text-primary-400 font-medium'
              )}
            >
              English
            </button>
          </div>
        )}
      </div>
      
      <button
        onClick={cycleTheme}
        data-testid="theme-toggle"
        className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100"
        aria-label={t('topBar.toggleTheme')}
      >
        {getThemeIcon()}
      </button>
    </header>
  )
}
