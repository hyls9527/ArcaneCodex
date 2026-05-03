export type AIStatusEnum = 'pending' | 'processing' | 'completed' | 'failed'

// Re-export AppImage from api.ts to ensure type consistency
export type { AppImage } from '../lib/api'

export type Page = 'gallery' | 'settings' | 'ai' | 'dedup' | 'dashboard'

export interface Toast {
  id: number
  message: string
  type: 'error' | 'success' | 'info'
}
