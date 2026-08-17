import { describe, expect, it } from 'vitest'
import { isSafeExternalReference } from './externalReferences'

describe('CVE external references', () => {
  it('allows only credential-free HTTPS URLs', () => {
    expect(isSafeExternalReference('https://example.invalid/advisory')).toBe(true)
    expect(isSafeExternalReference('http://example.invalid/advisory')).toBe(false)
    expect(isSafeExternalReference('https://user:password@example.invalid/advisory')).toBe(false)
    expect(isSafeExternalReference('javascript:alert(1)')).toBe(false)
    expect(isSafeExternalReference('not a URL')).toBe(false)
  })
})
