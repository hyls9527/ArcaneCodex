import i18n from '../index'

describe('i18n Language Switching', () => {
  it('should initialize with Chinese as default language', () => {
    expect(i18n.language).toBe('zh')
  })

  it('should switch to English', async () => {
    await i18n.changeLanguage('en')
    expect(i18n.language).toBe('en')
  })

  it('should switch back to Chinese', async () => {
    await i18n.changeLanguage('zh')
    expect(i18n.language).toBe('zh')
  })

  it('should have English as fallback language', () => {
    expect(i18n.options.fallbackLng).toContain('en')
  })

  it('should have both zh and en resources loaded', () => {
    expect(i18n.hasResourceBundle('zh', 'translation')).toBe(true)
    expect(i18n.hasResourceBundle('en', 'translation')).toBe(true)
  })

  it('should translate keys correctly in Chinese', async () => {
    await i18n.changeLanguage('zh')
    const result = i18n.t('app.title')
    expect(result).toBeTruthy()
    expect(typeof result).toBe('string')
  })

  it('should translate keys correctly in English', async () => {
    await i18n.changeLanguage('en')
    const result = i18n.t('app.title')
    expect(result).toBeTruthy()
    expect(typeof result).toBe('string')
  })

  it('should return key when translation is missing', () => {
    const result = i18n.t('nonexistent.key.12345')
    expect(result).toBe('nonexistent.key.12345')
  })
})
