import { convertFileSrc } from '@tauri-apps/api/core'

export function toAssetUrl(filePath: string | null | undefined): string {
  if (!filePath) return ''
  try {
    return convertFileSrc(filePath)
  } catch {
    return filePath
  }
}
