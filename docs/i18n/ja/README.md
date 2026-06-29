> この文書は [README.md](../../README.md) の日本語訳です。英語版が権威ある原本であり、より新しい可能性があります。
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
```

### Agent-led mode (MCP server)

`byoh serve` (`--features mcp`) starts a stdio MCP server so an LLM agent **drives BYOH** — the CLI becomes secondary (control inversion). 12 tools (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`) are discoverable via `tools/list`. The conversation *is* the interview/wizard.

```bash
cargo build --release --features mcp
byoh serve
```

## Core: synthesis + vendoring

- **Synthesis engine** — `synthesize(profile)` matches registry skills against profile tags, orders them into a pipeline, and forces a 3-gate re-pass (no bypass). Goal-oriented pipelines (product-launch / decision / research-report / secure-ship / …) overlay a skill ladder + agent set when the 30-day goal matches.
- **Community skill vendoring** (RFC M3) — `byoh vendor add` fetches an external `SKILL.md` (local path or git URL), runs static validation + sha256, and embeds it into **Ring 3** (most-restricted) at build time via `build.rs`. External skills join synthesis as untrusted code.

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
