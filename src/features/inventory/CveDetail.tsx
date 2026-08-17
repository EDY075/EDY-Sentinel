import { ArrowLeft, ExternalLink, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '../../components/ui/primitives'
import type { VulnerabilityMatch } from '../../types/inventory'

export function CveDetail({ match, softwareName, locale, onBack }: { match: VulnerabilityMatch; softwareName: string; locale: string; onBack: () => void }) {
  const { t } = useTranslation('inventory')
  const date = (value: string) => new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value))
  const number = new Intl.NumberFormat(locale, { maximumFractionDigits: 1 })
  const tone = match.matchState === 'confirmed' ? 'danger' : match.matchState === 'possible' ? 'warning' : 'neutral'
  return <div className="drawer-content cve-detail">
    <button type="button" className="button button--quiet cve-detail__back" onClick={onBack}><ArrowLeft size={14} />{t('cve.back')}</button>
    <header><span><ShieldAlert size={18} /></span><div><h3>{match.cveId}</h3><p>{match.description}</p></div><Badge tone={tone}>{t(`matchStates.${match.matchState}`)}</Badge></header>
    <section className="drawer-section"><h3>{t('cve.matching')}</h3><dl>
      <Detail label={t('cve.software')} value={softwareName} /><Detail label={t('cve.installedVersion')} value={match.installedVersion ?? t('unavailable')} mono /><Detail label="CPE" value={match.cpe} mono /><Detail label={t('cve.affectedRange')} value={match.affectedRange} mono /><Detail label={t('cve.result')} value={t(`comparison.${match.comparisonResult}`)} /><Detail label={t('cve.confidence')} value={t(`confidence.${match.confidence}`)} /><Detail label={t('cve.lastEvaluated')} value={date(match.lastEvaluatedAt)} /><Detail label={t('cve.engine')} value={`v${match.matchingEngineVersion}`} mono />
    </dl></section>
    <section className="drawer-section"><h3>{t('cve.nvd')}</h3><dl>
      <Detail label="CVSS" value={match.cvssScore === undefined ? t('unavailable') : `${number.format(match.cvssScore)}${match.cvssVersion ? ` · v${match.cvssVersion}` : ''}`} /><Detail label={t('cve.severity')} value={match.severity ?? t('unavailable')} /><Detail label={t('cve.published')} value={date(match.publishedAt)} /><Detail label={t('cve.modified')} value={date(match.lastModifiedAt)} /><Detail label={t('cve.sourceVersion')} value={match.nvdSourceVersion ?? t('unavailable')} mono />
    </dl></section>
    <section className="drawer-section"><h3>CISA KEV</h3>{match.kev ? <dl><Detail label={t('cve.kevName')} value={match.kev.vulnerabilityName} /><Detail label={t('cve.dateAdded')} value={match.kev.dateAdded} /><Detail label={t('cve.dueDate')} value={match.kev.dueDate ?? t('unavailable')} /><Detail label={t('cve.requiredAction')} value={match.kev.requiredAction} /></dl> : <p className="cve-detail__empty">{t('cve.notKev')}</p>}</section>
    <section className="drawer-section"><h3>{t('cve.evidence')}</h3>{match.evidence.map((evidence) => <article className="cve-evidence" key={evidence.evidenceId}><span><strong>{evidence.source}</strong><small>{date(evidence.observedAt)}</small></span><pre>{JSON.stringify(evidence.details, null, 2)}</pre></article>)}</section>
    <section className="drawer-section"><h3>{t('cve.references')}</h3><div className="cve-references">{match.references.length ? match.references.map((reference) => <a href={reference} target="_blank" rel="noreferrer" key={reference}>{reference}<ExternalLink size={12} /></a>) : <p>{t('cve.noReferences')}</p>}</div></section>
  </div>
}

function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined}>{value}</dd></div> }
