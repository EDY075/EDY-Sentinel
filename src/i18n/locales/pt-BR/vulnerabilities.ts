export default {
  title: 'Repositórios de vulnerabilidades',
  description: 'Sincronize dados públicos oficiais para avaliação local e conservadora entre software e CVE.',
  providers: { nvd: 'NVD', cisa_kev: 'CISA KEV' },
  providerDescriptions: {
    nvd: 'Registros de CVE e dados de aplicabilidade do National Vulnerability Database.',
    cisa_kev: 'Catálogo de Vulnerabilidades Conhecidas Exploradas da CISA.',
  },
  status: { idle: 'Não sincronizado', ready: 'Pronto', updating: 'Atualizando', error: 'Erro' },
  fields: { lastSync: 'Última sincronização', records: 'Registros processados', pages: 'Páginas processadas', lastPage: 'Última página concluída', elapsed: 'Tempo decorrido', elapsedValue: '{{value}} s', never: 'Nunca', localCache: 'Cache local' },
  actions: { synchronize: 'Sincronizar', cancel: 'Cancelar atualização', refreshStatus: 'Atualizar status' },
  messages: {
    loading: 'Carregando o status dos provedores', loadError: 'Não foi possível carregar o status dos provedores.',
    cacheUnavailable: 'O cache de vulnerabilidades está indisponível. O inventário e o monitoramento de segurança continuam disponíveis.',
    syncError: 'A atualização externa falhou. Os registros locais existentes foram preservados.',
    offline: 'Após uma sincronização bem-sucedida, o repositório local permanece disponível offline.',
    noApiKey: 'Nenhuma chave de API é armazenada na interface.',
  },
} as const
