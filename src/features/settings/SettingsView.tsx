import { Check, Languages } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { changeLanguage, getActiveLanguage, type SupportedLanguage } from '../../i18n'

const options: Array<{ language: SupportedLanguage; key: 'en' | 'ptBR'; localeLabel: string }> = [
  { language: 'pt-BR', key: 'ptBR', localeLabel: 'pt-BR' },
  { language: 'en', key: 'en', localeLabel: 'en' },
]

export function SettingsView() {
  const { t } = useTranslation(['settings', 'common'])
  const current = getActiveLanguage()
  const [busy, setBusy] = useState(false)
  const [result, setResult] = useState<'saved' | 'error' | null>(null)

  const selectLanguage = async (language: SupportedLanguage) => {
    if (language === current || busy) return
    setBusy(true)
    setResult(null)
    try {
      await changeLanguage(language)
      setResult('saved')
    } catch {
      setResult('error')
    } finally {
      setBusy(false)
    }
  }

  return (
    <section className="settings-view panel" aria-labelledby="language-settings-title" aria-busy={busy}>
      <header className="settings-view__header">
        <span><Languages size={19} /></span>
        <div>
          <h3 id="language-settings-title">{t('settings:sections.language')}</h3>
          <p>{t('settings:language.description')}</p>
        </div>
      </header>
      <fieldset className="language-options" disabled={busy}>
        <legend>{t('settings:language.label')}</legend>
        {options.map((option) => {
          const selected = current === option.language
          return (
            <label key={option.language} className="language-option" data-selected={selected || undefined}>
              <input type="radio" name="interface-language" value={option.language} checked={selected} onChange={() => void selectLanguage(option.language)} />
              <span>
                <strong>{t(`settings:language.options.${option.key}`)}</strong>
                <small lang={option.language}>{option.localeLabel}</small>
              </span>
              {selected && <Check size={17} aria-hidden="true" />}
            </label>
          )
        })}
      </fieldset>
      <footer className="settings-view__status">
        <span>{t('settings:language.current')}: <strong>{t(`settings:language.options.${current === 'pt-BR' ? 'ptBR' : 'en'}`)}</strong></span>
        {result && <p role={result === 'error' ? 'alert' : 'status'} data-error={result === 'error' || undefined}>{t(`settings:language.${result === 'error' ? 'saveError' : 'saved'}`)}</p>}
      </footer>
      <span className="sr-only" aria-live="polite">{busy ? t('common:states.loading') : result ? t(`settings:language.${result === 'error' ? 'saveError' : 'saved'}`) : ''}</span>
    </section>
  )
}
