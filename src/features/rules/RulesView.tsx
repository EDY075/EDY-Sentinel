import { ArrowLeft, DatabaseZap, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '../../components/ui/primitives'
import { formatNumber } from '../../i18n'
import type { RuleDefinition } from '../../types/rules'
import { severityLabel, severityTone } from '../detections/detectionPresentation'

export function RulesView({ rules, loading, error, busyRuleId, onBack, onToggle, onRetry }: {
  rules: RuleDefinition[]
  loading: boolean
  error?: string
  busyRuleId?: string
  onBack: () => void
  onToggle: (rule: RuleDefinition) => Promise<void>
  onRetry: () => void
}) {
  const { t } = useTranslation('rules')
  const { t: tDetections } = useTranslation('detections')
  return <section className="rules-view panel" aria-labelledby="rules-title" aria-busy={loading}>
    <header className="rules-view__header"><button type="button" className="button" onClick={onBack}><ArrowLeft size={14} /> {t('header.back')}</button><div><ShieldCheck size={18} /><span><strong id="rules-title">{t('header.title')}</strong><small>{t('header.subtitle')}</small></span></div><Badge>{t('header.count', { count: rules.length, formattedCount: formatNumber(rules.length) })}</Badge></header>
    {error && <div className="inline-error" role="alert">{t('error.load')} <button type="button" className="button" onClick={onRetry}>{t('error.retry')}</button></div>}
    <div className="rules-table-wrap">
      <table className="rules-table">
        <thead><tr><th>{t('table.rule')}</th><th>{t('table.category')}</th><th>{t('table.version')}</th><th>{t('table.defaultSeverity')}</th><th>{t('table.evidence')}</th><th>{t('table.enabled')}</th></tr></thead>
        <tbody>{rules.map((rule) => { const name = t(`definitions.${rule.ruleId}.name`, { defaultValue: rule.name }); return <tr key={`${rule.ruleId}-${rule.version}`}><td data-label={t('table.rule')}><strong>{name}</strong><small>{rule.ruleId}</small><p>{t(`definitions.${rule.ruleId}.description`, { defaultValue: rule.description })}</p></td><td data-label={t('table.category')}>{t(`categories.${rule.category}`, { defaultValue: rule.category })}</td><td data-label={t('table.version')}><code>v{formatNumber(rule.version)}</code></td><td data-label={t('table.defaultSeverity')}><Badge tone={severityTone(rule.defaultSeverity)}>{severityLabel(rule.defaultSeverity, tDetections)}</Badge></td><td data-label={t('table.evidence')}><span className="rule-evidence-count"><DatabaseZap size={13} />{t('table.required', { count: rule.requiredEvidence.length, formattedCount: formatNumber(rule.requiredEvidence.length) })}</span></td><td data-label={t('table.enabled')}><button type="button" className="rule-switch" role="switch" aria-checked={rule.enabled} aria-label={t(rule.enabled ? 'state.disableAria' : 'state.enableAria', { name })} disabled={busyRuleId === rule.ruleId} onClick={() => void onToggle(rule)}><span /><small>{t(rule.enabled ? 'state.enabled' : 'state.disabled')}</small></button></td></tr> })}</tbody>
      </table>
    </div>
    {!loading && !rules.length && !error && <div className="table-empty"><strong>{t('empty.title')}</strong><span>{t('empty.description')}</span></div>}
  </section>
}
