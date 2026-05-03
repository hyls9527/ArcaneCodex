import { useTranslation } from 'react-i18next'
import { Palette } from 'lucide-react'
import { useThemeStore } from '@/stores/useThemeStore'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'

interface DisplayConfigProps {
  onChange?: () => void
}

export function DisplayConfig({ onChange }: DisplayConfigProps) {
  const { t } = useTranslation()
  const { theme: currentTheme, thumbnailSize, updateField } = useConfigStore()
  const { applyTheme } = useThemeStore()

  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Palette className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.display.title')}</h2>
      </div>

      <div className="space-y-6">
        {/* Theme Selector */}
        <div>
          <label className="block text-sm font-medium mb-2">{t('settings.display.theme')}</label>
          <div className="grid grid-cols-3 gap-3">
            {[
              { id: 'light' as const, i18nKey: 'settings.display.themeLight', preview: 'bg-white border-gray-300' },
              { id: 'dark' as const, i18nKey: 'settings.display.themeDark', preview: 'bg-gray-900 border-gray-700' },
              { id: 'system' as const, i18nKey: 'settings.display.themeSystem', preview: 'bg-gradient-to-br from-white to-gray-900 border-gray-400' },
            ].map(({ id, i18nKey, preview }) => (
              <button
                key={id}
                onClick={() => {
                  updateField(CONFIG_KEYS.THEME, id)
                  applyTheme(id)
                  onChange?.()
                }}
                className={`p-3 rounded-lg border-2 transition-all focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100 ${
                  currentTheme === id ? 'border-primary-500 ring-2 ring-primary-500/30' : 'border-gray-200 dark:border-gray-700'
                }`}
              >
                <div className={`h-12 rounded ${preview} border mb-2`} />
                <span className="text-sm">{t(i18nKey)}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Thumbnail Size */}
        <div>
          <label className="block text-sm font-medium mb-2">{t('settings.display.thumbnailSize')}</label>
          <select
            value={thumbnailSize}
            onChange={(e) => {
              updateField(CONFIG_KEYS.THUMBNAIL_SIZE, e.target.value)
              onChange?.()
            }}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
          >
            <option value="200">{t('settings.display.sizeSmall')}</option>
            <option value="300">{t('settings.display.sizeMedium')}</option>
            <option value="400">{t('settings.display.sizeLarge')}</option>
          </select>
        </div>
      </div>
    </div>
  )
}
