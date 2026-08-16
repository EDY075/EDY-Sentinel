export const common = {
  appName: 'EDY Sentinel',
  actions: {
    cancel: 'Cancel',
    close: 'Close',
    refresh: 'Refresh',
    retry: 'Try again',
    save: 'Save',
  },
  states: {
    disabled: 'Disabled',
    enabled: 'Enabled',
    loading: 'Loading',
    unavailable: 'Unavailable',
  },
  status: {
    error: 'Error',
    offline: 'Offline',
    online: 'Online',
    partial: 'Partially available',
    success: 'Success',
  },
  accessibility: {
    closeDialog: 'Close dialog',
    closeDrawer: 'Close drawer',
    metricTrend: 'Metric trend',
  },
  observations: {
    processes_one: '{{count}} process observed',
    processes_other: '{{count}} processes observed',
  },
} as const
