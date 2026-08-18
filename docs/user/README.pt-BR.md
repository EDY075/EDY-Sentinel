<p align="center">
  <img src="../assets/edy-sentinel-banner.png" alt="EDY Sentinel — Inteligência de Endpoint Windows" width="100%">
</p>

<p align="center">
  <a href="../../README.md">English</a> ·
  <a href="README.pt-BR.md">Português (Brasil)</a>
</p>

# EDY Sentinel

EDY Sentinel é um aplicativo local-first de inteligência de endpoint para Windows. Ele reúne
telemetria real, baseline comportamental, detecções explicáveis, inteligência de vulnerabilidades
e uma Pontuação de Segurança auditável em um único espaço de trabalho desktop.

<p align="center">
  <a href="https://github.com/EDY075/EDY-Sentinel/releases/latest"><strong>Baixar EDY Sentinel para Windows →</strong></a>
</p>

> **Recomendado:** [EDY-Sentinel-1.0.0-Setup-x64.exe](https://github.com/EDY075/EDY-Sentinel/releases/download/v1.0.0/EDY-Sentinel-1.0.0-Setup-x64.exe).
> Os binários v1.0.0 atuais são um **UNSIGNED BUILD**; o Windows pode exibir um aviso de editor
> desconhecido. Verifique os hashes SHA-256 publicados antes de executar o download.

## Instalação rápida

1. Baixe o [Setup EXE recomendado](https://github.com/EDY075/EDY-Sentinel/releases/download/v1.0.0/EDY-Sentinel-1.0.0-Setup-x64.exe).
2. Execute o instalador e siga as instruções do Windows.
3. Abra o **EDY Sentinel** pelo menu Iniciar.

| Pacote | Indicação |
|---|---|
| [Setup EXE](https://github.com/EDY075/EDY-Sentinel/releases/download/v1.0.0/EDY-Sentinel-1.0.0-Setup-x64.exe) | Instalação interativa recomendada |
| [MSI](https://github.com/EDY075/EDY-Sentinel/releases/download/v1.0.0/EDY-Sentinel-1.0.0-x64.msi) | Instalação e gerenciamento Windows |
| [EXE standalone](https://github.com/EDY075/EDY-Sentinel/releases/download/v1.0.0/EDY-Sentinel-1.0.0-Standalone-x64.exe) | QA local e diagnóstico avançado; não é um pacote portátil |
| [SHA256SUMS.txt](https://github.com/EDY075/EDY-Sentinel/releases/download/v1.0.0/SHA256SUMS.txt) | Verificação de integridade |

Requer Windows 10/11 x64 e Microsoft Edge WebView2 Runtime. A internet é opcional para o
monitoramento local e usada somente na sincronização dos dados públicos NVD e CISA KEV.

## Veja o produto real

<p align="center">
  <img src="../assets/edy-sentinel-demo.gif" alt="Demonstração real do EDY Sentinel" width="960">
</p>

A demonstração usa capturas sanitizadas do aplicativo Tauri real. Os rótulos de redação indicam
onde dados privados do endpoint foram ocultados; a interface e os resultados visíveis não foram
fabricados.

## Principais recursos

- **Telemetria Windows** — hardware, armazenamento, serviços, rotas, adaptadores e DNS.
- **Processos e conexões** — processos atuais e endpoints TCP/UDP com atribuição factual.
- **Baseline comportamental** — aprendizado local versionado com estados explícitos.
- **Eventos de Segurança** — fatos do endpoint preservados separadamente das análises.
- **Detection Engine** — seis regras v1 conservadoras, com evidências, exclusões e histórico.
- **Inventário de software** — leitura nativa das áreas de Registro da máquina e do usuário.
- **Inteligência de vulnerabilidades** — identidade de produto e CVEs aplicáveis sem adivinhar ambiguidades.
- **NVD + CISA KEV** — contexto público aplicado somente após correspondência confirmada.
- **Pontuação de Segurança v2** — contribuições explicáveis de detecções e vulnerabilidades confirmadas.
- **Interface bilíngue e quatro temas** — pt-BR, English, Sentinel Blue, Cyber Green, Terminal e Spectrum.

## Telas

### Visão geral — Sentinel Blue

![Visão geral do EDY Sentinel](../assets/screenshots/overview-sentinel-blue.png)

### Inventário de software

![Inventário de software do EDY Sentinel](../assets/screenshots/software-inventory.png)

### Pontuação de Segurança v2

![Detalhes da Pontuação de Segurança v2](../assets/screenshots/security-score-v2.png)

### Telemetria de rede

![Telemetria de rede do EDY Sentinel](../assets/screenshots/network-telemetry.png)

### Endpoint Inspector

![Endpoint Inspector do EDY Sentinel](../assets/screenshots/endpoint-inspector.png)

### Tema Spectrum

![EDY Sentinel no tema Spectrum](../assets/screenshots/overview-spectrum.png)

## Como funciona

```text
Endpoint Windows
      ↓
Coletores nativos
      ↓
Baseline comportamental + Eventos de Segurança factuais
      ↓
Detection Engine + Inteligência de Vulnerabilidades
      ↓
Pontuação de Segurança v2 explicável
```

[Consultar a arquitetura completa →](../technical/architecture.md)

## Primeira execução

O EDY Sentinel começa a coletar telemetria local após a abertura. O baseline comportamental inicia
em **Aprendizado**, portanto a Pontuação de Segurança pode aparecer indisponível ou limitada até
que a cobertura necessária esteja pronta. A Inteligência de Vulnerabilidades usa o cache local e
pode sincronizar dados públicos NVD/CISA sob demanda. Esses estados iniciais são esperados.

## Segurança e privacidade

- Telemetria, inventário, baseline, eventos, detecções, pontuações e preferências ficam em bancos SQLite locais.
- Linhas de comando de processos são somente ao vivo e não são persistidas; conteúdo de arquivos não é coletado.
- A sincronização NVD/CISA baixa dados públicos por HTTPS e não envia o inventário do endpoint.
- A v1.0.0 não possui analytics, telemetria de nuvem própria, conta cloud ou chave de API obrigatória.
- O aplicativo roda como usuário atual e observa o sistema; não bloqueia processos nem altera políticas.

Consulte a [política de segurança](../../SECURITY.md), a
[assinatura de código](../security/code-signing.md) e o
[modelo de correspondência de vulnerabilidades](../technical/vulnerability-matching.md).

## Documentação

- [Guia do usuário](user-guide.md)
- [Índice da documentação](../README.md)
- [Arquitetura](../technical/architecture.md)
- [Regras de detecção](../technical/detection-rules.md)
- [Pontuação de Segurança](../technical/security-score.md)
- [Desenvolvimento](../development/setup.md)
- [Notas da v1.0.0](../development/releases/v1.0.0.md)

## Compilar do código-fonte

```powershell
git clone https://github.com/EDY075/EDY-Sentinel.git
cd EDY-Sentinel
pnpm install
pnpm tauri dev
```

Veja os pré-requisitos e gates no [guia de desenvolvimento](../development/setup.md).

## Limitações conhecidas

- Os pacotes v1.0.0 são unsigned e não possuem identidade autenticada de editor.
- Identidades de software ambíguas permanecem não resolvidas de forma intencional.
- Sem internet, o cache local continua disponível, mas NVD/CISA não pode ser atualizado.
- As seis regras v1 fornecem observabilidade explicável; não são classificação de malware ou EDR completo.
- A pontuação descreve a postura observada sob a cobertura atual; não é garantia de segurança.
- Remediação automática, bloqueio e serviços externos de reputação estão fora do escopo da v1.

**v1.0.0** é a versão pública atual para Windows.
