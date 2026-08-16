import { useEffect, useState } from 'react'
import { Activity, Bell, Boxes, ChevronLeft, CircleHelp, Command, FileText, Gauge, LayoutDashboard, Menu, Network, Palette, Pause, Play, RefreshCw, Search, Settings, Shield, ShieldAlert, SlidersHorizontal, Wifi } from 'lucide-react'
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
import { loadTheme, persistTheme } from './lib/tauri'
import type { ThemeName } from './types/system'

type Page = 'overview' | 'services' | 'network' | 'processes' | 'events'

const nav: Array<{ label: string; icon: typeof LayoutDashboard; page?: Page }> = [
  { label: 'Overview', icon: LayoutDashboard, page: 'overview' },
  { label: 'System', icon: Gauge, page: 'services' },
  { label: 'Network', icon: Network, page: 'network' },
  { label: 'Activity', icon: Activity, page: 'processes' },
  { label: 'Inventory', icon: Boxes },
  { label: 'Events', icon: FileText, page: 'events' },
]

const headings: Record<Page, { eyebrow: string; title: string; description: string; topbar: string }> = {
  overview: { eyebrow: 'Endpoint intelligence', title: 'Security overview', description: 'Review the current state of this Windows device.', topbar: 'Security overview' },
  processes: { eyebrow: 'Live activity', title: 'Processes', description: 'Observe real Windows processes and their runtime footprint.', topbar: 'Process activity' },
  network: { eyebrow: 'Network telemetry', title: 'Active connections', description: 'Inspect current TCP and UDP endpoints correlated with processes.', topbar: 'Active connections' },
  services: { eyebrow: 'System telemetry', title: 'Windows services', description: 'Review service state and startup configuration without changing the system.', topbar: 'Windows services' },
  events: { eyebrow: 'Evidence-based analysis', title: 'Security analysis', description: 'Review explainable detections separately from factual security events.', topbar: 'Security analysis' },
}

const securityHeadings: Record<SecuritySection, { eyebrow: string; title: string; description: string; topbar: string }> = {
  detections: { eyebrow: 'Evidence-based analysis', title: 'Detections', description: 'Review rule conclusions supported by correlated evidence.', topbar: 'Detections' },
  events: { eyebrow: 'Behavioral evidence', title: 'Security events', description: 'Review factual observations. No threat classification is assigned.', topbar: 'Security events' },
  rules: { eyebrow: 'Detection policy', title: 'Detection rules', description: 'Review versioned local rules and their operational state.', topbar: 'Detection rules' },
}

function App() {
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
  const { live, setLive, loading, refreshing, overview, database, snapshot, error, health, refresh, baseline, securityScore, securityError, securityRevision, securityNotice, refreshSecurity, startBaseline, resetBaseline, completeBaseline, setEventStatus, setDetectionStatus, clearSecurityNotice } = useTelemetry()
  const heading = page === 'events' ? securityHeadings[securitySection] : headings[page]
  const collectorsFailed = health.some(({ state }) => state === 'failed')
  const collectorsDegraded = health.some(({ state }) => state === 'degraded')
  const collectorStatus = !live ? { title: 'Live updates paused', detail: 'Latest snapshot retained' } : collectorsFailed ? { title: 'Collector failed', detail: 'Review the factual error details' } : collectorsDegraded ? { title: 'Collector degraded', detail: 'Available data remains visible' } : { title: 'Collectors healthy', detail: 'Coverage shown separately' }

  useEffect(() => { document.documentElement.dataset.theme = theme }, [theme])
  useEffect(() => { loadTheme().then(setTheme).catch(() => undefined) }, [])
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLocaleLowerCase() === 'k') { event.preventDefault(); setPaletteOpen((open) => !open) }
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
    setToast(securityNotice.message)
    clearSecurityNotice()
  }, [clearSecurityNotice, securityNotice])

  const changeTheme = async (nextTheme: ThemeName) => {
    const previous = theme
    setTheme(nextTheme)
    setThemeOpen(false)
    try { await persistTheme(nextTheme); setToast('Theme preference saved locally') }
    catch { setTheme(previous); setToast('Theme could not be saved') }
  }

  const openPage = (nextPage: Page) => { setPage(nextPage); setPaletteOpen(false); setMobileOpen(false) }
  const openSecurity = (section: SecuritySection) => { setSecuritySection(section); openPage('events') }
  const setLiveWithToast = (nextLive: boolean) => { setLive(nextLive); setToast(nextLive ? 'Live telemetry resumed' : 'Live telemetry paused at the latest snapshot') }
  const runRefresh = () => { void refresh(); setToast('Collecting current Windows telemetry') }
  const runSecurityRefresh = () => { void refreshSecurity(); setToast('Refreshing local security analysis') }
  const runBaselineAction = async (input: { confirmation: string; learningPeriodSeconds?: number }) => {
    if (!baselineAction) return
    setBaselineBusy(true)
    try {
      if (baselineAction === 'reset') await resetBaseline(input)
      else if (baselineAction === 'complete') await completeBaseline(input)
      else await startBaseline(input)
      setToast(baselineAction === 'complete' ? 'Behavioral baseline is Ready' : 'New behavioral baseline is Learning')
    } finally { setBaselineBusy(false) }
  }

  const commands = [
    { label: 'Open processes', detail: 'Observe active Windows processes', icon: Activity, run: () => openPage('processes') },
    { label: 'Open network connections', detail: 'Inspect TCP and UDP endpoints', icon: Network, run: () => openPage('network') },
    { label: 'Open services', detail: 'Review Windows service state', icon: Gauge, run: () => openPage('services') },
    { label: 'Open detections', detail: 'Review evidence-backed rule conclusions', icon: ShieldAlert, run: () => openSecurity('detections') },
    { label: 'Open security events', detail: 'Review factual baseline differences', icon: FileText, run: () => openSecurity('events') },
    { label: 'View Security Score', detail: 'Open the score and its explainable breakdown', icon: Gauge, run: () => { openPage('overview'); setScoreOpen(true) } },
    { label: 'View detection rules', detail: 'Review versioned local rule configuration', icon: SlidersHorizontal, run: () => openSecurity('rules') },
    { label: 'Refresh security analysis', detail: 'Evaluate current local security state', icon: RefreshCw, run: () => { setPaletteOpen(false); runSecurityRefresh() } },
    { label: 'View baseline', detail: 'Open behavioral baseline details', icon: Shield, run: () => openPage('overview') },
    { label: 'Start new baseline', detail: 'Preserve history and begin a new learning version', icon: Play, run: () => { setPaletteOpen(false); setBaselineAction('start') } },
    ...(baseline && baseline.status !== 'not_initialized' ? [{ label: 'Reset baseline', detail: 'Requires strong confirmation and preserves history', icon: RefreshCw, run: () => { setPaletteOpen(false); setBaselineAction('reset') } }] : []),
    { label: 'Refresh telemetry', detail: 'Collect a new real snapshot', icon: RefreshCw, run: () => { setPaletteOpen(false); runRefresh() } },
    ...(live ? [{ label: 'Pause live telemetry', detail: 'Keep the current snapshot navigable', icon: Pause, run: () => { setPaletteOpen(false); setLiveWithToast(false) } }] : [{ label: 'Resume live telemetry', detail: 'Continue automatic local collection', icon: Play, run: () => { setPaletteOpen(false); setLiveWithToast(true) } }]),
    { label: 'Change interface theme', detail: 'Choose from four persisted themes', icon: Palette, run: () => { setPaletteOpen(false); setThemeOpen(true) } },
    { label: 'Toggle sidebar', detail: 'Switch compact navigation mode', icon: Command, run: () => { setCollapsed((value) => !value); setPaletteOpen(false) } },
  ]
  const filteredCommands = commands.filter(({ label, detail }) => `${label} ${detail}`.toLocaleLowerCase().includes(paletteQuery.trim().toLocaleLowerCase()))

  return (
    <div className="app-shell" data-sidebar={collapsed ? 'compact' : 'expanded'} data-mobile-open={mobileOpen || undefined}>
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark"><Shield size={21} strokeWidth={1.7} /></span><span className="brand-copy"><strong>EDY</strong><small>SENTINEL</small></span></div>
        <nav aria-label="Primary navigation"><span className="nav-label">Workspace</span>{nav.map((item) => <Tooltip key={item.label} label={item.label}><button type="button" aria-label={item.label} className="nav-item" data-active={item.page === page || undefined} disabled={!item.page} title={!item.page ? 'Planned for a future sprint' : undefined} onClick={() => item.page && openPage(item.page)}><item.icon size={18} /><span>{item.label}</span>{item.page === page && <i />}</button></Tooltip>)}</nav>
        <div className="sidebar-spacer" />
        <div className="sidebar-status"><span className="pulse" data-paused={!live || undefined} /><div><strong>{live ? 'Live telemetry' : 'Telemetry paused'}</strong><small>{snapshot ? `${snapshot.processes.length} processes observed` : 'Local collection starting'}</small></div></div>
        <div className="sidebar-bottom"><button type="button" aria-label="Help center" className="nav-item" disabled><CircleHelp size={18} /><span>Help center</span></button><button type="button" aria-label="Settings" className="nav-item" disabled><Settings size={18} /><span>Settings</span></button></div>
        <button type="button" className="collapse-button" onClick={() => setCollapsed((value) => !value)} aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}><ChevronLeft size={16} /></button>
      </aside>
      {mobileOpen && <button type="button" className="mobile-sidebar-backdrop" aria-label="Close navigation" onClick={() => setMobileOpen(false)} />}

      <div className="app-main">
        <header className="topbar">
          <div className="topbar-title"><IconButton className="mobile-menu" aria-label="Toggle sidebar" aria-expanded={mobileOpen} onClick={() => setMobileOpen((value) => !value)}><Menu size={19} /></IconButton><div><span>Workspace</span><strong>{heading.topbar}</strong></div></div>
          <button type="button" className="search-trigger" onClick={() => setPaletteOpen(true)}><Search size={16} /><span>Search or run a command</span><kbd>Ctrl K</kbd></button>
          <div className="topbar-actions">
            <Tooltip label="Refresh telemetry"><IconButton aria-label="Refresh telemetry" onClick={runRefresh} disabled={refreshing}><RefreshCw size={17} className={refreshing ? 'spin' : ''} /></IconButton></Tooltip>
            <div className="theme-anchor"><Tooltip label="Change theme"><IconButton aria-label="Change theme" aria-haspopup="menu" aria-expanded={themeOpen} onClick={() => setThemeOpen((open) => !open)}><Palette size={17} /></IconButton></Tooltip>{themeOpen && <ThemeMenu theme={theme} onChange={changeTheme} onClose={() => setThemeOpen(false)} />}</div>
            <Tooltip label="No new notifications"><IconButton aria-label="No new notifications" disabled><Bell size={17} /></IconButton></Tooltip><span className="topbar-divider" />
            <div className="connection-state" data-paused={!live || collectorsFailed || collectorsDegraded || undefined}><Wifi size={15} /><span><strong>{collectorStatus.title}</strong><small>{collectorStatus.detail}</small></span></div>
          </div>
        </header>

        <main className="content">
          <div className="page-heading"><div><p>{heading.eyebrow}</p><h2>{page === 'overview' ? `Welcome, ${overview?.host.username ?? 'operator'}` : heading.title}</h2><span>{heading.description}</span></div><button type="button" className="button button--primary" onClick={runRefresh} disabled={refreshing}><RefreshCw size={16} className={refreshing ? 'spin' : ''} /> Refresh telemetry</button></div>
          <div className="operational-layout">
            <CollectorStrip health={health} live={live} refreshing={refreshing} onLiveChange={setLiveWithToast} onRefresh={runRefresh} />
            {page === 'overview' && <Overview data={overview} database={database} loading={loading} error={error} onRefresh={refresh} baseline={baseline} onBaselineAction={setBaselineAction} onOpenEvents={() => openSecurity('events')} securityScore={securityScore} onOpenScore={() => setScoreOpen(true)} />}
            {page !== 'overview' && page !== 'events' && loading && !snapshot && <OperationalLoading />}
            {page !== 'overview' && page !== 'events' && !loading && !snapshot && <div className="error-state"><span><Activity size={20} /></span><div><strong>Operational telemetry unavailable</strong><p>{error ?? 'The live collector did not return a snapshot.'}</p></div><button type="button" className="button" onClick={runRefresh}>Try again</button></div>}
            {page === 'processes' && snapshot && <ProcessesView processes={snapshot.processes} connections={snapshot.connections} currentUser={overview?.host.username} />}
            {page === 'network' && snapshot && <ConnectionsView connections={snapshot.connections} />}
            {page === 'services' && snapshot && <ServicesView services={snapshot.services} />}
            {page === 'events' && <SecurityWorkspace section={securitySection} revision={securityRevision} error={securityError} onSectionChange={setSecuritySection} onRefresh={refreshSecurity} onEventStatusChange={setEventStatus} onDetectionStatusChange={setDetectionStatus} />}
          </div>
        </main>
      </div>

      <Dialog open={paletteOpen} title="Command palette" onClose={() => { setPaletteOpen(false); setPaletteQuery('') }}>
        <div className="command-search"><Search size={17} /><input autoFocus aria-label="Command search" placeholder="Type a command…" value={paletteQuery} onChange={(event) => setPaletteQuery(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter' && filteredCommands[0]) filteredCommands[0].run() }} /></div>
        <div className="command-list"><span>Available now</span>{filteredCommands.map((command) => <button type="button" key={command.label} onClick={command.run}><command.icon size={16} /><div><strong>{command.label}</strong><small>{command.detail}</small></div></button>)}{!filteredCommands.length && <div className="command-empty">No available command matches this search.</div>}</div>
      </Dialog>
      <BaselineActionDialog mode={baselineAction} busy={baselineBusy} onClose={() => setBaselineAction(null)} onConfirm={runBaselineAction} />
      <ScoreBreakdownDrawer score={securityScore} open={scoreOpen} onClose={() => setScoreOpen(false)} />
      {toast && <div className="toast" role="status"><span className="toast-mark" />{toast}</div>}
    </div>
  )
}

function OperationalLoading() {
  return <div className="operational-loading" aria-label="Loading operational telemetry"><div className="operational-loading__toolbar"><Skeleton className="skeleton--search" /><Skeleton className="skeleton--filters" /></div>{Array.from({ length: 8 }, (_, index) => <Skeleton className="skeleton--table-row" key={index} />)}</div>
}

export default App
