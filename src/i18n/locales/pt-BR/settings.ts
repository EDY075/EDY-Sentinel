export const settings = {
  title: 'Configurações',
  description: 'Gerencie as preferências locais da interface.',
  open: 'Abrir configurações',
  sections: {
    appearance: 'Aparência',
    language: 'Idioma',
    about: 'Sobre',
  },
  language: {
    label: 'Idioma da interface',
    description: 'Escolha o idioma usado pela interface do EDY Sentinel.',
    current: 'Idioma atual',
    saved: 'Preferência de idioma salva localmente',
    saveError: 'Não foi possível salvar a preferência de idioma',
    options: {
      en: 'Inglês',
      ptBR: 'Português (Brasil)',
    },
  },
  about: {
    description: 'Consulte a versão instalada do produto e o canal de lançamento.',
    product: 'Produto',
    version: 'Versão',
    channel: 'Canal',
    stable: 'Estável',
    unavailable: 'Indisponível',
  },
} as const
