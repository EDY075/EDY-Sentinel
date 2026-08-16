# EDY Sentinel — Relatório da Sprint 1

Data de validação: 16/08/2026

Branch: `main`

Commit inicial: `540db4909f12c73e9756b219c13eb9aa7c81f2ac`

Commit final: commit desta entrega, `feat: add live process network and service telemetry` (hash registrado no handoff final).

## Resultado

A Sprint 1 transforma o EDY Sentinel em uma ferramenta operacional de observação do Windows. Processos, conexões TCP/UDP e serviços são coletados de fontes reais, correlacionados, rastreados entre snapshots e expostos em telas navegáveis sem atribuir reputação ou severidade fictícia. Overview, SQLite, quatro temas, command palette, responsividade e empacotamento Tauri foram preservados.

## Implementado

### Collectors e comandos Tauri

- `get_live_telemetry`: snapshot central tipado, executado fora da thread da UI.
- Processos: sampler `sysinfo` persistente, Toolhelp, APIs de processo/token, `IsWow64Process2`, version resources e `WinVerifyTrust` cache-only.
- Conexões: IP Helper API para TCP/UDP IPv4 e IPv6 com owner PID.
- Serviços: Service Control Manager para estado, configuração, conta, PID e delayed auto-start.
- Nenhum PowerShell, shell ou comando arbitrário no caminho de telemetria live.
- Falhas isoladas produzem collector `Partial` e preservam o último snapshot válido.

### UI

- Activity / Processes: tabela virtualizada, busca, ordenação, filtros, refresh, live/pause, teclado e drawer factual.
- Network / Active Connections: tabela virtualizada, filtros TCP/UDP, IPv4/IPv6 e estados, correlação com processo e detalhes/timeline.
- System / Windows Services: observação read-only, busca e filtros por estado/startup.
- Store central no `TelemetryProvider`, sem loops independentes por tela.
- Command palette com navegação, refresh e pause/resume reais.
- Saúde dos collectors com motivo técnico acessível.
- Quatro temas validados: Sentinel Blue, Cyber Green, Terminal e Spectrum.

### Tracking, diff e SQLite

- Identidade de processo: PID + creation time.
- Identidade de conexão: protocolo + família + tupla de endpoints + PID; UDP usa binding local + PID.
- Identidade de serviço: nome SCM.
- Baseline inicial não gera eventos sintéticos.
- Eventos factuais: processos iniciados/encerrados, conexões abertas/fechadas e mudanças reais de serviços.
- Falha parcial de uma família de rede não é convertida em fechamento em massa.
- Migration `0002_live_telemetry.sql` adiciona observações normalizadas e `telemetry_events` sem destruir o banco existente.
- Escrita de telemetria limitada a aproximadamente 60 segundos ou antecipada por eventos; snapshots do Overview a cada 5 minutos.
- Limpeza a cada 6 horas; retenção inicial de 7 dias para observações/snapshots inativos e 30 dias para eventos.
- Command line permanece live-only e não é persistida.

## Dados reais do Windows

### Processos

Nome, PID, PPID, usuário quando acessível, caminho, command line live-only, CPU por delta real após warm-up, memória, início, threads, arquitetura, descrição, CompanyName do version resource, resultado da validação de assinatura, acesso, first/last seen, observações e estado ativo.

### Conexões

TCP/UDP IPv4/IPv6, endereço e porta local, endpoint remoto quando aplicável, estado TCP, PID, processo/path correlacionado, first/last seen, observações e estado ativo.

### Serviços

Service name, display name, estado SCM, startup type, binary path, conta, PID quando em execução, first/last seen, observações e estado ativo.

## Política de atualização e performance

- Processos/UI hydrate: aproximadamente 2,5 s.
- Conexões: aproximadamente 4 s.
- Serviços: aproximadamente 15 s.
- Overview/system: aproximadamente 15 s na UI; persistência integral desacoplada e limitada.
- Coletas bloqueantes usam `spawn_blocking`; chamadas sobrepostas são evitadas.
- Metadados relativamente estáticos são cacheados por identidade do executável e mtime.
- Verificação de assinatura é cache-only e novos executáveis são enriquecidos com budget de 24 por ciclo.
- Tabelas usam virtualização para limitar os elementos renderizados.
- Em amostra ativa de 15 s do release real: 3,63% de um núcleo, working set de 43,92–45,92 MB, private bytes de 22,15–23,77 MB e 24 threads, sem travamento.
- SQLite observado com 1.474.560 bytes, integrity check `ok`, migrations 1 e 2 aplicadas e crescimento controlado pelo throttle/retention.

## Verificações

| Gate | Resultado |
| --- | --- |
| `pnpm lint` | PASS, sem warnings |
| `pnpm typecheck` | PASS |
| `pnpm test` | PASS, 5/5 |
| `pnpm build` | PASS, 1.812 módulos; JS 245,57 kB (75,93 kB gzip); CSS 30,59 kB (6,55 kB gzip) |
| `cargo fmt --check` | PASS |
| testes Rust | PASS, 12 testes; 1 smoke manual ignorado por padrão e executado separadamente com sucesso |
| Clippy all-targets/all-features `-D warnings` | PASS |
| `pnpm audit` | PASS, nenhuma vulnerabilidade conhecida |
| `cargo audit` | PASS, exit 0; nenhuma vulnerabilidade bloqueante; 17 warnings permitidos documentados |
| build Tauri release | PASS: EXE, MSI e NSIS |
| SQLite | PASS: migrations 1/2 e `integrity_check=ok` |
| scan de shell/segredos | PASS |
| `git diff --check` | PASS |

O `cargo audit` reportou warnings de manutenção em dependências GTK/unic/proc-macro do lockfile e um advisory `glib` marcado como unsound. `cargo tree --target x86_64-pc-windows-msvc -i glib` confirmou que `glib` não pertence ao grafo Windows desta entrega. O linker MSVC emitiu apenas a mensagem localizada conhecida ao criar `.lib/.exp`, sem falha de link.

## Smoke test do aplicativo real

- Release aberto em `D:\CodexBuild\EDY-Sentinel-sprint1\target\release\edy-sentinel.exe`.
- Overview carregou telemetria e SQLite schema v2.
- Processos reais apareceram; o processo do próprio Sentinel mostrou PID, path, CPU, memória e threads coerentes com o Windows; CPU/memória mudaram entre amostras; drawer abriu.
- Conexões reais TCP/UDP apareceram; uma conexão selecionada mostrou PID/processo, endpoints, estado e tracking factual.
- Serviços exibiram exatamente 280 registros, igual à contagem nativa do Windows na amostra.
- Pause manteve o snapshot navegável e Resume retomou as atualizações.
- Command palette executou navegação, refresh, pause/resume e troca de tema.
- Os cinco viewports solicitados ficaram sem overflow horizontal: 1920×1080, 1600×900, 1440×900, 1366×768 e 1280×720.

## Artefatos release

- EXE: `D:\CodexBuild\EDY-Sentinel-sprint1\target\release\edy-sentinel.exe`
- MSI: `D:\CodexBuild\EDY-Sentinel-sprint1\target\release\bundle\msi\EDY Sentinel_0.1.0_x64_en-US.msi`
- NSIS: `D:\CodexBuild\EDY-Sentinel-sprint1\target\release\bundle\nsis\EDY Sentinel_0.1.0_x64-setup.exe`

## Limitações honestas

- Processos protegidos podem omitir usuário, path, command line, arquitetura ou outros detalhes; continuam listados como restricted/unavailable.
- O primeiro valor de CPU de cada processo é `Unavailable` durante o warm-up necessário para obter delta real.
- `CompanyName` é metadata de versão do executável, não identidade criptográfica do signer.
- O status de assinatura é verificado, mas o subject do certificado não é extraído nesta Sprint.
- Hash sob demanda não foi exposto nesta Sprint; hashing contínuo permanece deliberadamente ausente.
- UDP não possui remote endpoint nem TCP state nas tabelas owner-PID do Windows.
- Ausência temporária de um serviço no SCM não é interpretada automaticamente como `Stopped`.
- Processos/conexões encerrados permanecem no snapshot live por um ciclo; o histórico completo fica no SQLite conforme retenção.
- Collectors podem ficar `Partial` por acesso negado sem comprometer os dados disponíveis.

## Evidências visuais

As sete capturas têm 1920×1080 e foram feitas diretamente do WebView2 do release Tauri real. Permanecem fora do repositório, no diretório de outputs do handoff:

1. `01-overview-sentinel-blue-1920x1080.png`
2. `02-activity-processes-1920x1080.png`
3. `03-process-detail-drawer-1920x1080.png`
4. `04-network-active-connections-1920x1080.png`
5. `05-connection-detail-1920x1080.png`
6. `06-services-sentinel-blue-1920x1080.png`
7. `07-services-terminal-theme-1920x1080.png`

## Recomendação para a Sprint 2

Construir o primeiro Detection Engine explicável sobre os eventos factuais já persistidos: regras locais versionadas, severidade somente quando sustentada por evidência, timeline de eventos, triagem e Security Score derivado de sinais reais. Integrações externas, ações destrutivas e automação de resposta devem continuar fora de escopo até existir modelo de confiança, auditoria e consentimento explícito.

Nenhuma funcionalidade da Sprint 2 foi implementada nesta entrega.
