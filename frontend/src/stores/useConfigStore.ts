import { create } from 'zustand'
import { getAllConfigs, setConfigs, type DiscoveredModel, discoverAvailableModels } from '@/lib/api'

// Config key constants to stay in sync with backend
export const CONFIG_KEYS = {
  LM_STUDIO_URL: 'lm_studio_url',
  AI_CONCURRENCY: 'ai_concurrency',
  AI_TIMEOUT: 'ai_timeout',
  THUMBNAIL_SIZE: 'thumbnail_size',
  THEME: 'theme',
  LANGUAGE: 'language',
  NOTIFICATION_ENABLED: 'notification_enabled',
  NOTIFICATION_AI_COMPLETE: 'notification_ai_complete',
  NOTIFICATION_DEDUP_COMPLETE: 'notification_dedup_complete',
  PRIVACY_SEND_ANALYTICS: 'privacy_send_analytics',
  PRIVACY_SHARE_DATA: 'privacy_share_data',
} as const

export type ConfigKey = (typeof CONFIG_KEYS)[keyof typeof CONFIG_KEYS]

interface ConfigState {
  // Persisted values (loaded from backend)
  lmStudioUrl: string
  aiConcurrency: number
  aiTimeout: number
  thumbnailSize: number
  theme: string
  language: string
  notificationEnabled: boolean
  notificationAiComplete: boolean
  notificationDedupComplete: boolean
  privacySendAnalytics: boolean
  privacyShareData: boolean

  // Whether settings have been loaded from backend
  isLoaded: boolean

  // Error from last loadConfigs call
  loadError: string | null

  // Pending (unsaved) changes
  pendingChanges: Partial<Record<ConfigKey, string>>

  // AI service discovery state
  discoveredModels: DiscoveredModel[]
  aiServiceReady: boolean
  aiServiceScanning: boolean

  // Actions
  loadConfigs: () => Promise<ConfigState>
  updateField: (key: ConfigKey, value: string) => void
  saveConfigs: () => Promise<void>
  hasPendingChanges: () => boolean
  scanAiServices: () => Promise<DiscoveredModel[]>
  setAiServiceReady: (ready: boolean) => void
}

function parseConfigValue(key: ConfigKey, value: string): unknown {
  switch (key) {
    case CONFIG_KEYS.AI_CONCURRENCY:
    case CONFIG_KEYS.AI_TIMEOUT:
    case CONFIG_KEYS.THUMBNAIL_SIZE:
      return Number(value)
    case CONFIG_KEYS.NOTIFICATION_ENABLED:
    case CONFIG_KEYS.NOTIFICATION_AI_COMPLETE:
    case CONFIG_KEYS.NOTIFICATION_DEDUP_COMPLETE:
    case CONFIG_KEYS.PRIVACY_SEND_ANALYTICS:
    case CONFIG_KEYS.PRIVACY_SHARE_DATA:
      return value === 'true' || value === '1'
    default:
      return value
  }
}

/**
 * Maps a config key and its parsed value to the corresponding ConfigState field.
 * Eliminates duplicated switch-case in loadConfigs and saveConfigs.
 */
function applyConfigToState(key: ConfigKey, parsedValue: unknown): Partial<ConfigState> {
  switch (key) {
    case CONFIG_KEYS.LM_STUDIO_URL:
      return { lmStudioUrl: parsedValue as string }
    case CONFIG_KEYS.AI_CONCURRENCY:
      return { aiConcurrency: parsedValue as number }
    case CONFIG_KEYS.AI_TIMEOUT:
      return { aiTimeout: parsedValue as number }
    case CONFIG_KEYS.THUMBNAIL_SIZE:
      return { thumbnailSize: parsedValue as number }
    case CONFIG_KEYS.THEME:
      return { theme: parsedValue as string }
    case CONFIG_KEYS.LANGUAGE:
      return { language: parsedValue as string }
    case CONFIG_KEYS.NOTIFICATION_ENABLED:
      return { notificationEnabled: parsedValue as boolean }
    case CONFIG_KEYS.NOTIFICATION_AI_COMPLETE:
      return { notificationAiComplete: parsedValue as boolean }
    case CONFIG_KEYS.NOTIFICATION_DEDUP_COMPLETE:
      return { notificationDedupComplete: parsedValue as boolean }
    case CONFIG_KEYS.PRIVACY_SEND_ANALYTICS:
      return { privacySendAnalytics: parsedValue as boolean }
    case CONFIG_KEYS.PRIVACY_SHARE_DATA:
      return { privacyShareData: parsedValue as boolean }
    default:
      return {}
  }
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  lmStudioUrl: 'http://localhost:1234',
  aiConcurrency: 3,
  aiTimeout: 60,
  thumbnailSize: 300,
  theme: 'system',
  language: 'zh',
  notificationEnabled: true,
  notificationAiComplete: true,
  notificationDedupComplete: true,
  privacySendAnalytics: false,
  privacyShareData: false,

  isLoaded: false,
  loadError: null,
  pendingChanges: {},
  discoveredModels: [],
  aiServiceReady: false,
  aiServiceScanning: false,

  loadConfigs: async () => {
    try {
      const configs = await getAllConfigs()
      const state: Partial<ConfigState> = {}

      for (const { key, value } of configs) {
        const configKey = key as ConfigKey
        const parsed = parseConfigValue(configKey, value)
        Object.assign(state, applyConfigToState(configKey, parsed))
      }

      const newState = { ...state, isLoaded: true, loadError: null, pendingChanges: {} }
      set(newState as Partial<ConfigState>)
      // Return the current state after update
      return get()
    } catch (err) {
      // Still mark as loaded so UI renders with defaults
      const errorMessage = err instanceof Error ? err.message : String(err)
      set({ isLoaded: true, loadError: errorMessage })
      return get()
    }
  },

  updateField: (key: ConfigKey, value: string) => {
    set((state) => ({
      pendingChanges: {
        ...state.pendingChanges,
        [key]: value,
      },
    }))
  },

  saveConfigs: async () => {
    const { pendingChanges } = get()
    const entries = Object.entries(pendingChanges) as [ConfigKey, string][]

    if (entries.length === 0) return

    await setConfigs(entries)

    // Apply saved values to persisted state and clear pending
    const state: Partial<ConfigState> = { pendingChanges: {} }
    for (const [key, value] of entries) {
      const configKey = key as ConfigKey
      const parsed = parseConfigValue(configKey, value)
      Object.assign(state, applyConfigToState(configKey, parsed))
    }

    set(state)
  },

  hasPendingChanges: () => {
    return Object.keys(get().pendingChanges).length > 0
  },

  scanAiServices: async () => {
    set({ aiServiceScanning: true })
    try {
      const models = await discoverAvailableModels()
      const ready = models.some(m => m.is_online)
      set({ discoveredModels: models, aiServiceReady: ready, aiServiceScanning: false })
      return models
    } catch {
      set({ discoveredModels: [], aiServiceReady: false, aiServiceScanning: false })
      return []
    }
  },

  setAiServiceReady: (ready: boolean) => {
    set({ aiServiceReady: ready })
  },
}))
