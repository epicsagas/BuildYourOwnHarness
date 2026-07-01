> Este documento é a versão em português de [README.md](../../../README.md). A versão em inglês é a fonte oficial.

<div align="center">

[English](../../../README.md) | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | **Português** | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Seu agente de IA, feito para você

*Não um template genérico — um harness compilado de acordo com seu papel, expertise e objetivos.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

A maioria das ferramentas de IA entrega um conjunto fixo de funções e diz "se vira". O BYOH faz o contrário: ele entrevista você, aprende como você realmente trabalha e gera um harness de agente personalizado — habilidades, memória, pipelines — que já funciona para o seu fluxo de trabalho desde o início.

## Para quem é?

- **Desenvolvedores** que querem um agente que já conhece sua stack, estilo de testes e cadência de entrega
- **Pesquisadores** que precisam de revisão bibliográfica, rastreamento de citações e síntese conectados numa só ferramenta
- **Criadores** que querem um parceiro de escrita alinhado com sua voz e estrutura de projeto
- **Analistas de negócio** que precisam de frameworks de decisão e pipelines de relatórios — não de um chat genérico

Se você já pensou "queria que minha IA entendesse meu contexto de verdade" — é exatamente isso que o BYOH faz.

## Como funciona em 60 segundos

O BYOH foi feito para ser conduzido pelo seu agente de IA — não por você digitando comandos. Instale o plugin e simplesmente converse. A conversa *é* a entrevista, o wizard e o build.

```
1. Instale o plugin        # Claude Code / Codex / agy — auto-instala o binário
2. "Build me a harness"    # seu agente escaneia seu repo e compila o resultado
```

Na próxima sessão, seu host carrega o harness automaticamente — agentes, habilidades, memória e pipelines ajustados para você.

## Instale o plugin (recomendado)

Usa **Claude Code, Codex ou agy**? Instale o plugin. Ele agrupa o servidor MCP e **auto-instala o binário no primeiro carregamento** — sem Rust toolchain, sem configuração manual:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

### Usa outro host compatível com MCP?

O BYOH fala MCP, então Cursor, Zed, Continue e afins também funcionam. Instale o [binário](#installation) uma vez e aponte seu host para o servidor:

```bash
byoh serve   # servidor MCP stdio
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Nota:** O repositório é atualmente privado. Use os caminhos acima. Quando for público, aparecerá no marketplace compartilhado `epicsagas/plugins`.

## Modo conduzido pelo agente — o caminho principal

Depois que seu host está conectado, você não digita comandos — você apenas conversa. Seu agente chama diretamente as ferramentas MCP do BYOH, e a conversa *é* a entrevista, o build e o ciclo de evolução:

> **Você:** *Sou um desenvolvedor backend em Go entregando uma API de pagamentos este mês. Build me a harness.*
>
> **Agente:** *(escaneia seu repo via `profile_scan`, faz algumas perguntas direcionadas via `profile_interview`, trava o gênero em `developer`)* → compila um `HarnessBundle` → instala agentes, habilidades, memória e um pipeline de entrega segura no Claude Code. Pronto — na próxima sessão, seu agente já fala a sua stack.

Esse mesmo fluxo, na ordem sugerida de ferramentas:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → rag_index / rag_search → compile → compile_dry_run
           → (opcional) registry_clone_skill → (depois) evolve_cycle
```

Ferramentas disponíveis: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `compile_dry_run`, `evolve_cycle`, `genre_list`, `rag_index`, `rag_search`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`, e mais.

Quer que o agente te conduza por tudo isso? Basta dizer *"build my harness"* — o agente `byoh-guide`, que vem embutido, orquestra o fluxo inteiro.

## Catálogo de plugins

O catálogo oferece uma lista curada dos 100 melhores plugins Claude (ordenados por estrelas, atualizada diariamente) para você descobrir e adicionar habilidades da comunidade sem sair da conversa.

```bash
# Indexação única — baixa um bundle pré-construído em segundos
byoh catalog index

# Busca offline — sem rede necessária após a indexação
byoh catalog search "memory" --genre developer --limit 5

# Adicionar um plugin ao seu harness
# licença, keywords e genre são detectados automaticamente do repositório clonado
byoh catalog vendor obra/superpowers --genre developer
```

O agente LLM (via ferramentas MCP `catalog_search` / `catalog_vendor`) pode fazer esse fluxo inteiro de forma autônoma — *"adiciona um plugin de memória ao meu harness"* — ou você pode conduzir diretamente pela CLI.

## Power users: a CLI (opcional)

Todos os fluxos acima também são acessíveis pelo terminal. A CLI é **auxiliar** — útil para scripts, CI ou quando você prefere não conversar — mas o caminho conduzido pelo agente é o caminho pretendido.

### Seu primeiro harness — pela CLI

```bash
byoh profile init me --paths ./src ./docs   # análise automática do projeto
byoh profile interview me                   # conversa de ~5 min
byoh profile confirm me --genre developer   # confirmar seu gênero

byoh compile me                             # gera o HarnessBundle (validado e controlado)
byoh render me --target claude              # ou: codex | agy | all
byoh install me                             # instalação segura em dist/

byoh run me                                 # iniciar com o harness ativo
byoh evolve me                              # melhorar o harness com base no feedback das sessões
```

O BYOH pergunta sobre seu papel, nível de expertise, ferramentas e objetivo de 30 dias. A entrevista se adapta — um pesquisador recebe perguntas diferentes de um desenvolvedor. `evolve` executa um ciclo com 3 gates (Critic / Seesaw / Stagnation) que nunca podem ser ignorados — a evolução é segura e auditável.

## Como funciona por dentro

O motor de síntese do BYOH combina suas tags de perfil com o registro de habilidades, as ordena em um pipeline com dependências resolvidas e emite um `HarnessBundle` — um artefato pronto para git que renderiza no formato nativo de cada host suportado.

- **Modelo de segurança com 4 anéis** — habilidades embutidas (anel 1) até habilidades da comunidade/não confiáveis (anel 4), com validação crescente em cada nível
- **Evolução com 3 gates** — cada ciclo `evolve` passa pelos gates Critic (qualidade), Seesaw (regressão) e Stagnation (estagnação); sem possibilidade de bypass
- **Pipelines por objetivo** — declarar um objetivo de 30 dias (lançamento de produto, relatório de pesquisa, entrega segura…) sobrepõe automaticamente uma escada de habilidades correspondente

Arquitetura hexagonal: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Veja `AGENTS.md` para o guia completo.

## Referência completa da CLI

```bash
# Perfil
byoh profile init <slug> [--paths ...]      # scan não-destrutivo do projeto
byoh profile interview <slug>               # entrevista guiada
byoh profile confirm <slug> --genre <g>     # confirmar e bloquear o perfil

# Build
byoh compile <slug> [--dry-run]             # validar + gerar HarnessBundle
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # deploy para dist/ ou diretório do plugin

# Executar e evoluir
byoh run <slug>
byoh evolve <slug>

# Habilidades da comunidade
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catálogo
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<consulta>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Installation

Necessário apenas se você **não** estiver usando o plugin (que auto-instala o binário) ou se quiser o BYOH em um host MCP sem plugin.

### Binário (sem necessidade de Rust toolchain)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**A partir do código-fonte:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verificar instalação
```

## Build e desenvolvimento

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # testes unitários + e2e
cvp                               # paralelo: check → clippy → test → fmt → build
```

A feature `mcp` (servidor MCP stdio) vem ativada por padrão. O BYOH não inclui nenhum acervo de conhecimento embutido — para recuperação, aponte seu harness gerado para um servidor de documentos como o [alcove](https://github.com/epicsagas/alcove).

## Licença

Apache-2.0.
