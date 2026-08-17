import { useEffect, useState } from 'react'
import { Activity, Bell, Boxes, ChevronLeft, CircleHelp, Command, FileText, Gauge, LayoutDashboard, Menu, Network, Palette, Pause, Play, RefreshCw, Search, Settings, Shield, ShieldAlert, SlidersHorizontal, Wifi } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import './App.css'
import { Dialog, IconButton, Skeleton, Tooltip } from './components/ui/primitives'
import { ConnectionsView } from './features/connections/ConnectionsView'
import { Overview } from './features/overview/Overview'
import { ProcessesView } from './features/processes/ProcessesView'
import { ServicesView } from './features/services/ServicesView'
import { CollectorStrip } from './features/telemetry/CollectorStrip'
import { useTelemetry } from './features/telemetry/TelemetryProvider'
import { ThemeMenu } from './features/theme/ThemeMenu'
import { BaselineActionDialog } from './features/baseline/BaselineActionDialog'
import type { BaselineActionMode } from './features/baseline/baseline'
import { SecurityWorkspace } from './features/security/SecurityWorkspace'
import type { SecuritySection } from './features/security/SecurityWorkspace'
import { ScoreBreakdownDrawer } from './features/score/ScoreBreakdownDrawer'
import { SettingsView } from './features/settings/SettingsView'
import { InventoryView } from './features/inventory/InventoryView'
import { loadTheme, persistTheme } from './lib/tauri'
import type { ThemeName } from './types/system'

type Page = 'overview' | 'services' | 'network' | 'processes' | 'inventory' | 'events' | 'settings'

function App() {
  const { t, i18n } = useTranslation(['shell', 'navigation', 'common', 'settings', 'errors'])
  const [page, setPage] = useState<Page>('overview')
  const [collapsed, setCollapsed] = useState(false)
  const [mobileOpen, setMobileOpen] = useState(false)
  const [theme, setTheme] = useState<ThemeName>(() => (localStorage.getItem('edy-sentinel-theme') as ThemeName | null) ?? 'sentinel-blue')
  const [themeOpen, setThemeOpen] = useState(false)
  const [paletteOpen, setPaletteOpen] = useState(false)
  const [paletteQuery, setPaletteQuery] = useState('')
  const [toast, setToast] = useState<string | null>(null)
  const [baselineAction, setBaselineAction] = useState<BaselineActionMode | null>(null)
  const [baselineBusy, setBaselineBusy] = useState(false)
  const [securitySection, setSecuritySection] = useState<SecuritySection>('detections')
  const [scoreOpen, setScoreOpen] = useState(false)
  const [inventoryTarget, setInventoryTarget] = useState<string>()
  const { live, setLive, loading, refreshing, overview, database, snapshot, error, health, refresh, baseline, securityScore, securityError, securityRevision, securityNotice, refreshSecurity, startBaseline, resetBaseline, completeBaseline, setEventStatus, setDetectionStatus, clearSecurityNotice } = useTelemetry()
  const nav: Array<{ label: string; icon: typeof LayoutDashboard; page?: Page }> = [
    { label: t('navigation:overview'), icon: LayoutDashboard, page: 'overview' },
    { label: t('navigation:system'), icon: Gauge, page: 'services' },
    { label: t('navigation:network'), icon: Network, page: 'network' },
    { label: t('navigation:activity'), icon: Activity, page: 'processes' },
    { label: t('navigation:inventory'), icon: Boxes, page: 'inventory' },
    { label: t('navigation:events'), icon: FileText, page: 'events' },
  ]
  const translatedHeading = (key: string) => ({
    eyebrow: t(`shell:headings.${key}.eyebrow`),
    title: t(`shell:headings.${key}.title`),
    description: t(`shell:headings.${key}.description`),
    topbar: t(`shell:headings.${key}.topbar`),
  })
  const headings: Record<Page, { eyebrow: string; title: string; description: string; topbar: string }> = {
    overview: translatedHeading('overview'),
    processes: translatedHeading('processes'),
    network: translatedHeading('network'),
    services: translatedHeading('services'),
    inventory: translatedHeading('inventory'),
    events: translatedHeading('events'),
    settings: translatedHeading('settings'),
  }
  const securityHeadings: Record<SecuritySection, { eyebrow: string; title: string; description: string; topbar: string }> = {
    detections: translatedHeading('detections'),
    events: translatedHeading('securityEvents'),
    rules: translatedHeading('rules'),
  }
  const heading = page === 'events' ? securityHeadings[securitySection] : headings[page]
  const collectorsFailed = health.some(({ state }) => state === 'failed')
  const collectorsDegraded = health.some(({ state }) => state === 'degraded')
  const collectorStatusKey = !live ? 'paused' : collectorsFailed ? 'failed' : collectorsDegraded ? 'degraded' : 'healthy'
  const collectorStatus = { title: t(`shell:collectorStatus.${collectorStatusKey}.title`), detail: t(`shell:collectorStatus.${collectorStatusKey}.detail`) }

  useEffect(() => { document.documentElement.dataset.theme = theme }, [theme])
  useEffect(() => { loadTheme().then(setTheme).catch(() => undefined) }, [])
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLocaleLowerCase() === 'k') { event.preventDefault(); setPaletteQuery(''); setPaletteOpen((open) => !open) }
      if (event.key === 'Escape') { setPaletteOpen(false); setThemeOpen(false) }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])
  useEffect(() => {
    if (!toast) return
    const timeout = window.setTimeout(() => setToast(null), 3200)
    return () => window.clearTimeout(timeout)
  }, [toast])
  useEffect(() => {
    if (!securityNotice) return
    setToast(t('shell:toast.newDetections', { count: securityNotice.count }))
    clearSecurityNotice()
  }, [clearSecurityNotice, securityNotice, t])

  const changeTheme = async (nextTheme: ThemeName) => {
    const previous = theme
    setTheme(nextTheme)
    setThemeOpen(false)
    try { await persistTheme(nextTheme); setToast(t('shell:toast.themeSaved')) }
    catch { setTheme(previous); setToast(t('shell:toast.themeSaveError')) }
  }

  const openPage = (nextPage: Page) => { setPage(nextPage); if (nextPage !== 'inventory') setInventoryTarget(undefined); setPaletteOpen(false); setMobileOpen(false) }
  const openScoreProduct = (softwareId: string) => { setInventoryTarget(softwareId); setScoreOpen(false); setPage('inventory'); setPaletteOpen(false); setMobileOpen(false) }
  const openSecurity = (section: SecuritySection) => { setSecuritySection(section); openPage('events') }
  const setLiveWithToast = (nextLive: boolean) => { setLive(nextLive); setToast(t(nextLive ? 'shell:toast.telemetryResumed' : 'shell:toast.telemetryPaused')) }
  const runRefresh = () => { void refresh(); setToast(t('shell:toast.collecting')) }
  const runSecurityRefresh = () => { void refreshSecurity(); setToast(t('shell:toast.refreshingSecurity')) }
  const runBaselineAction = async (input: { confirmation: string; learningPeriodSeconds?: number }) => {
    if (!baselineAction) return
    setBaselineBusy(true)
    try {
      if (baselineAction === 'reset') await resetBaseline(input)
      else if (baselineAction === 'complete') await completeBaseline(input)
      else await startBaseline(input)
      setToast(t(baselineAction === 'complete' ? 'shell:toast.baselineReady' : 'shell:toast.baselineLearning'))
    } finally { setBaselineBusy(false) }
  }

  const commands = [
    { label: t('shell:commands.processes.label'), detail: t('shell:commands.processes.detail'), icon: Activity, run: () => openPage('processes') },
    { label: t('shell:commands.connections.label'), detail: t('shell:commands.connections.detail'), icon: Network, run: () => openPage('network') },
    { label: t('shell:commands.services.label'), detail: t('shell:commands.services.detail'), icon: Gauge, run: () => openPage('services') },
    { label: t('shell:commands.inventory.label'), detail: t('shell:commands.inventory.detail'), icon: Boxes, run: () => openPage('inventory') },
    { label: t('shell:commands.detections.label'), detail: t('shell:commands.detections.detail'), icon: ShieldAlert, run: () => openSecurity('detections') },
    { label: t('shell:commands.events.label'), detail: t('shell:commands.events.detail'), icon: FileText, run: () => openSecurity('events') },
    { label: t('shell:commands.score.label'), detail: t('shell:commands.score.detail'), icon: Gauge, run: () => { openPage('overview'); setScoreOpen(true) } },
    { label: t('shell:commands.rules.label'), detail: t('shell:commands.rules.detail'), icon: SlidersHorizontal, run: () => openSecurity('rules') },
    { label: t('shell:commands.securityRefresh.label'), detail: t('shell:commands.securityRefresh.detail'), icon: RefreshCw, run: () => { setPaletteOpen(false); runSecurityRefresh() } },
    { label: t('shell:commands.baseline.label'), detail: t('shell:commands.baseline.detail'), icon: Shield, run: () => openPage('overview') },
    { label: t('shell:commands.baselineStart.label'), detail: t('shell:commands.baselineStart.detail'), icon: Play, run: () => { setPaletteOpen(false); setBaselineAction('start') } },
    ...(baseline && baseline.status !== 'not_initialized' ? [{ label: t('shell:commands.baselineReset.label'), detail: t('shell:commands.baselineReset.detail'), icon: RefreshCw, run: () => { setPaletteOpen(false); setBaselineAction('reset') } }] : []),
    { label: t('shell:commands.refresh.label'), detail: t('shell:commands.refresh.detail'), icon: RefreshCw, run: () => { setPaletteOpen(false); runRefresh() } },
    ...(live ? [{ label: t('shell:commands.pause.label'), detail: t('shell:commands.pause.detail'), icon: Pause, run: () => { setPaletteOpen(false); setLiveWithToast(false) } }] : [{ label: t('shell:commands.resume.label'), detail: t('shell:commands.resume.detail'), icon: Play, run: () => { setPaletteOpen(false); setLiveWithToast(true) } }]),
    { label: t('shell:commands.theme.label'), detail: t('shell:commands.theme.detail'), icon: Palette, run: () => { setPaletteOpen(false); setThemeOpen(true) } },
    { label: t('shell:commands.settings.label'), detail: t('shell:commands.settings.detail'), icon: Settings, run: () => openPage('settings') },
    { label: t('shell:commands.sidebar.label'), detail: t('shell:commands.sidebar.detail'), icon: Command, run: () => { setCollapsed((value) => !value); setPaletteOpen(false) } },
  ]
  const activeLocale = i18n.resolvedLanguage ?? 'en'
  const filteredCommands = commands.filter(({ label, detail }) => `${label} ${detail}`.toLocaleLowerCase(activeLocale).includes(paletteQuery.trim().toLocaleLowerCase(activeLocale)))

  return (
    <div className="app-shell" data-sidebar={collapsed ? 'compact' : 'expanded'} data-mobile-open={mobileOpen || undefined}>
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark"><Shield size={21} strokeWidth={1.7} /></span><span className="brand-copy"><strong>EDY</strong><small>SENTINEL</small></span></div>
        <nav aria-label={t('navigation:primaryLabel')}><span className="nav-label">{t('navigation:workspace')}</span>{nav.map((item) => <Tooltip key={item.label} label={item.label}><button type="button" aria-label={item.label} className="nav-item" data-active={item.page === page || undefined} disabled={!item.page} title={!item.page ? t('navigation:planned') : undefined} onClick={() => item.page && openPage(item.page)}><item.icon size={18} /><span>{item.label}</span>{item.page === page && <i />}</button></Tooltip>)}</nav>
        <div className="sidebar-spacer" />
        <div className="sidebar-status"><span className="pulse" data-paused={!live || undefined} /><div><strong>{t(live ? 'shell:sidebar.live' : 'shell:sidebar.paused')}</strong><small>{snapshot ? t('shell:sidebar.processesObserved', { count: snapshot.processes.length }) : t('shell:sidebar.starting')}</small></div></div>
        <div className="sidebar-bottom"><button type="button" aria-label={t('navigation:helpCenter')} className="nav-item" disabled><CircleHelp size={18} /><span>{t('navigation:helpCenter')}</span></button><button type="button" aria-label={t('navigation:settings')} className="nav-item" data-active={page === 'settings' || undefined} onClick={() => openPage('settings')}><Settings size={18} /><span>{t('navigation:settings')}</span>{page === 'settings' && <i />}</button></div>
        <button type="button" className="collapse-button" onClick={() => setCollapsed((value) => !value)} aria-label={t(collapsed ? 'navigation:expandSidebar' : 'navigation:collapseSidebar')}><ChevronLeft size={16} /></button>
      </aside>
      {mobileOpen && <button type="button" className="mobile-sidebar-backdrop" aria-label={t('navigation:closeNavigation')} onClick={() => setMobileOpen(false)} />}

      <div className="app-main">
        <header className="topbar">
          <div className="topbar-title"><IconButton className="mobile-menu" aria-label={t('shell:topbar.toggleSidebar')} aria-expanded={mobileOpen} onClick={() => setMobileOpen((value) => !value)}><Menu size={19} /></IconButton><div><span>{t('navigation:workspace')}</span><strong>{heading.topbar}</strong></div></div>
          <button type="button" className="search-trigger" onClick={() => { setPaletteQuery(''); setPaletteOpen(true) }}><Search size={16} /><span>{t('shell:topbar.search')}</span><kbd>Ctrl K</kbd></button>
          <div className="topbar-actions">
            <Tooltip label={t('shell:topbar.refresh')}><IconButton aria-label={t('shell:topbar.refresh')} onClick={runRefresh} disabled={refreshing}><RefreshCw size={17} className={refreshing ? 'spin' : ''} /></IconButton></Tooltip>
            <div className="theme-anchor"><Tooltip label={t('shell:topbar.changeTheme')}><IconButton aria-label={t('shell:topbar.changeTheme')} aria-haspopup="menu" aria-expanded={themeOpen} onClick={() => setThemeOpen((open) => !open)}><Palette size={17} /></IconButton></Tooltip>{themeOpen && <ThemeMenu theme={theme} onChange={changeTheme} onClose={() => setThemeOpen(false)} />}</div>
            <Tooltip label={t('shell:topbar.noNotifications')}><IconButton aria-label={t('shell:topbar.noNotifications')} disabled><Bell size={17} /></IconButton></Tooltip><span className="topbar-divider" />
            <div className="connection-state" data-paused={!live || collectorsFailed || collectorsDegraded || undefined}><Wifi size={15} /><span><strong>{collectorStatus.title}</strong><small>{collectorStatus.detail}</small></span></div>
          </div>
        </header>

        <main className="content">
          <div className="page-heading"><div><p>{heading.eyebrow}</p><h2>{page === 'overview' ? t('shell:welcome', { username: overview?.host.username ?? t('shell:operator') }) : heading.title}</h2><span>{heading.description}</span></div>{page !== 'settings' && page !== 'inventory' && <button type="button" className="button button--primary" onClick={runRefresh} disabled={refreshing}><RefreshCw size={16} className={refreshing ? 'spin' : ''} /> {t('shell:refresh')}</button>}</div>
          <div className="operational-layout">
            {page !== 'settings' && page !== 'inventory' && <CollectorStrip health={health} live={live} refreshing={refreshing} onLiveChange={setLiveWithToast} onRefresh={runRefresh} />}
            {page === 'overview' && <Overview data={overview} database={database} loading={loading} error={error} onRefresh={refresh} baseline={baseline} onBaselineAction={setBaselineAction} onOpenEvents={() => openSecurity('events')} securityScore={securityScore} onOpenScore={() => setScoreOpen(true)} />}
            {page !== 'overview' && page !== 'events' && page !== 'settings' && page !== 'inventory' && loading && !snapshot && <OperationalLoading label={t('shell:loading')} />}
            {page !== 'overview' && page !== 'events' && page !== 'settings' && page !== 'inventory' && !loading && !snapshot && <div className="error-state"><span><Activity size={20} /></span><div><strong>{t('shell:telemetryUnavailable.title')}</strong><p>{t(error ? 'errors:telemetryUnavailable' : 'shell:telemetryUnavailable.description')}</p></div><button type="button" className="button" onClick={runRefresh}>{t('shell:telemetryUnavailable.retry')}</button></div>}
            {page === 'processes' && snapshot && <ProcessesView processes={snapshot.processes} connections={snapshot.connections} currentUser={overview?.host.username} />}
            {page === 'network' && snapshot && <ConnectionsView connections={snapshot.connections} />}
            {page === 'services' && snapshot && <ServicesView services={snapshot.services} />}
            {page === 'inventory' && <InventoryView initialSoftwareId={inventoryTarget} />}
            {page === 'events' && <SecurityWorkspace section={securitySection} revision={securityRevision} error={securityError} onSectionChange={setSecuritySection} onRefresh={refreshSecurity} onEventStatusChange={setEventStatus} onDetectionStatusChange={setDetectionStatus} />}
            {page === 'settings' && <SettingsView />}
          </div>
        </main>
      </div>

      <Dialog open={paletteOpen} title={t('shell:palette.title')} onClose={() => { setPaletteOpen(false); setPaletteQuery('') }}>
        <div className="command-search"><Search size={17} /><input autoFocus data-initial-focus aria-label={t('shell:palette.searchLabel')} placeholder={t('shell:palette.placeholder')} value={paletteQuery} onChange={(event) => setPaletteQuery(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter' && filteredCommands[0]) filteredCommands[0].run() }} /></div>
        <div className="command-list"><span>{t('shell:palette.available')}</span>{filteredCommands.map((command) => <button type="button" key={command.label} onClick={command.run}><command.icon size={16} /><div><strong>{command.label}</strong><small>{command.detail}</small></div></button>)}{!filteredCommands.length && <div className="command-empty">{t('shell:palette.empty')}</div>}</div>
      </Dialog>
      <BaselineActionDialog mode={baselineAction} busy={baselineBusy} onClose={() => setBaselineAction(null)} onConfirm={runBaselineAction} />
      <ScoreBreakdownDrawer score={securityScore} open={scoreOpen} onClose={() => setScoreOpen(false)} onOpenProduct={openScoreProduct} />
      {toast && <div className="toast" role="status"><span className="toast-mark" />{toast}</div>}
    </div>
  )
}

function OperationalLoading({ label }: { label: string }) {
  return <div className="operational-loading" aria-label={label}><div className="operational-loading__toolbar"><Skeleton className="skeleton--search" /><Skeleton className="skeleton--filters" /></div>{Array.from({ length: 8 }, (_, index) => <Skeleton className="skeleton--table-row" key={index} />)}</div>
}

export default App
