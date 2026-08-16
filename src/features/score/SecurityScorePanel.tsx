import { Activity, Gauge, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '../../components/ui/primitives'
import { formatNumber } from '../../i18n'
import type { BaselineState } from '../../types/baseline'
import type { SecurityScore } from '../../types/score'
import { severityLabel, severityTone } from '../detections/detectionPresentation'
import { scoreAvailability } from './scorePresentation'

export function SecurityScorePanel({ score, baselineStatus, onOpen }: { score: SecurityScore | null; baselineStatus: BaselineState; onOpen: () => void }) {
  const { t } = useTranslation('score')
  const { t: tDetections } = useTranslation('detections')
  const presentation = scoreAvailability(score, t)
  return <section className="panel score-panel" aria-labelledby="security-score-title">
    <header className="panel__header"><div><Gauge size={18} /><span><strong id="security-score-title">{t('panel.title')}</strong><small>{t('panel.subtitle')}</small></span></div>{presentation.available && <Badge tone={presentation.value >= 90 ? 'good' : presentation.value >= 70 ? 'accent' : presentation.value >= 50 ? 'warning' : 'danger'}>{presentation.title}</Badge>}</header>
    {presentation.available && score ? <button type="button" className="score-summary" onClick={onOpen} aria-label={t('panel.ariaLabel', { score: formatNumber(presentation.value) })}><span className="score-summary__ring"><strong>{formatNumber(presentation.value)}</strong><small>/100</small></span><span><strong>{presentation.title}</strong><small>{presentation.detail}</small><em>{t('panel.viewBreakdown')}</em></span></button> : <div className="score-empty"><span className="score-empty__ring"><Gauge size={21} /></span><div><strong>{presentation.title}</strong><p>{presentation.detail}</p></div></div>}
    <footer><span><ShieldCheck size={13} /> {t('panel.baseline', { status: t(`baselineStatus.${baselineStatus}`) })}</span><span><Activity size={13} /> {t('panel.active', { count: score?.activeDetectionCount ?? 0, formattedCount: formatNumber(score?.activeDetectionCount ?? 0) })}</span>{score?.highestSeverity && <span><Badge tone={severityTone(score.highestSeverity)}>{severityLabel(score.highestSeverity, tDetections)}</Badge></span>}</footer>
  </section>
}
