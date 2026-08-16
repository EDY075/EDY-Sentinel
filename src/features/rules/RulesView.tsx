import { ArrowLeft, DatabaseZap, ShieldCheck } from 'lucide-react'
import { Badge } from '../../components/ui/primitives'
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
  return <section className="rules-view panel" aria-labelledby="rules-title" aria-busy={loading}>
    <header className="rules-view__header"><button type="button" className="button" onClick={onBack}><ArrowLeft size={14} /> Back to detections</button><div><ShieldCheck size={18} /><span><strong id="rules-title">Detection rules</strong><small>Versioned local rules · conditions are read-only</small></span></div><Badge>{rules.length} rules</Badge></header>
    {error && <div className="inline-error" role="alert">Rules could not be loaded: {error} <button type="button" className="button" onClick={onRetry}>Retry</button></div>}
    <div className="rules-table-wrap">
      <table className="rules-table">
        <thead><tr><th>Rule</th><th>Category</th><th>Version</th><th>Default severity</th><th>Evidence</th><th>Enabled</th></tr></thead>
        <tbody>{rules.map((rule) => <tr key={`${rule.ruleId}-${rule.version}`}><td data-label="Rule"><strong>{rule.name}</strong><small>{rule.ruleId}</small><p>{rule.description}</p></td><td data-label="Category">{rule.category}</td><td data-label="Version"><code>v{rule.version}</code></td><td data-label="Default severity"><Badge tone={severityTone(rule.defaultSeverity)}>{severityLabel(rule.defaultSeverity)}</Badge></td><td data-label="Evidence"><span className="rule-evidence-count"><DatabaseZap size={13} />{rule.requiredEvidence.length} required</span></td><td data-label="Enabled"><button type="button" className="rule-switch" role="switch" aria-checked={rule.enabled} aria-label={`${rule.enabled ? 'Disable' : 'Enable'} ${rule.name}`} disabled={busyRuleId === rule.ruleId} onClick={() => void onToggle(rule)}><span /><small>{rule.enabled ? 'Enabled' : 'Disabled'}</small></button></td></tr>)}</tbody>
      </table>
    </div>
    {!loading && !rules.length && !error && <div className="table-empty"><strong>No registered detection rules</strong><span>The Rust registry did not return any operational rule definitions.</span></div>}
  </section>
}
