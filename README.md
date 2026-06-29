**[English](README.md)** | [한국어](./docs/i18n/ko/README.md) | [日本語](./docs/i18n/ja/README.md) | [简体中文](./docs/i18n/zh-Hans/README.md) | [Español](./docs/i18n/es/README.md) | [Deutsch](./docs/i18n/de/README.md) | [Français](./docs/i18n/fr/README.md) | [Português](./docs/i18n/pt/README.md) | [Русский](./docs/i18n/ru/README.md) | [العربية](./docs/i18n/ar/README.md)

# BuildYourOwnHarness (BYOH)

> Interactively collect a user's tacit knowledge, data, business genre, and goals — then **generate, deploy, operate, and evolve a personalized AI agent harness**.

BYOH adds a **generation layer** on top of the validated building blocks of the [epiccounty](https://github.com/epicsagas) workspace. Instead of shipping a fixed skill/memory/pipeline set, it compiles a *unique* harness per user from an interview.

## What it does

A confirmed user profile (genre + expertise + 30-day goal) drives a synthesis engine that **recombines registry skills by keyword** into an ordered pipeline, producing a `HarnessBundle` that is *not* a fixed genre template. The whole pipeline is closed-loop and gated by three safety gates (Critic / Seesaw / Stagnation) that can never be bypassed.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## Installation

### Binary (recommended — no Rust toolchain required)

**macOS / Linux** (one line):
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows** (PowerShell):
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**Rust users**:
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness   # build from source
# cargo binstall byoh                                                        # once on crates.io
```

Verify:
```bash
byoh --version
```

Releases are built by [cargo-dist](https://github.com/axodotdev/cargo-dist) on tag push; the installers detect OS/arch and download the matching prebuilt binary to `~/.local/bin`.

### Load the BYOH plugin into your host

BYOH ships as a **polyglot plugin**: `.claude-plugin/` (Claude), `.codex-plugin/` (Codex), root `plugin.json` (agy), sharing `skills/`, `agents/`, and `mcp_config.json`. Load it so your host gets the skills/agents and the `byoh` MCP server (`byoh serve`).

- **Claude Code** — the plugin ships an in-repo marketplace (`epicsagas`), so add it then install:
  ```bash
  claude plugin marketplace add epicsagas/BuildYourOwnHarness   # public: or epicsagas/plugins
  claude plugin install byoh@epicsagas
  ```
- **agy (Antigravity)** — reads the plugin from a directory:
  ```bash
  agy plugin install /path/to/BuildYourOwnHarness
  agy plugin enable byoh
  ```
- **Codex** — register the repo as a local marketplace:
  ```bash
  codex plugin marketplace add /path/to/BuildYourOwnHarness
  codex plugin add byoh@epicsagas
  ```

The plugin's `SessionStart` hook (`.claude-plugin/hooks.json` → `registry/scripts/install.js`) auto-installs the `byoh` binary cross-platform if it's missing when the plugin loads — so Rust is never a prerequisite.

> The repo is currently **private**: use local paths / `epicsagas/BuildYourOwnHarness` above. Once public, BYOH will also be listed in the shared [`epicsagas/plugins`](https://github.com/epicsagas/plugins) marketplace (`/plugin marketplace add epicsagas/plugins` → `byoh@epicsagas`).

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

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. (Incremental re-index on source change is a follow-up.)

## License

Apache-2.0.
