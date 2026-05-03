import { useTranslation } from 'react-i18next'
import { Info, Github, BookOpen, Shield } from 'lucide-react'

export function AboutPage() {
  const { t } = useTranslation()
  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Info className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.about.title')}</h2>
      </div>
      
      <div className="space-y-6">
        {/* App Info */}
        <div className="text-center py-6">
          <div className="w-20 h-20 mx-auto mb-4 bg-gradient-to-br from-primary-500 to-primary-700 rounded-2xl flex items-center justify-center">
            <span className="text-3xl font-bold text-white">AC</span>
          </div>
          <h1 className="text-2xl font-bold mb-1">Arcane Codex</h1>
          <p className="text-gray-600 dark:text-gray-400">{t('settings.about.subtitle')}</p>
          <p className="text-sm text-gray-500 mt-2">{t('settings.about.version')} 1.0.0</p>
        </div>
        
        {/* Links */}
        <div className="grid grid-cols-2 gap-4">
          <a
            href="https://github.com/hyls9527/arcane-codex"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-dark-200 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-100 transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-dark-100"
          >
            <Github className="w-5 h-5" />
            <div>
              <div className="font-medium">{t('settings.about.github')}</div>
              <div className="text-xs text-gray-500">{t('settings.about.viewSource')}</div>
            </div>
          </a>
          
          <a
            href="https://github.com/hyls9527/arcane-codex#readme"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-dark-200 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-100 transition-colors"
          >
            <BookOpen className="w-5 h-5" />
            <div>
              <div className="font-medium">{t('settings.about.documentation')}</div>
              <div className="text-xs text-gray-500">{t('settings.about.userGuide')}</div>
            </div>
          </a>
        </div>
        
        {/* License */}
        <div className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-dark-200 rounded-lg">
          <Shield className="w-5 h-5" />
          <div>
            <div className="font-medium">{t('settings.about.license')}</div>
            <div className="text-xs text-gray-500">{t('settings.about.licenseDesc')}</div>
          </div>
        </div>
        
        {/* Tech Stack */}
        <div>
          <h3 className="font-medium mb-2">{t('settings.about.techStack')}</h3>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">{t('settings.about.techFrontend')}</span>
              <span className="font-mono">React 18</span>
            </div>
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">{t('settings.about.techBackend')}</span>
              <span className="font-mono">Tauri 2.x</span>
            </div>
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">{t('settings.about.techDatabase')}</span>
              <span className="font-mono">SQLite</span>
            </div>
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">{t('settings.about.techAI')}</span>
              <span className="font-mono">LM Studio</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
