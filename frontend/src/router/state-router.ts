import { useReducer, useEffect, useRef, useCallback } from 'react'
import type { AppRoute, RoutePayload, UnlistenFn } from './events'
import { ROUTE_CHANGE, ROUTE_BACK, ROUTE_FORWARD, appListen } from './events'

const HISTORY_STORAGE_KEY = 'tesr-history'
const MAX_HISTORY_LENGTH = 100

function loadPersistedHistory(): AppRoute[] {
  try {
    const stored = localStorage.getItem(HISTORY_STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored) as AppRoute[]
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed.slice(-MAX_HISTORY_LENGTH)
      }
    }
  } catch {
    // ignore
  }
  return []
}

function saveHistory(history: AppRoute[]) {
  try {
    localStorage.setItem(HISTORY_STORAGE_KEY, JSON.stringify(history.slice(-MAX_HISTORY_LENGTH)))
  } catch {
    // ignore
  }
}

interface RouterState {
  current: AppRoute
  history: AppRoute[]
  index: number
  params: Record<string, string>
  initialized: boolean
}

type RouterAction =
  | { type: 'NAVIGATE'; payload: RoutePayload }
  | { type: 'BACK' }
  | { type: 'FORWARD' }
  | { type: 'INIT' }

const VALID_TRANSITIONS: Record<AppRoute, AppRoute[]> = {
  gallery: ['dashboard', 'ai', 'dedup', 'settings', 'knowledge_graph'],
  dashboard: ['gallery', 'ai', 'dedup', 'settings', 'knowledge_graph'],
  ai: ['gallery', 'dashboard', 'dedup', 'settings', 'knowledge_graph'],
  dedup: ['gallery', 'dashboard', 'ai', 'settings', 'knowledge_graph'],
  settings: ['gallery', 'dashboard', 'ai', 'dedup', 'knowledge_graph'],
  knowledge_graph: ['gallery', 'dashboard', 'ai', 'dedup', 'settings'],
}

function routerReducer(state: RouterState, action: RouterAction): RouterState {
  switch (action.type) {
    case 'INIT': {
      const persisted = loadPersistedHistory()
      if (persisted.length > 0) {
        return {
          ...state,
          initialized: true,
          current: persisted[persisted.length - 1],
          history: persisted,
          index: persisted.length - 1,
        }
      }
      return { ...state, initialized: true }
    }
    case 'NAVIGATE': {
      const target = action.payload.route
      if (!VALID_TRANSITIONS[state.current]?.includes(target)) {
        return state
      }
      const newHistory = [...state.history.slice(0, state.index + 1), target]
      const trimmedHistory = newHistory.slice(-MAX_HISTORY_LENGTH)
      saveHistory(trimmedHistory)
      return {
        ...state,
        current: target,
        history: trimmedHistory,
        index: trimmedHistory.length - 1,
        params: action.payload.params ?? {},
      }
    }
    case 'BACK': {
      if (state.index <= 0) return state
      const newIndex = state.index - 1
      saveHistory(state.history)
      return {
        ...state,
        index: newIndex,
        current: state.history[newIndex],
      }
    }
    case 'FORWARD': {
      if (state.index >= state.history.length - 1) return state
      const newIndex = state.index + 1
      saveHistory(state.history)
      return {
        ...state,
        index: newIndex,
        current: state.history[newIndex],
      }
    }
  }
}

export function useStateRouter(initialRoute: AppRoute = 'gallery') {
  const [state, dispatch] = useReducer(routerReducer, {
    current: initialRoute,
    history: [initialRoute],
    index: 0,
    params: {},
    initialized: false,
  })

  const initRef = useRef(false)
  const unsubsRef = useRef<UnlistenFn[]>([])

  useEffect(() => {
    if (initRef.current) return
    initRef.current = true

    const unsubBack = appListen(ROUTE_BACK, () => dispatch({ type: 'BACK' }))
    const unsubForward = appListen(ROUTE_FORWARD, () => dispatch({ type: 'FORWARD' }))
    const unsubChange = appListen(ROUTE_CHANGE, (event) => {
      dispatch({ type: 'NAVIGATE', payload: event.payload as RoutePayload })
    })

    Promise.all([unsubBack, unsubForward, unsubChange]).then(([unsubBackFn, unsubForwardFn, unsubChangeFn]) => {
      unsubsRef.current = [unsubBackFn, unsubForwardFn, unsubChangeFn]
    })

    dispatch({ type: 'INIT' })

    return () => {
      unsubsRef.current.forEach(unsub => unsub())
    }
  }, [])

  const canGoBack = state.index > 0
  const canGoForward = state.index < state.history.length - 1

  const handleGoBack = useCallback(() => {
    if (canGoBack) {
      dispatch({ type: 'BACK' })
    }
  }, [canGoBack])

  const handleGoForward = useCallback(() => {
    if (canGoForward) {
      dispatch({ type: 'FORWARD' })
    }
  }, [canGoForward])

  return {
    current: state.current,
    params: state.params,
    initialized: state.initialized,
    canGoBack,
    canGoForward,
    history: state.history,
    goBack: handleGoBack,
    goForward: handleGoForward,
  }
}
