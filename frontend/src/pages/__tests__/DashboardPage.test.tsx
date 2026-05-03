/**
 * DashboardPage 页面测试
 * 覆盖: 初始渲染、数据展示、错误状态
 */

import React from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react'
import { DashboardPage } from '../DashboardPage'
import * as api from '@/lib/api'

// ===== Mocks =====
vi.mock('@/lib/api', () => ({
  getLibraryStats: vi.fn(),
  getAccuracyTrend: vi.fn(),
}))

vi.mock('@/components/dashboard/AccuracyChart', () => ({
  AccuracyChart: ({ initialData }: { initialData?: unknown }) => (
    <div data-testid="accuracy-chart">
      {initialData ? 'Chart with data' : 'Chart no data'}
    </div>
  ),
}))

// IMPORTANT: `t` must be a stable reference since DashboardPage uses useCallback([t])
vi.mock('react-i18next', () => {
  const map: Record<string, string> = {
    'dashboard.title': '数据仪表盘',
    'dashboard.loading': '加载中...',
    'dashboard.loadFailed': '加载失败',
    'dashboard.refresh': '刷新',
    'dashboard.totalImages': '图片总数',
    'dashboard.storageUsage': '存储使用',
    'dashboard.totalSize': '总大小',
    'dashboard.averageSize': '平均大小',
    'dashboard.largestSize': '最大文件',
    'dashboard.aiProgress': 'AI 打标进度',
    'dashboard.pending': '待处理',
    'dashboard.processing': '处理中',
    'dashboard.completed': '已完成',
    'dashboard.failed': '失败',
    'dashboard.verified': '已验证',
    'dashboard.provisional': '待验证',
    'dashboard.rejected': '已拒绝',
    'dashboard.categoryDistribution': '分类分布',
    'dashboard.uncategorized': '未分类',
    'dashboard.tagCloud': '标签云',
  }
  const t = (key: string) => map[key] || key
  return {
    useTranslation: () => ({
      t,
      i18n: { language: 'zh' },
    }),
  }
})

vi.mock('lucide-react', () => ({
  HardDrive: () => <span data-testid="icon-harddrive" />,
  BarChart3: () => <span data-testid="icon-barchart" />,
  Tag: () => <span data-testid="icon-tag" />,
  Sparkles: () => <span data-testid="icon-sparkles" />,
  RefreshCw: () => <span data-testid="icon-refresh" />,
  AlertCircle: () => <span data-testid="icon-alert" />,
  CheckCircle: () => <span data-testid="icon-check" />,
  Clock: () => <span data-testid="icon-clock" />,
}))

// ===== Test Data =====
const mockStats = {
  total_images: 1500,
  category_distribution: [
    ['landscape', 500],
    ['portrait', 300],
    ['street', 200],
    ['', 500], // uncategorized
  ] as [string, number][],
  ai_progress: {
    pending: 100,
    processing: 50,
    completed: 1200,
    failed: 30,
    verified: 800,
    provisional: 400,
    rejected: 20,
  },
  storage_usage: {
    total_size_bytes: 5 * 1024 * 1024 * 1024, // 5GB
    average_image_size: 3 * 1024 * 1024, // 3MB
    largest_image_size: 15 * 1024 * 1024, // 15MB
  },
  tag_cloud: [
    ['nature', 200],
    ['sunset', 150],
    ['beach', 100],
    ['mountain', 80],
  ] as [string, number][],
}

const mockTrend = {
  daily_data: [],
  category_accuracy: [],
  calibration_comparison: null,
}

describe('DashboardPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('加载中应显示加载状态', () => {
    vi.mocked(api.getLibraryStats).mockReturnValueOnce(new Promise(() => {}))
    vi.mocked(api.getAccuracyTrend).mockReturnValueOnce(new Promise(() => {}))

    render(<DashboardPage />)
    expect(screen.getByText('加载中...')).toBeInTheDocument()
  })

  it('数据加载完成后应显示仪表盘标题', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('数据仪表盘')).toBeInTheDocument()
    })
  })

  it('应显示图片总数', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('图片总数')).toBeInTheDocument()
      expect(screen.getByText('1,500')).toBeInTheDocument()
    })
  })

  it('应显示存储使用信息', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('存储使用')).toBeInTheDocument()
      expect(screen.getByText('总大小')).toBeInTheDocument()
      expect(screen.getByText('5.0 GB')).toBeInTheDocument()
      expect(screen.getByText('3.0 MB')).toBeInTheDocument()
      expect(screen.getByText('15.0 MB')).toBeInTheDocument()
    })
  })

  it('应显示 AI 打标进度', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('AI 打标进度')).toBeInTheDocument()
      expect(screen.getByText('待处理')).toBeInTheDocument()
      expect(screen.getByText('处理中')).toBeInTheDocument()
      expect(screen.getByText('已完成')).toBeInTheDocument()
      expect(screen.getByText('失败')).toBeInTheDocument()
      expect(screen.getByText('已验证')).toBeInTheDocument()
      expect(screen.getByText('待验证')).toBeInTheDocument()
      expect(screen.getByText('已拒绝')).toBeInTheDocument()
    })
  })

  it('应显示分类分布', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('分类分布')).toBeInTheDocument()
      expect(screen.getByText('landscape')).toBeInTheDocument()
      expect(screen.getByText('portrait')).toBeInTheDocument()
      expect(screen.getByText('street')).toBeInTheDocument()
    })
  })

  it('应显示标签云', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('标签云')).toBeInTheDocument()
      expect(screen.getByText('nature')).toBeInTheDocument()
      expect(screen.getByText('sunset')).toBeInTheDocument()
      expect(screen.getByText('beach')).toBeInTheDocument()
      expect(screen.getByText('mountain')).toBeInTheDocument()
    })
  })

  it('应渲染 AccuracyChart 组件', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    render(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getByTestId('accuracy-chart')).toBeInTheDocument()
      expect(screen.getByText('Chart with data')).toBeInTheDocument()
    })
  })

  it('加载失败应显示错误信息', async () => {
    vi.mocked(api.getLibraryStats).mockRejectedValueOnce(new Error('Failed'))
    vi.mocked(api.getAccuracyTrend).mockRejectedValueOnce(new Error('Failed'))

    await act(async () => {
      render(<DashboardPage />)
    })

    await waitFor(() => {
      expect(screen.getByText('加载失败')).toBeInTheDocument()
    })
  })

  it('刷新按钮应重新加载数据', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    await act(async () => {
      render(<DashboardPage />)
    })

    await waitFor(() => {
      expect(screen.getByText('数据仪表盘')).toBeInTheDocument()
    })

    // 点击刷新
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)
    fireEvent.click(screen.getByText('刷新'))

    await waitFor(() => {
      expect(api.getLibraryStats).toHaveBeenCalledTimes(2)
    })
  })

  it('应显示"刷新"按钮', async () => {
    vi.mocked(api.getLibraryStats).mockResolvedValueOnce(mockStats)
    vi.mocked(api.getAccuracyTrend).mockResolvedValueOnce(mockTrend)

    await act(async () => {
      render(<DashboardPage />)
    })

    await waitFor(() => {
      const refreshButtons = screen.getAllByText('刷新')
      expect(refreshButtons.length).toBeGreaterThanOrEqual(1)
    })
  })
})
