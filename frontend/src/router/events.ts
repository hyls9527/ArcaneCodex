import type { Page } from '@/types/image'

export type AppRoute = Page

export type NavigationSource = 'sidebar' | 'action' | 'tauri-event' | 'keyboard' | 'system'

export interface RoutePayload {
  route: AppRoute
  params?: Record<string, string>
  source?: NavigationSource
}

export const ROUTE_CHANGE = 'app:route-change'
export const ROUTE_BACK = 'app:route-back'
export const ROUTE_FORWARD = 'app:route-forward'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

function webEmit(event: string, payload: unknown) {
  window.dispatchEvent(new CustomEvent(event, { detail: payload }))
}

function webListen(event: string, handler: (event: { payload: unknown }) => void) {
  const listener = (e: Event) => handler({ payload: (e as CustomEvent).detail })
  window.addEventListener(event, listener)
  return () => window.removeEventListener(event, listener)
}

let tauriEmit: (event: string, payload: unknown) => Promise<void>
let tauriListen: (event: string, handler: (event: { payload: unknown }) => void) => Promise<() => void>

if (isTauri) {
  import('@tauri-apps/api/event').then(mod => {
    tauriEmit = mod.emit
    tauriListen = mod.listen
  })
}

export const navigate = (payload: RoutePayload) => {
  webEmit(ROUTE_CHANGE, payload)
  if (isTauri && tauriEmit) {
    tauriEmit(ROUTE_CHANGE, payload)
  }
  return Promise.resolve()
}

export const goBack = () => {
  if (isTauri && tauriEmit) {
    return tauriEmit(ROUTE_BACK, {})
  }
  webEmit(ROUTE_BACK, {})
  return Promise.resolve()
}

export const goForward = () => {
  if (isTauri && tauriEmit) {
    return tauriEmit(ROUTE_FORWARD, {})
  }
  webEmit(ROUTE_FORWARD, {})
  return Promise.resolve()
}

export type UnlistenFn = () => void

export function appListen(event: string, handler: (event: { payload: unknown }) => void): Promise<UnlistenFn> {
  if (isTauri && tauriListen) {
    return tauriListen(event, handler)
  }
  const unsub = webListen(event, handler)
  return Promise.resolve(unsub)
}

export function setupTauriRouteListeners(): Promise<UnlistenFn[]> {
  if (!isTauri) return Promise.resolve([])

  const unsubs: Promise<UnlistenFn>[] = []

  unsubs.push(
    appListen('tauri-navigate', (event) => {
      navigate({ ...event.payload as RoutePayload, source: 'tauri-event' })
    })
  )

  return Promise.all(unsubs)
}
