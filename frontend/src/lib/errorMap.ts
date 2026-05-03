import i18n from '../i18n'

const ERROR_CODE_MAP: Record<string, string> = {
  DB_001: 'errors.DB_001',
  NF_001: 'errors.NF_001',
  IO_001: 'errors.IO_001',
  VAL_001: 'errors.VAL_001',
  AUTH_001: 'errors.AUTH_001',
  AI_001: 'errors.AI_001',
  CFG_001: 'errors.CFG_001',
}

export function getErrorMessage(code: string, fallback?: string): string {
  const i18nKey = ERROR_CODE_MAP[code]
  if (i18nKey) {
    return i18n.t(i18nKey)
  }
  return fallback || i18n.t('errors.unknownError')
}
