import { describe, expect, it } from 'vitest'
import type { Detection } from '../../types/detection'
import { unseenDetectionIds } from './securityNotifications'

const detection = (detectionId: string) => ({ detectionId } as Detection)

describe('detection notification watermark', () => {
  it('seeds silently on first hydration', () => {
    expect(unseenDetectionIds(false, new Set(), [detection('one')])).toEqual([])
  })

  it('notifies only identifiers not observed after the seed', () => {
    expect(unseenDetectionIds(true, new Set(['one']), [detection('one'), detection('two')])).toEqual(['two'])
  })
})
