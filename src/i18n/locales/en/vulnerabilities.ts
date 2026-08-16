export default {
  title: 'Vulnerability repositories',
  description: 'Synchronize authoritative public data for offline local use. Software-to-CVE matching is not performed yet.',
  providers: { nvd: 'NVD', cisa_kev: 'CISA KEV' },
  providerDescriptions: {
    nvd: 'National Vulnerability Database CVE records and applicability data.',
    cisa_kev: 'Known Exploited Vulnerabilities catalog from CISA.',
  },
  status: { idle: 'Not synchronized', ready: 'Ready', updating: 'Updating', error: 'Error' },
  fields: { lastSync: 'Last sync', records: 'Records', never: 'Never', localCache: 'Local cache' },
  actions: { synchronize: 'Synchronize', cancel: 'Cancel update', refreshStatus: 'Refresh status' },
  messages: {
    loading: 'Loading provider status', loadError: 'Provider status could not be loaded.',
    syncError: 'The external update failed. Existing local records were preserved.',
    offline: 'After a successful sync, the local repository remains available offline.',
    noApiKey: 'No API key is stored in the interface.',
  },
} as const
