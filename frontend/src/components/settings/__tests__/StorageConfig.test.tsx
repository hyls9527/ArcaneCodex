/**
 * StorageConfig 组件测试
 * 覆盖: 初始渲染、备份按钮点击
 */

import React, { act } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { StorageConfig } from '../StorageConfig'

// ===== Mocks =====
vi.mock('@/lib/api', () => ({
  backupDatabase: vi.fn().mockResolvedValue('/backup/path.zip'),
  restoreDatabase: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const map: Record<string, string> = {
        'settings.storage.title': '存储设置',
        'settings.storage.dataDirectory': '数据目录',
        'settings.storage.openDirectory': '打开目录',
        'settings.storage.backup': '备份',
        'settings.storage.backupDesc': '备份数据库到压缩文件',
        'settings.storage.backupExport': '导出备份',
        'settings.storage.backupProgress': '备份中...',
        'settings.storage.backupComplete': '备份完成',
        'settings.storage.backupFailed': '备份失败',
        'settings.storage.backupSuccess': '备份成功',
        'settings.storage.restore': '恢复',
        'settings.storage.restoreDesc': '从备份文件恢复数据库',
        'settings.storage.restoreImport': '导入恢复',
        'settings.storage.restoreProgress': '恢复中...',
        'settings.storage.restoreComplete': '恢复完成',
        'settings.storage.restoreFailed': '恢复失败',
        'settings.storage.restoreSuccess': '恢复成功',
      }
      return map[key] || key
    },
    i18n: { language: 'zh' },
  }),
}))

vi.mock('lucide-react', () => ({
  Database: () => <span data-testid="icon-database" />,
  FolderOpen: () => <span data-testid="icon-folder" />,
  Download: () => <span data-testid="icon-download" />,
  Upload: () => <span data-testid="icon-upload" />,
  Loader2: () => <span data-testid="icon-loader" />,
  CheckCircle: () => <span data-testid="icon-check" />,
  AlertCircle: () => <span data-testid="icon-alert" />,
}))

describe('StorageConfig', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应该正确渲染存储设置标题', () => {
    render(<StorageConfig />)
    expect(screen.getByText('存储设置')).toBeInTheDocument()
  })

  it('应该显示数据目录路径', () => {
    render(<StorageConfig />)
    const input = screen.getByDisplayValue('%APPDATA%\\ArcaneCodex')
    expect(input).toBeInTheDocument()
    expect(input).toHaveAttribute('readOnly')
  })

  it('应该显示"打开目录"按钮', () => {
    render(<StorageConfig />)
    expect(screen.getByText('打开目录')).toBeInTheDocument()
  })

  it('应该显示备份区域', () => {
    render(<StorageConfig />)
    expect(screen.getByText('备份')).toBeInTheDocument()
    expect(screen.getByText('备份数据库到压缩文件')).toBeInTheDocument()
    expect(screen.getByText('导出备份')).toBeInTheDocument()
  })

  it('应该显示恢复区域', () => {
    render(<StorageConfig />)
    expect(screen.getByText('恢复')).toBeInTheDocument()
    expect(screen.getByText('从备份文件恢复数据库')).toBeInTheDocument()
    expect(screen.getByText('导入恢复')).toBeInTheDocument()
  })

  it('点击备份按钮应该调用 backupDatabase API', async () => {
    const { backupDatabase } = await import('@/lib/api')
    render(<StorageConfig />)

    const backupButton = screen.getByText('导出备份')
    fireEvent.click(backupButton)

    await waitFor(() => {
      expect(backupDatabase).toHaveBeenCalledTimes(1)
    })
  })

  it('备份成功后应显示成功消息', async () => {
    render(<StorageConfig />)

    fireEvent.click(screen.getByText('导出备份'))

    await waitFor(() => {
      expect(screen.getByText('备份成功')).toBeInTheDocument()
    })
  })

  it('备份失败时应显示错误消息', async () => {
    const { backupDatabase } = await import('@/lib/api')
    vi.mocked(backupDatabase).mockRejectedValueOnce(new Error('Failed'))

    render(<StorageConfig />)

    fireEvent.click(screen.getByText('导出备份'))

    await waitFor(() => {
      expect(screen.getByText('备份失败')).toBeInTheDocument()
    })
  })

  it('备份进行中时按钮应禁用', async () => {
    // 让 API 调用永不 resolve
    const { backupDatabase } = await import('@/lib/api')
    let resolveFn: ((value: string) => void) | undefined
    vi.mocked(backupDatabase).mockReturnValueOnce(new Promise((resolve) => { resolveFn = resolve }))

    render(<StorageConfig />)

    fireEvent.click(screen.getByText('导出备份'))

    await waitFor(() => {
      expect(screen.getByText('备份中...')).toBeInTheDocument()
    })

    // 清理
    await act(async () => {
      resolveFn?.('/path')
    })
  })

  it('点击恢复按钮应调用 restoreDatabase API', async () => {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const { restoreDatabase } = await import('@/lib/api')
    vi.mocked(open).mockResolvedValueOnce('/path/to/backup.zip')

    render(<StorageConfig />)

    fireEvent.click(screen.getByText('导入恢复'))

    await waitFor(() => {
      expect(restoreDatabase).toHaveBeenCalledTimes(1)
    })
  })

  it('恢复成功后应显示成功消息', async () => {
    const { open } = await import('@tauri-apps/plugin-dialog')
    vi.mocked(open).mockResolvedValueOnce('/path/to/backup.zip')

    render(<StorageConfig />)

    fireEvent.click(screen.getByText('导入恢复'))

    await waitFor(() => {
      expect(screen.getByText('恢复成功')).toBeInTheDocument()
    })
  })

  it('恢复失败时应显示错误消息', async () => {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const { restoreDatabase } = await import('@/lib/api')
    vi.mocked(open).mockResolvedValueOnce('/path/to/backup.zip')
    vi.mocked(restoreDatabase).mockRejectedValueOnce(new Error('Failed'))

    render(<StorageConfig />)

    fireEvent.click(screen.getByText('导入恢复'))

    await waitFor(() => {
      expect(screen.getByText('恢复失败')).toBeInTheDocument()
    })
  })
})
