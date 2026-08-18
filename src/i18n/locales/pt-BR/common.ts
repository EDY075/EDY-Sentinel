export const common = {
  appName: 'EDY Sentinel',
  actions: {
    cancel: 'Cancelar',
    close: 'Fechar',
    refresh: 'Atualizar',
    retry: 'Tentar novamente',
    save: 'Salvar',
  },
  states: {
    disabled: 'Desativado',
    enabled: 'Ativado',
    loading: 'Carregando',
    unavailable: 'Indisponível',
  },
  status: {
    error: 'Erro',
    offline: 'Fora de linha',
    online: 'Em operação',
    partial: 'Parcialmente disponível',
    success: 'Sucesso',
  },
  accessibility: {
    closeDialog: 'Fechar caixa de diálogo',
    closeDrawer: 'Fechar painel lateral',
    metricTrend: 'Tendência da métrica',
  },
  observations: {
    processes_one: '{{count}} processo observado',
    processes_other: '{{count}} processos observados',
  },
  inspector: { label: 'Inspetor do Endpoint' },
} as const
