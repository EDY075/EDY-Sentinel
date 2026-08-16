import { describe, expect, it } from 'vitest'
import { confidenceLabel, detectionStatusLabel, evidenceValue, severityLabel, severityTone } from './detectionPresentation'

describe('detection presentation', () => {
  it('always pairs semantic tones with readable labels', () => {
    expect(severityLabel('informational')).toBe('Informational')
    expect(severityTone('high')).toBe('danger')
    expect(confidenceLabel('medium')).toBe('Medium')
    expect(detectionStatusLabel('acknowledged')).toBe('Acknowledged')
  })

  it('renders evidence without inventing missing values', () => {
    expect(evidenceValue(null)).toBe('Unavailable')
    expect(evidenceValue({ path: 'C:\\test.exe' })).toBe('{"path":"C:\\\\test.exe"}')
  })
})
