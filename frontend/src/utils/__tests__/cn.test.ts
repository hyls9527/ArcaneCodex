/**
 * cn() 工具函数测试
 * 测试 clsx 集成的类名合并逻辑
 */

import { describe, it, expect } from 'vitest'
import { cn } from '../cn'

describe('cn() 类名合并工具', () => {
  it('应该合并多个字符串类名', () => {
    expect(cn('foo', 'bar')).toBe('foo bar')
  })

  it('应该过滤 falsy 值', () => {
    expect(cn('foo', false, null, undefined, 0, '', 'bar')).toBe('foo bar')
  })

  it('应该处理条件类名（对象语法）', () => {
    expect(cn('base', { active: true, disabled: false })).toBe('base active')
  })

  it('应该处理嵌套数组', () => {
    expect(cn(['foo', 'bar'])).toBe('foo bar')
  })

  it('应该处理混合参数类型', () => {
    expect(cn('base', { active: true }, ['extra', 'classes'])).toBe('base active extra classes')
  })

  it('空输入应返回空字符串', () => {
    expect(cn()).toBe('')
  })

  it('单个类名应原样返回', () => {
    expect(cn('single')).toBe('single')
  })

  it('所有 falsy 值应返回空字符串', () => {
    expect(cn(false, null, undefined, 0, '')).toBe('')
  })

  it('应该处理 Tailwind 冲突类名（clsx 行为）', () => {
    // cn 基于 clsx，不做 tailwind-merge，相同键会并存
    const result = cn('p-4', 'p-8')
    expect(result).toBe('p-4 p-8')
  })

  it('应该支持数字作为类名', () => {
    // clsx 将数字转为字符串
    expect(cn(123)).toBe('123')
  })

  it('应该处理动态条件的复杂场景', () => {
    const isActive = true
    const isDisabled = false
    const variant = 'primary'

    const result = cn(
      'btn',
      `btn-${variant}`,
      {
        'btn-active': isActive,
        'btn-disabled': isDisabled,
      }
    )

    expect(result).toContain('btn')
    expect(result).toContain('btn-primary')
    expect(result).toContain('btn-active')
    expect(result).not.toContain('btn-disabled')
  })
})
