import { useCallback, useEffect, useState } from 'react'
import type { KeyboardEvent } from 'react'
import { RefreshCw, Settings2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { getDetectionRules, setDetectionRuleEnabled } from '../../lib/tauri'
import type { SecurityEventStatus } from '../../types/baseline'
import type { DetectionEvidenceRecord, DetectionStatus } from '../../types/detection'
import type { RuleDefinition } from '../../types/rules'
import { DetectionsView } from '../detections/DetectionsView'
import { SecurityEventsView } from '../events/SecurityEventsView'
import type { SecurityEventTarget } from '../events/SecurityEventsView'
import { RulesView } from '../rules/RulesView'

export type SecuritySection = 'detections' | 'events' | 'rules'

export function SecurityWorkspace({ section, revision, error, onSectionChange, onRefresh, onEventStatusChange, onDetectionStatusChange }: {
  section: SecuritySection
  revision: number
  error: string | null
  onSectionChange: (section: SecuritySection) => void
  onRefresh: () => Promise<void>
  onEventStatusChange: (eventId: string, status: SecurityEventStatus) => Promise<void>
  onDetectionStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
}) {
  const { t } = useTranslation('security')
  const [rules, setRules] = useState<RuleDefinition[]>([])
  const [rulesLoading, setRulesLoading] = useState(true)
  const [rulesError, setRulesError] = useState<string>()
  const [busyRuleId, setBusyRuleId] = useState<string>()
  const [requestedEvent, setRequestedEvent] = useState<SecurityEventTarget>()

  const loadRules = useCallback(async () => {
    setRulesLoading(true)
    setRulesError(undefined)
    try { setRules(await getDetectionRules()) }
    catch (reason) { setRulesError(reason instanceof Error ? reason.message : String(reason)) }
    finally { setRulesLoading(false) }
  }, [])
  useEffect(() => { void loadRules() }, [loadRules])

  const toggleRule = async (rule: RuleDefinition) => {
    setBusyRuleId(rule.ruleId)
    try {
      await setDetectionRuleEnabled(rule.ruleId, !rule.enabled)
      await Promise.all([loadRules(), onRefresh()])
    } finally { setBusyRuleId(undefined) }
  }
  const openSourceEvent = (evidence: DetectionEvidenceRecord) => {
    setRequestedEvent({ requestId: Date.now(), eventId: evidence.eventId, eventType: evidence.eventType, entityType: evidence.entityType, entityKey: evidence.entityKey })
    onSectionChange('events')
  }
  const onTabKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    const tabs: SecuritySection[] = ['detections', 'events']
    const current = tabs.indexOf(section === 'events' ? 'events' : 'detections')
    const next = event.key === 'ArrowRight' ? (current + 1) % tabs.length : event.key === 'ArrowLeft' ? (current - 1 + tabs.length) % tabs.length : event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : -1
    if (next < 0) return
    event.preventDefault()
    onSectionChange(tabs[next])
    const buttons = event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('[role="tab"]')
    buttons?.[next]?.focus()
  }

  if (section === 'rules') return <RulesView rules={rules} loading={rulesLoading} error={rulesError} busyRuleId={busyRuleId} onBack={() => onSectionChange('detections')} onToggle={toggleRule} onRetry={loadRules} />

  return <div className="security-workspace">
    <header className="security-workspace__nav">
      <div className="security-tabs" role="tablist" aria-label={t('workspace.ariaLabel')}>
        <button type="button" role="tab" aria-selected={section === 'detections'} tabIndex={section === 'detections' ? 0 : -1} onKeyDown={onTabKeyDown} onClick={() => onSectionChange('detections')}>{t('workspace.detections')}</button>
        <button type="button" role="tab" aria-selected={section === 'events'} tabIndex={section === 'events' ? 0 : -1} onKeyDown={onTabKeyDown} onClick={() => onSectionChange('events')}>{t('workspace.events')}</button>
      </div>
      <div><button type="button" className="button" onClick={() => void onRefresh()}><RefreshCw size={14} /> {t('workspace.refresh')}</button><button type="button" className="button" onClick={() => onSectionChange('rules')}><Settings2 size={14} /> {t('workspace.rules')}</button></div>
    </header>
    {error && <div className="security-boundary-warning" role="status">{t('workspace.warning')}</div>}
    <div role="tabpanel" aria-label={section === 'detections' ? t('workspace.detectionsPanel') : t('workspace.eventsPanel')}>
      {section === 'detections' ? <DetectionsView revision={revision} rules={rules} onStatusChange={onDetectionStatusChange} onOpenSourceEvent={openSourceEvent} /> : <SecurityEventsView revision={revision} requestedEvent={requestedEvent} onStatusChange={onEventStatusChange} />}
    </div>
  </div>
}
