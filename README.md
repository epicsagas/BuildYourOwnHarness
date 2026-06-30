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

BYOH is built to be driven by your AI agent. Install it, connect your host over MCP, then just talk — the conversation *is* the interview, the wizard, and the build.

```
1. Install byoh              # one-line install (see below)
2. Connect your host via MCP # byoh serve — any MCP-compatible agent
3. "Build me a harness"      # your agent scans your repo and compiles the result
```

On the next session your host loads the harness automatically — agents, skills, memory, and pipelines tuned to you.

**Prefer the terminal?** The same flow from the CLI:
```
byoh profile init me        # scan your project — non-destructive, read-only
byoh profile interview me   # a short conversation about your role and goals
byoh compile me             # generates your personal harness
byoh install me             # deploys it to Claude / Codex / agy
```

**Already know what you need?** Browse the community catalog:
```bash
byoh catalog index                                 # fetch the top-100 plugin list (seconds)
byoh catalog search "code review"                  # find relevant plugins
byoh catalog vendor anthropics/claude-code-review  # add one to your harness
```

## Installation

### Binary (recommended — no Rust toolchain required)

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

### Connect your AI host

BYOH speaks MCP, so any MCP-compatible agent can drive it. Install the binary above, start the server, and your host calls every BYOH tool directly:

```bash
byoh serve   # stdio MCP server
```

For **other agents** (Cursor, Zed, Continue, …), add `byoh` to your host's MCP config:
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

Using **Claude Code, Codex, or agy**? Install the plugin instead — it bundles the MCP server and auto-installs the binary on first load (no Rust required):

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

> **Note:** The repo is currently private. Use the paths above. Once public it will appear in the shared `epicsagas/plugins` marketplace.

## Agent-led mode

Once your host is connected, you don't type commands — you just talk. Your agent calls BYOH's 14 tools directly, and the conversation *is* the interview, the build, and the evolve cycle:

Available tools: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`, and more.

## Your first harness — from the CLI

The same steps, driven from the terminal:

### Step 1 — Profile
```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile interview me                   # ~5 min conversation
byoh profile confirm me --genre developer   # lock in your genre
```

BYOH asks about your role, expertise level, tools, and 30-day goal. The interview adapts — a researcher gets different questions than a developer.

### Step 2 — Compile & install
```bash
byoh compile me          # generates HarnessBundle (validated + gated)
byoh render me --target claude   # or: codex | agy | all
byoh install me          # safe install to dist/
```

### Step 3 — Run & evolve
```bash
byoh run me              # launch with your harness active
byoh evolve me           # improve the harness based on session feedback
```

`evolve` runs a 3-gate cycle (Critic / Seesaw / Stagnation) that can never be bypassed — so evolution is safe and auditable.

## Plugin catalog

The catalog gives you a curated list of the top 100 Claude plugins (sorted by stars, refreshed daily) so you can discover and add community skills without leaving the terminal.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

The LLM agent (via `catalog_search` / `catalog_vendor` MCP tools) can do this entire flow autonomously — or you can drive it directly from the CLI.

## Full CLI reference

```bash
# Profile
byoh profile init <slug> [--paths ...]      # non-destructive project scan
byoh profile interview <slug>               # guided interview
byoh profile confirm <slug> --genre <g>     # confirm and lock profile

# Build
byoh compile <slug> [--dry-run]             # validate + generate HarnessBundle
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # deploy to dist/ or live plugin dir

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

## How it works under the hood

BYOH's synthesis engine matches your profile tags against the skill registry, orders them into a dependency-resolved pipeline, and emits a `HarnessBundle` — a git-ready artifact that renders into the native format of any supported host.

- **4-ring security model** — built-in skills (Ring 1) through community/untrusted skills (Ring 4), each with escalating validation
- **3-gate evolution** — every `evolve` cycle passes Critic (quality), Seesaw (regression), and Stagnation (plateau) gates; no bypass
- **Goal-oriented pipelines** — declaring a 30-day goal (product launch, research report, secure ship…) overlays a matching skill ladder automatically

Architecture: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. See `AGENTS.md` for the full guide.

## Build & develop

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

The `mcp` feature (stdio MCP server) is on by default. BYOH ships no embedded knowledge base — for retrieval, point your generated harness at a doc server like [alcove](https://github.com/epicsagas/alcove).

## License

Apache-2.0.
