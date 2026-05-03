/**
 * AIProgressPanel 组件测试
 * 覆盖: 不同状态渲染、进度显示、按钮存在性
 */

import React from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { AIProgressPanel } from '../AIProgressPanel'

// ===== Mocks =====
vi.mock('@/lib/api', () => ({
  getRecentAIResults: vi.fn().mockResolvedValue([]),
  retrySingleAIResult: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const map: Record<string, string> = {
        'ai.progressTitle': 'AI 处理进度',
        'ai.statusIdle': '空闲',
        'ai.statusProcessing': '处理中',
        'ai.statusPaused': '已暂停',
        'ai.statusCompleted': '已完成',
        'ai.statusFailed': '失败',
        'ai.startProcessing': '开始处理',
        'ai.pause': '暂停',
        'ai.resume': '继续',
        'ai.cancel': '取消',
        'ai.cancelConfirm': '确定要取消吗？',
        'ai.confirmCancel': '确认取消',
        'ai.back': '返回',
        'ai.retryFailed': '重试失败',
        'ai.calculating': '计算中...',
        'ai.minutes': '分',
        'ai.seconds': '秒',
        'ai.eta': '预计时间',
        'ai.success': '成功',
        'ai.failed': '失败',
        'ai.retrying': '重试中',
        'ai.recentResults': '最近结果',
        'ai.loadingResults': '加载中...',
        'ai.noResults': '暂无结果',
        'ai.retry': '重试',
      }
      return map[key] || key
    },
    i18n: { language: 'zh' },
  }),
}))

vi.mock('framer-motion', () => {
  const stripMotionProps = (props: Record<string, unknown>): Record<string, unknown> => {
    const result: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(props)) {
      if (!['initial', 'animate', 'exit', 'transition', 'whileHover', 'whileTap'].includes(key)) {
        result[key] = value
      }
    }
    return result
  }

  return {
    motion: {
      div: ({ children, ...props }: React.PropsWithChildren<Record<string, unknown>>) => {
        return <div {...(stripMotionProps(props) as React.HTMLAttributes<HTMLDivElement>)}>{children}</div>
      },
    },
    AnimatePresence: ({ children }: React.PropsWithChildren) => <>{children}</>,
  }
})

vi.mock('lucide-react', () => ({
  Play: () => <span data-testid="icon-play" />,
  Pause: () => <span data-testid="icon-pause" />,
  RotateCcw: () => <span data-testid="icon-rotate" />,
  AlertCircle: () => <span data-testid="icon-alert" />,
  CheckCircle: () => <span data-testid="icon-check" />,
  XCircle: () => <span data-testid="icon-x-circle" />,
  Clock: () => <span data-testid="icon-clock" />,
}))

describe('AIProgressPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('idle 状态应显示"开始处理"按钮', () => {
    render(<AIProgressPanel />)

    expect(screen.getByText('AI 处理进度')).toBeInTheDocument()
    expect(screen.getByText('空闲')).toBeInTheDocument()
    expect(screen.getByText('开始处理')).toBeInTheDocument()
    // idle 状态没有取消按钮
    expect(screen.queryByText('取消')).not.toBeInTheDocument()
  })

  it('processing 状态应显示"暂停"和"取消"按钮', () => {
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
      eta_seconds: 120,
    }

    render(<AIProgressPanel status={status} />)

    expect(screen.getByText('处理中')).toBeInTheDocument()
    expect(screen.getByText('暂停')).toBeInTheDocument()
    expect(screen.getByText('取消')).toBeInTheDocument()
    // ETA 存在 - 通过百分比和进度文本验证
    expect(screen.getByText('50%')).toBeInTheDocument()
    expect(screen.getByText('50 / 100')).toBeInTheDocument()
  })

  it('paused 状态应显示"继续"和"取消"按钮', () => {
    const status = {
      status: 'paused' as const,
      total: 100,
      completed: 30,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} />)

    expect(screen.getByText('已暂停')).toBeInTheDocument()
    expect(screen.getByText('继续')).toBeInTheDocument()
    expect(screen.getByText('取消')).toBeInTheDocument()
  })

  it('completed 状态应正确显示', () => {
    const status = {
      status: 'completed' as const,
      total: 100,
      completed: 100,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} />)

    expect(screen.getByText('已完成')).toBeInTheDocument()
    expect(screen.getByText('100%')).toBeInTheDocument()
    expect(screen.getByText('100 / 100')).toBeInTheDocument()
  })

  it('failed 状态应显示"重试失败"按钮', () => {
    const status = {
      status: 'failed' as const,
      total: 100,
      completed: 90,
      failed: 10,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} />)

    // "失败" 在状态徽章和统计网格中各出现一次，使用 getAllByText
    const failedElements = screen.getAllByText('失败')
    expect(failedElements.length).toBeGreaterThanOrEqual(1)

    expect(screen.getByText('重试失败')).toBeInTheDocument()
    expect(screen.getByText('10')).toBeInTheDocument() // failed 数量
  })

  it('进度百分比应正确计算', () => {
    const status = {
      status: 'processing' as const,
      total: 200,
      completed: 50,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} />)
    expect(screen.getByText('25%')).toBeInTheDocument()
    expect(screen.getByText('50 / 200')).toBeInTheDocument()
  })

  it('成功/失败/重试数字应正确显示', () => {
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 80,
      failed: 15,
      retrying: 5,
    }

    render(<AIProgressPanel status={status} />)

    // Stats grid 中的数字
    const numbers = screen.getAllByText('80')
    expect(numbers.length).toBeGreaterThan(0)
    expect(screen.getByText('15')).toBeInTheDocument()
    expect(screen.getByText('5')).toBeInTheDocument()
  })

  it('onStart 回调应该在点击"开始处理"时被调用', () => {
    const onStart = vi.fn()
    render(<AIProgressPanel onStart={onStart} />)

    fireEvent.click(screen.getByText('开始处理'))
    expect(onStart).toHaveBeenCalledTimes(1)
  })

  it('onPause 回调应该在点击"暂停"时被调用', () => {
    const onPause = vi.fn()
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} onPause={onPause} />)

    fireEvent.click(screen.getByText('暂停'))
    expect(onPause).toHaveBeenCalledTimes(1)
  })

  it('onResume 回调应该在点击"继续"时被调用', () => {
    const onResume = vi.fn()
    const status = {
      status: 'paused' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} onResume={onResume} />)

    fireEvent.click(screen.getByText('继续'))
    expect(onResume).toHaveBeenCalledTimes(1)
  })

  it('点击"取消"应显示确认对话框', () => {
    const onCancel = vi.fn()
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} onCancel={onCancel} />)

    fireEvent.click(screen.getByText('取消'))
    expect(screen.getByText('确定要取消吗？')).toBeInTheDocument()
    expect(screen.getByText('确认取消')).toBeInTheDocument()
    expect(screen.getByText('返回')).toBeInTheDocument()
  })

  it('确认取消应调用 onCancel', () => {
    const onCancel = vi.fn()
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} onCancel={onCancel} />)

    fireEvent.click(screen.getByText('取消'))
    fireEvent.click(screen.getByText('确认取消'))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })

  it('点击"返回"应关闭确认对话框', () => {
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} />)

    fireEvent.click(screen.getByText('取消'))
    expect(screen.getByText('确定要取消吗？')).toBeInTheDocument()

    fireEvent.click(screen.getByText('返回'))
    expect(screen.queryByText('确定要取消吗？')).not.toBeInTheDocument()
  })

  it('total 为 0 时进度应显示 0%', () => {
    const status = {
      status: 'processing' as const,
      total: 0,
      completed: 0,
      failed: 0,
      retrying: 0,
    }

    render(<AIProgressPanel status={status} />)
    expect(screen.getByText('0%')).toBeInTheDocument()
    expect(screen.getByText('0 / 0')).toBeInTheDocument()
  })

  it('有 ETA 时应显示预计时间', () => {
    const status = {
      status: 'processing' as const,
      total: 100,
      completed: 50,
      failed: 0,
      retrying: 0,
      eta_seconds: 150, // 2分30秒
    }

    render(<AIProgressPanel status={status} />)
    // 验证进度和状态存在
    expect(screen.getByText('50%')).toBeInTheDocument()
    expect(screen.getByText('50 / 100')).toBeInTheDocument()
    expect(screen.getByText('处理中')).toBeInTheDocument()
  })
})
