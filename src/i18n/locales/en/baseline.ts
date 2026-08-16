export default {
  status: {
    not_initialized: { label: 'Not initialized', detail: 'No behavior has been learned yet.' },
    learning: { label: 'Learning', detail: 'Real observations are being added without generating new-behavior events.' },
    ready: { label: 'Ready', detail: 'New factual differences can create deduplicated security events.' },
    stale: { label: 'Stale', detail: 'The baseline has not received a recent successful observation.' },
    error: { label: 'Error', detail: 'The baseline could not be updated. Existing telemetry remains available.' },
  },
  panel: {
    title: 'Behavioral baseline', subtitle: 'Local factual learning · no threat classification', observationPeriod: 'Observation period', observationCycles: 'Observation cycles', learnedFacts: 'Learned facts', processing: 'Baseline processing', version: 'Baseline version',
    executables_one: '{{formattedCount}} executable', executables_other: '{{formattedCount}} executables', relationships_one: '{{formattedCount}} relationship', relationships_other: '{{formattedCount}} relationships', destinations_one: '{{formattedCount}} destination', destinations_other: '{{formattedCount}} destinations', services_one: '{{formattedCount}} service', services_other: '{{formattedCount}} services',
    timelineAria: 'Baseline timeline', started: 'Started', completed: 'Completed', learningInProgress: 'Learning in progress',
    start: 'Start baseline', complete: 'Complete learning', startNew: 'Start new baseline', reset: 'Reset baseline', securityEvents: 'Security events', localPersistenceReady: 'Local persistence ready', persistenceUnavailable: 'Persistence unavailable',
  },
  dialog: {
    title: { reset: 'Reset behavioral baseline', complete: 'Complete baseline learning', start: 'Start new behavioral baseline' },
    warning: {
      reset: 'Resetting starts a new learning period. Previous baseline versions and other Sentinel history are preserved.',
      complete: 'Manual completion is intended for controlled development validation. Only real observations already collected are used.',
      start: 'The current baseline is preserved as history. The new version learns from real local telemetry and produces no new-behavior events while Learning.',
    },
    learningPeriod: 'Learning period', confirmation: 'Type <code>{{phrase}}</code> to confirm', confirmationAria: 'Baseline confirmation phrase', cancel: 'Cancel', working: 'Working…', actionError: 'The baseline action could not be completed. Review the current state and try again.',
    periods: { minute: '1 minute · controlled development test', hour: '1 hour', hours6: '6 hours', hours24: '24 hours · recommended default', days3: '3 days', days7: '7 days' },
  },
} as const
