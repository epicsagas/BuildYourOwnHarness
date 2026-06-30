> Este documento é a versão em português de [README.md](../../../README.md). A versão em inglês é a fonte oficial.

<div align="center">

[English](../../../README.md) | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | **Português** | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Seu agente de IA, feito para você

*Não um template genérico — um harness compilado de acordo com seu papel, expertise e objetivos.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

A maioria das ferramentas de IA entrega um conjunto fixo de funções e diz "se vira". O BYOH faz o contrário: uma entrevista rápida aprende como você realmente trabalha e gera um harness de agente personalizado — habilidades, memória, pipelines — que já funciona para o seu fluxo de trabalho desde o início.

## Para quem é?

- **Desenvolvedores** que querem um agente que já conhece sua stack, estilo de testes e cadência de entrega
- **Pesquisadores** que precisam de revisão bibliográfica, rastreamento de citações e síntese conectados numa só ferramenta
- **Criadores** que querem um parceiro de escrita alinhado com sua voz e estrutura de projeto
- **Analistas de negócio** que precisam de frameworks de decisão e pipelines de relatórios — não de um chat genérico

Se você já pensou "queria que minha IA entendesse meu contexto de verdade" — é exatamente isso que o BYOH faz.

## Comece em 60 segundos

```bash
byoh profile init eu        # escaneia seu projeto — só leitura, sem modificações
byoh profile interview eu   # uma conversa curta sobre seu papel e objetivos
byoh compile eu             # gera seu harness pessoal
byoh install eu             # faz o deploy para Claude / Codex / agy
```

Na próxima sessão, seu host carrega o harness automaticamente — agentes, habilidades, memória e pipelines ajustados para você.

**Já sabe o que precisa?** Explore o catálogo da comunidade:
```bash
byoh catalog index                              # baixa a lista dos 100 melhores plugins (segundos)
byoh catalog search "code review"               # encontre plugins relevantes
byoh catalog vendor anthropics/claude-code-review   # adicione um ao seu harness
```

## Instalação

### Binário (recomendado — sem necessidade de Rust)

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

### Carregar o plugin no seu host de IA

O BYOH vem como um plugin poliglota compatível com Claude Code, Codex e agy — um só repositório, três hosts.

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /caminho/para/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /caminho/para/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

O plugin instala o binário `byoh` automaticamente no primeiro carregamento — sem necessidade de Rust no seu lado.

> **Nota:** O repositório é atualmente privado. Use os caminhos acima. Quando for público, aparecerá no marketplace compartilhado `epicsagas/plugins`.

## Seu primeiro harness — passo a passo

### Etapa 1 — Perfil
```bash
byoh profile init eu --paths ./src ./docs   # análise automática do projeto
byoh profile interview eu                   # conversa de ~5 min
byoh profile confirm eu --genre developer   # confirmar seu gênero
```

O BYOH pergunta sobre seu papel, nível de expertise, ferramentas e objetivo de 30 dias. A entrevista se adapta — um pesquisador recebe perguntas diferentes de um desenvolvedor.

### Etapa 2 — Compilar e instalar
```bash
byoh compile eu                     # gera o HarnessBundle (validado e controlado)
byoh render eu --target claude      # ou: codex | agy | all
byoh install eu                     # instalação segura em dist/
```

### Etapa 3 — Executar e evoluir
```bash
byoh run eu       # iniciar com o harness ativo
byoh evolve eu    # melhorar o harness com base no feedback das sessões
```

`evolve` executa um ciclo com 3 gates (Critic / Seesaw / Stagnation) que nunca podem ser ignorados — a evolução é segura e auditável.

## Catálogo de plugins

O catálogo oferece uma lista curada dos 100 melhores plugins Claude (ordenados por estrelas, atualizada diariamente) para você descobrir e adicionar habilidades da comunidade sem sair do terminal.

```bash
# Indexação única — baixa um bundle pré-construído em segundos
byoh catalog index

# Busca offline — sem rede necessária após a indexação
byoh catalog search "memória" --genre developer --limit 5

# Adicionar um plugin ao seu harness
# licença, keywords e genre são detectados automaticamente do repositório clonado
byoh catalog vendor obra/superpowers --genre developer
```

O agente LLM (via ferramentas MCP `catalog_search` / `catalog_vendor`) pode fazer esse fluxo inteiro de forma autônoma — ou você pode conduzir diretamente pela CLI.

## Modo agente

`byoh serve` inicia um servidor MCP stdio. Em vez de digitar comandos, seu host de IA chama diretamente as 14 ferramentas do BYOH — a conversa *é* a entrevista, o wizard e a execução.

```bash
byoh serve   # Claude / Codex / agy conecta e conduz tudo
```

Ferramentas disponíveis: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `rag_index`, `rag_search`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`, e mais.

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

# Base de conhecimento (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<consulta>" [--genre <g>] [--k N]
```

## Como funciona por dentro

O motor de síntese do BYOH combina suas tags de perfil com o registro de habilidades, as ordena em um pipeline com dependências resolvidas e emite um `HarnessBundle` — um artefato pronto para git que renderiza no formato nativo de cada host suportado.

- **Modelo de segurança com 4 anéis** — habilidades embutidas (anel 1) até habilidades da comunidade/não confiáveis (anel 4), com validação crescente em cada nível
- **Evolução com 3 gates** — cada ciclo `evolve` passa pelos gates Critic (qualidade), Seesaw (regressão) e Stagnation (estagnação); sem possibilidade de bypass
- **RAG persistente** — re-embedding incremental a cada mudança (`+adicionado ~modificado -removido`); a busca reutiliza o índice salvo sem re-embedding
- **Pipelines por objetivo** — declarar um objetivo de 30 dias (lançamento de produto, relatório de pesquisa, entrega segura…) sobrepõe automaticamente uma escada de habilidades correspondente

Arquitetura hexagonal: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Veja `AGENTS.md` para o guia completo.

## Build e desenvolvimento

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # testes unitários + e2e
cvp                               # paralelo: check → clippy → test → fmt → build
```

Features opcionais: `--features mcp` (servidor MCP), `--features native-rag` (embeddings locais), `--features rag-openai` (embeddings OpenAI). Os binários de release incluem todas as features.

## Licença

Apache-2.0.
