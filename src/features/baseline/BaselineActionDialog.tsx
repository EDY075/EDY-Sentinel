import { useEffect, useState } from 'react'
import { AlertTriangle } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { Dialog } from '../../components/ui/primitives'
import type { BaselineAction } from '../../types/baseline'
import { isBaselineConfirmationValid, requiredConfirmation } from './baseline'
import type { BaselineActionMode } from './baseline'

export function BaselineActionDialog({ mode, busy, onClose, onConfirm }: { mode: BaselineActionMode | null; busy: boolean; onClose: () => void; onConfirm: (input: BaselineAction) => Promise<void> }) {
  const { t } = useTranslation('baseline')
  const [confirmation, setConfirmation] = useState('')
  const [period, setPeriod] = useState(86_400)
  const [error, setError] = useState<string>()
  useEffect(() => { if (mode) { setConfirmation(''); setError(undefined); setPeriod(86_400) } }, [mode])
  if (!mode) return null
  const phrase = requiredConfirmation(mode)
  const valid = isBaselineConfirmationValid(mode, confirmation)
  const title = t(`dialog.title.${mode}`)
  const periods = [
    { value: 60, label: t('dialog.periods.minute') },
    { value: 3_600, label: t('dialog.periods.hour') },
    { value: 21_600, label: t('dialog.periods.hours6') },
    { value: 86_400, label: t('dialog.periods.hours24') },
    { value: 259_200, label: t('dialog.periods.days3') },
    { value: 604_800, label: t('dialog.periods.days7') },
  ]
  const submit = async () => {
    try { setError(undefined); await onConfirm({ confirmation, learningPeriodSeconds: mode === 'complete' ? undefined : period }); onClose() }
    catch { setError(t('dialog.actionError')) }
  }
  return <Dialog open title={title} onClose={onClose}>
    <div className="baseline-dialog">
      <div className="baseline-dialog__warning"><AlertTriangle size={18} /><p>{t(`dialog.warning.${mode}`)}</p></div>
      {mode !== 'complete' && <label>{t('dialog.learningPeriod')}<select value={period} onChange={(event) => setPeriod(Number(event.target.value))}>{periods.map((option) => <option value={option.value} key={option.value}>{option.label}</option>)}</select></label>}
      <label><Trans ns="baseline" i18nKey="dialog.confirmation" values={{ phrase }} components={{ code: <code /> }} /><input autoFocus data-initial-focus value={confirmation} onChange={(event) => setConfirmation(event.target.value)} aria-label={t('dialog.confirmationAria')} /></label>
      {error && <p className="form-error" role="alert">{error}</p>}
      <div className="dialog-actions"><button type="button" className="button" onClick={onClose}>{t('dialog.cancel')}</button><button type="button" className={mode === 'reset' ? 'button button--danger-subtle' : 'button button--primary'} disabled={!valid || busy} onClick={() => void submit()}>{busy ? t('dialog.working') : title}</button></div>
    </div>
  </Dialog>
}
