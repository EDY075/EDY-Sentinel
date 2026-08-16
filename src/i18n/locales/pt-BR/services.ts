export default {
  filters: { all: 'Todos', running: 'Em execução', stopped: 'Parados', automatic: 'Automáticos', manual: 'Manuais', disabled: 'Desabilitados' },
  search: 'Buscar por nome do serviço, nome de exibição, caminho do binário ou conta',
  ofTotal: 'de {{total}}',
  columns: { service: 'Serviço', status: 'Status', startup: 'Inicialização', pid: 'PID', account: 'Conta' },
  binaryPathUnavailable: 'Caminho do binário indisponível',
  empty: { title: 'Nenhum serviço corresponde a esta visualização', description: 'Altere a busca ou o filtro de estado do serviço.' },
  ariaLabel: 'Serviços do Windows',
  unavailable: 'Indisponível',
  status: { stopped: 'Parado', start_pending: 'Início pendente', stop_pending: 'Parada pendente', running: 'Em execução', continue_pending: 'Continuação pendente', pause_pending: 'Pausa pendente', paused: 'Pausado', unknown: 'Desconhecido' },
  startupType: { boot: 'Inicialização do sistema', system: 'Sistema', automatic_delayed: 'Automática (atrasada)', automatic: 'Automática', manual: 'Manual', disabled: 'Desabilitada', unknown: 'Desconhecida' },
} as const
