import { create } from 'zustand'
import { useConfigStore } from './useConfigStore'

export type Theme = 'light' | 'dark' | 'system'

interface ThemeStore {
  applyTheme: (theme: Theme) => void
}

function applyTheme(theme: Theme) {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
    document.documentElement.classList.remove('light')
  } else if (theme === 'light') {
    document.documentElement.classList.remove('dark')
    document.documentElement.classList.add('light')
  } else {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    document.documentElement.classList.toggle('dark', prefersDark)
    document.documentElement.classList.toggle('light', !prefersDark)
  }
}

export const useThemeStore = create<ThemeStore>()(() => ({
  applyTheme,
}))

if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const { theme } = useConfigStore.getState()
    if (theme === 'system') {
      applyTheme('system')
    }
  })
}
