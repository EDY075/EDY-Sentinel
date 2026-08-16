export default {
  workspace: { ariaLabel: 'Security analysis views', detections: 'Detections', events: 'Events', refresh: 'Refresh analysis', rules: 'Detection rules', warning: 'Some security analysis state is unavailable. Try refreshing the analysis.', detectionsPanel: 'Detections', eventsPanel: 'Factual security events' },
  pagination: { ariaLabel: '{{noun}} pagination', status: 'Page {{page}} · {{count}} {{noun}}', refresh: 'Refresh {{noun}}', previous: 'Previous', next: 'Next', nouns: { detections_one: 'detection', detections_other: 'detections', events_one: 'event', events_other: 'events', evidence_one: 'evidence record', evidence_other: 'evidence records', history_one: 'history record', history_other: 'history records' } },
} as const
