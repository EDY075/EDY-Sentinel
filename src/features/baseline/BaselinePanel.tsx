import { Activity, Binary, CalendarClock, CheckCircle2, Clock3, Database, GitBranch, Network, Radar, ServerCog } from 'lucide-react'
import { Badge } from '../../components/ui/primitives'
import type { BaselineSummary } from '../../types/baseline'
import type { DatabaseStatus } from '../../types/system'
import { baselinePresentation, formatLearningDuration, learningElapsedSeconds } from './baseline'
import type { BaselineActionMode } from './baseline'
import { formatDateTime } from '../telemetry/format'

export function BaselinePanel({ baseline, database, onAction, onOpenEvents }: { baseline: BaselineSummary | null; database: DatabaseStatus | null; onAction: (mode: BaselineActionMode) => void; onOpenEvents: () => void }) {
  const current = baseline ?? { status: 'not_initialized', observationCount: 0, schemaVersion: 1, learningPeriodSeconds: 0, lastProcessingDurationMs: 0, entities: { executables: 0, processPatterns: 0, parentChildRelationships: 0, networkDestinations: 0, services: 0, networkConfigurations: 0 } } as BaselineSummary
  const presentation = baselinePresentation(current.status)
  const elapsed = formatLearningDuration(learningElapsedSeconds(current))
  const entityTotal = Object.values(current.entities).reduce((sum, count) => sum + count, 0)

  return <section className="panel baseline-panel" aria-labelledby="baseline-title">
    <header className="panel__header"><div><Radar size={18} /><span><strong id="baseline-title">Behavioral baseline</strong><small>Local factual learning · no threat classification</small></span></div><Badge tone={presentation.tone}>{presentation.label}</Badge></header>
    <div className="baseline-summary">
      <p>{presentation.detail}</p>
      <div className="baseline-metrics">
        <span><Clock3 size={14} /><strong>{current.status === 'not_initialized' ? '—' : elapsed}</strong><small>Observation period</small></span>
        <span><Database size={14} /><strong>{current.observationCount}</strong><small>Observation cycles</small></span>
        <span><Binary size={14} /><strong>{entityTotal}</strong><small>Learned facts</small></span>
        <span><Activity size={14} /><strong>{current.lastProcessingDurationMs} ms</strong><small>Baseline processing</small></span>
        <span><GitBranch size={14} /><strong>{current.version ? `v${current.version}` : '—'}</strong><small>Baseline version</small></span>
      </div>
      {current.baselineId && <div className="baseline-entity-grid">
        <span><Binary size={13} /> {current.entities.executables} executables</span>
        <span><GitBranch size={13} /> {current.entities.parentChildRelationships} relationships</span>
        <span><Network size={13} /> {current.entities.networkDestinations} destinations</span>
        <span><ServerCog size={13} /> {current.entities.services} services</span>
      </div>}
      {current.baselineId && <div className="baseline-timeline" aria-label="Baseline timeline">
        <span><CalendarClock size={13} /><small>Started</small><strong>{formatDateTime(current.learningStartedAt)}</strong></span>
        <span><CheckCircle2 size={13} /><small>Completed</small><strong>{current.learningCompletedAt ? formatDateTime(current.learningCompletedAt) : 'Learning in progress'}</strong></span>
      </div>}
      <div className="baseline-actions">
        {current.status === 'not_initialized' && <button type="button" className="button button--primary" onClick={() => onAction('start')}>Start baseline</button>}
        {current.status === 'learning' && <button type="button" className="button" onClick={() => onAction('complete')}>Complete learning</button>}
        {current.status !== 'not_initialized' && <button type="button" className="button" onClick={() => onAction('start')}>Start new baseline</button>}
        {current.status !== 'not_initialized' && <button type="button" className="button button--danger-subtle" onClick={() => onAction('reset')}>Reset baseline</button>}
        <button type="button" className="button" onClick={onOpenEvents}>Security events</button>
      </div>
    </div>
    <footer className="baseline-score-boundary"><span><strong>SQLite v{database?.schemaVersion ?? '—'}</strong><small>{database?.writable ? 'Local persistence ready' : 'Persistence unavailable'}</small></span></footer>
  </section>
}
