<div align="center">

**[English](README.md)** | [한국어](./docs/i18n/ko/README.md) | [日本語](./docs/i18n/ja/README.md) | [简体中文](./docs/i18n/zh-Hans/README.md) | [Español](./docs/i18n/es/README.md) | [Deutsch](./docs/i18n/de/README.md) | [Français](./docs/i18n/fr/README.md) | [Português](./docs/i18n/pt/README.md) | [Русский](./docs/i18n/ru/README.md) | [العربية](./docs/i18n/ar/README.md)

# BuildYourOwnHarness (BYOH)

### Your AI agent, built around you

*Not a generic template — a harness compiled from your role, expertise, and goals.*

<img src="assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Most AI setups hand you a fixed set of tools and say "good luck." BYOH flips that: it interviews you, learns what you actually do, and generates a personalized agent harness — skills, agents, goal pipelines — that fits your workflow out of the box.

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
2. "Build me a harness"    # your agent interviews you, builds, fills any gaps
                           # itself, and installs — all in conversation
```

On the next session your host loads the harness automatically — agents, skills, and goal pipelines tuned to you.

## Install the plugin (recommended)

### Grok Build (xAI)

```bash
grok plugin install epicsagas/BuildYourOwnHarness --trust
```

Grok reads skills from `skills/`, agents from `agents/`, and the `hooks/hooks.json` SessionStart hook (same `install.js` bootstrap as Claude Code — Grok injects `CLAUDE_PLUGIN_ROOT` for plugin hooks). The plugin's `.mcp.json` wires the `byoh serve` MCP server, which must be on PATH. Repo-local (non-plugin) `.mcp.json` MCP servers additionally require folder trust (`--trust` or `/hooks-trust`).
Note: verified on Grok 1.0.13 — the hook loads, but Grok does not yet deliver `SessionStart` events to plugin hooks (other events like `Stop`/`Notification` fire). Until then the bootstrap is a no-op on Grok; keep `byoh` on PATH or run `node registry/scripts/install.js` once.

Using **Claude Code, Codex, or agy**? Install the plugin. It bundles the MCP server and **auto-installs the binary on first load** — no Rust toolchain, no manual setup:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@byoh
```

**agy (Antigravity):**
```bash
agy plugin install https://github.com/epicsagas/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add epicsagas/BuildYourOwnHarness
codex plugin add byoh@byoh
```

### Using any other MCP-compatible host?

BYOH speaks MCP, so Cursor, Zed, Continue, and friends work too. Install the [binary](#install-the-binary-directly) once, then point your host at the server:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Note:** This repository ships its own `byoh` marketplace (`.claude-plugin/marketplace.json`) — install it standalone, no hub required.

## Install the binary directly

Only needed if you're **not** using the plugin (the plugin auto-installs the binary on first session) or you want BYOH on a non-plugin MCP host. Prebuilt binaries are published for every release — no Rust toolchain required.

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

### From source

```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## Agent-led mode — the main path

Once your host is connected, you don't type commands — you just talk. Your agent calls BYOH's MCP tools directly, and the conversation *is* the interview, the build, and the evolve cycle:

> **You:** *I'm a backend Go developer shipping a payments API this month. Build me a harness.*
>
> **Agent:** *(scans your repo via `profile_scan`, asks a few targeted questions via `profile_interview`, locks the genre to `developer`)* → `build` synthesizes a `HarnessBundle` and classifies every skill as `matched` / `authored` / `skeleton` → for any skeleton the profile needs (say, a payments-specific verification skill), the agent authors it on the spot via `author_skill`, then `build`s again to confirm → installs agents, skills, and a secure-ship goal pipeline into Claude Code. Authored content persists across rebuilds. Done — next session, your agent already speaks your stack.

That same flow, in the suggested tool order:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → (author_skill / author_doc to fill skeletons) → build → install_plugin
```

`build` synthesizes the bundle (compile + preset injection + static gate) and
classifies every skill as `matched` (real preset body), `authored` (filled by you
via `author_skill`), or `skeleton` (genre-template placeholder) — the agent reads
that to decide whether to install now, author the skeletons, or iterate the
profile first. `author_skill` / `author_doc` write to a profile overlay that
`build` reads back, so authored content survives every rebuild.

Available tools: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

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

A few companion tools are seeded into search results as **reference material** (not dependencies): BYOH's own execution-layer tools — [alcove](https://github.com/epicsagas/alcove) (doc server), [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (vault automation), [epic-harness](https://github.com/epicsagas/epic-harness) (hook/skill runtime) — surface contextually (a "doc server" / "search backend" query finds alcove) so an agent can recommend them when relevant. Vendor one only if you actually want it; bundles ship dependency-free either way.

## Example harnesses

Two harnesses built with the flow above and published to GitHub — each a multi-host polyglot plugin (Claude Code · Codex · Antigravity), made of skills, manifests, and (where matched) agents only, so it runs on any machine with no extra binaries:

- **[byoh-paper-whisperer](https://github.com/epicsagas/byoh-paper-whisperer)** — `researcher` genre · 13 skills · 1 agent. Survey, read, analyze, and take notes on AI/ML papers. The bundled *Research Analyst* agent tags every claim with an evidence tier (primary > peer-reviewed > secondary > anecdotal) and never overstates a source; a `search_citations` MCP tool wires citation checks to a knowledge base ([alcove](https://github.com/epicsagas/alcove)).
- **[byoh-market-gtm-analyst](https://github.com/epicsagas/byoh-market-gtm-analyst)** — `business` genre · 9 skills. Scopes a market/competitive research engagement (`goal`), runs evidence-tiered market and competitor analysis (`analyze`), builds a positioning read (`decide`), and produces a GTM strategy doc with per-channel ROI and a risk register (`execute` / `roi_evaluation` / `risk_assessment`) — all gated by the same Critic / Seesaw / Stagnation safety gates.

```bash
# install byoh-paper-whisperer into Claude Code
claude plugin marketplace add epicsagas/byoh-paper-whisperer
claude plugin install byoh-paper-whisperer@byoh-paper-whisperer
```

Each repo's README carries per-host install steps for Codex and Antigravity too.

## Power users: the CLI (optional)

Every flow above is also reachable from the terminal. The CLI is **auxiliary** — useful for scripting, CI, or when you'd rather not chat — but the agent-led path is the intended one.

### Your first harness — from the CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # synthesize (compile + preset injection + static gate) and write the HarnessBundle; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

The interview itself is agent-led (the `profile_interview` MCP tool) — the conversation is the interview, so there is no interactive CLI interview. The build's static gate (Critic / Seesaw / Stagnation safety-gate presence, MCP schema, hook input) always runs and can never be bypassed — so the bundle is structurally valid before it ships. Post-install improvement is a conversational retrospective in later sessions, not a tool call.

## How it works under the hood

BYOH's synthesis engine matches your profile tags against the skill registry, orders them into a dependency-resolved pipeline, and emits a `HarnessBundle` — a git-ready artifact that renders into the native format of any supported host. Skills the registry doesn't cover stay as skeletons; the agent fills them via `author_skill`, writing to a profile-scoped overlay that the next build reads back — so authored content persists and the static gate still runs on every build.

- **4-ring security model** — lifecycle spec (Ring 0) and built-in pipeline skills (Ring 1) through community/untrusted skills (Ring 3), each with escalating validation; vendored skills are sha256-pinned and verified at read + embed time
- **3-gate safety floor** — every build's static gate confirms Critic (quality), Seesaw (regression), and Stagnation (plateau) gates are present; no bypass
- **Goal-oriented pipelines** — declaring a 30-day goal (product launch, research report, secure ship…) overlays a matching skill ladder automatically

Architecture: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. See `AGENTS.md` for the full guide.

## Full CLI reference

The CLI is intentionally small: machine entry points (`serve`, `catalog index` in CI, `vendor` for maintainers) plus a scriptable mirror of the core build flow. Interview and evolution are MCP-only (agent-led).

```bash
# Profile
byoh profile init <slug> [--paths ...]      # non-destructive project scan
byoh profile confirm <slug> --genre <g> [--goal <text>]  # confirm and lock profile
byoh profile show <slug>                    # print the profile YAML

# Build (static gate always runs; render synthesizes: compile + preset injection)
byoh render <slug> [--target <host>]        # claude | codex | agy | all (default: all); writes the HarnessBundle
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # dist/ tree; --scope decides where it goes (local=this project, global=HOME, publish=+LICENSE/.gitignore+git steps). --host is legacy for --scope global.

# Community skills (maintainer/build-time; sha256-pinned and verified at read + embed time)
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catalog
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# Diagnostics / server
byoh doctor                                 # check execution-layer tools
byoh serve                                  # stdio MCP server (agent-led mode)
```

Profiles and the catalog cache live under `~/.byoh` by default (override with `BYOH_HOME`).

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
