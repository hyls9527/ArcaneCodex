import { useTranslation } from 'react-i18next'
import { Bell } from 'lucide-react'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'

export function NotificationConfig() {
  const { t } = useTranslation()
  const { notificationEnabled, notificationAiComplete, notificationDedupComplete, updateField } = useConfigStore()

  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Bell className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.notifications.title')}</h2>
      </div>

      <div className="space-y-6">
        {/* 全局通知开关 */}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t('settings.notifications.enabled')}</p>
            <p className="text-xs text-gray-500">{t('settings.notifications.enabledDesc')}</p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={notificationEnabled}
              onChange={() => updateField(CONFIG_KEYS.NOTIFICATION_ENABLED, (!notificationEnabled).toString())}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-500/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        {/* AI 打标完成通知 */}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t('settings.notifications.aiComplete')}</p>
            <p className="text-xs text-gray-500">{t('settings.notifications.aiCompleteDesc')}</p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={notificationAiComplete}
              onChange={() => updateField(CONFIG_KEYS.NOTIFICATION_AI_COMPLETE, (!notificationAiComplete).toString())}
              className="sr-only peer"
              disabled={!notificationEnabled}
            />
            <div className={`w-11 h-6 bg-gray-200 rounded-full peer after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all ${
              notificationAiComplete ? 'after:translate-x-full peer-checked:bg-primary-500' : ''
            } ${!notificationEnabled ? 'opacity-50 cursor-not-allowed' : ''}`}></div>
          </label>
        </div>

        {/* 去重完成通知 */}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t('settings.notifications.dedupComplete')}</p>
            <p className="text-xs text-gray-500">{t('settings.notifications.dedupCompleteDesc')}</p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={notificationDedupComplete}
              onChange={() => updateField(CONFIG_KEYS.NOTIFICATION_DEDUP_COMPLETE, (!notificationDedupComplete).toString())}
              className="sr-only peer"
              disabled={!notificationEnabled}
            />
            <div className={`w-11 h-6 bg-gray-200 rounded-full peer after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all ${
              notificationDedupComplete ? 'after:translate-x-full peer-checked:bg-primary-500' : ''
            } ${!notificationEnabled ? 'opacity-50 cursor-not-allowed' : ''}`}></div>
          </label>
        </div>
      </div>
    </div>
  )
}
