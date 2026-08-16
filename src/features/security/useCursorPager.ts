import { useCallback, useEffect, useRef, useState } from 'react'

export interface CursorPage<TItem, TCursor> {
  items: TItem[]
  nextCursor?: TCursor
  hasMore: boolean
}

export const advanceCursorStack = <TCursor,>(stack: Array<TCursor | undefined>, cursor: TCursor) => [...stack, cursor]
export const retreatCursorStack = <TCursor,>(stack: Array<TCursor | undefined>) => stack.length > 1 ? stack.slice(0, -1) : stack

export function useCursorPager<TItem, TCursor>({ load, queryKey, refreshKey }: {
  load: (cursor?: TCursor) => Promise<CursorPage<TItem, TCursor>>
  queryKey: string
  refreshKey: number
}) {
  const [cursors, setCursors] = useState<Array<TCursor | undefined>>([undefined])
  const [page, setPage] = useState<CursorPage<TItem, TCursor>>({ items: [], hasMore: false })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string>()
  const [reloadKey, setReloadKey] = useState(0)
  const requestId = useRef(0)
  const cursor = cursors[cursors.length - 1]

  useEffect(() => { setCursors([undefined]) }, [queryKey])

  useEffect(() => {
    const currentRequest = ++requestId.current
    setLoading(true)
    setError(undefined)
    load(cursor).then((result) => {
      if (requestId.current === currentRequest) setPage(result)
    }).catch((reason: unknown) => {
      if (requestId.current === currentRequest) setError(reason instanceof Error ? reason.message : String(reason))
    }).finally(() => {
      if (requestId.current === currentRequest) setLoading(false)
    })
  }, [cursor, load, queryKey, refreshKey, reloadKey])

  const next = useCallback(() => {
    if (page.nextCursor) setCursors((current) => advanceCursorStack(current, page.nextCursor as TCursor))
  }, [page.nextCursor])
  const previous = useCallback(() => setCursors(retreatCursorStack), [])
  const reload = useCallback(() => setReloadKey((value) => value + 1), [])

  return {
    items: page.items,
    hasMore: page.hasMore,
    loading,
    error,
    pageNumber: cursors.length,
    canPrevious: cursors.length > 1,
    next,
    previous,
    reload,
  }
}
