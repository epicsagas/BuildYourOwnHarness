> Este documento é a versão em português de [README.md](../../../README.md). A versão em inglês é a fonte oficial.

<div align="center">

[English](../../../README.md) | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | **Português** | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Seu agente de IA, construído em torno de você

*Não é um template genérico — é um harness compilado a partir do seu papel, especialidade e objetivos.*

<img src="assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

A maioria das configurações de IA entrega um conjunto fixo de ferramentas e diz "boa sorte". O BYOH inverte isso: ele entrevista você, aprende o que você realmente faz e gera um harness de agente personalizado — skills, agentes, pipelines de objetivos — que se encaixa no seu fluxo de trabalho desde o início.

## Para quem é isto?

- **Desenvolvedores** que querem um agente que já conheça sua stack, estilo de teste e cadência de entrega
- **Pesquisadores** que precisam de revisão de literatura, rastreamento de citações e síntese integrados
- **Criadores** que querem um parceiro de escrita que combine com sua voz e estrutura de projeto
- **Analistas de negócios** que precisam de frameworks de decisão e pipelines de relatórios, não um chat genérico

Se você já pensou "quem dera a minha IA realmente conhecesse o meu contexto" — é isso que o BYOH faz.

## Como funciona em 60 segundos

O BYOH foi projetado para ser conduzido pelo seu agente de IA — não por você digitando comandos. Instale o plugin e apenas converse. A conversa *é* a entrevista, o wizard e o build.

```
1. Instale o plugin        # Claude Code / Codex / agy — auto-instala o binário
2. "Construa um harness"   # seu agente te entrevista, constrói, preenche as lacunas
                           # ele mesmo e instala — tudo na conversa
```

Na próxima sessão, seu host carrega o harness automaticamente — agentes, skills e pipelines de objetivos ajustados para você.

## Instalar o plugin (recomendado)

Usando **Claude Code, Codex ou agy**? Instale o plugin. Ele empacota o servidor MCP e **auto-instala o binário no primeiro carregamento** — sem toolchain Rust, sem configuração manual:

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

### Usando qualquer outro host compatível com MCP?

O BYOH fala MCP, então Cursor, Zed, Continue e afins também funcionam. Instale o [binário](#installation) uma vez e aponte seu host para o servidor:

```bash
byoh serve   # servidor MCP stdio
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Nota:** O repo é atualmente privado. Use os caminhos acima. Quando se tornar público, ele aparecerá no marketplace compartilhado `epicsagas/plugins`.

## Modo conduzido por agente — o caminho principal

Depois que seu host está conectado, você não digita comandos — apenas conversa. Seu agente chama as ferramentas MCP do BYOH diretamente, e a conversa *é* a entrevista, o build e o ciclo de evolve:

> **Você:** *Sou um desenvolvedor Go de backend entregando uma API de pagamentos este mês. Construa um harness para mim.*
>
> **Agente:** *(examina seu repo via `profile_scan`, faz algumas perguntas direcionadas via `profile_interview`, trava o gênero como `developer`)* → `build` sintetiza um `HarnessBundle` e classifica cada skill como `matched` / `authored` / `skeleton` → para qualquer skeleton que o perfil precise (digamos, uma skill de verificação específica de pagamentos), o agente a escreve na hora via `author_skill`, e roda `build` de novo para confirmar → instala agentes, skills e um pipeline de objetivos de secure-ship no Claude Code. O conteúdo autorizado persiste entre reconstruções. Pronto — na próxima sessão, seu agente já fala a sua stack.

O mesmo fluxo, na ordem sugerida de ferramentas:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → (author_skill / author_doc to fill skeletons) → build → install_plugin
```

Ferramentas disponíveis: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

Quer que o agente te guie pelo processo? Basta dizer *"construa meu harness"* — o agente `byoh-guide`, incluído no bundle, orquestra todo o fluxo.

## Catálogo de plugins

O catálogo é construído a partir do README de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — uma lista mantida pela comunidade, ordenada por estrelas, com os 100 principais repositórios de plugins do Claude. O BYOH distribui um bundle pré-construído (reconstruído **semanalmente**, toda segunda-feira às 03:17 UTC) para que `byoh catalog index` resolva em segundos; passe `--no-bundle` para analisar a lista upstream diretamente.

```bash
# Indexação única — baixa um bundle pré-construído em segundos
byoh catalog index

# Busca offline — sem rede necessária após a indexação
byoh catalog search "memory" --genre developer --limit 5

# Adiciona um plugin ao seu harness
# licença, palavras-chave e gênero são detectados automaticamente do repo clonado
byoh catalog vendor obra/superpowers --genre developer
```

O agente LLM (via ferramentas MCP `catalog_search` / `catalog_vendor`) pode executar todo esse fluxo de forma autônoma — *"adicione um plugin de memória ao meu harness"* — ou você pode conduzi-lo diretamente pelo CLI.

Algumas ferramentas complementares são inseridas nos resultados de busca como **material de referência** (não dependências): as próprias ferramentas de camada de execução do BYOH — [alcove](https://github.com/epicsagas/alcove) (servidor de docs), [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (automação de vault), [epic-harness](https://github.com/epicsagas/epic-harness) (runtime de hook/skill) — aparecem contextualmente (uma consulta por "doc server" / "search backend" encontra o alcove) para que um agente possa recomendá-las quando relevante. Faça vendor de uma apenas se você realmente quiser; de qualquer forma, os bundles são distribuídos sem dependências.

## Power users: o CLI (opcional)

Todos os fluxos acima também são acessíveis pelo terminal. O CLI é **auxiliar** — útil para scripts, CI ou quando você prefere não conversar — mas o caminho conduzido por agente é o pretendido.

### Seu primeiro harness — pelo CLI

```bash
byoh profile init me --paths ./src ./docs   # examina seu projeto automaticamente
byoh profile confirm me --genre developer   # trava seu gênero (+ opcional --goal)

byoh render me --target claude              # sintetiza (compile + injeção de presets + static gate) e escreve o HarnessBundle; ou: codex | agy | all (padrão: all)
byoh install me --scope local               # renderiza para dist/, ativa apenas no .claude/ deste projeto
byoh install me --scope global              # ...ou ~/.claude + ~/.codex + ~/.gemini (antigo --host)
byoh install me --scope publish             # ...ou adiciona LICENSE + .gitignore e exibe instruções do git
```

A própria entrevista é conduzida pelo agente (a ferramenta MCP `profile_interview`) — a conversa é a entrevista, então não há entrevista interativa no CLI. O static gate do build sempre roda, então o bundle é estruturalmente válido antes de ser entregue. A melhoria pós-instalação é uma retrospectiva conversacional em sessões posteriores, não uma chamada de ferramenta.

## Como funciona por baixo dos panos

O motor de síntese do BYOH faz a correspondência das tags do seu perfil com o registro de skills, ordena-as em um pipeline resolvido por dependências e emite um `HarnessBundle` — um artefato pronto para git que é renderizado no formato nativo de qualquer host suportado.

- **Modelo de segurança de 4 anéis** — spec de ciclo de vida (Ring 0) e skills de pipeline embutidas (Ring 1) até skills da comunidade/não confiáveis (Ring 3), cada uma com validação crescente; skills com vendor têm sha256 fixado e verificado no momento de leitura + embed
- **Piso de segurança de 3 gates** — todo build passa por um static gate que confirma a presença dos gates Critic (qualidade), Seesaw (regressão) e Stagnation (platô); sem bypass
- **Pipelines orientados a objetivos** — declarar um objetivo de 30 dias (lançamento de produto, relatório de pesquisa, secure ship…) sobrepõe automaticamente uma escada de skills correspondente

Arquitetura: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. Veja `AGENTS.md` para o guia completo.

## Referência completa do CLI

O CLI é intencionalmente enxuto: entry points de máquina (`serve`, `catalog index` em CI, `vendor` para mantenedores) mais um espelho scriptável do fluxo central de build. Entrevista e evolução são exclusivas do MCP (conduzidas por agente).

```bash
# Profile
byoh profile init <slug> [--paths ...]      # scan não destrutivo do projeto
byoh profile confirm <slug> --genre <g> [--goal <text>]  # confirma e trava o perfil
byoh profile show <slug>                    # exibe o YAML do perfil

# Build (o static gate sempre roda; render sintetiza: compile + injeção de presets)
byoh render <slug> [--target <host>]        # claude | codex | agy | all (padrão: all); escreve o HarnessBundle
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # árvore em dist/; --scope decide o destino (local=este projeto, global=HOME, publish=+LICENSE/.gitignore+passos git). --host é o legado para --scope global.

# Skills da comunidade (mantenedor/build-time; sha256 fixado e verificado no momento de leitura + embed)
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catálogo
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# Diagnóstico / servidor
byoh doctor                                 # verifica ferramentas da camada de execução
byoh serve                                  # servidor MCP stdio (modo conduzido por agente)
```

Perfis e o cache do catálogo ficam em `~/.byoh` por padrão (sobrescreva com `BYOH_HOME`).

## Instalação

Necessária apenas se você **não** estiver usando o plugin (que auto-instala o binário) ou se quiser o BYOH em um host MCP sem plugin.

### Binário (sem toolchain Rust necessária)

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
byoh --version   # verifique
```

## Build & desenvolvimento

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # paralelo: check → clippy → test → fmt → build
```

A feature `mcp` (servidor MCP stdio) vem ativada por padrão. O BYOH não distribui nenhuma base de conhecimento embutida — para recuperação, aponte seu harness gerado para um servidor de docs como o [alcove](https://github.com/epicsagas/alcove).

## Agradecimentos

O BYOH está apoiado sobre vários esforços da comunidade:

- **Catálogo de plugins** — originado de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), uma lista comunitária ordenada por estrelas com os 100 principais repositórios de plugins do Claude. Sem ele, o catálogo não existiria.
- **Ferramentas complementares** — projetadas para interoperar com [alcove](https://github.com/epicsagas/alcove) (servidor de docs / RAG), [Episteme](https://github.com/epicsagas/Episteme) (knowledge graph) e [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (automação de vault).
- **Stack open-source** — construído sobre [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) e o ecossistema Rust.

Entradas do catálogo e skills da comunidade com vendor mantêm suas próprias licenças (detectadas automaticamente no momento do vendor). O BYOH em si é Apache-2.0.

## Licença

Apache-2.0.
