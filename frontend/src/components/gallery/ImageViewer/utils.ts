/**
 * ImageViewer 工具函数
 * 可被其他模块复用
 */

export function formatFileSize(bytes?: number): string {
  if (!bytes) return ''
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function parseExifData(exifData?: Record<string, string | number | undefined>) {
  if (!exifData || Object.keys(exifData).length === 0) return null
  try {
    if (typeof exifData === 'string') {
      return JSON.parse(exifData)
    }
    return exifData
  } catch {
    return null
  }
}

export function generatePlaceholderGradient(fileName: string, tags: string[]): string {
  const hash = fileName.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  const tagHash = tags.join('').split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  const hue1 = (hash * 137) % 360
  const hue2 = (tagHash * 173) % 360
  const hue3 = ((hash + tagHash) * 97) % 360
  return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%), hsl(${hue2}, 65%, 55%), hsl(${hue3}, 75%, 50%))`
}
