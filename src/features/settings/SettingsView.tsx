import { Check, Database, Download, Languages, RefreshCw, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { changeLanguage, getActiveLanguage, type SupportedLanguage } from '../../i18n'
import { Badge } from '../../components/ui/primitives'
import { cancelVulnerabilitySync, getVulnerabilityProviderStatus, syncVulnerabilityProvider } from '../../lib/tauri'
import type { VulnerabilityProviderName, VulnerabilityProviderStatus } from '../../types/inventory'

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

  return <div className="settings-stack">
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
    <ProviderSettings />
  </div>
}

function ProviderSettings() {
  const { t, i18n } = useTranslation(['vulnerabilities', 'common'])
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const dateTime = new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' })
  const [providers, setProviders] = useState<VulnerabilityProviderStatus[]>([])
  const [loading, setLoading] = useState(true)
  const [loadError, setLoadError] = useState(false)
  const [syncError, setSyncError] = useState(false)
  const [busyProvider, setBusyProvider] = useState<VulnerabilityProviderName>()

  const load = async () => {
    try { setProviders(await getVulnerabilityProviderStatus()); setLoadError(false) }
    catch { setLoadError(true) }
    finally { setLoading(false) }
  }
  useEffect(() => { void load() }, [])
  const syncActive = Boolean(busyProvider) || providers.some((provider) => provider.status === 'updating')
  useEffect(() => {
    if (!syncActive) return
    const interval = window.setInterval(() => {
      void getVulnerabilityProviderStatus().then((statuses) => {
        setProviders(statuses)
        setLoadError(false)
      }).catch(() => setLoadError(true))
    }, 2_000)
    return () => window.clearInterval(interval)
  }, [syncActive])

  const synchronize = async (provider: VulnerabilityProviderName) => {
    setBusyProvider(provider)
    setSyncError(false)
    setProviders((current) => current.map((item) => item.provider === provider ? { ...item, status: 'updating' } : item))
    try {
      const status = await syncVulnerabilityProvider(provider)
      setProviders((current) => current.map((item) => item.provider === provider ? status : item))
    } catch {
      setSyncError(true)
      await load()
    } finally { setBusyProvider(undefined) }
  }
  const cancel = async () => { await cancelVulnerabilitySync() }

  return <section className="settings-view panel provider-settings" aria-labelledby="provider-settings-title" aria-busy={loading || Boolean(busyProvider)}>
    <header className="settings-view__header">
      <span><Database size={19} /></span>
      <div><h3 id="provider-settings-title">{t('vulnerabilities:title')}</h3><p>{t('vulnerabilities:description')}</p></div>
    </header>
    <div className="provider-list">
      {loading && <div className="provider-message"><RefreshCw className="spin" size={15} />{t('vulnerabilities:messages.loading')}</div>}
      {!loading && providers.map((provider) => <article className="provider-card" key={provider.provider}>
        <div className="provider-card__identity"><strong>{t(`vulnerabilities:providers.${provider.provider}`)}</strong><span>{t(`vulnerabilities:providerDescriptions.${provider.provider}`)}</span></div>
        <Badge tone={provider.status === 'ready' ? 'good' : provider.status === 'error' ? 'danger' : provider.status === 'updating' ? 'accent' : 'neutral'}>{t(`vulnerabilities:status.${provider.status}`)}</Badge>
        <dl>
          <div><dt>{t('vulnerabilities:fields.lastSync')}</dt><dd>{provider.lastSuccessfulSyncAt ? dateTime.format(new Date(provider.lastSuccessfulSyncAt)) : t('vulnerabilities:fields.never')}</dd></div>
          <div><dt>{t('vulnerabilities:fields.records')}</dt><dd>{number.format(provider.recordCount)}</dd></div>
          <div><dt>{t('vulnerabilities:fields.pages')}</dt><dd>{number.format(provider.pagesProcessed)}</dd></div>
          <div><dt>{t('vulnerabilities:fields.lastPage')}</dt><dd>{provider.lastSuccessfulPage ? number.format(provider.lastSuccessfulPage) : t('vulnerabilities:fields.never')}</dd></div>
          <div><dt>{t('vulnerabilities:fields.elapsed')}</dt><dd>{provider.syncElapsedMs === undefined ? t('vulnerabilities:fields.never') : t('vulnerabilities:fields.elapsedValue', { value: number.format(Math.round(provider.syncElapsedMs / 1_000)) })}</dd></div>
        </dl>
        <div className="provider-card__actions">
          <button type="button" className="button" disabled={Boolean(busyProvider)} onClick={() => void synchronize(provider.provider)}><Download size={14} />{t('vulnerabilities:actions.synchronize')}</button>
          {busyProvider === provider.provider && <button type="button" className="button" onClick={() => void cancel()}><X size={14} />{t('vulnerabilities:actions.cancel')}</button>}
        </div>
      </article>)}
      {loadError && <div className="provider-message" role="alert">{t('vulnerabilities:messages.loadError')}<button type="button" className="button" onClick={() => void load()}>{t('vulnerabilities:actions.refreshStatus')}</button></div>}
      {syncError && <div className="provider-message provider-message--error" role="alert">{t('vulnerabilities:messages.syncError')}</div>}
    </div>
    <footer className="settings-view__status"><span>{t('vulnerabilities:messages.offline')}</span><span>{t('vulnerabilities:messages.noApiKey')}</span></footer>
  </section>
}
