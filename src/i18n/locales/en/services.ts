export default {
  filters: { all: 'All', running: 'Running', stopped: 'Stopped', automatic: 'Automatic', manual: 'Manual', disabled: 'Disabled' },
  search: 'Search service name, display name, binary path, or account',
  ofTotal: 'of {{total}}',
  columns: { service: 'Service', status: 'Status', startup: 'Startup', pid: 'PID', account: 'Account' },
  binaryPathUnavailable: 'Binary path unavailable',
  empty: { title: 'No services match this view', description: 'Change the search or service-state filter.' },
  ariaLabel: 'Windows services',
  unavailable: 'Unavailable',
  status: { stopped: 'Stopped', start_pending: 'Start pending', stop_pending: 'Stop pending', running: 'Running', continue_pending: 'Continue pending', pause_pending: 'Pause pending', paused: 'Paused', unknown: 'Unknown' },
  startupType: { boot: 'Boot', system: 'System', automatic_delayed: 'Automatic (delayed)', automatic: 'Automatic', manual: 'Manual', disabled: 'Disabled', unknown: 'Unknown' },
} as const
