import { createInstance } from 'i18next'
import { describe, expect, it } from 'vitest'
import { I18N_INIT_OPTIONS, changeLanguage, getActiveLanguage, i18n } from './index'
import { DEFAULT_LANGUAGE, detectInitialLanguage, normalizeLanguage } from './language'
import { flattenResourceKeys, namespaces, resources } from './resources'

function interpolationSignature(resource: unknown, prefix = ''): string[] {
  if (typeof resource === 'string') {
    const variables = [...resource.matchAll(/{{\s*([^},\s]+).*?}}/g)].map((match) => match[1]).sort()
    return [`${prefix}:${variables.join(',')}`]
  }
  if (!resource || typeof resource !== 'object') return []
  return Object.entries(resource).flatMap(([key, value]) => interpolationSignature(value, prefix ? `${prefix}.${key}` : key)).sort()
}

describe('language normalization and detection', () => {
  it('normalizes every Portuguese locale to pt-BR', () => {
    expect(normalizeLanguage('pt')).toBe('pt-BR')
    expect(normalizeLanguage('pt-PT')).toBe('pt-BR')
    expect(normalizeLanguage('pt_BR')).toBe('pt-BR')
  })

  it('uses saved preference before navigator language', () => {
    expect(detectInitialLanguage('en', ['pt-BR'])).toBe('en')
    expect(detectInitialLanguage(null, ['pt-PT', 'en-US'])).toBe('pt-BR')
  })

  it('falls back to English for unsupported languages', () => {
    expect(detectInitialLanguage('de-DE', ['es-ES'])).toBe(DEFAULT_LANGUAGE)
    expect(detectInitialLanguage(null, ['es-ES', 'pt-BR'])).toBe(DEFAULT_LANGUAGE)
  })
})

describe('translation resources', () => {
  it('keeps English and Portuguese keys in parity for every namespace', () => {
    for (const namespace of namespaces) {
      expect(flattenResourceKeys(resources['pt-BR'][namespace])).toEqual(
        flattenResourceKeys(resources.en[namespace]),
      )
    }
  })

  it('keeps interpolation variables aligned between both locales', () => {
    for (const namespace of namespaces) {
      expect(interpolationSignature(resources['pt-BR'][namespace])).toEqual(
        interpolationSignature(resources.en[namespace]),
      )
    }
  })

  it('uses English resources as the configured fallback', async () => {
    const instance = createInstance()
    await instance.init({
      ...I18N_INIT_OPTIONS,
      lng: 'pt-BR',
      resources: {
        en: { common: { fallbackOnly: 'English fallback' } },
        'pt-BR': { common: {} },
      },
      ns: ['common'],
    })
    expect(instance.t('fallbackOnly')).toBe('English fallback')
  })

  it('switches the active language at runtime', async () => {
    await i18n.init({ ...I18N_INIT_OPTIONS, lng: 'en' })
    await changeLanguage('pt-BR')
    expect(getActiveLanguage()).toBe('pt-BR')
    expect(i18n.t('actions.cancel')).toBe('Cancelar')
    await changeLanguage('en')
    expect(i18n.t('actions.cancel')).toBe('Cancel')
  })

  it('updates the document language during a runtime switch', async () => {
    const documentElement = { lang: '' }
    Object.defineProperty(globalThis, 'document', { configurable: true, value: { documentElement } })
    try {
      await i18n.init({ ...I18N_INIT_OPTIONS, lng: 'en' })
      await changeLanguage('pt-BR')
      expect(documentElement.lang).toBe('pt-BR')
    } finally {
      Reflect.deleteProperty(globalThis, 'document')
    }
  })

  it('uses locale plural rules instead of manual concatenation', async () => {
    const instance = createInstance()
    await instance.init({ ...I18N_INIT_OPTIONS, lng: 'pt-BR' })
    expect(instance.t('observations.processes', { count: 1 })).toBe('1 processo observado')
    expect(instance.t('observations.processes', { count: 267 })).toBe('267 processos observados')
    await instance.changeLanguage('en')
    expect(instance.t('observations.processes', { count: 1 })).toBe('1 process observed')
    expect(instance.t('observations.processes', { count: 267 })).toBe('267 processes observed')
  })
})
