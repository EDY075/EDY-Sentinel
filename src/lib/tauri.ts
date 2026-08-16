import { invoke } from '@tauri-apps/api/core'
import type { DatabaseStatus, SystemOverview, ThemeName } from '../types/system'
import type { LiveTelemetrySnapshot } from '../types/telemetry'
import type { BaselineAction, BaselineSummary, SecurityEvent, SecurityEventHistoryPage, SecurityEventHistoryQuery, SecurityEventPage, SecurityEventQuery, SecurityEventStatus } from '../types/baseline'
import type { DetectionEvidencePage, DetectionEvidenceQuery, DetectionPage, DetectionQuery, DetectionStatus } from '../types/detection'
import type { SecurityScore } from '../types/score'
import type { RuleDefinition } from '../types/rules'
import type { InstalledSoftware, SoftwareInventorySnapshot, VulnerabilityProviderName, VulnerabilityProviderStatus } from '../types/inventory'
import { LANGUAGE_STORAGE_KEY, normalizeLanguage, type SupportedLanguage } from '../i18n/language'

export const isTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

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

export async function loadLanguage(): Promise<SupportedLanguage | null> {
  if (typeof window === 'undefined') return null
  const persisted = isTauri()
    ? await invoke<string | null>('get_language')
    : window.localStorage.getItem(LANGUAGE_STORAGE_KEY)
  return normalizeLanguage(persisted)
}

export async function loadSystemLocale(): Promise<string | null> {
  if (!isTauri()) return null
  const locale = await invoke<string>('get_system_locale')
  return locale.trim() || null
}

export async function persistLanguage(language: SupportedLanguage): Promise<void> {
  if (isTauri()) {
    await invoke('set_language', { input: { language } })
    return
  }
  if (typeof window !== 'undefined') window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language)
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

export async function getSecurityEventsPage(input: SecurityEventQuery = {}): Promise<SecurityEventPage> {
  if (!isTauri()) throw new Error('Security events are available only inside the EDY Sentinel desktop app.')
  return invoke<SecurityEventPage>('get_security_events_page', { input })
}

export async function getSecurityEventHistory(input: SecurityEventHistoryQuery): Promise<SecurityEventHistoryPage> {
  if (!isTauri()) throw new Error('Security event history is available only inside the EDY Sentinel desktop app.')
  return invoke<SecurityEventHistoryPage>('get_security_event_history', { input })
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

export async function getDetectionsPage(input: DetectionQuery = {}): Promise<DetectionPage> {
  if (!isTauri()) throw new Error('Detections are available only inside the EDY Sentinel desktop app.')
  return invoke<DetectionPage>('get_detections_page', { input })
}

export async function getDetectionEvidence(input: DetectionEvidenceQuery): Promise<DetectionEvidencePage> {
  if (!isTauri()) throw new Error('Detection evidence is available only inside the EDY Sentinel desktop app.')
  return invoke<DetectionEvidencePage>('get_detection_evidence', { input })
}

export async function updateDetectionStatus(detectionId: string, status: DetectionStatus): Promise<void> {
  return invoke('set_detection_status', { input: { detectionId, status } })
}

export async function getDetectionRules(): Promise<RuleDefinition[]> {
  if (!isTauri()) throw new Error('Detection rules are available only inside the EDY Sentinel desktop app.')
  return invoke<RuleDefinition[]>('get_detection_rules')
}

export async function setDetectionRuleEnabled(ruleId: string, enabled: boolean): Promise<void> {
  return invoke('set_detection_rule_enabled', { input: { ruleId, enabled } })
}

export async function getSecurityScore(): Promise<SecurityScore> {
  if (!isTauri()) throw new Error('Security Score is available only inside the EDY Sentinel desktop app.')
  return invoke<SecurityScore>('get_security_score')
}

export async function getSoftwareInventory(): Promise<InstalledSoftware[]> {
  if (!isTauri()) throw new Error('Software inventory is available only inside the EDY Sentinel desktop app.')
  return invoke<InstalledSoftware[]>('get_software_inventory')
}

export async function refreshSoftwareInventory(): Promise<SoftwareInventorySnapshot> {
  if (!isTauri()) throw new Error('Software inventory is available only inside the EDY Sentinel desktop app.')
  return invoke<SoftwareInventorySnapshot>('refresh_software_inventory')
}

export async function getVulnerabilityProviderStatus(): Promise<VulnerabilityProviderStatus[]> {
  if (!isTauri()) return []
  return invoke<VulnerabilityProviderStatus[]>('get_vulnerability_provider_status')
}

export async function syncVulnerabilityProvider(provider: VulnerabilityProviderName): Promise<VulnerabilityProviderStatus> {
  if (!isTauri()) throw new Error('Vulnerability repository sync is available only inside the EDY Sentinel desktop app.')
  return invoke<VulnerabilityProviderStatus>('sync_vulnerability_provider', { input: { provider } })
}

export async function cancelVulnerabilitySync(): Promise<void> {
  if (isTauri()) await invoke('cancel_vulnerability_sync')
}
