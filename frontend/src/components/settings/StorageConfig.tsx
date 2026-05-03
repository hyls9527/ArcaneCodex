import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Database, FolderOpen, Download, Upload, Loader2, CheckCircle, AlertCircle } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { open as openPath } from '@tauri-apps/plugin-shell'
import { backupDatabase, restoreDatabase } from '@/lib/api'

export function StorageConfig() {
  const { t } = useTranslation()
  const appDataPath = '%APPDATA%\\ArcaneCodex'
  
  const [backingUp, setBackingUp] = useState(false)
  const [backupResult, setBackupResult] = useState<'success' | 'error' | null>(null)
  const [restoring, setRestoring] = useState(false)
  const [restoreResult, setRestoreResult] = useState<'success' | 'error' | null>(null)

  const handleBackup = async () => {
    setBackingUp(true)
    setBackupResult(null)
    try {
      const timestamp = Date.now()
      const filename = `ArcaneCodex_backup_${timestamp}.zip`
      await backupDatabase(filename)
      setBackupResult('success')
    } catch {
      setBackupResult('error')
    } finally {
      setBackingUp(false)
    }
  }

  const handleRestore = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Backup', extensions: ['zip'] }],
    })
    if (!selected) return

    setRestoring(true)
    setRestoreResult(null)
    try {
      await restoreDatabase(selected as string)
      setRestoreResult('success')
    } catch {
      setRestoreResult('error')
    } finally {
      setRestoring(false)
    }
  }

  const handleOpenDirectory = async () => {
    try {
      await openPath(appDataPath)
    } catch {
    }
  }
  
  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Database className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.storage.title')}</h2>
      </div>
      
      <div className="space-y-6">
        {/* Data Directory */}
        <div>
          <label className="block text-sm font-medium mb-2">{t('settings.storage.dataDirectory')}</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={appDataPath}
              readOnly
              className="flex-1 px-3 py-2 bg-gray-100 dark:bg-dark-200 rounded-lg border border-gray-300 dark:border-gray-600"
            />
            <button onClick={handleOpenDirectory} className="btn-secondary flex items-center gap-2">
              <FolderOpen className="w-4 h-4" />
              {t('settings.storage.openDirectory')}
            </button>
          </div>
        </div>
        
        {/* Backup & Restore */}
        <div className="grid grid-cols-2 gap-4">
          <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
            <Download className="w-6 h-6 text-green-600 mb-2" />
            <h3 className="font-medium mb-1">{t('settings.storage.backup')}</h3>
            <p className="text-xs text-gray-600 dark:text-gray-400 mb-3">
              {t('settings.storage.backupDesc')}
            </p>
            <button 
              onClick={handleBackup}
              disabled={backingUp}
              className="btn-primary bg-green-600 hover:bg-green-700 w-full flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {backingUp ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  {t('settings.storage.backupProgress')}
                </>
              ) : backupResult === 'success' ? (
                <>
                  <CheckCircle className="w-4 h-4" />
                  {t('settings.storage.backupComplete')}
                </>
              ) : (
                t('settings.storage.backupExport')
              )}
            </button>
            
            {backupResult === 'error' && (
              <p className="mt-2 text-xs text-red-600 flex items-center gap-1">
                <AlertCircle className="w-3 h-3" />
                {t('settings.storage.backupFailed')}
              </p>
            )}
            {backupResult === 'success' && (
              <p className="mt-2 text-xs text-green-700 dark:text-green-400 flex items-center gap-1">
                <CheckCircle className="w-3 h-3" />
                {t('settings.storage.backupSuccess')}
              </p>
            )}
          </div>
          
          <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
            <Upload className="w-6 h-6 text-blue-600 mb-2" />
            <h3 className="font-medium mb-1">{t('settings.storage.restore')}</h3>
            <p className="text-xs text-gray-600 dark:text-gray-400 mb-3">
              {t('settings.storage.restoreDesc')}
            </p>
            <button 
              onClick={handleRestore}
              disabled={restoring}
              className="btn-primary bg-blue-600 hover:bg-blue-700 w-full flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {restoring ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  {t('settings.storage.restoreProgress')}
                </>
              ) : restoreResult === 'success' ? (
                <>
                  <CheckCircle className="w-4 h-4" />
                  {t('settings.storage.restoreComplete')}
                </>
              ) : (
                t('settings.storage.restoreImport')
              )}
            </button>
            
            {restoreResult === 'error' && (
              <p className="mt-2 text-xs text-red-600 flex items-center gap-1">
                <AlertCircle className="w-3 h-3" />
                {t('settings.storage.restoreFailed')}
              </p>
            )}
            {restoreResult === 'success' && (
              <p className="mt-2 text-xs text-blue-700 dark:text-blue-400">
                {t('settings.storage.restoreSuccess')}
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
