export default {
  status: {
    not_initialized: { label: 'Não inicializada', detail: 'Nenhum comportamento foi aprendido ainda.' },
    learning: { label: 'Aprendendo', detail: 'Observações reais estão sendo adicionadas sem gerar eventos de comportamento novo.' },
    ready: { label: 'Pronta', detail: 'Novas diferenças factuais podem criar eventos de segurança deduplicados.' },
    stale: { label: 'Desatualizada', detail: 'A baseline não recebeu uma observação bem-sucedida recentemente.' },
    error: { label: 'Erro', detail: 'Não foi possível atualizar a baseline. A telemetria existente continua disponível.' },
  },
  panel: {
    title: 'Baseline comportamental', subtitle: 'Aprendizado factual local · sem classificação de ameaça', observationPeriod: 'Período de observação', observationCycles: 'Ciclos de observação', learnedFacts: 'Fatos aprendidos', processing: 'Processamento da baseline', version: 'Versão da baseline',
    executables_one: '{{formattedCount}} executável', executables_other: '{{formattedCount}} executáveis', relationships_one: '{{formattedCount}} relação', relationships_other: '{{formattedCount}} relações', destinations_one: '{{formattedCount}} destino', destinations_other: '{{formattedCount}} destinos', services_one: '{{formattedCount}} serviço', services_other: '{{formattedCount}} serviços',
    timelineAria: 'Linha do tempo da baseline', started: 'Iniciada', completed: 'Concluída', learningInProgress: 'Aprendizado em andamento',
    start: 'Iniciar baseline', complete: 'Concluir aprendizado', startNew: 'Iniciar nova baseline', reset: 'Redefinir baseline', securityEvents: 'Eventos de segurança', localPersistenceReady: 'Persistência local pronta', persistenceUnavailable: 'Persistência indisponível',
  },
  dialog: {
    title: { reset: 'Redefinir baseline comportamental', complete: 'Concluir aprendizado da baseline', start: 'Iniciar nova baseline comportamental' },
    warning: {
      reset: 'A redefinição inicia um novo período de aprendizado. As versões anteriores da baseline e o restante do histórico do Sentinel são preservados.',
      complete: 'A conclusão manual destina-se à validação controlada de desenvolvimento. Somente observações reais já coletadas são usadas.',
      start: 'A baseline atual é preservada como histórico. A nova versão aprende com a telemetria local real e não produz eventos de comportamento novo enquanto estiver aprendendo.',
    },
    learningPeriod: 'Período de aprendizado', confirmation: 'Digite <code>{{phrase}}</code> para confirmar', confirmationAria: 'Frase de confirmação da baseline', cancel: 'Cancelar', working: 'Processando…', actionError: 'Não foi possível concluir a ação da baseline. Revise o estado atual e tente novamente.',
    periods: { minute: '1 minuto · teste controlado de desenvolvimento', hour: '1 hora', hours6: '6 horas', hours24: '24 horas · padrão recomendado', days3: '3 dias', days7: '7 dias' },
  },
} as const
