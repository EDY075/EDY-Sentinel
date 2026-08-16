import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }))

import { LANGUAGE_STORAGE_KEY } from '../i18n/language'
import { loadLanguage, loadSystemLocale, persistLanguage } from './tauri'

function memoryStorage(): Storage {
  const values = new Map<string, string>()
  return {
    get length() { return values.size },
    clear: () => values.clear(),
    getItem: (key) => values.get(key) ?? null,
    key: (index) => [...values.keys()][index] ?? null,
    removeItem: (key) => { values.delete(key) },
    setItem: (key, value) => { values.set(key, value) },
  }
}

describe('language preference boundary', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { localStorage: memoryStorage() },
    })
  })

  afterEach(() => {
    Reflect.deleteProperty(globalThis, 'window')
  })

  it('normalizes a locally persisted browser preference', async () => {
    window.localStorage.setItem(LANGUAGE_STORAGE_KEY, 'pt-PT')
    await expect(loadLanguage()).resolves.toBe('pt-BR')
  })

  it('uses the narrow desktop commands when Tauri is available', async () => {
    Object.assign(window, { __TAURI_INTERNALS__: {} })
    invokeMock.mockResolvedValueOnce('en-US').mockResolvedValueOnce(undefined)

    await expect(loadLanguage()).resolves.toBe('en')
    await persistLanguage('pt-BR')

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'get_language')
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'set_language', { input: { language: 'pt-BR' } })
    expect(window.localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBeNull()
  })

  it('reads the Windows user locale through a dedicated desktop command', async () => {
    Object.assign(window, { __TAURI_INTERNALS__: {} })
    invokeMock.mockResolvedValueOnce('pt-BR')

    await expect(loadSystemLocale()).resolves.toBe('pt-BR')
    expect(invokeMock).toHaveBeenCalledWith('get_system_locale')
  })

  it('leaves system-locale detection to the browser outside Tauri', async () => {
    await expect(loadSystemLocale()).resolves.toBeNull()
    expect(invokeMock).not.toHaveBeenCalled()
  })

  it('persists locally only in browser development mode', async () => {
    await persistLanguage('en')
    expect(window.localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe('en')
    expect(invokeMock).not.toHaveBeenCalled()
  })
})
