export default {
  collectors: {
    ariaLabel: 'Integridade dos coletores', label: 'Coletores', observed_one: '{{formattedCount}} observado', observed_other: '{{formattedCount}} observados', restricted: '{{formattedCount}} com metadados restritos', summary: '{{label}}: {{state}}. {{coverage}}{{timing}}{{issue}}', timing: ' · {{duration}} ms', issue: '. Os detalhes da coleta estão temporariamente indisponíveis.', restrictedBadge: '{{formattedCount}} restritos', warning: 'Um ou mais coletores relataram uma falha factual de coleta', live: 'Ao vivo', paused: 'Pausado', refresh: 'Atualizar toda a telemetria',
    names: { system: 'Sistema', processes: 'Processos', network: 'Rede', services: 'Serviços' },
    states: { healthy: 'Saudável', degraded: 'Degradado', failed: 'Com falha', paused: 'Pausado', loading: 'Carregando' },
  },
  toolbar: { filtersAria: 'Filtros da tabela' },
  table: { unavailable: 'Indisponível' },
} as const
