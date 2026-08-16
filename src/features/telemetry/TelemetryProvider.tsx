/* oxlint-disable react/only-export-components */
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { getDatabaseStatus, getLiveTelemetry, getSystemOverview } from '../../lib/tauri'
import type { DatabaseStatus, SystemOverview } from '../../types/system'
import type { CollectorHealth, LiveTelemetrySnapshot } from '../../types/telemetry'
import { applyPausedState, deriveCollectorHealth } from './health'

const LIVE_INTERVAL_MS = 2_500
const SYSTEM_INTERVAL_MS = 15_000

interface TelemetryContextValue {
  live: boolean
  loading: boolean
  refreshing: boolean
  overview: SystemOverview | null
  database: DatabaseStatus | null
  snapshot: LiveTelemetrySnapshot | null
  error: string | null
  health: CollectorHealth[]
  setLive: (live: boolean) => void
  refresh: () => Promise<void>
}

const TelemetryContext = createContext<TelemetryContextValue | null>(null)

const reasonText = (reason: unknown) => reason instanceof Error ? reason.message : String(reason)

export function TelemetryProvider({ children }: { children: ReactNode }) {
  const [live, setLive] = useState(true)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [overview, setOverview] = useState<SystemOverview | null>(null)
  const [database, setDatabase] = useState<DatabaseStatus | null>(null)
  const [snapshot, setSnapshot] = useState<LiveTelemetrySnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [systemFailure, setSystemFailure] = useState<string>()
  const [liveFailure, setLiveFailure] = useState<string>()
  const collecting = useRef(false)
  const lastSystemAt = useRef(0)
  const overviewRef = useRef<SystemOverview | null>(null)

  const collect = useCallback(async (forceSystem = false) => {
    if (collecting.current) return
    collecting.current = true
    setRefreshing(true)
    setError(null)
    try {
      const now = Date.now()
      const shouldCollectSystem = forceSystem || !overviewRef.current || now - lastSystemAt.current >= SYSTEM_INTERVAL_MS
      const liveRequest = getLiveTelemetry()
      const systemRequest = shouldCollectSystem ? getSystemOverview() : Promise.resolve(overviewRef.current)
      const [liveResult, systemResult] = await Promise.allSettled([liveRequest, systemRequest])

      if (liveResult.status === 'fulfilled') { setSnapshot(liveResult.value); setLiveFailure(undefined) }
      else { const message = reasonText(liveResult.reason); setLiveFailure(message); setError(message) }

      if (systemResult.status === 'fulfilled' && systemResult.value) {
        setOverview(systemResult.value)
        overviewRef.current = systemResult.value
        setSystemFailure(undefined)
        if (shouldCollectSystem) lastSystemAt.current = now
      } else if (systemResult.status === 'rejected') {
        const message = reasonText(systemResult.reason)
        setSystemFailure(message)
        if (liveResult.status === 'rejected') setError(message)
      }

      if (shouldCollectSystem) {
        try { setDatabase(await getDatabaseStatus()) } catch { setDatabase(null) }
      }
    } finally {
      collecting.current = false
      setLoading(false)
      setRefreshing(false)
    }
  }, [])

  useEffect(() => { void collect(true) }, [collect])
  useEffect(() => {
    if (!live) return
    const interval = window.setInterval(() => void collect(), LIVE_INTERVAL_MS)
    return () => window.clearInterval(interval)
  }, [collect, live])

  const health = useMemo<CollectorHealth[]>(() => applyPausedState(deriveCollectorHealth(loading, overview, snapshot, { system: systemFailure, live: liveFailure }), live), [live, liveFailure, loading, overview, snapshot, systemFailure])

  const value = useMemo(() => ({ live, loading, refreshing, overview, database, snapshot, error, health, setLive, refresh: () => collect(true) }), [collect, database, error, health, live, loading, overview, refreshing, snapshot])
  return <TelemetryContext.Provider value={value}>{children}</TelemetryContext.Provider>
}

export function useTelemetry() {
  const context = useContext(TelemetryContext)
  if (!context) throw new Error('useTelemetry must be used within TelemetryProvider')
  return context
}

export const telemetryIntervals = { live: LIVE_INTERVAL_MS, system: SYSTEM_INTERVAL_MS }
