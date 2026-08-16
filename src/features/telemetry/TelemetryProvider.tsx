/* oxlint-disable react/only-export-components */
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { completeBaselineLearning, getBaselineSummary, getDatabaseStatus, getDetectionsPage, getLiveTelemetry, getSecurityScore, getSystemOverview, resetBaseline, startNewBaseline, updateDetectionStatus, updateSecurityEventStatus } from '../../lib/tauri'
import type { DatabaseStatus, SystemOverview } from '../../types/system'
import type { CollectorHealth, LiveTelemetrySnapshot } from '../../types/telemetry'
import type { BaselineAction, BaselineSummary, SecurityEventStatus } from '../../types/baseline'
import type { DetectionStatus } from '../../types/detection'
import type { SecurityScore } from '../../types/score'
import { applyPausedState, deriveCollectorHealth } from './health'
import { unseenDetectionIds } from '../security/securityNotifications'

const LIVE_INTERVAL_MS = 2_500
const SYSTEM_INTERVAL_MS = 15_000
const SECURITY_INTERVAL_MS = 5_000

interface TelemetryContextValue {
  live: boolean
  loading: boolean
  refreshing: boolean
  overview: SystemOverview | null
  database: DatabaseStatus | null
  snapshot: LiveTelemetrySnapshot | null
  error: string | null
  health: CollectorHealth[]
  baseline: BaselineSummary | null
  securityScore: SecurityScore | null
  securityError: string | null
  securityRevision: number
  securityNotice: { id: number; count: number } | null
  setLive: (live: boolean) => void
  refresh: () => Promise<void>
  refreshSecurity: () => Promise<void>
  startBaseline: (input: BaselineAction) => Promise<void>
  resetBaseline: (input: BaselineAction) => Promise<void>
  completeBaseline: (input: BaselineAction) => Promise<void>
  setEventStatus: (eventId: string, status: SecurityEventStatus) => Promise<void>
  setDetectionStatus: (detectionId: string, status: DetectionStatus) => Promise<void>
  clearSecurityNotice: () => void
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
  const [baseline, setBaseline] = useState<BaselineSummary | null>(null)
  const [securityScore, setSecurityScore] = useState<SecurityScore | null>(null)
  const [securityError, setSecurityError] = useState<string | null>(null)
  const [securityRevision, setSecurityRevision] = useState(0)
  const [securityNotice, setSecurityNotice] = useState<{ id: number; count: number } | null>(null)
  const [systemFailure, setSystemFailure] = useState<string>()
  const [liveFailure, setLiveFailure] = useState<string>()
  const collecting = useRef(false)
  const lastSystemAt = useRef(0)
  const overviewRef = useRef<SystemOverview | null>(null)
  const securityCollecting = useRef(false)
  const lastSecurityAt = useRef(0)
  const seededDetections = useRef(false)
  const observedDetectionIds = useRef(new Set<string>())
  const noticeSequence = useRef(0)

  const refreshSecurity = useCallback(async (force = true) => {
    if (securityCollecting.current) return
    if (!force && Date.now() - lastSecurityAt.current < SECURITY_INTERVAL_MS) return
    securityCollecting.current = true
    try {
      const [baselineResult, scoreResult, detectionsResult] = await Promise.allSettled([
        getBaselineSummary(),
        getSecurityScore(),
        getDetectionsPage({ statuses: ['new'], limit: 50 }),
      ])
      if (baselineResult.status === 'fulfilled') setBaseline(baselineResult.value)
      if (scoreResult.status === 'fulfilled') setSecurityScore(scoreResult.value)
      if (detectionsResult.status === 'fulfilled') {
        const unseen = unseenDetectionIds(seededDetections.current, observedDetectionIds.current, detectionsResult.value.items)
        detectionsResult.value.items.forEach(({ detectionId }) => observedDetectionIds.current.add(detectionId))
        if (seededDetections.current && unseen.length) {
          noticeSequence.current += 1
          setSecurityNotice({ id: noticeSequence.current, count: unseen.length })
        }
        seededDetections.current = true
      }
      if (baselineResult.status === 'rejected' || scoreResult.status === 'rejected' || detectionsResult.status === 'rejected') {
        const reason = baselineResult.status === 'rejected' ? baselineResult.reason : scoreResult.status === 'rejected' ? scoreResult.reason : detectionsResult.status === 'rejected' ? detectionsResult.reason : 'Security state unavailable'
        setSecurityError(reasonText(reason))
      } else setSecurityError(null)
      setSecurityRevision((value) => value + 1)
      lastSecurityAt.current = Date.now()
    } finally {
      securityCollecting.current = false
    }
  }, [])

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
      await refreshSecurity(forceSystem)
    } finally {
      collecting.current = false
      setLoading(false)
      setRefreshing(false)
    }
  }, [refreshSecurity])

  useEffect(() => { void collect(true) }, [collect])
  useEffect(() => {
    if (!live) return
    const interval = window.setInterval(() => void collect(), LIVE_INTERVAL_MS)
    return () => window.clearInterval(interval)
  }, [collect, live])

  const health = useMemo<CollectorHealth[]>(() => applyPausedState(deriveCollectorHealth(loading, overview, snapshot, { system: systemFailure, live: liveFailure }), live), [live, liveFailure, loading, overview, snapshot, systemFailure])

  const startBaseline = useCallback(async (input: BaselineAction) => { setBaseline(await startNewBaseline(input)); lastSecurityAt.current = 0; await refreshSecurity(true) }, [refreshSecurity])
  const resetCurrentBaseline = useCallback(async (input: BaselineAction) => { setBaseline(await resetBaseline(input)); lastSecurityAt.current = 0; await refreshSecurity(true) }, [refreshSecurity])
  const completeBaseline = useCallback(async (input: BaselineAction) => { setBaseline(await completeBaselineLearning(input)); lastSecurityAt.current = 0; await refreshSecurity(true) }, [refreshSecurity])
  const setEventStatus = useCallback(async (eventId: string, status: SecurityEventStatus) => { await updateSecurityEventStatus(eventId, status); await refreshSecurity(true) }, [refreshSecurity])
  const setCurrentDetectionStatus = useCallback(async (detectionId: string, status: DetectionStatus) => { await updateDetectionStatus(detectionId, status); await refreshSecurity(true) }, [refreshSecurity])
  const clearSecurityNotice = useCallback(() => setSecurityNotice(null), [])

  const value = useMemo(() => ({ live, loading, refreshing, overview, database, snapshot, error, health, baseline, securityScore, securityError, securityRevision, securityNotice, setLive, refresh: () => collect(true), refreshSecurity: () => refreshSecurity(true), startBaseline, resetBaseline: resetCurrentBaseline, completeBaseline, setEventStatus, setDetectionStatus: setCurrentDetectionStatus, clearSecurityNotice }), [baseline, clearSecurityNotice, collect, completeBaseline, database, error, health, live, loading, overview, refreshSecurity, refreshing, resetCurrentBaseline, securityError, securityNotice, securityRevision, securityScore, setCurrentDetectionStatus, setEventStatus, snapshot, startBaseline])
  return <TelemetryContext.Provider value={value}>{children}</TelemetryContext.Provider>
}

export function useTelemetry() {
  const context = useContext(TelemetryContext)
  if (!context) throw new Error('useTelemetry must be used within TelemetryProvider')
  return context
}

export const telemetryIntervals = { live: LIVE_INTERVAL_MS, system: SYSTEM_INTERVAL_MS }
