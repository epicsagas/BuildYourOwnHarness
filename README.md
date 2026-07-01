<div align="center">

**[English](README.md)** | [한국어](./docs/i18n/ko/README.md) | [日本語](./docs/i18n/ja/README.md) | [简体中文](./docs/i18n/zh-Hans/README.md) | [Español](./docs/i18n/es/README.md) | [Deutsch](./docs/i18n/de/README.md) | [Français](./docs/i18n/fr/README.md) | [Português](./docs/i18n/pt/README.md) | [Русский](./docs/i18n/ru/README.md) | [العربية](./docs/i18n/ar/README.md)

# BuildYourOwnHarness (BYOH)

### Your AI agent, built around you

*Not a generic template — a harness compiled from your role, expertise, and goals.*

<img src="assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Most AI setups hand you a fixed set of tools and say "good luck." BYOH flips that: it interviews you, learns what you actually do, and generates a personalized agent harness — skills, memory, pipelines — that fits your workflow out of the box.

## Who is this for?

- **Developers** who want an agent that already knows their stack, test style, and delivery cadence
- **Researchers** who need literature review, citation tracking, and synthesis wired together
- **Creators** who want a writing partner that matches their voice and project structure
- **Business analysts** who need decision frameworks and reporting pipelines, not raw chat

If you've ever thought "I wish my AI actually knew my context" — this is what BYOH does.

## How it works in 60 seconds

BYOH is built to be driven by your AI agent — not by you typing commands. Install the plugin, then just talk. The conversation *is* the interview, the wizard, and the build.

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # your agent scans your repo and compiles the result
```

On the next session your host loads the harness automatically — agents, skills, memory, and pipelines tuned to you.

## Install the plugin (recommended)

Using **Claude Code, Codex, or agy**? Install the plugin. It bundles the MCP server and **auto-installs the binary on first load** — no Rust toolchain, no manual setup:

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

### Using any other MCP-compatible host?

BYOH speaks MCP, so Cursor, Zed, Continue, and friends work too. Install the [binary](#installation) once, then point your host at the server:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Note:** The repo is currently private. Use the paths above. Once public it will appear in the shared `epicsagas/plugins` marketplace.

## Agent-led mode — the main path

Once your host is connected, you don't type commands — you just talk. Your agent calls BYOH's MCP tools directly, and the conversation *is* the interview, the build, and the evolve cycle:

> **You:** *I'm a backend Go developer shipping a payments API this month. Build me a harness.*
>
> **Agent:** *(scans your repo via `profile_scan`, asks a few targeted questions via `profile_interview`, locks the genre to `developer`)* → compiles a `HarnessBundle` → installs agents, skills, memory, and a secure-ship pipeline into Claude Code. Done — next session, your agent already speaks your stack.

That same flow, in the suggested tool order:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → compile → compile_dry_run → render_plugin → install_plugin
           → (optional) registry_clone_skill → (later) evolve_cycle
```

Available tools: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `genre_list`, `compile`, `compile_dry_run`, `render_plugin`, `install_plugin`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`.

Want the agent to walk you through it? Just say *"build my harness"* — the bundled `byoh-guide` agent orchestrates the whole flow.

## Plugin catalog

The catalog is built from the [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) README — a community-maintained, star-ranked list of the top 100 Claude plugin repositories. BYOH ships a prebuilt bundle (rebuilt **weekly**, every Monday 03:17 UTC) so `byoh catalog index` resolves in seconds; pass `--no-bundle` to parse the upstream list directly.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

The LLM agent (via `catalog_search` / `catalog_vendor` MCP tools) can do this entire flow autonomously — *"add a memory plugin to my harness"* — or you can drive it directly from the CLI.

## Power users: the CLI (optional)

Every flow above is also reachable from the terminal. The CLI is **auxiliary** — useful for scripting, CI, or when you'd rather not chat — but the agent-led path is the intended one.

### Your first harness — from the CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile interview me                   # ~5 min conversation
byoh profile confirm me --genre developer   # lock in your genre

byoh compile me --no-dry-run                # validate + write the HarnessBundle (dry-run is default)
byoh render me --target claude              # or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions

byoh run me                                 # launch with your harness active
byoh evolve me                              # improve the harness based on session feedback
```

BYOH asks about your role, expertise level, tools, and 30-day goal. The interview adapts — a researcher gets different questions than a developer. `evolve` runs a 3-gate cycle (Critic / Seesaw / Stagnation) that can never be bypassed — so evolution is safe and auditable.

## How it works under the hood

BYOH's synthesis engine matches your profile tags against the skill registry, orders them into a dependency-resolved pipeline, and emits a `HarnessBundle` — a git-ready artifact that renders into the native format of any supported host.

- **4-ring security model** — built-in skills (Ring 1) through community/untrusted skills (Ring 4), each with escalating validation
- **3-gate evolution** — every `evolve` cycle passes Critic (quality), Seesaw (regression), and Stagnation (plateau) gates; no bypass
- **Goal-oriented pipelines** — declaring a 30-day goal (product launch, research report, secure ship…) overlays a matching skill ladder automatically

Architecture: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. See `AGENTS.md` for the full guide.

## Full CLI reference

```bash
# Profile
byoh profile init <slug> [--paths ...]      # non-destructive project scan
byoh profile interview <slug>               # guided interview
byoh profile confirm <slug> --genre <g>     # confirm and lock profile

# Build
byoh compile <slug> [--no-dry-run]          # dry-run is on by default; pass --no-dry-run to write the bundle
byoh render <slug> [--target <host>]        # claude | codex | agy | all (default: all)
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # dist/ tree; --scope decides where it goes (local=this project, global=HOME, publish=+LICENSE/.gitignore+git steps). --host is legacy for --scope global.

# Run & evolve
byoh run <slug>
byoh evolve <slug>

# Community skills
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catalog
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Installation

Only needed if you're **not** using the plugin (which auto-installs the binary) or you want BYOH on a non-plugin MCP host.

### Binary (no Rust toolchain required)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**From source:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## Build & develop

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

The `mcp` feature (stdio MCP server) is on by default. BYOH ships no embedded knowledge base — for retrieval, point your generated harness at a doc server like [alcove](https://github.com/epicsagas/alcove).

## Acknowledgments

BYOH stands on the shoulders of several community efforts:

- **Plugin catalog** — sourced from [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), a star-ranked community list of the top 100 Claude plugin repositories. Without it, the catalog would not exist.
- **Companion tools** — designed to interoperate with [alcove](https://github.com/epicsagas/alcove) (doc server / RAG), [Episteme](https://github.com/epicsagas/Episteme) (knowledge graph), and [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (vault automation).
- **Open-source stack** — built on [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq), and the Rust ecosystem.

Catalog entries and vendored community skills keep their own licenses (detected automatically at vendor time). BYOH itself is Apache-2.0.

## License

Apache-2.0.
