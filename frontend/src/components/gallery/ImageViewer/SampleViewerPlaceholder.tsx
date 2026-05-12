import { useMemo } from 'react'
import { generatePlaceholderGradient } from './utils'

interface SampleViewerPlaceholderProps {
  fileName: string
  tags: string[]
}

export function SampleViewerPlaceholder({ fileName, tags }: SampleViewerPlaceholderProps) {
  const gradient = useMemo(() => generatePlaceholderGradient(fileName, tags), [fileName, tags])
  const initials = fileName
    .split(/[_\-.]/)
    .map(w => w[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()

  return (
    <div
      className="flex-1 flex items-center justify-center"
      style={{ background: gradient }}
    >
      <div className="text-center">
        <div className="w-24 h-24 rounded-full bg-white/20 flex items-center justify-center mb-4 backdrop-blur-sm mx-auto">
          <span className="text-4xl font-bold text-white">{initials}</span>
        </div>
        <p className="text-lg text-white font-medium mb-2">{fileName}</p>
        <p className="text-sm text-white/70">示例图片</p>
        {tags.length > 0 && (
          <div className="flex flex-wrap gap-2 mt-4 justify-center">
            {tags.map((tag, i) => (
              <span key={i} className="text-xs px-2 py-1 rounded-full bg-white/20 text-white">
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
