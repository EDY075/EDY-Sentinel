import { Activity, Binary, CalendarClock, CheckCircle2, Clock3, Database, GitBranch, Network, Radar, ServerCog } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '../../components/ui/primitives'
import type { BaselineSummary } from '../../types/baseline'
import type { DatabaseStatus } from '../../types/system'
import { baselinePresentation, formatLearningDuration, learningElapsedSeconds } from './baseline'
import type { BaselineActionMode } from './baseline'
import { formatDateTime } from '../telemetry/format'

export function BaselinePanel({ baseline, database, onAction, onOpenEvents }: { baseline: BaselineSummary | null; database: DatabaseStatus | null; onAction: (mode: BaselineActionMode) => void; onOpenEvents: () => void }) {
  const { t, i18n } = useTranslation('baseline')
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const current = baseline ?? { status: 'not_initialized', observationCount: 0, schemaVersion: 1, learningPeriodSeconds: 0, lastProcessingDurationMs: 0, entities: { executables: 0, processPatterns: 0, parentChildRelationships: 0, networkDestinations: 0, services: 0, networkConfigurations: 0 } } as BaselineSummary
  const presentation = baselinePresentation(current.status)
  const elapsed = formatLearningDuration(learningElapsedSeconds(current), locale)
  const entityTotal = Object.values(current.entities).reduce((sum, count) => sum + count, 0)

  return <section className="panel baseline-panel" aria-labelledby="baseline-title">
    <header className="panel__header"><div><Radar size={18} /><span><strong id="baseline-title">{t('panel.title')}</strong><small>{t('panel.subtitle')}</small></span></div><Badge tone={presentation.tone}>{t(`status.${presentation.key}.label`)}</Badge></header>
    <div className="baseline-summary">
      <p>{t(`status.${presentation.key}.detail`)}</p>
      <div className="baseline-metrics">
        <span><Clock3 size={14} /><strong>{current.status === 'not_initialized' ? '—' : elapsed}</strong><small>{t('panel.observationPeriod')}</small></span>
        <span><Database size={14} /><strong>{number.format(current.observationCount)}</strong><small>{t('panel.observationCycles')}</small></span>
        <span><Binary size={14} /><strong>{number.format(entityTotal)}</strong><small>{t('panel.learnedFacts')}</small></span>
        <span><Activity size={14} /><strong>{number.format(current.lastProcessingDurationMs)} ms</strong><small>{t('panel.processing')}</small></span>
        <span><GitBranch size={14} /><strong>{current.version ? `v${number.format(current.version)}` : '—'}</strong><small>{t('panel.version')}</small></span>
      </div>
      {current.baselineId && <div className="baseline-entity-grid">
        <span><Binary size={13} /> {t('panel.executables', { count: current.entities.executables, formattedCount: number.format(current.entities.executables) })}</span>
        <span><GitBranch size={13} /> {t('panel.relationships', { count: current.entities.parentChildRelationships, formattedCount: number.format(current.entities.parentChildRelationships) })}</span>
        <span><Network size={13} /> {t('panel.destinations', { count: current.entities.networkDestinations, formattedCount: number.format(current.entities.networkDestinations) })}</span>
        <span><ServerCog size={13} /> {t('panel.services', { count: current.entities.services, formattedCount: number.format(current.entities.services) })}</span>
      </div>}
      {current.baselineId && <div className="baseline-timeline" aria-label={t('panel.timelineAria')}>
        <span><CalendarClock size={13} /><small>{t('panel.started')}</small><strong>{formatDateTime(current.learningStartedAt, locale, '—')}</strong></span>
        <span><CheckCircle2 size={13} /><small>{t('panel.completed')}</small><strong>{current.learningCompletedAt ? formatDateTime(current.learningCompletedAt, locale, '—') : t('panel.learningInProgress')}</strong></span>
      </div>}
      <div className="baseline-actions">
        {current.status === 'not_initialized' && <button type="button" className="button button--primary" onClick={() => onAction('start')}>{t('panel.start')}</button>}
        {current.status === 'learning' && <button type="button" className="button" onClick={() => onAction('complete')}>{t('panel.complete')}</button>}
        {current.status !== 'not_initialized' && <button type="button" className="button" onClick={() => onAction('start')}>{t('panel.startNew')}</button>}
        {current.status !== 'not_initialized' && <button type="button" className="button button--danger-subtle" onClick={() => onAction('reset')}>{t('panel.reset')}</button>}
        <button type="button" className="button" onClick={onOpenEvents}>{t('panel.securityEvents')}</button>
      </div>
    </div>
    <footer className="baseline-score-boundary"><span><strong>SQLite v{database?.schemaVersion ?? '—'}</strong><small>{database?.writable ? t('panel.localPersistenceReady') : t('panel.persistenceUnavailable')}</small></span></footer>
  </section>
}
