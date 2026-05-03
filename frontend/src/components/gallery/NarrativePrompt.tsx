import { useState, useEffect, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import i18n from '@/i18n'
import { Send, MessageCircle, X, User, MapPin, Clock } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from '@/utils/cn'
import { type Narrative } from '@/lib/api'

interface NarrativePromptProps {
  imageId: number
  narratives: Narrative[]
  onWriteNarrative: (imageId: number, content: string) => Promise<void>
  onDeleteNarrative?: (narrativeId: number) => Promise<void>
}

interface ParsedEntities {
  persons: string[]
  locations: string[]
  times: string[]
  events: string[]
}

const PLACEHOLDERS_ZH = [
  '这张照片让你想起什么？',
  '用一句话讲讲这张照片的故事',
  '这张图对你意味着什么？',
  '这是在哪拍的？和谁一起？',
  '关于这张照片，你想记住什么？',
]

const PLACEHOLDERS_EN = [
  'What does this photo remind you of?',
  'Tell the story of this photo in one sentence',
  'What does this image mean to you?',
  'Where was this taken? Who were you with?',
  'What do you want to remember about this photo?',
]

const ENTITY_STYLES: Record<string, { bg: string; text: string; icon: typeof User }> = {
  persons: { bg: 'bg-blue-500/30', text: 'text-blue-300', icon: User },
  locations: { bg: 'bg-green-500/30', text: 'text-green-300', icon: MapPin },
  times: { bg: 'bg-orange-500/30', text: 'text-orange-300', icon: Clock },
  events: { bg: 'bg-purple-500/30', text: 'text-purple-300', icon: MessageCircle },
}

const ENTITY_LABEL_KEYS: Record<string, string> = {
  persons: 'narrative.person',
  locations: 'narrative.location',
  times: 'narrative.time',
  events: 'narrative.event',
}

function parseEntities(entitiesJson: string): ParsedEntities {
  try {
    const parsed = JSON.parse(entitiesJson)
    return {
      persons: Array.isArray(parsed.persons) ? parsed.persons : [],
      locations: Array.isArray(parsed.locations) ? parsed.locations : [],
      times: Array.isArray(parsed.times) ? parsed.times : [],
      events: Array.isArray(parsed.events) ? parsed.events : [],
    }
  } catch {
    return { persons: [], locations: [], times: [], events: [] }
  }
}

function renderEntityTags(entitiesJson: string) {
  const entities = parseEntities(entitiesJson)
  const allEntities = Object.entries(entities).flatMap(([type, values]) =>
    values.map(value => ({ type, value }))
  )

  if (allEntities.length === 0) return null

  return (
    <div className="flex flex-wrap gap-1.5 mb-2">
      {allEntities.map(({ type, value }, i) => {
        const style = ENTITY_STYLES[type] || ENTITY_STYLES.events
        const Icon = style.icon
        const labelKey = ENTITY_LABEL_KEYS[type]
        const label = labelKey ? i18n.t(labelKey) : type
        return (
          <motion.span
            key={`${type}-${value}-${i}`}
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            className={cn(
              'inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs',
              style.bg,
              style.text
            )}
          >
            <Icon className="w-3 h-3" />
            <span className="opacity-70">{label}:</span>
            <span>{value}</span>
          </motion.span>
        )
      })}
    </div>
  )
}

export function NarrativePrompt({
  imageId,
  narratives,
  onWriteNarrative,
  onDeleteNarrative,
}: NarrativePromptProps) {
  const { t, i18n } = useTranslation()
  const [input, setInput] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [isExpanded, setIsExpanded] = useState(false)
  const [placeholder, setPlaceholder] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    const placeholders = i18n.language === 'zh' ? PLACEHOLDERS_ZH : PLACEHOLDERS_EN
    const idx = Math.floor(Math.random() * placeholders.length)
    setPlaceholder(placeholders[idx])
  }, [imageId, i18n.language])

  const handleSubmit = useCallback(async () => {
    const trimmed = input.trim()
    if (!trimmed || isSubmitting) return

    setIsSubmitting(true)
    try {
      await onWriteNarrative(imageId, trimmed)
      setInput('')
    } catch {
    } finally {
      setIsSubmitting(false)
    }
  }, [input, imageId, isSubmitting, onWriteNarrative])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSubmit()
    }
  }, [handleSubmit])

  const handleDelete = useCallback(async (narrativeId: number) => {
    if (!onDeleteNarrative) return
    await onDeleteNarrative(narrativeId)
  }, [onDeleteNarrative])

  return (
    <div className="w-full">
      <AnimatePresence>
        {narratives.length > 0 && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="overflow-hidden mb-2"
          >
            <div className="flex flex-col gap-2">
              {narratives.map((narrative) => (
                <motion.div
                  key={narrative.id}
                  layout
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="group relative bg-white/10 rounded-lg px-3 py-2"
                >
                  {renderEntityTags(narrative.entities_json)}
                  <p className="text-sm text-gray-200 leading-relaxed">{narrative.content}</p>
                  {onDeleteNarrative && (
                    <button
                      onClick={() => handleDelete(narrative.id)}
                      className="absolute top-1.5 right-1.5 p-1 rounded-full opacity-0 group-hover:opacity-100 hover:bg-white/20 transition-opacity"
                      aria-label={t('common.delete')}
                    >
                      <X className="w-3.5 h-3.5 text-gray-400" />
                    </button>
                  )}
                </motion.div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <motion.div
        layout
        className={cn(
          'bg-black/60 backdrop-blur-sm rounded-xl border border-white/10',
          'transition-all duration-200'
        )}
      >
        <AnimatePresence mode="wait">
          {!isExpanded ? (
            <motion.button
              key="trigger"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0, height: 0 }}
              onClick={() => {
                setIsExpanded(true)
                setTimeout(() => inputRef.current?.focus(), 50)
              }}
              className="w-full flex items-center gap-2 px-4 py-2.5 text-gray-400 hover:text-gray-200 transition-colors"
            >
              <MessageCircle className="w-4 h-4" />
              <span className="text-sm">{placeholder}</span>
            </motion.button>
          ) : (
            <motion.div
              key="input"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="flex items-center gap-2 px-3 py-2"
            >
              <input
                ref={inputRef}
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                onBlur={() => {
                  if (!input.trim()) setIsExpanded(false)
                }}
                placeholder={placeholder}
                disabled={isSubmitting}
                className="flex-1 bg-transparent text-sm text-white placeholder-gray-500 focus:outline-none"
              />
              <motion.button
                onClick={handleSubmit}
                disabled={!input.trim() || isSubmitting}
                whileTap={{ scale: 0.9 }}
                className={cn(
                  'p-1.5 rounded-lg transition-colors',
                  input.trim()
                    ? 'text-blue-400 hover:bg-blue-500/20'
                    : 'text-gray-600 cursor-not-allowed'
                )}
                aria-label={t('narrative.send')}
              >
                <Send className="w-4 h-4" />
              </motion.button>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  )
}
