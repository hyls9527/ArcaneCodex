import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'motion/react'
import {
  AlertCircle,
  Download,
  Play,
  Brain,
  X,
  Clock,
  CheckCircle2,
  ExternalLink,
  Loader2,
} from 'lucide-react'
import { useConfigStore } from '@/stores/useConfigStore'
import i18n from '@/i18n'

// i18n keys
const I18N_PREFIX = 'ai.lmStudioGuide'

// localStorage key for permanent dismissal (until next app restart)
const LM_STUDIO_GUIDE_DISMISSED_KEY = 'lm_studio_guide_dismissed'

// localStorage key for first-run AI guide (set once, never auto-show again)
const AI_GUIDE_SEEN_KEY = 'has_seen_ai_guide'

// Session-level dismissal (not persisted, resets on page reload)
let sessionDismissed = false

function t(key: string): string {
  const fullKey = `${I18N_PREFIX}.${key}`
  const value = i18n.t(fullKey)
  return value === fullKey ? key : value
}

export interface LMStudioGuideProps {
  /** Auto-detect mode: mount时自动检测AI服务，未就绪则显示引导 */
  autoDetect?: boolean
  /** Callback when user dismisses the guide (close/skip) */
  onDismiss?: () => void
  /** Callback when user clicks "skip for now" */
  onSkip?: () => void
}

export function LMStudioGuide({ autoDetect = false, onDismiss, onSkip }: LMStudioGuideProps) {
  const { lmStudioUrl } = useConfigStore()
  const lmStudioUrlRef = useRef(lmStudioUrl)

  // Keep ref in sync without triggering effect re-runs
  useEffect(() => {
    lmStudioUrlRef.current = lmStudioUrl
  }, [lmStudioUrl])
  const [isOpen, setIsOpen] = useState(false)
  const [isChecking, setIsChecking] = useState(false)
  const [isConnected, setIsConnected] = useState(false)
  const [connectionError, setConnectionError] = useState<string | null>(null)
  const [step, setStep] = useState<'downloading' | 'setup' | 'success'>('setup')
  // autoDetect mode: tracks whether we've already performed the initial detection
  const autoDetectedRef = useRef(false)

  // Check LM Studio connectivity (via Tauri backend — used by manual guide)
  const checkConnection = useCallback(async () => {
    const url = lmStudioUrlRef.current || 'http://localhost:1234'
    try {
      setIsChecking(true)
      setConnectionError(null)
      const result = await invoke<boolean>('test_lm_studio_connection', { url })
      setIsConnected(result)
      if (result) {
        setStep('success')
      } else {
        setConnectionError(t('connectionFailed'))
      }
    } catch {
      setIsConnected(false)
      setConnectionError(t('cannotConnect'))
    } finally {
      setIsChecking(false)
    }
  }, [])

  // Lightweight HTTP detection (used by autoDetect mode — no Tauri invoke)
  const detectViaHttp = useCallback(async (): Promise<boolean> => {
    try {
      const resp = await fetch('http://localhost:1234/v1/models', {
        signal: AbortSignal.timeout(3000),
      })
      return resp.ok
    } catch {
      return false
    }
  }, [])

  // Determine if guide should show on mount
  useEffect(() => {
    if (autoDetect) {
      // Auto-detect mode: only run once, check via HTTP
      if (autoDetectedRef.current) return
      autoDetectedRef.current = true

      // If user has already seen the guide, don't show again
      if (localStorage.getItem(AI_GUIDE_SEEN_KEY)) {
        onDismiss?.()
        return
      }

      detectViaHttp().then((ok) => {
        if (ok) {
          // Service is ready — mark as seen and notify parent
          localStorage.setItem(AI_GUIDE_SEEN_KEY, 'true')
          onDismiss?.()
        } else {
          // Service not ready — show the guide
          setIsOpen(true)
        }
      })
      return
    }

    // Original behavior (non-autoDetect mode)
    const isDismissed = localStorage.getItem(LM_STUDIO_GUIDE_DISMISSED_KEY)
    if (isDismissed || sessionDismissed) {
      return
    }
    // Initial check on mount
    checkConnection()
  }, [autoDetect, checkConnection, detectViaHttp, onDismiss])

  // Auto-recheck connectivity every 30 seconds when modal is open
  useEffect(() => {
    if (!isOpen) return

    const interval = setInterval(() => {
      if (autoDetect) {
        detectViaHttp().then((ok) => {
          if (ok) {
            setIsConnected(true)
            setStep('success')
          }
        })
      } else {
        checkConnection()
      }
    }, 30000)

    return () => clearInterval(interval)
  }, [isOpen, autoDetect, checkConnection, detectViaHttp])

  // Auto-close and auto-recheck when connection becomes available
  useEffect(() => {
    if (isConnected && isOpen && step === 'success') {
      // Mark guide as seen in autoDetect mode
      if (autoDetect) {
        localStorage.setItem(AI_GUIDE_SEEN_KEY, 'true')
      }
      // Give user a moment to see the success state, then close
      const timeout = setTimeout(() => {
        setIsOpen(false)
        sessionDismissed = false // Reset for next time
        localStorage.removeItem(LM_STUDIO_GUIDE_DISMISSED_KEY)
        onDismiss?.()
      }, 2000)
      return () => clearTimeout(timeout)
    }
  }, [isConnected, isOpen, step, autoDetect, onDismiss])

  // Dismiss for current session only
  const handleRemindLater = () => {
    sessionDismissed = true
    setIsOpen(false)
    onSkip?.()
  }

  // Permanently dismiss (until localStorage cleared / next app restart)
  const handleDismiss = () => {
    localStorage.setItem(LM_STUDIO_GUIDE_DISMISSED_KEY, 'true')
    // In autoDetect mode, also mark as seen so it won't auto-show again
    if (autoDetect) {
      localStorage.setItem(AI_GUIDE_SEEN_KEY, 'true')
    }
    sessionDismissed = true
    setIsOpen(false)
    onDismiss?.()
  }

  // Manual retry button
  const handleRetryConnection = () => {
    if (autoDetect) {
      setIsChecking(true)
      setConnectionError(null)
      detectViaHttp().then((ok) => {
        setIsChecking(false)
        if (ok) {
          setIsConnected(true)
          setStep('success')
        } else {
          setConnectionError(t('connectionFailed'))
        }
      })
    } else {
      checkConnection()
    }
  }

  if (!isOpen) return null

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-labelledby="lm-studio-guide-title"
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
            className="relative w-full max-w-lg mx-4 bg-white dark:bg-zinc-900 rounded-2xl shadow-2xl border border-zinc-200 dark:border-zinc-700 overflow-hidden"
          >
            {/* Close Button */}
            <button
              onClick={handleDismiss}
              className="absolute top-4 right-4 z-10 p-1.5 rounded-lg text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
              aria-label={t('common.close')}
            >
              <X className="w-4 h-4" />
            </button>

            {/* Header */}
            <div className="px-6 pt-6 pb-4">
              <div className="flex items-center gap-3 mb-2">
                <div className="p-2.5 bg-gradient-to-br from-violet-500 to-indigo-600 rounded-xl shadow-lg shadow-violet-500/20">
                  <Brain className="w-6 h-6 text-white" />
                </div>
                <h2
                  id="lm-studio-guide-title"
                  className="text-xl font-bold text-zinc-900 dark:text-zinc-100"
                >
                  {autoDetect ? t('guideTitle') : t('title')}
                </h2>
              </div>
              <p className="text-sm text-zinc-500 dark:text-zinc-400 leading-relaxed">
                {autoDetect ? t('guideDescription') : t('description')}
              </p>
            </div>

            {/* Steps */}
            <div className="px-6 pb-4 space-y-3">
              {step === 'setup' && (
                <>
                  {/* Step 1: Download */}
                  <div className="flex items-start gap-3 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-xl border border-zinc-100 dark:border-zinc-700/50">
                    <div className="flex-shrink-0 w-7 h-7 bg-violet-100 dark:bg-violet-900/30 text-violet-600 dark:text-violet-400 rounded-lg flex items-center justify-center text-sm font-bold">
                      1
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200 mb-1">
                        {autoDetect ? t('guideStep1') : t('step1Title')}
                      </h3>
                      <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-2">
                        {t('step1Desc')}
                      </p>
                      <a
                        href="https://lmstudio.ai"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-1.5 text-xs font-medium text-violet-600 dark:text-violet-400 hover:text-violet-700 dark:hover:text-violet-300 hover:underline"
                      >
                        <ExternalLink className="w-3.5 h-3.5" />
                        {autoDetect ? t('downloadLMStudio') : t('step1Link')}
                      </a>
                    </div>
                    <Download className="w-5 h-5 text-zinc-300 dark:text-zinc-600 flex-shrink-0 mt-1" />
                  </div>

                  {/* Step 2: Start Service */}
                  <div className="flex items-start gap-3 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-xl border border-zinc-100 dark:border-zinc-700/50">
                    <div className="flex-shrink-0 w-7 h-7 bg-violet-100 dark:bg-violet-900/30 text-violet-600 dark:text-violet-400 rounded-lg flex items-center justify-center text-sm font-bold">
                      2
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200 mb-1">
                        {autoDetect ? t('guideStep2') : t('step2Title')}
                      </h3>
                      <p className="text-xs text-zinc-500 dark:text-zinc-400">
                        {t('step2Desc')}
                      </p>
                    </div>
                    <Play className="w-5 h-5 text-zinc-300 dark:text-zinc-600 flex-shrink-0 mt-1" />
                  </div>

                  {/* Step 3: Verify */}
                  <div className="flex items-start gap-3 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-xl border border-zinc-100 dark:border-zinc-700/50">
                    <div className="flex-shrink-0 w-7 h-7 bg-violet-100 dark:bg-violet-900/30 text-violet-600 dark:text-violet-400 rounded-lg flex items-center justify-center text-sm font-bold">
                      3
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200 mb-1">
                        {autoDetect ? t('guideStep3') : t('step3Title')}
                      </h3>
                      <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-2">
                        {t('step3Desc')}
                      </p>
                      <button
                        onClick={handleRetryConnection}
                        disabled={isChecking}
                        className="inline-flex items-center gap-1.5 text-xs font-medium text-violet-600 dark:text-violet-400 hover:text-violet-700 dark:hover:text-violet-300 disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        {isChecking ? (
                          <>
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            {t('checking')}
                          </>
                        ) : (
                          <>
                            <ExternalLink className="w-3.5 h-3.5" />
                            {t('testConnection')}
                          </>
                        )}
                      </button>
                    </div>
                    <CheckCircle2 className="w-5 h-5 text-zinc-300 dark:text-zinc-600 flex-shrink-0 mt-1" />
                  </div>

                  {/* Error Message */}
                  <AnimatePresence>
                    {connectionError && (
                      <motion.div
                        initial={{ opacity: 0, height: 0 }}
                        animate={{ opacity: 1, height: 'auto' }}
                        exit={{ opacity: 0, height: 0 }}
                        className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800/50"
                      >
                        <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
                        <p className="text-xs text-red-600 dark:text-red-400">
                          {connectionError}
                        </p>
                      </motion.div>
                    )}
                  </AnimatePresence>
                </>
              )}

              {step === 'success' && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="flex flex-col items-center justify-center py-6"
                >
                  <motion.div
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ type: 'spring', stiffness: 300, damping: 20 }}
                    className="w-14 h-14 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full flex items-center justify-center shadow-lg shadow-green-500/30 mb-4"
                  >
                    <CheckCircle2 className="w-8 h-8 text-white" />
                  </motion.div>
                  <h3 className="text-lg font-bold text-zinc-800 dark:text-zinc-200 mb-1">
                    {t('successTitle')}
                  </h3>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {t('successDesc')}
                  </p>
                </motion.div>
              )}
            </div>

            {/* Footer Actions */}
            {step !== 'success' && (
              <div className="px-6 pb-6 pt-2 flex items-center gap-3">
                <button
                  onClick={handleRemindLater}
                  className="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium text-zinc-600 dark:text-zinc-400 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-xl transition-colors"
                >
                  <Clock className="w-4 h-4" />
                  {autoDetect ? t('skipForNow') : t('remindLater')}
                </button>
                <button
                  onClick={handleDismiss}
                  className="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium text-zinc-500 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 rounded-xl transition-colors hover:bg-zinc-50 dark:hover:bg-zinc-800/50"
                >
                  {t('dismiss')}
                </button>
              </div>
            )}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}

export default LMStudioGuide
