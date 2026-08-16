import { Activity, Gauge, ShieldCheck } from 'lucide-react'
import { Badge } from '../../components/ui/primitives'
import type { BaselineState } from '../../types/baseline'
import type { SecurityScore } from '../../types/score'
import { severityLabel, severityTone } from '../detections/detectionPresentation'
import { scoreAvailability } from './scorePresentation'

export function SecurityScorePanel({ score, baselineStatus, onOpen }: { score: SecurityScore | null; baselineStatus: BaselineState; onOpen: () => void }) {
  const presentation = scoreAvailability(score)
  return <section className="panel score-panel" aria-labelledby="security-score-title">
    <header className="panel__header"><div><Gauge size={18} /><span><strong id="security-score-title">Security Score</strong><small>Observed posture under current coverage</small></span></div>{presentation.available && <Badge tone={presentation.value >= 90 ? 'good' : presentation.value >= 70 ? 'accent' : presentation.value >= 50 ? 'warning' : 'danger'}>{presentation.title}</Badge>}</header>
    {presentation.available && score ? <button type="button" className="score-summary" onClick={onOpen} aria-label={`Security Score ${presentation.value} out of 100. View breakdown.`}><span className="score-summary__ring"><strong>{presentation.value}</strong><small>/100</small></span><span><strong>{presentation.title}</strong><small>{presentation.detail}</small><em>View score breakdown</em></span></button> : <div className="score-empty"><span className="score-empty__ring"><Gauge size={21} /></span><div><strong>{presentation.title}</strong><p>{presentation.detail}</p></div></div>}
    <footer><span><ShieldCheck size={13} /> Baseline {baselineStatus.replace('_', ' ')}</span><span><Activity size={13} /> {score?.activeDetectionCount ?? 0} active</span>{score?.highestSeverity && <span><Badge tone={severityTone(score.highestSeverity)}>{severityLabel(score.highestSeverity)}</Badge></span>}</footer>
  </section>
}
