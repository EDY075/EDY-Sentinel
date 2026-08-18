export const settings = {
  title: 'Settings',
  description: 'Manage local interface preferences.',
  open: 'Open settings',
  sections: {
    appearance: 'Appearance',
    language: 'Language',
    about: 'About',
  },
  language: {
    label: 'Interface language',
    description: 'Choose the language used by the EDY Sentinel interface.',
    current: 'Current language',
    saved: 'Language preference saved locally',
    saveError: 'Language preference could not be saved',
    options: {
      en: 'English',
      ptBR: 'Portuguese (Brazil)',
    },
  },
  about: {
    description: 'Review the installed product version and release channel.',
    product: 'Product',
    version: 'Version',
    channel: 'Channel',
    stable: 'Stable',
    unavailable: 'Unavailable',
  },
} as const
