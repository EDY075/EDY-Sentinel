import { useEffect, useState } from 'react'
import { Activity, ArrowLeft, Calculator, Clock3, ExternalLink, Gauge, Layers3, PackageSearch, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { formatDateTime, formatNumber } from '../../i18n'
import type { ProductVulnerabilityRisk, SecurityScore, VulnerabilityCoverageStatus } from '../../types/score'
import { confidenceLabel, localizedDetectionTitle, severityLabel } from '../detections/detectionPresentation'
import { formatBreakdownValue, scoreAvailability } from './scorePresentation'

interface ScoreBreakdownDrawerProps {
  score: SecurityScore | null
  open: boolean
  onClose: () => void
  onOpenProduct: (softwareId: string) => void
}

export function ScoreBreakdownDrawer({ score, open, onClose, onOpenProduct }: ScoreBreakdownDrawerProps) {
  const { t } = useTranslation('score')
  const { t: tDetections } = useTranslation('detections')
  const [selectedProductKey, setSelectedProductKey] = useState<string>()
  useEffect(() => { if (!open) setSelectedProductKey(undefined) }, [open])
  const presentation = scoreAvailability(score, t)
  const selectedProduct = score?.productVulnerabilityRisks.find(({ productRiskKey }) => productRiskKey === selectedProductKey)
  const coverageTone = score ? vulnerabilityCoverageTone(score.vulnerabilityCoverage.status) : 'neutral'
  const confirmedTotal = score?.productVulnerabilityRisks.reduce((sum, product) => sum + product.confirmedCount, 0) ?? 0

  return <Drawer open={open} title={selectedProduct ? t('productDetail.title') : t('drawer.title')} onClose={onClose} wide><div className="drawer-content score-drawer-content">
    {selectedProduct ? <ProductRiskDetail product={selectedProduct} onBack={() => setSelectedProductKey(undefined)} onOpenProduct={onOpenProduct} /> : <>
      <section className="drawer-section"><h3><Gauge size={15} />{t('drawer.score')}</h3>{presentation.available && score ? <div className="score-drawer-summary" data-limited={presentation.limited || undefined}><strong>{formatNumber(presentation.value)}<small>/100</small></strong><div><Badge tone={presentation.limited ? 'warning' : presentation.value >= 90 ? 'good' : presentation.value >= 70 ? 'accent' : presentation.value >= 50 ? 'warning' : 'danger'}>{presentation.title}</Badge><p>{presentation.detail}</p></div></div> : <div className="score-unavailable"><strong>{presentation.title}</strong><p>{presentation.detail}</p></div>}<p className="drawer-copy score-posture-copy">{t('drawer.observablePosture')}</p></section>
      {score && <>
        <section className="drawer-section"><h3><Calculator size={15} />{t('drawer.why')}</h3><div className="score-breakdown-list score-breakdown-list--categories">
          <div><span><strong>{t('drawer.baseScore')}</strong><small>{t('drawer.baseScoreDetail')}</small></span><code>{formatNumber(100)}</code></div>
          <div><span><strong>{t('drawer.detections')}</strong><small>{t('drawer.detectionsDetail', { count: score.activeDetectionCount, formattedCount: formatNumber(score.activeDetectionCount) })}</small></span><code>{formatBreakdownValue(-score.detectionPenalty, formatNumber)}</code></div>
          <div><span><strong>{t('drawer.vulnerabilities')}</strong><small>{t('drawer.vulnerabilitiesDetail', { count: confirmedTotal, formattedCount: formatNumber(confirmedTotal) })}</small></span><code>{formatBreakdownValue(-score.vulnerabilityPenalty, formatNumber)}</code></div>
          {score.score != null && <div><span><strong>{t('drawer.finalScore')}</strong><small>{t('drawer.formulaShort', { version: formatNumber(score.formulaVersion) })}</small></span><code>{formatNumber(score.score)}</code></div>}
        </div></section>
        {score.breakdown.length > 0 && <section className="drawer-section"><h3><Activity size={15} />{t('drawer.detectionInput')}</h3><div className="score-breakdown-list">{score.breakdown.map((item) => <div key={item.correlationKey}><span><strong>{localizedDetectionTitle(item.title, tDetections)}</strong><small>{severityLabel(item.severity, tDetections)} · {t('drawer.confidence', { confidence: confidenceLabel(item.confidence, tDetections) })}</small></span><code>{formatBreakdownValue(-item.penalty, formatNumber)}</code></div>)}</div></section>}
        <section className="drawer-section"><h3><ShieldAlert size={15} />{t('vulnerability.category')}</h3><p className="drawer-copy">{t('vulnerability.included')}</p><div className="product-risk-list">{score.productVulnerabilityRisks.length ? score.productVulnerabilityRisks.map((product) => <button type="button" key={product.productRiskKey} onClick={() => setSelectedProductKey(product.productRiskKey)} aria-label={t('product.openAria', { product: product.displayNames[0] ?? product.canonicalProduct })}><span className="product-risk-list__identity"><PackageSearch size={16} /><span><strong>{product.displayNames[0] ?? product.canonicalProduct}</strong><small>{t('product.confirmed', { count: product.confirmedCount, formattedCount: formatNumber(product.confirmedCount) })}{product.possibleCount ? ` · ${t('product.possible', { count: product.possibleCount, formattedCount: formatNumber(product.possibleCount) })}` : ''}</small></span></span><span className="product-risk-list__badges">{product.confirmedKevCount > 0 && <Badge tone="warning">{t('product.kev', { count: product.confirmedKevCount, formattedCount: formatNumber(product.confirmedKevCount) })}</Badge>}{product.possibleCount > 0 && <Badge>{t('product.possibleImpactZero')}</Badge>}<code>{formatBreakdownValue(-product.impact, formatNumber)}</code></span></button>) : <p className="drawer-empty">{t('product.none')}</p>}</div><p className="drawer-copy">{t('vulnerability.possibleNoImpact')}</p></section>
        <section className="drawer-section"><h3><Layers3 size={15} />{t('vulnerability.coverage')}</h3><div className="vulnerability-coverage-heading"><span><strong>{t(`vulnerability.states.${score.vulnerabilityCoverage.status}`)}</strong><small>{t('vulnerability.coverageBasis')}</small></span><Badge tone={coverageTone}>{t(`vulnerability.states.${score.vulnerabilityCoverage.status}`)}</Badge></div><dl className="coverage-metrics"><Metric label={t('coverage.totalSoftware')} value={score.vulnerabilityCoverage.totalSoftware} /><Metric label={t('coverage.eligible')} value={score.vulnerabilityCoverage.eligibleSoftware} /><Metric label={t('coverage.resolved')} value={score.vulnerabilityCoverage.resolvedEligible} /><Metric label={t('coverage.ambiguous')} value={score.vulnerabilityCoverage.ambiguousEligible} /><Metric label={t('coverage.unresolved')} value={score.vulnerabilityCoverage.unresolvedEligible} /><Metric label={t('coverage.notMappable')} value={score.vulnerabilityCoverage.notMappable} /><Metric label={t('coverage.pending')} value={score.vulnerabilityCoverage.pendingEvaluation} /></dl><p className="drawer-copy">{t('coverage.denominatorNote')}</p></section>
        <section className="drawer-section"><h3><Layers3 size={15} />{t('drawer.collectorCoverage')}</h3><div className="coverage-list">{score.coverage.map((item) => <div key={item.component}><span><strong>{t(`coverageComponent.${item.component}`, { defaultValue: item.component })}</strong><small>{t(`coverageDetail.${item.status}`, { defaultValue: item.detail })}</small></span><Badge tone={item.status === 'healthy' || item.status === 'available' ? 'good' : item.status === 'degraded' ? 'warning' : 'neutral'}>{t(`coverageStatus.${item.status}`, { defaultValue: item.status })}</Badge></div>)}</div></section>
        <section className="drawer-section"><h3><Clock3 size={15} />{t('drawer.formula')}</h3><dl><div><dt>{t('drawer.formulaVersion')}</dt><dd>v{formatNumber(score.formulaVersion)}</dd></div><div><dt>{t('drawer.calculated')}</dt><dd>{score.generatedAt ? formatDateTime(score.generatedAt) : t('drawer.notCalculated')}</dd></div></dl><p className="drawer-copy">{t('drawer.disclaimer')}</p></section>
      </>}
    </>}
  </div></Drawer>
}

export function ProductRiskDetail({ product, onBack, onOpenProduct }: { product: ProductVulnerabilityRisk; onBack: () => void; onOpenProduct: (softwareId: string) => void }) {
  const { t } = useTranslation('score')
  return <>
    <button type="button" className="button button--ghost score-product-back" onClick={onBack}><ArrowLeft size={14} />{t('productDetail.back')}</button>
    <section className="drawer-section product-risk-detail__hero"><h3><PackageSearch size={15} />{t('productDetail.product')}</h3><strong>{product.displayNames[0] ?? product.canonicalProduct}</strong><span>{product.canonicalVendor} / {product.canonicalProduct}</span><div><Badge tone="accent">{t('productDetail.impact', { value: formatNumber(-product.impact) })}</Badge>{product.confirmedKevCount > 0 && <Badge tone="warning">{t('productDetail.knownExploited')}</Badge>}</div></section>
    <section className="drawer-section"><h3><Calculator size={15} />{t('productDetail.summary')}</h3><dl><TextMetric label={t('productDetail.versions')} value={product.installedVersions.join(', ') || t('drawer.none')} /><TextMetric label={t('productDetail.confirmed')} value={formatNumber(product.confirmedCount)} /><TextMetric label={t('productDetail.possible')} value={formatNumber(product.possibleCount)} /><TextMetric label={t('productDetail.highestCvss')} value={product.highestCvss == null ? t('drawer.none') : formatNumber(product.highestCvss)} /><TextMetric label={t('productDetail.kev')} value={formatNumber(product.confirmedKevCount)} /><TextMetric label={t('productDetail.scoreImpact')} value={formatBreakdownValue(-product.impact, formatNumber)} /></dl></section>
    <section className="drawer-section"><h3><ShieldAlert size={15} />{t('productDetail.confirmedCves')}</h3><div className="product-cve-list">{product.confirmedCveIds.map((cveId) => <code key={cveId}>{cveId}</code>)}</div></section>
    {product.possibleCveIds.length > 0 && <section className="drawer-section product-possible-section"><h3><ShieldAlert size={15} />{t('productDetail.possibleCves')}</h3><div className="product-possible-note"><Badge>{t('productDetail.possibleMatch')}</Badge><strong>{t('productDetail.possibleImpactZero')}</strong><p>{t('vulnerability.possibleNoImpact')}</p></div><div className="product-cve-list">{product.possibleCveIds.map((cveId) => <code key={cveId}>{cveId}</code>)}</div></section>}
    {product.confirmedKevCount > 0 && <section className="drawer-section product-kev-note"><h3><ShieldAlert size={15} />{t('productDetail.knownExploited')}</h3><p>{t('vulnerability.kevAttention')}</p></section>}
    <section className="drawer-section"><h3><Clock3 size={15} />{t('productDetail.evidence')}</h3><dl><TextMetric label={t('productDetail.matchingEngine')} value={product.matchingEngineVersions.map((version) => `v${version}`).join(', ')} /><TextMetric label={t('productDetail.identityResolver')} value={product.identityResolverVersions.map((version) => `v${version}`).join(', ')} /><TextMetric label={t('productDetail.evaluated')} value={formatDateTime(product.evaluatedAt)} /></dl>{product.softwareIds[0] && <button type="button" className="button button--primary product-risk-open-inventory" onClick={() => onOpenProduct(product.softwareIds[0])}>{t('productDetail.openInventory')}<ExternalLink size={14} /></button>}</section>
  </>
}

function Metric({ label, value }: { label: string; value: number }) { return <div><dt>{label}</dt><dd>{formatNumber(value)}</dd></div> }
function TextMetric({ label, value }: { label: string; value: string }) { return <div><dt>{label}</dt><dd>{value}</dd></div> }
function vulnerabilityCoverageTone(status: VulnerabilityCoverageStatus): 'neutral' | 'good' | 'warning' { return status === 'complete' || status === 'not_applicable' ? 'good' : status === 'limited' || status === 'updating' ? 'warning' : 'neutral' }
