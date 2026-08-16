import { common as commonEn } from './locales/en/common'
import { errors as errorsEn } from './locales/en/errors'
import { navigation as navigationEn } from './locales/en/navigation'
import { settings as settingsEn } from './locales/en/settings'
import baselineEn from './locales/en/baseline'
import connectionsEn from './locales/en/connections'
import detectionsEn from './locales/en/detections'
import eventsEn from './locales/en/events'
import overviewEn from './locales/en/overview'
import processesEn from './locales/en/processes'
import rulesEn from './locales/en/rules'
import scoreEn from './locales/en/score'
import securityEn from './locales/en/security'
import servicesEn from './locales/en/services'
import shellEn from './locales/en/shell'
import telemetryEn from './locales/en/telemetry'
import { common as commonPtBR } from './locales/pt-BR/common'
import { errors as errorsPtBR } from './locales/pt-BR/errors'
import { navigation as navigationPtBR } from './locales/pt-BR/navigation'
import { settings as settingsPtBR } from './locales/pt-BR/settings'
import baselinePtBR from './locales/pt-BR/baseline'
import connectionsPtBR from './locales/pt-BR/connections'
import detectionsPtBR from './locales/pt-BR/detections'
import eventsPtBR from './locales/pt-BR/events'
import overviewPtBR from './locales/pt-BR/overview'
import processesPtBR from './locales/pt-BR/processes'
import rulesPtBR from './locales/pt-BR/rules'
import scorePtBR from './locales/pt-BR/score'
import securityPtBR from './locales/pt-BR/security'
import servicesPtBR from './locales/pt-BR/services'
import shellPtBR from './locales/pt-BR/shell'
import telemetryPtBR from './locales/pt-BR/telemetry'

export const namespaces = [
  'common', 'navigation', 'settings', 'errors', 'shell', 'overview', 'processes',
  'connections', 'services', 'baseline', 'telemetry', 'events', 'detections',
  'rules', 'score', 'security',
] as const
export type TranslationNamespace = (typeof namespaces)[number]

export const resources = {
  en: {
    common: commonEn,
    navigation: navigationEn,
    settings: settingsEn,
    errors: errorsEn,
    shell: shellEn,
    overview: overviewEn,
    processes: processesEn,
    connections: connectionsEn,
    services: servicesEn,
    baseline: baselineEn,
    telemetry: telemetryEn,
    events: eventsEn,
    detections: detectionsEn,
    rules: rulesEn,
    score: scoreEn,
    security: securityEn,
  },
  'pt-BR': {
    common: commonPtBR,
    navigation: navigationPtBR,
    settings: settingsPtBR,
    errors: errorsPtBR,
    shell: shellPtBR,
    overview: overviewPtBR,
    processes: processesPtBR,
    connections: connectionsPtBR,
    services: servicesPtBR,
    baseline: baselinePtBR,
    telemetry: telemetryPtBR,
    events: eventsPtBR,
    detections: detectionsPtBR,
    rules: rulesPtBR,
    score: scorePtBR,
    security: securityPtBR,
  },
} as const

type ResourceValue = string | { readonly [key: string]: ResourceValue }

export function flattenResourceKeys(resource: { readonly [key: string]: ResourceValue }, prefix = ''): string[] {
  return Object.entries(resource).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key
    return typeof value === 'string' ? [path] : flattenResourceKeys(value, path)
  }).sort()
}
