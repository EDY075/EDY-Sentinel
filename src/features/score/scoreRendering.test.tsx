import { createInstance } from 'i18next'
import type { ReactNode } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { I18nextProvider } from 'react-i18next'
import { describe, expect, it } from 'vitest'
import en from '../../i18n/locales/en/score'
import ptBR from '../../i18n/locales/pt-BR/score'
import type { ProductVulnerabilityRisk, SecurityScore } from '../../types/score'
import { ProductRiskDetail, ScoreBreakdownDrawer } from './ScoreBreakdownDrawer'
import { SecurityScorePanel } from './SecurityScorePanel'

const pythonRisk: ProductVulnerabilityRisk = {
  productRiskKey: 'python/python', canonicalVendor: 'python', canonicalProduct: 'python',
  displayNames: ['Python 3.12.10 (64-bit)'], softwareIds: ['python-software'], installedVersions: ['3.12.10'],
  confirmedCveIds: Array.from({ length: 10 }, (_, index) => `CVE-2026-${1000 + index}`),
  possibleCveIds: ['CVE-2026-3087'], confirmedCount: 10, possibleCount: 1,
  cvssBands: { unrated: 0, low: 3, medium: 5, high: 2, critical: 0 }, highestCvss: 8.1,
  confirmedKevCount: 0, severityAnchor: 2, marginalBreadth: 2, kevBoost: 0, uncappedImpact: 4, impact: 4,
  matchingEngineVersions: [1], identityResolverVersions: [1], nvdSourceVersions: ['nvd-2026'], kevSourceVersions: ['kev-2026'], evaluatedAt: '2026-08-17T12:00:00Z',
}

const jreRisk: ProductVulnerabilityRisk = {
  ...pythonRisk, productRiskKey: 'oracle/jre', canonicalVendor: 'oracle', canonicalProduct: 'jre',
  displayNames: ['Java 8 Update 401'], softwareIds: ['jre-software'], installedVersions: ['8u401'],
  confirmedCveIds: Array.from({ length: 6 }, (_, index) => `CVE-2023-${4100 + index}`), possibleCveIds: [],
  confirmedCount: 6, possibleCount: 0, cvssBands: { unrated: 0, low: 5, medium: 0, high: 1, critical: 0 },
  highestCvss: 8.1, confirmedKevCount: 1, severityAnchor: 2, marginalBreadth: 0.625, kevBoost: 3, uncappedImpact: 5.625, impact: 6,
}

const score: SecurityScore = {
  state: 'limited', score: 86, label: 'Good', generatedAt: '2026-08-17T12:00:00Z', formulaVersion: 2,
  activeDetectionCount: 0, coverage: [], breakdown: [], detectionPenalty: 0, vulnerabilityPenalty: 14,
  vulnerabilityCoverage: { status: 'updating', basis: 'software_record_proxy', totalSoftware: 81, eligibleSoftware: 66, resolvedEligible: 19, ambiguousEligible: 0, unresolvedEligible: 47, notMappable: 15, pendingEvaluation: 81 },
  productVulnerabilityRisks: [jreRisk, { ...pythonRisk, productRiskKey: 'oracle/vm_virtualbox', canonicalVendor: 'oracle', canonicalProduct: 'vm_virtualbox', displayNames: ['Oracle VM VirtualBox 7.2.12'], softwareIds: ['virtualbox-software'], confirmedCveIds: Array.from({ length: 15 }, (_, index) => `CVE-2026-${4700 + index}`), possibleCveIds: [], confirmedCount: 15, possibleCount: 0 }, pythonRisk],
  reason: 'Limited coverage — Vulnerability Intelligence is updating',
}

async function renderer(language: 'pt-BR' | 'en') {
  const instance = createInstance()
  await instance.init({ lng: language, resources: { 'pt-BR': { score: ptBR }, en: { score: en } }, defaultNS: 'score', interpolation: { escapeValue: false } })
  return (node: ReactNode) => renderToStaticMarkup(<I18nextProvider i18n={instance}>{node}</I18nextProvider>)
}

describe('Security Score v2 rendering', () => {
  it('keeps the numeric score visible while clearly qualifying limited coverage in pt-BR', async () => {
    const render = await renderer('pt-BR')
    const html = render(<SecurityScorePanel score={score} baselineStatus="ready" onOpen={() => undefined} />)
    expect(html).toContain('86')
    expect(html).toContain('/100')
    expect(html).toContain('Análise com cobertura limitada')
    expect(html).toContain('Detecções -0')
    expect(html).toContain('Vulnerabilidades -14')
    expect(html).not.toContain('86%')
    expect(html).toContain('aria-label="Pontuação de Segurança 86 de 100.')
  })

  it('renders explicit category math, products, KEV, coverage, and formula v2 in English', async () => {
    const render = await renderer('en')
    const html = render(<ScoreBreakdownDrawer score={score} open onClose={() => undefined} onOpenProduct={() => undefined} />)
    expect(html).toContain('Security Score breakdown')
    expect(html).toContain('Detections')
    expect(html).toContain('Vulnerabilities')
    expect(html).toContain('Java 8 Update 401')
    expect(html).toContain('Oracle VM VirtualBox 7.2.12')
    expect(html).toContain('Python 3.12.10')
    expect(html).toContain('Possible: score impact 0')
    expect(html).toContain('81')
    expect(html).toContain('Not mappable')
    expect(html).toContain('v2')
    expect(html).toContain('role="dialog"')
  })

  it('renders Possible as neutral context with zero score impact and KEV as enrichment', async () => {
    const renderPt = await renderer('pt-BR')
    const possibleHtml = renderPt(<ProductRiskDetail product={pythonRisk} onBack={() => undefined} onOpenProduct={() => undefined} />)
    expect(possibleHtml).toContain('Correspondência possível')
    expect(possibleHtml).toContain('Impacto na Pontuação de Segurança: 0')
    expect(possibleHtml).toContain('CVE-2026-3087')
    expect(possibleHtml).toContain('badge--neutral')
    const kevHtml = renderPt(<ProductRiskDetail product={jreRisk} onBack={() => undefined} onOpenProduct={() => undefined} />)
    expect(kevHtml).toContain('Vulnerabilidade com exploração conhecida')
    expect(kevHtml).toContain('não confirma a relação entre software e CVE')
    expect(kevHtml).toContain('Abrir produto no Inventário de Software')
  })
})
