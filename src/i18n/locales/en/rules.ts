export default {
  header: { back: 'Back to detections', title: 'Detection rules', subtitle: 'Versioned local rules · conditions are read-only', count_one: '{{formattedCount}} rule', count_other: '{{formattedCount}} rules' },
  table: { rule: 'Rule', category: 'Category', version: 'Version', defaultSeverity: 'Default severity', evidence: 'Evidence', enabled: 'Enabled', required_one: '{{formattedCount}} required', required_other: '{{formattedCount}} required' },
  state: { enabled: 'Enabled', disabled: 'Disabled', enableAria: 'Enable {{name}}', disableAria: 'Disable {{name}}' },
  error: { load: 'Rules could not be loaded. Try again.', retry: 'Retry' },
  empty: { title: 'No registered detection rules', description: 'The Rust registry did not return any operational rule definitions.' },
  categories: { process_execution: 'Process execution', network_activity: 'Network activity', service_persistence: 'Service persistence', network_configuration: 'Network configuration' },
  definitions: {
    'EDY-PROC-001': { name: 'New unsigned executable in a temporary user-writable path', description: 'A new executable was actually launched from a temporary user-writable path and Windows reported it as unsigned.' },
    'EDY-PROC-002': { name: 'New unsigned child execution with a known signed parent', description: 'A baseline-known signed parent launched a new unsigned child from a temporary user-writable path through a previously unseen relationship.' },
    'EDY-NET-001': { name: 'New outbound activity from a new unsigned temporary executable', description: 'A newly executed unsigned binary in a temporary user-writable path initiated first-seen outbound activity with an unambiguous process association.' },
    'EDY-SVC-001': { name: 'New privileged automatic service from a user-writable path', description: 'A newly observed running service uses an automatic startup mode, a privileged system account, and a binary under a user-writable location.' },
    'EDY-SVC-002': { name: 'Correlated persistent service reconfiguration', description: 'A known service changed its binary path together with its startup type or account in the same collection window.' },
    'EDY-NET-002': { name: 'Persistent coordinated network configuration change', description: 'The default gateway and DNS resolver set changed together and remained different from the baseline for at least two observations.' },
  },
} as const
