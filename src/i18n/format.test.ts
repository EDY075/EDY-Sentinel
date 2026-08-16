import { describe, expect, it } from 'vitest'
import { formatDate, formatNumber, formatRelativeTime } from './format'

describe('locale-aware formatting', () => {
  it('formats numbers with the selected locale', () => {
    expect(formatNumber(1234.5, undefined, 'en')).toBe('1,234.5')
    expect(formatNumber(1234.5, undefined, 'pt-BR')).toBe('1.234,5')
  })

  it('formats dates and relative time through Intl', () => {
    const date = new Date('2026-08-16T12:00:00Z')
    expect(formatDate(date, { timeZone: 'UTC', year: 'numeric' }, 'en')).toBe('2026')
    expect(formatRelativeTime(-1, 'day', { numeric: 'auto' }, 'en')).toBe('yesterday')
    expect(formatRelativeTime(-1, 'day', { numeric: 'auto' }, 'pt-BR')).toBe('ontem')
  })
})
