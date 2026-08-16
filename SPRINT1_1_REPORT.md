# EDY Sentinel — Relatório da Sprint 1.1

Data de validação: 16/08/2026

Branch: `main`

Commit inicial: `ef7809d30d839702bd0897a0779521ed5f311070`

Commit final: commit desta entrega, `fix: harden telemetry accuracy and collector semantics` (hash exato registrado no handoff final).

## Resultado

A Sprint 1.1 endurece a precisão e a semântica da telemetria já entregue, sem iniciar
funcionalidades da Sprint 2. CPU, saúde dos collectors, cobertura de dados, correlação
de conexões, rota primária, identidade de executáveis, eventos e retenção agora têm
contratos factuais explícitos e testados. Nenhum score, alerta inferido, reputação
externa, ação de resposta ou integração foi adicionado.

## CPU

- O valor principal de CPU de processo representa porcentagem da capacidade lógica
  total da máquina: agregado do sampler dividido pelo número de processadores lógicos.
- O valor fica limitado a 0–100% e o primeiro ciclo permanece `Calculating`, pois não
  existe delta factual no warm-up.
- O agregado original é preservado como `coreEquivalentCpuPercent` somente no detalhe,
  com o rótulo explícito “Core-equivalent CPU”.
- Na validação real do EDY Sentinel: 0,3% total e 3,8% core-equivalent em host com 12
  processadores lógicos, coerente com a fórmula.

## Collectors: saúde e cobertura

- Estados de execução: `healthy`, `degraded` e `failed`; `paused` é aplicado apenas na
  apresentação quando a coleta automática está pausada.
- Cada collector informa última tentativa, último sucesso, duração, observações,
  registros restritos e diagnóstico nativo quando existe falha.
- Acesso restrito a metadata não degrada um collector que concluiu a enumeração. No
  smoke real, Processes permaneceu `Healthy` com aproximadamente 262 observações e
  116 restrições, exibidas separadamente como cobertura.
- Respostas servidas dentro da cadência preservam os timestamps reais da última
  execução do collector, sem fingir uma nova tentativa em cada hidratação da UI.

## Conexões e correlação de PID

- Identidade de processo continua PID + creation time; conexão usa protocolo, família,
  endpoints e PID.
- Cache bounded de processos recentes: 10 s. Janela de proteção contra reutilização de
  PID: 5 s.
- Estados explícitos: `associated`, `recently_exited`, `unresolved`, `system_kernel` e
  `not_applicable`. A UI não inventa um processo quando a identidade é ambígua.
- PID 0/4 pode ser identificado como System/Kernel quando aplicável.
- Fechamento exige ausência em dois snapshots de rede bem-sucedidos. Falha de uma
  família TCP/UDP conserva o last-good e não gera fechamento em massa.
- No aplicativo real foram observadas conexões associadas e conexões `Unresolved`,
  ambas apresentadas explicitamente.

## Network Overview

- A rota primária é obtida por `GetBestRoute2` no IP Helper, não pela ordem dos
  adaptadores.
- São mostrados interface, tipo, origem da classificação, endereço de origem, gateway,
  métrica e demais interfaces.
- Amostra real: `Ethernet`, `Physical Ethernet`, origem `192.168.10.56`, gateway
  `192.168.10.1`, métrica 25. O adaptador virtual Radmin não foi classificado como rota
  primária porque não era a melhor rota do Windows.
- A classificação usa o IfType nativo e fallback heurístico documentado apenas para
  distinguir VPN/virtual quando o tipo do Windows é insuficiente.

## Executáveis, assinatura e privacidade

- `company`: CompanyName do version resource; não é prova criptográfica.
- `signatureStatus`: resultado local do WinVerifyTrust.
- `signer`: subject extraído do certificado via CryptoAPI quando disponível.
- Cache é invalidado por path, mtime e tamanho; no máximo 24 novos executáveis são
  enriquecidos por ciclo.
- Smoke real do `explorer.exe`: Company `Microsoft Corporation`, assinatura `signed`,
  signer `Microsoft Windows`.
- Command line permanece live-only, nunca é persistida ou registrada. Hash contínuo,
  VirusTotal e qualquer reputação externa continuam fora do escopo.

## Eventos, tracking e persistência

- Evento v1: ID estável, tipo, entidade/tipo/chave, timestamp, collector, payload
  factual, versão do schema e mensagem.
- O baseline inicial não cria eventos sintéticos; timestamps são monotônicos; uma
  identidade duplicada no mesmo ciclo incrementa somente uma vez.
- Migration `0003_telemetry_accuracy.sql` adiciona company/signature/signer,
  associação de conexão/processLastSeen e metadados do evento sem apagar dados.
- SQLite real: migrations 1/2/3, `PRAGMA integrity_check = ok` e nenhuma coluna de
  command line nas tabelas persistidas.
- Writes live permanecem limitados a cerca de 60 s ou antecipados por eventos;
  snapshots completos a 5 min; cleanup a cada 6 h; sete dias para observações inativas
  e snapshots, 30 dias para eventos; registros ativos não são removidos.

## Diff funcional da Sprint 1 para a Sprint 1.1

| Área | Sprint 1 | Sprint 1.1 |
| --- | --- | --- |
| CPU | agregado multi-core exposto | total-capacity + detalhe core-equivalent |
| Health | restrições podiam sugerir partial | execução separada de cobertura |
| Conexão | correlação pelo snapshot atual | recent cache + PID-reuse guard + estados |
| Fechamento | ausência imediata | debounce de dois snapshots válidos |
| Network | interface/gateway enumerados | melhor rota nativa, tipo e métrica |
| Executável | CompanyName tratado como publisher | company, trust e signer separados |
| Evento | campos factuais básicos | schema v1 estruturado e deduplicado |
| SQLite | schema v2 | schema v3 append-only |

## Performance

Release real observado durante 15,06 s:

- CPU: 0,688 s acumulados, média 4,56% de um núcleo; aproximadamente 0,38% da
  capacidade total em 12 processadores lógicos.
- Working set: 43,00–46,16 MB.
- Private bytes: 20,18–23,84 MB.
- Threads: 23–24.
- Crescimento SQLite durante a janela: 49.152 bytes, compatível com eventos/heartbeat
  e sem persistência por hidratação de UI.

Baseline da Sprint 1: 3,63% de um núcleo, 43,92–45,92 MB working set e
22,15–23,77 MB private bytes. A variação de CPU foi +0,93 ponto percentual de um
núcleo; pico de working set +0,24 MB e private bytes +0,07 MB, sem regressão material,
travamento ou crescimento descontrolado observado.

## Testes e gates

| Gate | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| testes Rust | PASS, 22/22; 1 smoke manual ignorado por padrão |
| smoke nativo manual | PASS, 1/1 executado separadamente |
| Clippy all-targets/all-features `-D warnings` | PASS |
| `pnpm lint` | PASS, sem warnings |
| `pnpm typecheck` | PASS |
| `pnpm test` | PASS, 9/9 |
| `pnpm build` | PASS, 1.813 módulos; JS 247,39 kB (76,57 kB gzip); CSS 30,71 kB (6,57 kB gzip) |
| `pnpm audit --audit-level high` | PASS, nenhuma vulnerabilidade conhecida |
| `cargo audit` | PASS, exit 0; nenhuma vulnerabilidade bloqueante; 17 warnings permitidos |
| build Tauri release | PASS: EXE, MSI e NSIS |
| SQLite | PASS: migrations 1/2/3 e `integrity_check=ok` |
| responsividade | PASS, 20 combinações sem overflow horizontal |

Os 17 warnings do `cargo audit` são de manutenção em dependências GTK/unic/proc-macro
do lockfile e do advisory `glib`. `cargo tree --target x86_64-pc-windows-msvc -i glib`
retornou `nothing to print`, confirmando que `glib` não pertence ao grafo Windows.
O linker MSVC emitiu somente a mensagem localizada conhecida ao criar `.lib/.exp`,
sem warning de código ou falha de link.

## Smoke real do aplicativo Tauri

- Binário: `D:\CodexBuild\EDY-Sentinel-sprint1-1\target\release\edy-sentinel.exe`.
- MSI: `D:\CodexBuild\EDY-Sentinel-sprint1-1\target\release\bundle\msi\EDY Sentinel_0.1.0_x64_en-US.msi`.
- NSIS: `D:\CodexBuild\EDY-Sentinel-sprint1-1\target\release\bundle\nsis\EDY Sentinel_0.1.0_x64-setup.exe`.
- Overview mostrou collector health/cobertura e rota primária factual.
- Processos mostraram CPU normalizada; drawer validou Company/Signature/Signer.
- Active Connections mostrou associação explícita, inclusive `Unresolved`.
- Windows Services mostrou 280 serviços; `Schedule` foi conferido contra SCM nativo
  (Running, Automatic, PID 1580, LocalSystem e binário `svchost.exe -k netsvcs -p`).
- Pause/Resume e ações reais da command palette funcionaram.
- Sentinel Blue, Cyber Green, Terminal e Spectrum foram exercitados.
- 1920×1080, 1600×900, 1440×900, 1366×768 e 1280×720 ficaram sem overflow horizontal
  em Overview, Processes, Connections e Services: 20 verificações.

## Limitações honestas

- Processos protegidos podem omitir path, command line, usuário, arquitetura, company,
  assinatura ou signer; o registro continua factual e a cobertura informa restrição.
- CPU precisa de um ciclo de warm-up.
- `recently_exited` depende da janela bounded de 10 s e pode não aparecer em toda
  sessão; ausência desta classe na amostra não foi simulada.
- UDP não possui endpoint remoto/estado TCP nas owner-PID tables.
- Assinatura válida não é reputação nem garantia de segurança.
- Hash sob demanda não foi exposto; hashing contínuo permanece deliberadamente ausente.
- Nenhuma funcionalidade da Sprint 2 foi iniciada.

## Evidências visuais

As seis capturas são 1920×1080, geradas diretamente do WebView2 do release Tauri real
e mantidas fora do Git no diretório de outputs do handoff:

1. `01-overview-health-primary-route-1920x1080.png`
2. `02-process-activity-normalized-cpu-1920x1080.png`
3. `03-process-detail-company-signature-signer-1920x1080.png`
4. `04-active-connections-association-1920x1080.png`
5. `05-network-primary-route-terminal-1920x1080.png`
6. `06-windows-services-1920x1080.png`

As imagens contêm metadata real do host local (hostname, usuário, IPs, PIDs e paths) e
devem ser tratadas como evidência privada.

## Git

- Nenhuma captura, banco, log, dump, `.env`, output de build ou segredo é incluído.
- Um único commit final é criado com a mensagem solicitada.
- Nenhum push é executado.
