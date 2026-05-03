/**
 * errorMap.ts 错误映射逻辑测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock i18next (used by src/i18n/index.ts)
vi.mock('i18next', () => ({
  default: {
    use: vi.fn().mockReturnThis(),
    init: vi.fn(),
    t: vi.fn((key: string) => key),
    language: 'zh',
  },
}))

// Mock react-i18next
vi.mock('react-i18next', () => ({
  initReactI18next: { type: '3rdParty', init: vi.fn() },
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'zh', changeLanguage: vi.fn() },
  }),
}))

// Mock i18n module - errorMap.ts imports `import i18n from '../i18n'`
// From test file at src/lib/__tests__/, the resolved path is ../../i18n
vi.mock('../../i18n', () => {
  const translations: Record<string, string> = {
    'errors.DB_001': '数据库错误',
    'errors.NF_001': '资源未找到',
    'errors.IO_001': 'IO 错误',
    'errors.VAL_001': '验证错误',
    'errors.AUTH_001': '认证错误',
    'errors.AI_001': 'AI 处理错误',
    'errors.CFG_001': '配置错误',
    'errors.unknownError': '未知错误',
  }
  return {
    default: {
      t: (key: string) => translations[key] || key,
    },
  }
})

import { getErrorMessage } from '../errorMap'

describe('getErrorMessage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应该返回已知错误码对应的翻译', () => {
    expect(getErrorMessage('DB_001')).toBe('数据库错误')
    expect(getErrorMessage('NF_001')).toBe('资源未找到')
    expect(getErrorMessage('IO_001')).toBe('IO 错误')
    expect(getErrorMessage('VAL_001')).toBe('验证错误')
    expect(getErrorMessage('AUTH_001')).toBe('认证错误')
    expect(getErrorMessage('AI_001')).toBe('AI 处理错误')
    expect(getErrorMessage('CFG_001')).toBe('配置错误')
  })

  it('未知错误码应返回 unknownError 翻译', () => {
    expect(getErrorMessage('UNKNOWN_CODE')).toBe('未知错误')
  })

  it('未知错误码且提供 fallback 时应返回 fallback', () => {
    expect(getErrorMessage('UNKNOWN_CODE', '自定义回退')).toBe('自定义回退')
  })

  it('已知错误码应忽略 fallback 参数', () => {
    expect(getErrorMessage('DB_001', '自定义回退')).toBe('数据库错误')
  })

  it('空字符串错误码应返回 unknownError', () => {
    expect(getErrorMessage('')).toBe('未知错误')
  })

  it('空字符串错误码且提供 fallback 时应返回 fallback', () => {
    expect(getErrorMessage('', '回退文本')).toBe('回退文本')
  })
})
