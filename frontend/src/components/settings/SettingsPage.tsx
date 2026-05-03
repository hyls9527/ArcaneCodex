import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import i18n from '@/i18n'
import { Settings as SettingsIcon, Brain, Palette, Database, Info, FileText, Bell, Shield } from 'lucide-react'
import { cn } from '@/utils/cn'
import { AIConfig } from './AIConfig'
import { DisplayConfig } from './DisplayConfig'
import { StorageConfig } from './StorageConfig'
import { AboutPage } from './AboutPage'
import { LogViewer } from './LogViewer'
import { NotificationConfig } from './NotificationConfig'
import { PrivacyConfig } from './PrivacyConfig'
import { useConfigStore } from '@/stores/useConfigStore'
import { useThemeStore } from '@/stores/useThemeStore'
import type { Theme } from '@/stores/useThemeStore'

type SettingsTab = 'ai' | 'display' | 'storage' | 'notifications' | 'privacy' | 'logs' | 'about'

const tabKeys = [
  { id: 'ai' as const, icon: Brain, i18nKey: 'settings.ai.title' },
  { id: 'display' as const, icon: Palette, i18nKey: 'settings.display.title' },
  { id: 'storage' as const, icon: Database, i18nKey: 'settings.storage.title' },
  { id: 'notifications' as const, icon: Bell, i18nKey: 'settings.notifications.title' },
  { id: 'privacy' as const, icon: Shield, i18nKey: 'settings.privacy.title' },
  { id: 'logs' as const, icon: FileText, i18nKey: 'logs.title' },
  { id: 'about' as const, icon: Info, i18nKey: 'settings.about.title' },
]

export function SettingsPage() {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState<SettingsTab>('ai')
  const [isSaving, setIsSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  const { hasPendingChanges, isLoaded, loadConfigs, saveConfigs } = useConfigStore()
  const { applyTheme } = useThemeStore()

  useEffect(() => {
    if (isLoaded) return
    loadConfigs()
      .then((configs) => {
        if (configs?.theme) {
          applyTheme(configs.theme as Theme)
        }
        if (configs?.language) {
          i18n.changeLanguage(configs.language)
        }
      })
      .catch(() => {
      })
  }, [isLoaded, loadConfigs, applyTheme])

  const handleSave = async () => {
    setIsSaving(true)
    setSaveError(null)
    try {
      await saveConfigs()
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : t('settings.saveFailed'))
    } finally {
      setIsSaving(false)
    }
  }

  const ActiveComponent = {
    ai: AIConfig,
    display: DisplayConfig,
    storage: StorageConfig,
    notifications: NotificationConfig,
    privacy: PrivacyConfig,
    logs: LogViewer,
    about: AboutPage,
  }[activeTab]

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="flex items-center gap-3 mb-6">
        <SettingsIcon className="w-6 h-6" />
        <h1 className="text-2xl font-bold">{t('settings.title')}</h1>
      </div>

      <div className="flex gap-6">
        {/* Sidebar Tabs */}
        <nav className="w-48 flex-shrink-0">
          <ul className="space-y-1">
            {tabKeys.map(({ id, icon: Icon, i18nKey }) => (
              <li key={id}>
                <button
                  onClick={() => setActiveTab(id)}
                  data-testid={`settings-tab-${id}`}
                  role="tab"
                  aria-selected={activeTab === id}
                  className={cn(
                    'flex items-center gap-3 w-full px-3 py-2 rounded-lg text-left transition-colors',
                    'hover:bg-gray-100 dark:hover:bg-gray-800',
                    'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100',
                    activeTab === id && 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
                  )}
                >
                  <Icon className="w-4 h-4 flex-shrink-0" />
                  <span className="text-sm">{t(i18nKey)}</span>
                </button>
              </li>
            ))}
          </ul>
        </nav>

        {/* Content Area */}
        <div className="flex-1">
          <div className="bg-white dark:bg-dark-100 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
            <ActiveComponent />
          </div>

          {/* Save Button */}
          {hasPendingChanges() && activeTab !== 'about' && (
            <div className="mt-4 flex items-center gap-4">
              {saveError && (
                <p className="text-sm text-red-600">{saveError}</p>
              )}
              <button
                onClick={handleSave}
                disabled={isSaving}
                className="btn-primary ml-auto disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSaving ? t('common.saving') : t('common.saveChanges')}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
