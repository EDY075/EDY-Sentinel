import { Activity, Calculator, Clock3, Gauge, Layers3 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { formatDateTime, formatNumber } from '../../i18n'
import type { SecurityScore } from '../../types/score'
import { confidenceLabel, localizedDetectionTitle, severityLabel, severityTone } from '../detections/detectionPresentation'
import { formatBreakdownValue, scoreAvailability } from './scorePresentation'

export function ScoreBreakdownDrawer({ score, open, onClose }: { score: SecurityScore | null; open: boolean; onClose: () => void }) {
  const { t } = useTranslation('score')
  const { t: tDetections } = useTranslation('detections')
  const presentation = scoreAvailability(score, t)
  return <Drawer open={open} title={t('drawer.title')} onClose={onClose} wide><div className="drawer-content score-drawer-content">
    <section className="drawer-section"><h3><Gauge size={15} />{t('drawer.score')}</h3>{presentation.available && score ? <div className="score-drawer-summary"><strong>{formatNumber(presentation.value)}<small>/100</small></strong><div><Badge tone={presentation.value >= 90 ? 'good' : presentation.value >= 70 ? 'accent' : presentation.value >= 50 ? 'warning' : 'danger'}>{presentation.title}</Badge><p>{presentation.detail}</p></div></div> : <div className="score-unavailable"><strong>{presentation.title}</strong><p>{presentation.detail}</p></div>}</section>
    {score && <>
      <section className="drawer-section"><h3><Calculator size={15} />{t('drawer.why')}</h3><div className="score-breakdown-list"><div><span><strong>{t('drawer.baseScore')}</strong><small>{t('drawer.baseScoreDetail')}</small></span><code>{formatNumber(100)}</code></div>{score.breakdown.map((item) => <div key={item.correlationKey}><span><strong>{localizedDetectionTitle(item.title, tDetections)}</strong><small>{severityLabel(item.severity, tDetections)} · {t('drawer.confidence', { confidence: confidenceLabel(item.confidence, tDetections) })}</small></span><code>{formatBreakdownValue(-item.penalty, formatNumber)}</code></div>)}{score.score != null && <div><span><strong>{t('drawer.finalScore')}</strong><small>{t('drawer.formulaShort', { version: formatNumber(score.formulaVersion) })}</small></span><code>{formatNumber(score.score)}</code></div>}</div></section>
      <section className="drawer-section"><h3><Layers3 size={15} />{t('drawer.coverage')}</h3><div className="coverage-list">{score.coverage.map((item) => <div key={item.component}><span><strong>{t(`coverageComponent.${item.component}`, { defaultValue: item.component })}</strong><small>{t(`coverageDetail.${item.status}`, { defaultValue: item.detail })}</small></span><Badge tone={item.status === 'healthy' || item.status === 'available' ? 'good' : item.status === 'degraded' ? 'warning' : 'neutral'}>{t(`coverageStatus.${item.status}`, { defaultValue: item.status })}</Badge></div>)}</div></section>
      <section className="drawer-section"><h3><Activity size={15} />{t('drawer.detectionInput')}</h3><dl><div><dt>{t('drawer.activeDetections')}</dt><dd>{formatNumber(score.activeDetectionCount)}</dd></div><div><dt>{t('drawer.highestSeverity')}</dt><dd>{score.highestSeverity ? <Badge tone={severityTone(score.highestSeverity)}>{severityLabel(score.highestSeverity, tDetections)}</Badge> : t('drawer.none')}</dd></div></dl></section>
      <section className="drawer-section"><h3><Clock3 size={15} />{t('drawer.formula')}</h3><dl><div><dt>{t('drawer.formulaVersion')}</dt><dd>v{formatNumber(score.formulaVersion)}</dd></div><div><dt>{t('drawer.calculated')}</dt><dd>{score.generatedAt ? formatDateTime(score.generatedAt) : t('drawer.notCalculated')}</dd></div></dl><p className="drawer-copy">{t('drawer.disclaimer')}</p></section>
    </>}
  </div></Drawer>
}
