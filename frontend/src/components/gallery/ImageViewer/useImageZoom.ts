import { useState, useCallback, useRef, useEffect } from 'react'

export interface UseImageZoomReturn {
  scale: number
  position: { x: number; y: number }
  isDragging: boolean
  handlers: {
    onWheel: (e: React.WheelEvent) => void
    onMouseDown: (e: React.MouseEvent) => void
    onMouseMove: (e: React.MouseEvent) => void
    onMouseUp: () => void
  }
  zoomIn: (step?: number) => void
  zoomOut: (step?: number) => void
  reset: () => void
}

interface UseImageZoomOptions {
  onClose: () => void
  onToggleInfoPanel: () => void
}

export function useImageZoom({ onClose, onToggleInfoPanel }: UseImageZoomOptions): UseImageZoomReturn {
  const [scale, setScale] = useState(1)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [isDragging, setIsDragging] = useState(false)
  const dragStart = useRef({ x: 0, y: 0 })

  // 用 ref 保存最新回调，避免键盘 listener 闭包陈旧问题
  const onCloseRef = useRef(onClose)
  onCloseRef.current = onClose
  const onToggleInfoPanelRef = useRef(onToggleInfoPanel)
  onToggleInfoPanelRef.current = onToggleInfoPanel

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    setScale(prev => Math.max(0.1, Math.min(5, prev + delta)))
  }, [])

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (scale > 1) {
      setIsDragging(true)
      dragStart.current = { x: e.clientX - position.x, y: e.clientY - position.y }
    }
  }, [scale, position])

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isDragging) {
      setPosition({
        x: e.clientX - dragStart.current.x,
        y: e.clientY - dragStart.current.y,
      })
    }
  }, [isDragging])

  const handleMouseUp = useCallback(() => {
    setIsDragging(false)
  }, [])

  const zoomIn = useCallback((step = 0.2) => {
    setScale(prev => Math.min(5, prev + step))
  }, [])

  const zoomOut = useCallback((step = 0.2) => {
    setScale(prev => Math.max(0.1, prev - step))
  }, [])

  const reset = useCallback(() => {
    setScale(1)
    setPosition({ x: 0, y: 0 })
  }, [])

  // 键盘快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          onCloseRef.current()
          break
        case 'ArrowLeft':
        case 'ArrowDown':
          setScale(prev => Math.max(0.1, prev - 0.1))
          break
        case 'ArrowUp':
        case 'ArrowRight':
          setScale(prev => Math.min(5, prev + 0.1))
          break
        case '0':
          setScale(1)
          setPosition({ x: 0, y: 0 })
          break
        case 'i':
          onToggleInfoPanelRef.current()
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  return {
    scale,
    position,
    isDragging,
    handlers: {
      onWheel: handleWheel,
      onMouseDown: handleMouseDown,
      onMouseMove: handleMouseMove,
      onMouseUp: handleMouseUp,
    },
    zoomIn,
    zoomOut,
    reset,
  }
}
