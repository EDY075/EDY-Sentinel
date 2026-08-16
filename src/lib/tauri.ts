import { invoke } from '@tauri-apps/api/core'
import type { DatabaseStatus, SystemOverview, ThemeName } from '../types/system'
import type { LiveTelemetrySnapshot } from '../types/telemetry'
import type { BaselineAction, BaselineSummary, SecurityEvent, SecurityEventStatus } from '../types/baseline'

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

export async function getBaselineSummary(): Promise<BaselineSummary> {
  if (!isTauri()) throw new Error('Behavioral baseline is available only inside the EDY Sentinel desktop app.')
  return invoke<BaselineSummary>('get_baseline_summary')
}

export async function getSecurityEvents(): Promise<SecurityEvent[]> {
  if (!isTauri()) throw new Error('Security events are available only inside the EDY Sentinel desktop app.')
  return invoke<SecurityEvent[]>('get_security_events')
}

export async function startNewBaseline(input: BaselineAction): Promise<BaselineSummary> {
  return invoke<BaselineSummary>('start_new_baseline', { input })
}

export async function resetBaseline(input: BaselineAction): Promise<BaselineSummary> {
  return invoke<BaselineSummary>('reset_baseline', { input })
}

export async function completeBaselineLearning(input: BaselineAction): Promise<BaselineSummary> {
  return invoke<BaselineSummary>('complete_baseline_learning', { input })
}

export async function updateSecurityEventStatus(eventId: string, status: SecurityEventStatus): Promise<void> {
  return invoke('set_security_event_status', { input: { eventId, status } })
}
