import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Brain, CheckCircle, AlertCircle, Loader2, Search, Circle } from 'lucide-react'
import { cn } from '@/utils/cn'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'
import { invoke } from '@tauri-apps/api/core'

const PROVIDER_OPTIONS = [
  { value: 'lm_studio', label: 'LM Studio', type: 'local', defaultUrl: 'http://127.0.0.1:1234', defaultModel: 'Qwen2.5-VL-7B-Instruct' },
  { value: 'ollama', label: 'Ollama', type: 'local', defaultUrl: 'http://127.0.0.1:11434', defaultModel: 'llava:7b' },
  { value: 'openai', label: 'OpenAI', type: 'cloud', defaultUrl: '', defaultModel: 'gpt-4o' },
  { value: 'openrouter', label: 'OpenRouter', type: 'cloud', defaultUrl: '', defaultModel: '' },
]

interface DiscoveredModel {
  provider: string
  provider_name: string
  base_url: string
  model_id: string
  model_name: string | null
  port: number
  is_online: boolean
  /** Whether the model is currently loaded into VRAM and ready for inference */
  loaded: boolean
}

function isValidUrl(url: string): boolean {
  if (!url || url.trim().length === 0) return true
  try {
    const parsed = new URL(url)
    return parsed.protocol === 'http:' || parsed.protocol === 'https:'
  } catch {
    return false
  }
}

export function AIConfig({ onChange }: { onChange?: () => void }) {
  const { t } = useTranslation()
  const { lmStudioUrl, aiConcurrency, aiTimeout, updateField } = useConfigStore()

  const [provider, setProvider] = useState('lm_studio')
  const [model, setModel] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)
  const [testDetail, setTestDetail] = useState<string>('')
  const [discovering, setDiscovering] = useState(false)
  const [discoveredModels, setDiscoveredModels] = useState<DiscoveredModel[]>([])
  const [discoverError, setDiscoverError] = useState<string | null>(null)
  const [urlValidationError, setUrlValidationError] = useState<string | null>(null)

  useEffect(() => {
    loadInferenceConfig()
  }, [])

  const loadInferenceConfig = async () => {
    try {
      const config = await invoke<{ provider: string; model: string; api_key: string }>('get_inference_config')
      setProvider(config.provider)
      setModel(config.model || '')
      setApiKey(config.api_key || '')
    } catch {
      // use defaults
    }
  }

  const handleProviderChange = (value: string) => {
    setProvider(value)
    const opt = PROVIDER_OPTIONS.find(o => o.value === value)
    if (opt) {
      updateField(CONFIG_KEYS.LM_STUDIO_URL, opt.defaultUrl)
      setModel(opt.defaultModel)
      if (opt.type === 'local') {
        setApiKey('')
      }
      invoke('set_inference_provider', { provider: value, model: opt.defaultModel, apiKey: null }).catch(() => {})
    }
    setTestResult(null)
    onChange?.()
  }

  const handleUrlChange = (value: string) => {
    updateField(CONFIG_KEYS.LM_STUDIO_URL, value)
    onChange?.()
    if (value.trim().length > 0 && !isValidUrl(value)) {
      setUrlValidationError(t('settings.ai.urlInvalid'))
    } else {
      setUrlValidationError(null)
    }
  }

  const handleTestConnection = async () => {
    if (!isValidUrl(lmStudioUrl) && provider !== 'openai' && provider !== 'openrouter') {
      setTestResult('error')
      setUrlValidationError(t('settings.ai.urlInvalidTest'))
      return
    }

    setTesting(true)
    setTestResult(null)
    setTestDetail('')
    setUrlValidationError(null)

    try {
      await invoke('set_inference_provider', { provider, model, apiKey: apiKey || null })
      const detail = await invoke<string>('test_inference_connection')
      setTestResult('success')
      setTestDetail(detail)
    } catch (err: any) {
      setTestResult('error')
      setTestDetail(err?.toString?.() || '')
    } finally {
      setTesting(false)
    }
  }

  const handleDiscoverModels = async () => {
    setDiscovering(true)
    setDiscoverError(null)
    setDiscoveredModels([])

    try {
      const models = await invoke<DiscoveredModel[]>('discover_available_models')
      setDiscoveredModels(models)
      if (models.length === 0) {
        setDiscoverError(t('settings.ai.noModelsFound'))
      }
    } catch {
      setDiscoverError(t('settings.ai.discoverFailed'))
    } finally {
      setDiscovering(false)
    }
  }

  const handleSelectDiscoveredModel = (m: DiscoveredModel) => {
    setProvider(m.provider)
    setModel(m.model_id)
    const opt = PROVIDER_OPTIONS.find(o => o.value === m.provider)
    if (opt) {
      updateField(CONFIG_KEYS.LM_STUDIO_URL, m.base_url)
    }
  }

  const isCloud = PROVIDER_OPTIONS.find(o => o.value === provider)?.type === 'cloud'

  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Brain className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.ai.title')}</h2>
      </div>

      <div className="space-y-6">
        {/* Provider Selector */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.provider')}
          </label>
          <select
            value={provider}
            onChange={(e) => handleProviderChange(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
          >
            {PROVIDER_OPTIONS.map(opt => (
              <option key={opt.value} value={opt.value}>
                {opt.label} ({opt.type === 'local' ? t('settings.ai.typeLocal') : t('settings.ai.typeCloud')})
              </option>
            ))}
          </select>
        </div>

        {/* Model Name */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.model')}
          </label>
          <input
            type="text"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            placeholder={t('settings.ai.modelPlaceholder')}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
          />
        </div>

        {/* API Key (Cloud only) */}
        {isCloud && (
          <div>
            <label className="block text-sm font-medium mb-2">
              {t('settings.ai.apiKey')}
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={t('settings.ai.apiKeyPlaceholder')}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                         bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
            />
          </div>
        )}

        {/* Local URL */}
        {!isCloud && (
          <div>
            <label className="block text-sm font-medium mb-2">
              {t('settings.ai.localUrl')}
            </label>
            <div className="flex gap-2">
              <input
                type="url"
                value={lmStudioUrl}
                onChange={(e) => handleUrlChange(e.target.value)}
                placeholder={t('settings.ai.lmStudioUrlPlaceholder')}
                className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                           bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
              />
              <button
                onClick={handleTestConnection}
                disabled={testing}
                className="btn-secondary flex items-center gap-2 disabled:opacity-50"
              >
                {testing ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : testResult === 'success' ? (
                  <CheckCircle className="w-4 h-4 text-green-500" />
                ) : testResult === 'error' ? (
                  <AlertCircle className="w-4 h-4 text-red-500" />
                ) : null}
                {t('settings.ai.testConnection')}
              </button>
            </div>
            {urlValidationError && (
              <p className="mt-2 text-sm text-red-600">{urlValidationError}</p>
            )}
          </div>
        )}

        {/* Test Connection (Cloud) */}
        {isCloud && (
          <div>
            <button
              onClick={handleTestConnection}
              disabled={testing || !apiKey}
              className="btn-secondary flex items-center gap-2 disabled:opacity-50"
            >
              {testing ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : testResult === 'success' ? (
                <CheckCircle className="w-4 h-4 text-green-500" />
              ) : testResult === 'error' ? (
                <AlertCircle className="w-4 h-4 text-red-500" />
              ) : null}
              {t('settings.ai.testConnection')}
            </button>
          </div>
        )}

        {/* Test Result */}
        {testResult && (
          <div className={cn(
            'rounded-lg p-3 text-sm border',
            testResult === 'success'
              ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800 text-green-800 dark:text-green-200'
              : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-800 dark:text-red-200'
          )}>
            <div className="flex items-center gap-2 font-medium mb-1">
              {testResult === 'success' ? (
                <CheckCircle className="w-4 h-4 text-green-500" />
              ) : (
                <AlertCircle className="w-4 h-4 text-red-500" />
              )}
              {testResult === 'success' ? t('settings.ai.connectionSuccess') : t('settings.ai.connectionFailed')}
            </div>
            {testDetail && (
              <pre className="text-xs whitespace-pre-wrap mt-1 opacity-80 leading-relaxed">{testDetail}</pre>
            )}
          </div>
        )}

        {/* Discover Models */}
        <div>
          <button
            onClick={handleDiscoverModels}
            disabled={discovering}
            className="btn-secondary flex items-center gap-2 disabled:opacity-50"
          >
            {discovering ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Search className="w-4 h-4" />
            )}
            {t('settings.ai.discoverModels')}
          </button>

          {discoveredModels.length > 0 && (
            <div className="mt-3 space-y-2">
              {discoveredModels.map((m, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700
                             bg-gray-50 dark:bg-dark-200 cursor-pointer hover:border-primary-500 transition-colors"
                  onClick={() => handleSelectDiscoveredModel(m)}
                >
                  <div>
                    <p className="text-sm font-medium">{m.model_id}</p>
                    <p className="text-xs text-gray-500">{m.provider_name} (:{m.port})</p>
                  </div>
                  <div
                    className="flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-full"
                    style={{
                      backgroundColor: m.loaded
                        ? 'rgba(34, 197, 94, 0.1)'
                        : 'rgba(245, 158, 11, 0.1)',
                      color: m.loaded
                        ? 'rgb(21, 128, 61)'
                        : 'rgb(180, 83, 9)',
                    }}
                    title={m.loaded
                      ? 'Model is loaded into VRAM and ready for inference'
                      : 'Model is available but not yet loaded into VRAM. Selecting it may trigger loading.'}
                  >
                    <Circle
                      className="w-2.5 h-2.5"
                      fill={m.loaded ? 'rgb(34, 197, 94)' : 'rgb(245, 158, 11)'}
                      stroke="none"
                    />
                    {m.loaded ? 'Ready' : 'Available'}
                  </div>
                </div>
              ))}
            </div>
          )}

          {discoverError && (
            <p className="mt-2 text-sm text-red-600">{discoverError}</p>
          )}
        </div>

        {/* Concurrency */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.concurrency')}: {aiConcurrency}
          </label>
          <input
            type="range"
            min="1"
            max="10"
            value={aiConcurrency}
            onChange={(e) => {
              updateField(CONFIG_KEYS.AI_CONCURRENCY, e.target.value)
              onChange?.()
            }}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>1 ({t('settings.ai.concurrencySlow')})</span>
            <span>10 ({t('settings.ai.concurrencyFast')})</span>
          </div>
        </div>

        {/* Timeout */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.timeout')}
          </label>
          <input
            type="number"
            min="10"
            max="120"
            value={aiTimeout}
            onChange={(e) => {
              updateField(CONFIG_KEYS.AI_TIMEOUT, e.target.value)
              onChange?.()
            }}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
          />
        </div>
      </div>
    </div>
  )
}
