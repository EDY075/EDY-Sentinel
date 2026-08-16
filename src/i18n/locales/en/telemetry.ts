export default {
  collectors: {
    ariaLabel: 'Collector health', label: 'Collectors', observed_one: '{{formattedCount}} observed', observed_other: '{{formattedCount}} observed', restricted: '{{formattedCount}} with restricted metadata', summary: '{{label}}: {{state}}. {{coverage}}{{timing}}{{issue}}', timing: ' · {{duration}} ms', issue: '. Collection details are temporarily unavailable.', restrictedBadge: '{{formattedCount}} restricted', warning: 'One or more collectors reported a factual collection failure', live: 'Live', paused: 'Paused', refresh: 'Refresh all telemetry',
    names: { system: 'System', processes: 'Processes', network: 'Network', services: 'Services' },
    states: { healthy: 'Healthy', degraded: 'Degraded', failed: 'Failed', paused: 'Paused', loading: 'Loading' },
  },
  toolbar: { filtersAria: 'Table filters' },
  table: { unavailable: 'Unavailable' },
} as const
