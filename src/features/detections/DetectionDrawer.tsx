import { useCallback, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, BookOpenCheck, Clock3, FileSearch, Fingerprint, GitBranch, Lightbulb, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { formatDateTime, formatNumber } from '../../i18n'
import { getDetectionEvidence } from '../../lib/tauri'
import type { Detection, DetectionEvidenceCursor, DetectionEvidenceRecord, DetectionStatus } from '../../types/detection'
import type { RuleDefinition } from '../../types/rules'
import { CursorPagination } from '../security/CursorPagination'
import { useCursorPager } from '../security/useCursorPager'
import { confidenceLabel, confidenceTone, detectionStatusLabel, evidenceValue, localizedCollectorSource, localizedDetectionField, localizedDetectionGuidance, localizedEntityType, localizedEvidenceField, localizedEvidenceLabel, localizedEvidenceType, severityLabel, severityTone } from './detectionPresentation'

export function DetectionDrawer({ detection, rule, revision, onClose, onStatusChange, onOpenSourceEvent }: {
  detection: Detection | null
  rule?: RuleDefinition
  revision: number
  onClose: () => void
  onStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
  onOpenSourceEvent: (evidence: DetectionEvidenceRecord) => void
}) {
  const { t } = useTranslation('detections')
  return <Drawer open={Boolean(detection)} title={t('drawer.title')} onClose={onClose} wide>{detection && <DetectionContent detection={detection} rule={rule} revision={revision} onStatusChange={onStatusChange} onOpenSourceEvent={onOpenSourceEvent} />}</Drawer>
}

function DetectionContent({ detection, rule, revision, onStatusChange, onOpenSourceEvent }: {
  detection: Detection
  rule?: RuleDefinition
  revision: number
  onStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
  onOpenSourceEvent: (evidence: DetectionEvidenceRecord) => void
}) {
  const { t } = useTranslation('detections')
  const { t: tRules } = useTranslation('rules')
  const [busy, setBusy] = useState(false)
  const load = useCallback((cursor?: DetectionEvidenceCursor) => getDetectionEvidence({ detectionId: detection.detectionId, cursor, limit: 25 }), [detection.detectionId])
  const evidence = useCursorPager({ load, queryKey: detection.detectionId, refreshKey: revision })
  const changeStatus = async (status: DetectionStatus) => {
    setBusy(true)
    try { await onStatusChange(detection.detectionId, status) } finally { setBusy(false) }
  }
  const guidance = localizedDetectionGuidance(detection.ruleId, detection.remediationGuidance, t)

  return <div className="drawer-content detection-drawer-content">
    <DrawerSection icon={<Activity size={15} />} title={t('drawer.detection')}>
      <Detail label={t('drawer.titleLabel')} value={localizedDetectionField(detection.ruleId, 'title', detection.title, t)} />
      <Detail label={t('drawer.description')} value={localizedDetectionField(detection.ruleId, 'summary', detection.summary, t)} />
      <Detail label={t('drawer.severity')} value={<Badge tone={severityTone(detection.severity)}>{severityLabel(detection.severity, t)}</Badge>} />
      <Detail label={t('drawer.confidence')} value={<Badge tone={confidenceTone(detection.confidence)}>{confidenceLabel(detection.confidence, t)}</Badge>} />
      <Detail label={t('drawer.status')} value={detectionStatusLabel(detection.status, t)} />
    </DrawerSection>
    <DrawerSection icon={<Lightbulb size={15} />} title={t('drawer.whyFlagged')}>
      <Detail label={t('drawer.whatHappened')} value={localizedDetectionField(detection.ruleId, 'whatHappened', detection.explanation.whatHappened, t)} />
      <Detail label={t('drawer.ruleExplanation')} value={localizedDetectionField(detection.ruleId, 'whyFlagged', detection.explanation.whyFlagged, t)} />
      <Detail label={t('drawer.severityRationale')} value={localizedDetectionField(detection.ruleId, 'severityReason', detection.explanation.severityReason, t)} />
      <Detail label={t('drawer.confidenceRationale')} value={localizedDetectionField(detection.ruleId, 'confidenceReason', detection.explanation.confidenceReason, t)} />
    </DrawerSection>
    <DrawerSection icon={<FileSearch size={15} />} title={t('drawer.evidence')}>
      {evidence.error && <div className="inline-error" role="alert">{t('drawer.evidenceLoadError')}</div>}
      <div className="detection-evidence-list">{evidence.items.map((entry, index) => <article key={entry.evidenceId}><span>{formatNumber(index + 1)}</span><div><strong>{localizedEvidenceLabel(entry.label, t)}</strong><EvidenceSnapshot value={entry.value} unavailable={t('unavailable')} t={t} /><small>{localizedEvidenceType(entry.evidenceType, t)} · {localizedCollectorSource(entry.source, t)} · {formatDateTime(entry.observedAt)}</small></div><button type="button" className="button" onClick={() => onOpenSourceEvent(entry)}>{t('drawer.viewEvent')}</button></article>)}</div>
      {!evidence.loading && !evidence.items.length && !evidence.error && <p className="drawer-empty">{t('drawer.noEvidence')}</p>}
      {evidence.hasMore || evidence.canPrevious ? <CursorPagination pageNumber={evidence.pageNumber} itemCount={evidence.items.length} nounKey="evidence" hasMore={evidence.hasMore} canPrevious={evidence.canPrevious} loading={evidence.loading} onPrevious={evidence.previous} onNext={evidence.next} onRetry={evidence.reload} /> : null}
    </DrawerSection>
    <DrawerSection icon={<Fingerprint size={15} />} title={t('drawer.entityAndBaseline')}><Detail label={t('drawer.entityType')} value={localizedEntityType(detection.entityType, t)} /><Detail label={t('drawer.entityKey')} value={detection.entityKey} mono /><Detail label={t('drawer.knownInBaseline')} value={detection.baselineId ? t('drawer.comparedBaseline') : t('drawer.noComparableBaseline')} /><Detail label={t('drawer.baselineId')} value={detection.baselineId ?? t('drawer.notApplicable')} mono /></DrawerSection>
    <DrawerSection icon={<GitBranch size={15} />} title={t('drawer.rule')}><Detail label={t('drawer.ruleId')} value={detection.ruleId} mono /><Detail label={t('drawer.ruleName')} value={rule ? tRules(`definitions.${rule.ruleId}.name`, { defaultValue: rule.name }) : detection.ruleId} /><Detail label={t('drawer.version')} value={formatNumber(detection.ruleVersion)} /><Detail label={t('drawer.category')} value={rule ? tRules(`categories.${rule.category}`, { defaultValue: rule.category }) : t('drawer.unavailable')} /></DrawerSection>
    <DrawerSection icon={<Clock3 size={15} />} title={t('drawer.timeline')}><Detail label={t('drawer.firstDetected')} value={formatDateTime(detection.firstDetectedAt)} /><Detail label={t('drawer.lastDetected')} value={formatDateTime(detection.lastDetectedAt)} /><Detail label={t('drawer.occurrences')} value={formatNumber(detection.occurrenceCount)} /><Detail label={t('drawer.condition')} value={detection.conditionActive ? t('drawer.active') : t('drawer.inactive')} /></DrawerSection>
    <DrawerSection icon={<BookOpenCheck size={15} />} title={t('drawer.recommendedAction')}><ul className="guidance-list">{guidance.map((item, index) => <li key={`${detection.ruleId}-${index}`}>{item}</li>)}</ul></DrawerSection>
    <DrawerSection icon={<ShieldCheck size={15} />} title={t('drawer.provenance')}><p className="drawer-copy">{t('drawer.provenanceCopy')}</p></DrawerSection>
    <div className="event-actions" aria-label={t('drawer.actionsAriaLabel')}><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('investigating')}>{t('drawer.investigate')}</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('acknowledged')}>{t('drawer.acknowledge')}</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('resolved')}>{t('drawer.resolve')}</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('ignored')}>{t('drawer.ignore')}</button></div>
  </div>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: ReactNode; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined}>{value}</dd></div> }
function EvidenceSnapshot({ value, unavailable, t }: { value: unknown; unavailable: string; t: ReturnType<typeof useTranslation<'detections'>>['t'] }) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return <code>{evidenceValue(value, unavailable)}</code>
  return <div className="evidence-snapshot">{Object.entries(value as Record<string, unknown>).map(([key, item]) => <div key={key}><span>{localizedEvidenceField(key, t)}</span><code>{evidenceValue(item, unavailable)}</code></div>)}</div>
}
