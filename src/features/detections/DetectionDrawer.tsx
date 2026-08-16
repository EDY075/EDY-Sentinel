import { useCallback, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, BookOpenCheck, Clock3, FileSearch, Fingerprint, GitBranch, Lightbulb, ShieldCheck } from 'lucide-react'
import { Badge, Drawer } from '../../components/ui/primitives'
import { getDetectionEvidence } from '../../lib/tauri'
import type { Detection, DetectionEvidenceCursor, DetectionEvidenceRecord, DetectionStatus } from '../../types/detection'
import type { RuleDefinition } from '../../types/rules'
import { CursorPagination } from '../security/CursorPagination'
import { useCursorPager } from '../security/useCursorPager'
import { formatDateTime } from '../telemetry/format'
import { confidenceLabel, confidenceTone, detectionStatusLabel, evidenceValue, severityLabel, severityTone, titleCase } from './detectionPresentation'

export function DetectionDrawer({ detection, rule, revision, onClose, onStatusChange, onOpenSourceEvent }: {
  detection: Detection | null
  rule?: RuleDefinition
  revision: number
  onClose: () => void
  onStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
  onOpenSourceEvent: (evidence: DetectionEvidenceRecord) => void
}) {
  return <Drawer open={Boolean(detection)} title="Detection details" onClose={onClose} wide>{detection && <DetectionContent detection={detection} rule={rule} revision={revision} onStatusChange={onStatusChange} onOpenSourceEvent={onOpenSourceEvent} />}</Drawer>
}

function DetectionContent({ detection, rule, revision, onStatusChange, onOpenSourceEvent }: {
  detection: Detection
  rule?: RuleDefinition
  revision: number
  onStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
  onOpenSourceEvent: (evidence: DetectionEvidenceRecord) => void
}) {
  const [busy, setBusy] = useState(false)
  const load = useCallback((cursor?: DetectionEvidenceCursor) => getDetectionEvidence({ detectionId: detection.detectionId, cursor, limit: 25 }), [detection.detectionId])
  const evidence = useCursorPager({ load, queryKey: detection.detectionId, refreshKey: revision })
  const changeStatus = async (status: DetectionStatus) => {
    setBusy(true)
    try { await onStatusChange(detection.detectionId, status) } finally { setBusy(false) }
  }

  return <div className="drawer-content detection-drawer-content">
    <DrawerSection icon={<Activity size={15} />} title="Detection">
      <Detail label="Title" value={detection.title} />
      <Detail label="Description" value={detection.summary} />
      <Detail label="Severity" value={<Badge tone={severityTone(detection.severity)}>{severityLabel(detection.severity)}</Badge>} />
      <Detail label="Confidence" value={<Badge tone={confidenceTone(detection.confidence)}>{confidenceLabel(detection.confidence)}</Badge>} />
      <Detail label="Status" value={detectionStatusLabel(detection.status)} />
    </DrawerSection>
    <DrawerSection icon={<Lightbulb size={15} />} title="Why this was flagged">
      <Detail label="What happened" value={detection.explanation.whatHappened} />
      <Detail label="Rule explanation" value={detection.explanation.whyFlagged} />
      <Detail label="Severity rationale" value={detection.explanation.severityReason} />
      <Detail label="Confidence rationale" value={detection.explanation.confidenceReason} />
    </DrawerSection>
    <DrawerSection icon={<FileSearch size={15} />} title="Evidence">
      {evidence.error && <div className="inline-error" role="alert">Evidence could not be loaded: {evidence.error}</div>}
      <div className="detection-evidence-list">{evidence.items.map((entry, index) => <article key={entry.evidenceId}><span>{index + 1}</span><div><strong>{entry.label}</strong><EvidenceSnapshot value={entry.value} /><small>{titleCase(entry.evidenceType)} · {entry.source} · {formatDateTime(entry.observedAt)}</small></div><button type="button" className="button" onClick={() => onOpenSourceEvent(entry)}>View event</button></article>)}</div>
      {!evidence.loading && !evidence.items.length && !evidence.error && <p className="drawer-empty">No evidence records were returned. This detection should be reviewed as incomplete.</p>}
      {evidence.hasMore || evidence.canPrevious ? <CursorPagination pageNumber={evidence.pageNumber} itemCount={evidence.items.length} noun="evidence records" hasMore={evidence.hasMore} canPrevious={evidence.canPrevious} loading={evidence.loading} onPrevious={evidence.previous} onNext={evidence.next} onRetry={evidence.reload} /> : null}
    </DrawerSection>
    <DrawerSection icon={<Fingerprint size={15} />} title="Entity and baseline"><Detail label="Entity type" value={detection.entityType} /><Detail label="Entity key" value={detection.entityKey} mono /><Detail label="Known in baseline" value={detection.baselineId ? 'Compared with the active baseline' : 'No comparable baseline reference'} /><Detail label="Baseline ID" value={detection.baselineId ?? 'Not applicable'} mono /></DrawerSection>
    <DrawerSection icon={<GitBranch size={15} />} title="Rule"><Detail label="Rule ID" value={detection.ruleId} mono /><Detail label="Rule name" value={rule?.name ?? detection.ruleId} /><Detail label="Version" value={String(detection.ruleVersion)} /><Detail label="Category" value={rule?.category ?? 'Unavailable'} /></DrawerSection>
    <DrawerSection icon={<Clock3 size={15} />} title="Timeline"><Detail label="First detected" value={formatDateTime(detection.firstDetectedAt)} /><Detail label="Last detected" value={formatDateTime(detection.lastDetectedAt)} /><Detail label="Occurrences" value={String(detection.occurrenceCount)} /><Detail label="Condition" value={detection.conditionActive ? 'Active' : 'No longer active'} /></DrawerSection>
    <DrawerSection icon={<BookOpenCheck size={15} />} title="Recommended action"><ul className="guidance-list">{detection.remediationGuidance.map((item) => <li key={item}>{item}</li>)}</ul></DrawerSection>
    <DrawerSection icon={<ShieldCheck size={15} />} title="Provenance"><p className="drawer-copy">Each evidence record links to its original factual Security Event. Those events retain their collector, baseline context, timestamps and append-only history.</p></DrawerSection>
    <div className="event-actions" aria-label="Detection status actions"><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('investigating')}>Investigate</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('acknowledged')}>Acknowledge</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('resolved')}>Resolve</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('ignored')}>Ignore</button></div>
  </div>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: ReactNode; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined}>{value}</dd></div> }
function EvidenceSnapshot({ value }: { value: unknown }) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return <code>{evidenceValue(value)}</code>
  return <div className="evidence-snapshot">{Object.entries(value as Record<string, unknown>).map(([key, item]) => <div key={key}><span>{titleCase(key)}</span><code>{evidenceValue(item)}</code></div>)}</div>
}
