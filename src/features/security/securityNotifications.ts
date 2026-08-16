import type { Detection } from '../../types/detection'

export function unseenDetectionIds(seeded: boolean, observedIds: ReadonlySet<string>, current: Detection[]) {
  if (!seeded) return []
  return current.filter(({ detectionId }) => !observedIds.has(detectionId)).map(({ detectionId }) => detectionId)
}
