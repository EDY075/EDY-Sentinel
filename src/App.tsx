import { useCallback, useEffect, useState } from 'react'
import { Activity, Bell, Boxes, ChevronLeft, CircleHelp, Command, FileText, Gauge, LayoutDashboard, Menu, Network, Palette, RefreshCw, Search, Settings, Shield, Wifi } from 'lucide-react'
import './App.css'
import { Dialog, IconButton, Tooltip } from './components/ui/primitives'
import { Overview } from './features/overview/Overview'
import { ThemeMenu } from './features/theme/ThemeMenu'
import { getDatabaseStatus, getSystemOverview, loadTheme, persistTheme } from './lib/tauri'
import type { DatabaseStatus, SystemOverview, ThemeName } from './types/system'

const nav = [
  { label: 'Overview', icon: LayoutDashboard, active: true },
  { label: 'System', icon: Gauge },
  { label: 'Network', icon: Network },
  { label: 'Activity', icon: Activity },
  { label: 'Inventory', icon: Boxes },
  { label: 'Reports', icon: FileText },
]

function App() {
  const [collapsed, setCollapsed] = useState(false)
  const [theme, setTheme] = useState<ThemeName>(() => (localStorage.getItem('edy-sentinel-theme') as ThemeName | null) ?? 'sentinel-blue')
  const [themeOpen, setThemeOpen] = useState(false)
  const [paletteOpen, setPaletteOpen] = useState(false)
  const [overview, setOverview] = useState<SystemOverview | null>(null)
  const [database, setDatabase] = useState<DatabaseStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [toast, setToast] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [systemData, databaseData] = await Promise.all([getSystemOverview(), getDatabaseStatus()])
      setOverview(systemData)
      setDatabase(databaseData)
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { document.documentElement.dataset.theme = theme }, [theme])
  useEffect(() => { loadTheme().then(setTheme).catch(() => undefined); refresh() }, [refresh])
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setPaletteOpen((open) => !open)
      }
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

  const changeTheme = async (nextTheme: ThemeName) => {
    const previous = theme
    setTheme(nextTheme)
    setThemeOpen(false)
    try {
      await persistTheme(nextTheme)
      setToast('Theme preference saved locally')
    } catch {
      setTheme(previous)
      setToast('Theme could not be saved')
    }
  }

  return (
    <div className="app-shell" data-sidebar={collapsed ? 'compact' : 'expanded'}>
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark"><Shield size={21} strokeWidth={1.7} /></span><span className="brand-copy"><strong>EDY</strong><small>SENTINEL</small></span></div>
        <nav aria-label="Primary navigation">
          <span className="nav-label">Workspace</span>
          {nav.map((item) => <Tooltip key={item.label} label={item.label}><button type="button" className="nav-item" data-active={item.active || undefined} disabled={!item.active} title={!item.active ? 'Planned for a future sprint' : undefined}><item.icon size={18} /><span>{item.label}</span>{item.active && <i />}</button></Tooltip>)}
        </nav>
        <div className="sidebar-spacer" />
        <div className="sidebar-status"><span className="pulse" /><div><strong>Local mode</strong><small>No cloud connection</small></div></div>
        <div className="sidebar-bottom">
          <button type="button" className="nav-item" disabled><CircleHelp size={18} /><span>Help center</span></button>
          <button type="button" className="nav-item" disabled><Settings size={18} /><span>Settings</span></button>
        </div>
        <button type="button" className="collapse-button" onClick={() => setCollapsed((value) => !value)} aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}><ChevronLeft size={16} /></button>
      </aside>

      <div className="app-main">
        <header className="topbar">
          <div className="topbar-title"><IconButton className="mobile-menu" aria-label="Toggle sidebar" onClick={() => setCollapsed((value) => !value)}><Menu size={19} /></IconButton><div><span>Workspace</span><strong>Security overview</strong></div></div>
          <button type="button" className="search-trigger" onClick={() => setPaletteOpen(true)}><Search size={16} /><span>Search or run a command</span><kbd>Ctrl K</kbd></button>
          <div className="topbar-actions">
            <Tooltip label="Refresh telemetry"><IconButton aria-label="Refresh telemetry" onClick={refresh} disabled={loading}><RefreshCw size={17} className={loading ? 'spin' : ''} /></IconButton></Tooltip>
            <div className="theme-anchor"><Tooltip label="Change theme"><IconButton aria-label="Change theme" onClick={() => setThemeOpen((open) => !open)}><Palette size={17} /></IconButton></Tooltip>{themeOpen && <ThemeMenu theme={theme} onChange={changeTheme} />}</div>
            <Tooltip label="No new notifications"><IconButton aria-label="Notifications"><Bell size={17} /></IconButton></Tooltip>
            <span className="topbar-divider" />
            <div className="connection-state"><Wifi size={15} /><span><strong>Protected locally</strong><small>Collectors on device</small></span></div>
          </div>
        </header>

        <main className="content">
          <div className="page-heading"><div><p>Endpoint intelligence</p><h2>Good evening, {overview?.host.username ?? 'operator'}</h2><span>Review the current state of this Windows device.</span></div><button type="button" className="button button--primary" onClick={refresh} disabled={loading}><RefreshCw size={16} className={loading ? 'spin' : ''} /> Refresh telemetry</button></div>
          <Overview data={overview} database={database} loading={loading} error={error} onRefresh={refresh} />
        </main>
      </div>

      <Dialog open={paletteOpen} title="Command palette" onClose={() => setPaletteOpen(false)}>
        <div className="command-search"><Search size={17} /><input autoFocus aria-label="Command search" placeholder="Type a command…" /></div>
        <div className="command-list"><span>Available now</span><button type="button" onClick={() => { setPaletteOpen(false); refresh() }}><RefreshCw size={16} /><div><strong>Refresh system telemetry</strong><small>Collect a new real snapshot</small></div><kbd>↵</kbd></button><button type="button" onClick={() => { setPaletteOpen(false); setThemeOpen(true) }}><Palette size={16} /><div><strong>Change interface theme</strong><small>Choose from four persisted themes</small></div></button><button type="button" onClick={() => { setCollapsed((value) => !value); setPaletteOpen(false) }}><Command size={16} /><div><strong>Toggle sidebar</strong><small>Switch compact navigation mode</small></div></button></div>
      </Dialog>
      {toast && <div className="toast" role="status"><span className="toast-mark" />{toast}</div>}
    </div>
  )
}

export default App
