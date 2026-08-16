import { Activity, Calculator, Clock3, Gauge, Layers3 } from 'lucide-react'
import { Badge, Drawer } from '../../components/ui/primitives'
import type { SecurityScore } from '../../types/score'
import { severityLabel, severityTone, titleCase } from '../detections/detectionPresentation'
import { formatDateTime } from '../telemetry/format'
import { formatBreakdownValue, scoreAvailability } from './scorePresentation'

export function ScoreBreakdownDrawer({ score, open, onClose }: { score: SecurityScore | null; open: boolean; onClose: () => void }) {
  const presentation = scoreAvailability(score)
  return <Drawer open={open} title="Security Score breakdown" onClose={onClose} wide><div className="drawer-content score-drawer-content">
    <section className="drawer-section"><h3><Gauge size={15} />Score</h3>{presentation.available && score ? <div className="score-drawer-summary"><strong>{presentation.value}<small>/100</small></strong><div><Badge tone={presentation.value >= 90 ? 'good' : presentation.value >= 70 ? 'accent' : presentation.value >= 50 ? 'warning' : 'danger'}>{presentation.title}</Badge><p>{presentation.detail}</p></div></div> : <div className="score-unavailable"><strong>{presentation.title}</strong><p>{presentation.detail}</p></div>}</section>
    {score && <>
      <section className="drawer-section"><h3><Calculator size={15} />Why this score?</h3><div className="score-breakdown-list"><div><span><strong>Base score</strong><small>Starting point before active detection penalties</small></span><code>100</code></div>{score.breakdown.map((item) => <div key={item.correlationKey}><span><strong>{item.title}</strong><small>{severityLabel(item.severity)} · {titleCase(item.confidence)} confidence</small></span><code>{formatBreakdownValue(-item.penalty)}</code></div>)}{score.score != null && <div><span><strong>Final score</strong><small>Formula v{score.formulaVersion}</small></span><code>{score.score}</code></div>}</div></section>
      <section className="drawer-section"><h3><Layers3 size={15} />Coverage</h3><div className="coverage-list">{score.coverage.map((item) => <div key={item.component}><span><strong>{item.component}</strong><small>{item.detail}</small></span><Badge tone={item.status === 'available' ? 'good' : item.status === 'degraded' ? 'warning' : 'neutral'}>{titleCase(item.status)}</Badge></div>)}</div></section>
      <section className="drawer-section"><h3><Activity size={15} />Detection input</h3><dl><div><dt>Active detections</dt><dd>{score.activeDetectionCount}</dd></div><div><dt>Highest severity</dt><dd>{score.highestSeverity ? <Badge tone={severityTone(score.highestSeverity)}>{severityLabel(score.highestSeverity)}</Badge> : 'None'}</dd></div></dl></section>
      <section className="drawer-section"><h3><Clock3 size={15} />Formula</h3><dl><div><dt>Formula version</dt><dd>v{score.formulaVersion}</dd></div><div><dt>Calculated</dt><dd>{score.generatedAt ? formatDateTime(score.generatedAt) : 'Not calculated'}</dd></div></dl><p className="drawer-copy">This is an observed posture score under current Sentinel coverage, not a percentage guarantee of security.</p></section>
    </>}
  </div></Drawer>
}
