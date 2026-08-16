import { invoke } from '@tauri-apps/api/core'
import type { DatabaseStatus, SystemOverview, ThemeName } from '../types/system'
import type { LiveTelemetrySnapshot } from '../types/telemetry'

export const isTauri = () => '__TAURI_INTERNALS__' in window

export async function getSystemOverview(): Promise<SystemOverview> {
  if (!isTauri()) {
    throw new Error('System collection is available only inside the EDY Sentinel desktop app.')
  }
  return invoke<SystemOverview>('get_system_overview')
}

export async function loadTheme(): Promise<ThemeName> {
  if (!isTauri()) {
    return (localStorage.getItem('edy-sentinel-theme') as ThemeName | null) ?? 'sentinel-blue'
  }
  return invoke<ThemeName>('get_theme')
}

export async function persistTheme(theme: ThemeName): Promise<void> {
  localStorage.setItem('edy-sentinel-theme', theme)
  if (isTauri()) {
    await invoke('set_theme', { input: { theme } })
  }
}

export async function getDatabaseStatus(): Promise<DatabaseStatus | null> {
  if (!isTauri()) return null
  return invoke<DatabaseStatus>('get_database_status')
}

export async function getLiveTelemetry(): Promise<LiveTelemetrySnapshot> {
  if (!isTauri()) {
    throw new Error('Live telemetry is available only inside the EDY Sentinel desktop app.')
  }
  return invoke<LiveTelemetrySnapshot>('get_live_telemetry')
}
