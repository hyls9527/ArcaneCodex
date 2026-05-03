import { useTranslation } from 'react-i18next'
import { Shield } from 'lucide-react'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'

export function PrivacyConfig() {
  const { t } = useTranslation()
  const { privacySendAnalytics, privacyShareData, updateField } = useConfigStore()

  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Shield className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.privacy.title')}</h2>
      </div>

      <div className="space-y-6">
        {/* 发送分析数据 */}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t('settings.privacy.sendAnalytics')}</p>
            <p className="text-xs text-gray-500">{t('settings.privacy.sendAnalyticsDesc')}</p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={privacySendAnalytics}
              onChange={() => updateField(CONFIG_KEYS.PRIVACY_SEND_ANALYTICS, (!privacySendAnalytics).toString())}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-500/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        {/* 共享使用数据 */}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t('settings.privacy.shareData')}</p>
            <p className="text-xs text-gray-500">{t('settings.privacy.shareDataDesc')}</p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={privacyShareData}
              onChange={() => updateField(CONFIG_KEYS.PRIVACY_SHARE_DATA, (!privacyShareData).toString())}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-500/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        {/* 隐私提示 */}
        <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
          <p className="text-sm text-blue-800 dark:text-blue-300">
            {t('settings.privacy.note')}
          </p>
        </div>
      </div>
    </div>
  )
}
