export default {
  header: { back: 'Voltar às detecções', title: 'Regras de detecção', subtitle: 'Regras locais versionadas · as condições são somente leitura', count_one: '{{formattedCount}} regra', count_other: '{{formattedCount}} regras' },
  table: { rule: 'Regra', category: 'Categoria', version: 'Versão', defaultSeverity: 'Severidade padrão', evidence: 'Evidências', enabled: 'Habilitada', required_one: '{{formattedCount}} obrigatória', required_other: '{{formattedCount}} obrigatórias' },
  state: { enabled: 'Habilitada', disabled: 'Desabilitada', enableAria: 'Habilitar {{name}}', disableAria: 'Desabilitar {{name}}' },
  error: { load: 'Não foi possível carregar as regras. Tente novamente.', retry: 'Tentar novamente' },
  empty: { title: 'Nenhuma regra de detecção registrada', description: 'O registro Rust não retornou nenhuma definição de regra operacional.' },
  categories: { process_execution: 'Execução de processos', network_activity: 'Atividade de rede', service_persistence: 'Persistência de serviços', network_configuration: 'Configuração de rede' },
  definitions: {
    'EDY-PROC-001': { name: 'Novo executável não assinado em caminho temporário gravável pelo usuário', description: 'Um novo executável foi realmente iniciado a partir de um caminho temporário gravável pelo usuário e o Windows o identificou como não assinado.' },
    'EDY-PROC-002': { name: 'Nova execução filha não assinada com pai conhecido e assinado', description: 'Um processo pai assinado e conhecido pela baseline iniciou um novo filho não assinado em um caminho temporário gravável pelo usuário, por uma relação até então não observada.' },
    'EDY-NET-001': { name: 'Nova atividade de saída de um novo executável temporário não assinado', description: 'Um binário não assinado recém-executado em caminho temporário gravável pelo usuário iniciou atividade de saída vista pela primeira vez, com associação inequívoca ao processo.' },
    'EDY-SVC-001': { name: 'Novo serviço automático privilegiado em caminho gravável pelo usuário', description: 'Um serviço em execução recém-observado usa inicialização automática, uma conta privilegiada do sistema e um binário em local gravável pelo usuário.' },
    'EDY-SVC-002': { name: 'Reconfiguração persistente correlacionada de serviço', description: 'Um serviço conhecido alterou o caminho do binário junto com o tipo de inicialização ou a conta na mesma janela de coleta.' },
    'EDY-NET-002': { name: 'Alteração persistente e coordenada da configuração de rede', description: 'O gateway padrão e o conjunto de resolvedores DNS mudaram em conjunto e permaneceram diferentes da baseline por pelo menos duas observações.' },
  },
} as const
