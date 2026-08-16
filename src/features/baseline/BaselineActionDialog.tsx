import { useEffect, useState } from 'react'
import { AlertTriangle } from 'lucide-react'
import { Dialog } from '../../components/ui/primitives'
import type { BaselineAction } from '../../types/baseline'
import { isBaselineConfirmationValid, requiredConfirmation } from './baseline'
import type { BaselineActionMode } from './baseline'

const periods = [
  { value: 60, label: '1 minute · controlled development test' },
  { value: 3_600, label: '1 hour' },
  { value: 21_600, label: '6 hours' },
  { value: 86_400, label: '24 hours · recommended default' },
  { value: 259_200, label: '3 days' },
  { value: 604_800, label: '7 days' },
]

export function BaselineActionDialog({ mode, busy, onClose, onConfirm }: { mode: BaselineActionMode | null; busy: boolean; onClose: () => void; onConfirm: (input: BaselineAction) => Promise<void> }) {
  const [confirmation, setConfirmation] = useState('')
  const [period, setPeriod] = useState(86_400)
  const [error, setError] = useState<string>()
  useEffect(() => { if (mode) { setConfirmation(''); setError(undefined); setPeriod(86_400) } }, [mode])
  if (!mode) return null
  const phrase = requiredConfirmation(mode)
  const valid = isBaselineConfirmationValid(mode, confirmation)
  const title = mode === 'reset' ? 'Reset behavioral baseline' : mode === 'complete' ? 'Complete baseline learning' : 'Start new behavioral baseline'
  const submit = async () => {
    try { setError(undefined); await onConfirm({ confirmation, learningPeriodSeconds: mode === 'complete' ? undefined : period }); onClose() }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)) }
  }
  return <Dialog open title={title} onClose={onClose}>
    <div className="baseline-dialog">
      <div className="baseline-dialog__warning"><AlertTriangle size={18} /><p>{mode === 'reset' ? 'Resetting starts a new learning period. Previous baseline versions and other Sentinel history are preserved.' : mode === 'complete' ? 'Manual completion is intended for controlled development validation. Only real observations already collected are used.' : 'The current baseline is preserved as history. The new version learns from real local telemetry and produces no new-behavior events while Learning.'}</p></div>
      {mode !== 'complete' && <label>Learning period<select value={period} onChange={(event) => setPeriod(Number(event.target.value))}>{periods.map((option) => <option value={option.value} key={option.value}>{option.label}</option>)}</select></label>}
      <label>Type <code>{phrase}</code> to confirm<input autoFocus value={confirmation} onChange={(event) => setConfirmation(event.target.value)} aria-label="Baseline confirmation phrase" /></label>
      {error && <p className="form-error" role="alert">{error}</p>}
      <div className="dialog-actions"><button type="button" className="button" onClick={onClose}>Cancel</button><button type="button" className={mode === 'reset' ? 'button button--danger-subtle' : 'button button--primary'} disabled={!valid || busy} onClick={() => void submit()}>{busy ? 'Working…' : title}</button></div>
    </div>
  </Dialog>
}
