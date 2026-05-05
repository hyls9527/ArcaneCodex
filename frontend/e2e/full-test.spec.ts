import { test, expect } from '@playwright/test'

test.describe('Arcane Codex - 全链路测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:1420/')
    await page.waitForLoadState('networkidle')
  })

  test('1. 页面加载 - 侧边栏和主区域渲染', async ({ page }) => {
    await expect(page.locator('text=图库').first()).toBeVisible()
    await expect(page.locator('text=AI 打标').first()).toBeVisible()
    await expect(page.locator('text=去重').first()).toBeVisible()
    await expect(page.locator('text=Knowledge Graph').first()).toBeVisible()
    await expect(page.locator('text=数据仪表盘').first()).toBeVisible()
    await expect(page.locator('text=设置').first()).toBeVisible()
  })

  test('2. 导航切换 - 各页面可访问', async ({ page }) => {
    await page.locator('text=AI 打标').first().click()
    await expect(page.locator('text=还没有图片需要处理')).toBeVisible()

    await page.locator('text=去重').first().click()
    await expect(page.locator('text=去重').first()).toBeVisible()

    await page.locator('text=数据仪表盘').first().click()
    await expect(page.locator('text=仪表盘')).toBeVisible()

    await page.locator('text=设置').first().click()
    await expect(page.locator('text=设置').first()).toBeVisible()

    await page.locator('text=图库').first().click()
    await expect(page.locator('text=图库').first()).toBeVisible()
  })

  test('3. 图片列表 - 分页和显示', async ({ page }) => {
    await page.waitForTimeout(1000)
    const cards = page.locator('[data-testid="image-card"]')
    const count = await cards.count()

    if (count > 0) {
      const firstCard = cards.first()
      await expect(firstCard.locator('img')).toBeVisible()
    }
  })

  test('4. 搜索功能 - 输入和响应', async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="搜索"]')
    await searchInput.fill('test')
    await searchInput.press('Enter')
    await page.waitForTimeout(500)
    await expect(page.locator('text=图库').first()).toBeVisible()
  })

  test('5. 设置页面 - 配置项渲染', async ({ page }) => {
    await page.locator('text=设置').first().click()
    await expect(page.locator('text=AI 配置').first()).toBeVisible()
    await expect(page.locator('text=显示设置').first()).toBeVisible()
    await expect(page.locator('text=存储配置').first()).toBeVisible()
  })
})
