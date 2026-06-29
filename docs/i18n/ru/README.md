> Этот документ — перевод [README.md](../../README.md). Английская версия является авторитетным источником и может быть новее.
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
byoh evolve <slug>                       # 3-gate evolution cycle
byoh catalog index [--limit N]           # обход awesomeclaudeplugins.com → ~/.byoh/catalog.json
byoh catalog search "<запрос>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### Agent-led mode (MCP server)

`byoh serve` (`--features mcp`) starts a stdio MCP server so an LLM agent **drives BYOH** — the CLI becomes secondary (control inversion). 14 tools (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`) are discoverable via `tools/list`. The conversation *is* the interview/wizard.

```bash
cargo build --release --features mcp
byoh serve
```

## Основные механизмы: синтез, вендоринг и каталог

- **Синтез-движок** — `synthesize(profile)` подбирает скиллы реестра по тегам профиля, выстраивает их в упорядоченный пайплайн и принудительно прогоняет через 3 защитных шлюза (без обходов). Целевые пайплайны (product-launch / decision / research-report / secure-ship / …) накладывают лестницу скиллов + набор агентов, если 30-дневная цель совпадает.
- **Вендоринг скиллов сообщества** (RFC M3) — `byoh vendor add` загружает внешний `SKILL.md` (локальный путь или git URL), выполняет статическую валидацию + sha256 и встраивает его в **Ring 3** (наиболее ограниченный) во время сборки через `build.rs`. Внешние скиллы включаются в синтез как недоверенный код.
- **Каталог плагинов** — `byoh catalog index` (требует `--features catalog`) обходит [awesomeclaudeplugins.com](https://awesomeclaudeplugins.com) (24 000+ плагинов через `sitemap.xml` + JSON-LD) и сохраняет офлайн-кэш в `~/.byoh/catalog.json`. После этого `catalog search` и `catalog vendor` работают полностью офлайн. Во время S2-интервью мастера `profile_interview` автоматически включает `catalog_suggestions` — до 5 плагинов, подобранных по жанру, которые LLM может рекомендовать без дополнительных вызовов инструментов.

  ```bash
  # Однократная индексация (сеть; ~24 000 страниц)
  byoh catalog index --limit 500          # начать с малого; 0 = полный обход

  # Офлайн-поиск — без сети
  byoh catalog search "test driven development" --genre developer --limit 5

  # Вендоринг найденного плагина в registry/vendored/
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
