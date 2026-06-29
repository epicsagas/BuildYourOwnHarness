> Este documento é uma tradução de [README.md](../../README.md). A versão em inglês é a fonte autoritativa e pode estar mais atualizada.
>
> ⚠️ Auto-translation pending — the English source below awaits translation via the i18n workflow.

# BuildYourOwnHarness (BYOH)

> Interactively collect a user's tacit knowledge, data, business genre, and goals — then **generate, deploy, operate, and evolve a personalized AI agent harness**.

BYOH adds a **generation layer** on top of the validated building blocks of the [epiccounty](https://github.com/epicsagas) workspace. Instead of shipping a fixed skill/memory/pipeline set, it compiles a *unique* harness per user from an interview.

## What it does

A confirmed user profile (genre + expertise + 30-day goal) drives a synthesis engine that **recombines registry skills by keyword** into an ordered pipeline, producing a `HarnessBundle` that is *not* a fixed genre template. The whole pipeline is closed-loop and gated by three safety gates (Critic / Seesaw / Stagnation) that can never be bypassed.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## Build & verify

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                       # unit + e2e
./target/release/byoh --help
```

Hexagonal architecture: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`.

## CLI

```bash
byoh profile init <slug> [--paths ...]   # S1 autoscan (non-destructive)
byoh profile interview <slug>            # S2 interview (Suggest + Council)
byoh profile confirm <slug> --genre <g>  # S3 wizard confirm
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # static gate + dry-run gate → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # safe dist/ install (--host for live plugin dir)
byoh run <slug>
byoh evolve <slug>                       # ciclo de evolução com 3 gates
byoh catalog index [--limit N]           # rastrear awesomeclaudeplugins.com → ~/.byoh/catalog.json
byoh catalog search "<consulta>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### Modo agente (servidor MCP)

`byoh serve` (`--features mcp`) inicia um servidor MCP via stdio para que um agente LLM **controle o BYOH** — a CLI torna-se secundária (inversão de controle). 14 ferramentas (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`) são descobríveis via `tools/list`. A conversa *é* a entrevista/assistente.

```bash
cargo build --release --features mcp
byoh serve
```

## Núcleo: síntese, vendoring e catálogo

- **Motor de síntese** — `synthesize(profile)` combina habilidades do registry com as tags do perfil, organiza-as em um pipeline e força uma re-execução com 3 gates (sem desvio). Pipelines orientados a objetivos (lançamento de produto / decisão / relatório de pesquisa / entrega segura / …) aplicam uma escada de habilidades + conjunto de agentes quando o objetivo de 30 dias corresponde.
- **Vendoring de habilidades da comunidade** (RFC M3) — `byoh vendor add` busca um `SKILL.md` externo (caminho local ou URL git), executa validação estática + sha256 e o incorpora no **Ring 3** (mais restrito) em tempo de build via `build.rs`. Habilidades externas entram na síntese como código não confiável.
- **Catálogo de plugins** — `byoh catalog index` (requer `--features catalog`) rastreia [awesomeclaudeplugins.com](https://awesomeclaudeplugins.com) (mais de 24.000 plugins via `sitemap.xml` + JSON-LD) e salva um cache offline em `~/.byoh/catalog.json`. Após isso, `catalog search` e `catalog vendor` funcionam completamente offline. Durante a entrevista S2 do assistente, `profile_interview` inclui automaticamente `catalog_suggestions` — até 5 plugins correspondentes ao gênero que o LLM pode recomendar sem chamadas de ferramenta adicionais.

  ```bash
  # Indexação única (rede; ~24.000 páginas)
  byoh catalog index --limit 500          # comece pequeno; 0 = rastreamento completo

  # Busca offline — sem rede
  byoh catalog search "test driven development" --genre developer --limit 5

  # Incorporar plugin encontrado em registry/vendored/
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
