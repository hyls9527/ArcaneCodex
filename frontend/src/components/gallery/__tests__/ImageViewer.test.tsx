/**
 * ImageViewer 组件测试
 * 覆盖: 渲染、缩放控制、关闭按钮、键盘导航
 */

import React from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ImageViewer } from '../ImageViewer'

// ===== Mocks =====
vi.mock('@/lib/api', () => ({
  getNarratives: vi.fn().mockResolvedValue([]),
  writeNarrative: vi.fn().mockResolvedValue({ id: 1, image_id: 1, content: 'test', entities_json: '[]' }),
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const map: Record<string, string> = {
        'imageViewer.close': '关闭',
        'imageViewer.zoomIn': '放大',
        'imageViewer.zoomOut': '缩小',
        'imageViewer.resetZoom': '重置缩放',
        'imageViewer.toggleInfo': '切换信息面板',
        'imageViewer.export': '导出',
        'imageViewer.delete': '删除',
        'imageViewer.imageInfo': '图片信息',
        'imageViewer.fileInfo': '文件信息',
        'imageViewer.aiDescription': 'AI 描述',
        'imageViewer.aiTags': 'AI 标签',
        'imageViewer.exifData': 'EXIF 数据',
        'imageViewer.actions': '操作',
        'imageViewer.reAnalyze': '重新分析',
        'imageViewer.archive': '归档',
        'imageViewer.safeExport': '安全导出',
        'imageViewer.category': '分类',
      }
      return map[key] || key
    },
    i18n: { language: 'zh' },
  }),
  initReactI18next: {
    type: '3rdParty',
    init: vi.fn(),
  },
}))

vi.mock('motion/react', () => {
  const MOTION_PROPS = new Set([
    'initial', 'animate', 'exit', 'transition', 'whileHover', 'whileTap',
    'whileFocus', 'whileDrag', 'whileInView', 'layout', 'drag', 'dragConstraints',
    'dragElastic', 'dragMomentum', 'dragSnapToOrigin', 'dragTransition',
    'variants', 'onAnimationStart', 'onAnimationComplete', 'onUpdate',
    'transformTemplate', 'isStatic', 'layoutDependency', 'layoutScroll',
    'layoutId', 'onLayoutAnimationComplete', 'onBeforeLayoutMeasure',
    'onLayoutMeasure', 'inherit', 'static', 'custom',
    'onPan', 'onPanStart', 'onPanEnd', 'onTap', 'onTapStart', 'onTapCancel',
    'onPointerDown', 'onPointerUp', 'onPointerCancel', 'onHoverStart',
    'onHoverEnd', 'style',
  ])

  const EVENT_HANDLER_WHITELIST = new Set([
    'onClick', 'onChange', 'onKeyDown', 'onKeyUp', 'onBlur',
    'onFocus', 'onSubmit', 'onMouseDown', 'onMouseUp', 'onMouseMove',
    'onScroll', 'onInput', 'onContextMenu', 'onDoubleClick',
  ])

  const stripMotionProps = (props: Record<string, unknown>) => {
    const cleaned: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(props)) {
      if (MOTION_PROPS.has(key)) continue
      if (key.startsWith('while')) continue
      if (key.startsWith('on') && !EVENT_HANDLER_WHITELIST.has(key)) continue
      cleaned[key] = value
    }
    return cleaned
  }

  // Pre-define stable component references (not a Proxy that creates new types per render)
  const MotionDiv = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('div', stripMotionProps(props) as React.HTMLAttributes<HTMLDivElement>, children)
  const MotionButton = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('button', stripMotionProps(props) as React.HTMLAttributes<HTMLButtonElement>, children)
  const MotionSpan = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('span', stripMotionProps(props) as React.HTMLAttributes<HTMLSpanElement>, children)
  const MotionImg = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('img', stripMotionProps(props) as React.HTMLAttributes<HTMLImageElement>, children)
  const MotionA = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('a', stripMotionProps(props) as React.HTMLAttributes<HTMLAnchorElement>, children)
  const MotionP = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('p', stripMotionProps(props) as React.HTMLAttributes<HTMLParagraphElement>, children)
  const MotionH1 = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('h1', stripMotionProps(props) as React.HTMLAttributes<HTMLHeadingElement>, children)
  const MotionH2 = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('h2', stripMotionProps(props) as React.HTMLAttributes<HTMLHeadingElement>, children)
  const MotionH3 = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('h3', stripMotionProps(props) as React.HTMLAttributes<HTMLHeadingElement>, children)
  const MotionH4 = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('h4', stripMotionProps(props) as React.HTMLAttributes<HTMLHeadingElement>, children)
  const MotionUl = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('ul', stripMotionProps(props) as React.HTMLAttributes<HTMLUListElement>, children)
  const MotionLi = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('li', stripMotionProps(props) as React.HTMLAttributes<HTMLLIElement>, children)
  const MotionSection = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('section', stripMotionProps(props) as React.HTMLAttributes<HTMLElement>, children)
  const MotionHeader = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('header', stripMotionProps(props) as React.HTMLAttributes<HTMLElement>, children)
  const MotionFooter = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('footer', stripMotionProps(props) as React.HTMLAttributes<HTMLElement>, children)
  const MotionNav = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('nav', stripMotionProps(props) as React.HTMLAttributes<HTMLElement>, children)
  const MotionInput = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('input', stripMotionProps(props) as React.HTMLAttributes<HTMLInputElement>, children)
  const MotionTextarea = ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => React.createElement('textarea', stripMotionProps(props) as React.HTMLAttributes<HTMLTextAreaElement>, children)

  return {
    motion: {
      div: MotionDiv,
      button: MotionButton,
      span: MotionSpan,
      img: MotionImg,
      a: MotionA,
      p: MotionP,
      h1: MotionH1,
      h2: MotionH2,
      h3: MotionH3,
      h4: MotionH4,
      ul: MotionUl,
      li: MotionLi,
      section: MotionSection,
      header: MotionHeader,
      footer: MotionFooter,
      nav: MotionNav,
      input: MotionInput,
      textarea: MotionTextarea,
    },
    AnimatePresence: ({ children }: React.PropsWithChildren) => <>{children}</>,
  }
})

vi.mock('lucide-react', () => ({
  X: () => <span data-testid="icon-x" />,
  ZoomIn: () => <span data-testid="icon-zoom-in" />,
  ZoomOut: () => <span data-testid="icon-zoom-out" />,
  RotateCw: () => <span data-testid="icon-rotate" />,
  Download: () => <span data-testid="icon-download" />,
  Trash2: () => <span data-testid="icon-trash" />,
  Camera: () => <span data-testid="icon-camera" />,
  Clock: () => <span data-testid="icon-clock" />,
  MapPin: () => <span data-testid="icon-mappin" />,
  Tag: () => <span data-testid="icon-tag" />,
  Archive: () => <span data-testid="icon-archive" />,
  RefreshCw: () => <span data-testid="icon-refresh" />,
  Info: () => <span data-testid="icon-info" />,
  User: () => <span data-testid="icon-user" />,
  Send: () => <span data-testid="icon-send" />,
  MessageCircle: () => <span data-testid="icon-message" />,
  Loader2: () => <span data-testid="icon-loader" />,
  AlertCircle: () => <span data-testid="icon-alert" />,
  CheckCircle: () => <span data-testid="icon-check" />,
}))

// ===== Test Data =====
const mockImage = {
  id: 1,
  file_path: '/photos/sunset.jpg',
  file_name: 'sunset.jpg',
  width: 1920,
  height: 1080,
  file_size: 2048000,
  ai_tags: ['sunset', 'beach', 'nature'],
  ai_description: 'A beautiful sunset over the ocean',
  ai_category: 'landscape',
  exif_data: {
    DateTimeOriginal: '2024:01:15 18:30:00',
    Make: 'Canon',
    Model: 'EOS R5',
  },
}

const defaultProps = {
  image: mockImage,
  onClose: vi.fn(),
}

describe('ImageViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应该正确渲染图片', () => {
    render(<ImageViewer {...defaultProps} />)

    const img = screen.getByRole('img')
    expect(img).toBeInTheDocument()
    expect(img).toHaveAttribute('src', '/photos/sunset.jpg')
    expect(img).toHaveAttribute('alt', 'A beautiful sunset over the ocean')
  })

  it('当没有 AI 描述时应使用文件名作为 alt', () => {
    const image = { ...mockImage, ai_description: undefined }
    render(<ImageViewer image={image} onClose={vi.fn()} />)

    const img = screen.getByRole('img')
    expect(img).toHaveAttribute('alt', 'sunset.jpg')
  })

  it('应该显示文件名', () => {
    render(<ImageViewer {...defaultProps} />)
    expect(screen.getByText('sunset.jpg')).toBeInTheDocument()
  })

  it('应该显示 AI 描述', () => {
    render(<ImageViewer {...defaultProps} />)
    expect(screen.getByText('A beautiful sunset over the ocean')).toBeInTheDocument()
  })

  it('应该显示 AI 标签', () => {
    render(<ImageViewer {...defaultProps} />)
    expect(screen.getByText('sunset')).toBeInTheDocument()
    expect(screen.getByText('beach')).toBeInTheDocument()
    expect(screen.getByText('nature')).toBeInTheDocument()
  })

  it('关闭按钮应该调用 onClose', () => {
    const onClose = vi.fn()
    render(<ImageViewer image={mockImage} onClose={onClose} />)

    const closeButton = screen.getByRole('button', { name: '关闭' })
    // 点击关闭按钮会冒泡到父级 overlay (也有 onClick={onClose})，因此触发 2 次
    fireEvent.click(closeButton)
    expect(onClose).toHaveBeenCalled()
  })

  it('放大按钮应该增加缩放比例', () => {
    render(<ImageViewer {...defaultProps} />)

    const zoomInButton = screen.getByRole('button', { name: '放大' })
    // 初始显示 100%
    expect(screen.getByText('100%')).toBeInTheDocument()

    fireEvent.click(zoomInButton)
    expect(screen.getByText('120%')).toBeInTheDocument()
  })

  it('缩小按钮应该减小缩放比例', () => {
    render(<ImageViewer {...defaultProps} />)

    const zoomOutButton = screen.getByRole('button', { name: '缩小' })
    fireEvent.click(zoomOutButton)
    expect(screen.getByText('80%')).toBeInTheDocument()
  })

  it('重置按钮应该恢复 100% 缩放', () => {
    render(<ImageViewer {...defaultProps} />)

    const zoomInButton = screen.getByRole('button', { name: '放大' })
    fireEvent.click(zoomInButton) // 100 -> 120
    fireEvent.click(zoomInButton) // 120 -> 140
    expect(screen.getByText('140%')).toBeInTheDocument()

    const resetButton = screen.getByRole('button', { name: '重置缩放' })
    fireEvent.click(resetButton) // 140 -> 100
    expect(screen.getByText('100%')).toBeInTheDocument()
  })

  it('Escape 键应该调用 onClose', () => {
    const onClose = vi.fn()
    render(<ImageViewer {...defaultProps} onClose={onClose} />)

    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('ArrowUp/ArrowRight 应该增加缩放比例', () => {
    render(<ImageViewer {...defaultProps} />)

    fireEvent.keyDown(window, { key: 'ArrowRight' })
    expect(screen.getByText('110%')).toBeInTheDocument()

    fireEvent.keyDown(window, { key: 'ArrowUp' })
    expect(screen.getByText('120%')).toBeInTheDocument()
  })

  it('ArrowLeft/ArrowDown 应该减小缩放比例', () => {
    render(<ImageViewer {...defaultProps} />)

    fireEvent.keyDown(window, { key: 'ArrowLeft' })
    expect(screen.getByText('90%')).toBeInTheDocument()

    fireEvent.keyDown(window, { key: 'ArrowDown' })
    expect(screen.getByText('80%')).toBeInTheDocument()
  })

  it('"0" 键应该重置缩放和位置', () => {
    render(<ImageViewer {...defaultProps} />)

    fireEvent.keyDown(window, { key: 'ArrowRight' })
    fireEvent.keyDown(window, { key: 'ArrowRight' })
    expect(screen.getByText('120%')).toBeInTheDocument()

    fireEvent.keyDown(window, { key: '0' })
    expect(screen.getByText('100%')).toBeInTheDocument()
  })

  it('当有 onDelete 回调时应该显示删除按钮', () => {
    const onDelete = vi.fn()
    render(<ImageViewer {...defaultProps} onDelete={onDelete} />)

    const deleteButton = screen.getByRole('button', { name: '删除' })
    expect(deleteButton).toBeInTheDocument()

    fireEvent.click(deleteButton)
    expect(onDelete).toHaveBeenCalledWith(1)
  })

  it('当有 onExport 回调时应该显示导出按钮', () => {
    const onExport = vi.fn()
    render(<ImageViewer {...defaultProps} onExport={onExport} />)

    const exportButton = screen.getByRole('button', { name: '导出' })
    expect(exportButton).toBeInTheDocument()

    fireEvent.click(exportButton)
    expect(onExport).toHaveBeenCalledWith(1)
  })

  it('没有 onDelete 回调时不应显示删除按钮', () => {
    render(<ImageViewer {...defaultProps} />)
    expect(screen.queryByRole('button', { name: '删除' })).not.toBeInTheDocument()
  })

  it('没有 onExport 回调时不应显示导出按钮', () => {
    render(<ImageViewer {...defaultProps} />)
    expect(screen.queryByRole('button', { name: '导出' })).not.toBeInTheDocument()
  })

  it('没有 AI 标签时不应显示标签区域', () => {
    const image = { ...mockImage, ai_tags: undefined }
    render(<ImageViewer image={image} onClose={vi.fn()} />)

    // 底部信息栏中不应该有标签
    const bottomTags = screen.queryAllByText('sunset')
    expect(bottomTags).toHaveLength(0)
  })

  it('分类信息应正确显示', () => {
    render(<ImageViewer {...defaultProps} />)
    expect(screen.getByText('分类: landscape')).toBeInTheDocument()
  })
})
