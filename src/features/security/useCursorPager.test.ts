import { describe, expect, it } from 'vitest'
import { advanceCursorStack, retreatCursorStack } from './useCursorPager'

describe('bounded cursor navigation', () => {
  it('retains cursor history without retaining result pages', () => {
    const second = advanceCursorStack([undefined], { id: 'a' })
    const third = advanceCursorStack(second, { id: 'b' })
    expect(third).toEqual([undefined, { id: 'a' }, { id: 'b' }])
    expect(retreatCursorStack(third)).toEqual([undefined, { id: 'a' }])
  })

  it('does not navigate before the first page', () => {
    expect(retreatCursorStack([undefined])).toEqual([undefined])
  })
})
