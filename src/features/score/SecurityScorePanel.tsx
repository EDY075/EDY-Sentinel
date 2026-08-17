import { Activity, Gauge, ShieldCheck, ShieldQuestion } from 'lucide-react'
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
  const confirmedCount = score?.productVulnerabilityRisks.reduce((total, product) => total + product.confirmedCount, 0) ?? 0
  return <section className="panel score-panel" aria-labelledby="security-score-title">
    <header className="panel__header"><div><Gauge size={18} /><span><strong id="security-score-title">{t('panel.title')}</strong><small>{t('panel.subtitle')}</small></span></div>{presentation.available && <Badge tone={presentation.limited ? 'warning' : presentation.value >= 90 ? 'good' : presentation.value >= 70 ? 'accent' : presentation.value >= 50 ? 'warning' : 'danger'}>{presentation.limited ? t('state.limited') : presentation.title}</Badge>}</header>
    {presentation.available && score ? <button type="button" className="score-summary" data-limited={presentation.limited || undefined} onClick={onOpen} aria-label={t('panel.ariaLabel', { score: formatNumber(presentation.value), state: presentation.title })}><span className="score-summary__ring"><strong>{formatNumber(presentation.value)}</strong><small>/100</small></span><span><strong>{presentation.title}</strong><small>{presentation.detail}</small><span className="score-summary__components"><span>{t('panel.detectionsImpact', { value: formatNumber(-score.detectionPenalty) })}</span><span>{t('panel.vulnerabilitiesImpact', { value: formatNumber(-score.vulnerabilityPenalty) })}</span></span><em>{t('panel.viewBreakdown')}</em></span></button> : <div className="score-empty"><span className="score-empty__ring"><Gauge size={21} /></span><div><strong>{presentation.title}</strong><p>{presentation.detail}</p></div></div>}
    <footer><span><ShieldCheck size={13} /> {t('panel.baseline', { status: t(`baselineStatus.${baselineStatus}`) })}</span><span><ShieldQuestion size={13} /> {t('panel.attention', { count: confirmedCount + (score?.activeDetectionCount ?? 0), formattedCount: formatNumber(confirmedCount + (score?.activeDetectionCount ?? 0)) })}</span><span><Activity size={13} /> {score ? t(`vulnerability.states.${score.vulnerabilityCoverage.status}`) : t('vulnerability.states.unavailable')}</span>{score?.highestSeverity && <span><Badge tone={severityTone(score.highestSeverity)}>{severityLabel(score.highestSeverity, tDetections)}</Badge></span>}</footer>
  </section>
}
