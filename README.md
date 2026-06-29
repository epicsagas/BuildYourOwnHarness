**[English](README.md)** | [한국어](./docs/i18n/ko/README.md) | [日本語](./docs/i18n/ja/README.md) | [简体中文](./docs/i18n/zh-Hans/README.md) | [Español](./docs/i18n/es/README.md) | [Deutsch](./docs/i18n/de/README.md) | [Français](./docs/i18n/fr/README.md) | [Português](./docs/i18n/pt/README.md) | [Русский](./docs/i18n/ru/README.md) | [العربية](./docs/i18n/ar/README.md)

# BuildYourOwnHarness (BYOH)

> Interactively collect a user's tacit knowledge, data, business genre, and goals — then **generate, deploy, operate, and evolve a personalized AI agent harness**.

BYOH adds a **generation layer** on top of the validated building blocks of the [epiccounty](https://github.com/epicsagas) workspace. Instead of shipping a fixed skill/memory/pipeline set, it compiles a *unique* harness per user from an interview.

<img src="assets/features.png" width="100%" alt="Build Your Own Harness">

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
byoh vendor remove <id> --genre <g>           # positional: id first, then --genre
byoh compile <slug> [--dry-run]          # static gate + dry-run gate → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # safe dist/ install (--host for live plugin dir)
byoh run <slug>
byoh evolve <slug> [--genre <g>] [--edit-type <t>] [--score-with <f>] [--score-without <f>] [--samples <n>]
```

### Agent-led mode (MCP server)

`byoh serve` (`--features mcp`) starts a stdio MCP server so an LLM agent **drives BYOH** — the CLI becomes secondary (control inversion). 12 tools (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`) are discoverable via `tools/list`. The conversation *is* the interview/wizard.

```bash
cargo build --release --features mcp
byoh serve
```

## Genres

BYOH ships four genre templates. `mvp=true` means the genre is fully wired end-to-end; non-MVP genres compile and deploy without warnings — the MVP flag is classification metadata only, not a compile gate.

| Genre | Ring 1 pipeline | Ring 2 quality skills | Skills | Agents | MVP |
|-------|----------------|----------------------|--------|--------|-----|
| `creator` | draft → edit → proofread → publish | continuity, character_consistency | 9 | 2 | ✅ |
| `developer` | spec → go → check → ship | tdd, debug, secure, perf | 19 | 3 | ✅ |
| `researcher` | spec → literature_review → go → check → ship | citation_accuracy, source_verification | 13 | 1 | — |
| `business` | goal → analyze → decide → execute | roi_evaluation, risk_assessment | 16 | 1 | — |

Skills/Agents counts are from `byoh render --target all`. `developer` has the most (19 skills, 3 agents: code-reviewer, debugger, tech-debt-auditor) because Ring 2 carries four quality gates.

Pass `--genre <g>` to `profile confirm`. The 30-day goal in your profile steers which skill ladder the synthesis engine selects — the pipeline above is the base; goals like "ship a secure API" or "finish a research report" overlay an extra skill set.

## Vendor: extending the skill registry

`byoh vendor` lets you pull an external `SKILL.md` into the local registry without forking BYOH. Vendored skills are committed to disk (`registry/vendored/`) and sha256-pinned — no network at runtime.

### Local path

```bash
# Single file
byoh vendor add ./my-hook-writer.md --genre creator --id hook-writer \
  --keywords hook,opening,youtube

# Directory laid out as skills/<id>/SKILL.md
byoh vendor add /path/to/plugin-repo --genre developer --id my-linter \
  --keywords lint,quality
```

### Remote git repo

```bash
# Trusted source (github.com/anthropics/*) — no --trust needed
byoh vendor add https://github.com/anthropics/claude-plugins-official \
  --genre developer --id official-tdd --keywords tdd,test

# Untrusted third-party source — explicit --trust required
byoh vendor add https://github.com/someone/my-skills --genre creator \
  --id scene-writer --keywords scene,screenplay --trust

# Pin to a specific commit sha (recommended for reproducible builds)
byoh vendor add https://github.com/anthropics/claude-plugins-official \
  --genre developer --id official-tdd --sha abc1234
```

### Security model

`vendor add` runs a static blocklist scan before writing anything. Patterns like `curl`, `wget`, `rm -rf`, `~/`, `$HOME` cause an immediate refusal:

```
Error: vendor add refused — static validation flagged: [curl]
```

Pass `--trust` only for remote sources not in the `github.com/anthropics/` allowlist. The vendored `.md` is stored verbatim and sha256-stamped in `registry/vendored/MANIFEST.toml`.

### List / remove

```bash
byoh vendor list
# skill_id      genre     license    sha256
# hook-writer   creator   unknown    e98eb282d625...

byoh vendor remove hook-writer --genre creator
# removed vendored 'hook-writer' (creator)
```

### How vendored skills enter synthesis

At `byoh compile` time, `build.rs` embeds all files in `registry/vendored/` into the binary's preset catalog. The synthesis engine matches vendored skills by their `--keywords` against profile tags — a vendored skill with `keywords = ["hook","youtube"]` activates when the profile's automation targets include those keywords. Vendored skills land in **Ring 3** (highest restriction, 3-gate protected).

> **Note:** After `vendor add`, run `cargo build --release` and replace your binary for the vendored skill to take effect in `byoh compile`. A pre-built binary cannot pick up new vendor entries added after the build.

## Evolution gates

`byoh evolve` runs three safety gates in sequence — none can be bypassed:

| Gate | Triggers on | Outcome |
|------|-------------|---------|
| **Critic** | reward-hacking pattern detected | `Rejected` |
| **Seesaw** | edit score < baseline (catastrophic forgetting) | `RolledBack` or `AutoTuned` |
| **Stagnation** | consecutive cycles with no improvement | `AutoTuned` |

`AutoTuned` means the gate intervened to adjust the edit rather than fully rolling back — seen on early cycles when the Seesaw has limited history. `RolledBack` is a hard revert; it appears once the Seesaw has enough cycle history to be certain the edit regresses quality. `Approved` passes all three.

```
# Approved path (score improves)
byoh evolve solo-creator --genre creator --edit-type AddSkill \
  --score-with 0.82 --score-without 0.5 --samples 3
# → cycle #N: Approved  (Critic: no reward-hacking)

# Regression path (score drops) → Seesaw blocks
byoh evolve solo-creator --genre creator --edit-type AddSkill \
  --score-with 0.3 --score-without 0.5 --samples 3
# → cycle #N: RolledBack  (seesaw: catastrophic forgetting detected)
```

## Core: synthesis + vendoring

- **Synthesis engine** — `synthesize(profile)` matches registry skills against profile tags, orders them into a pipeline, and forces a 3-gate re-pass (no bypass). Goal-oriented pipelines (product-launch / decision / research-report / secure-ship / …) overlay a skill ladder + agent set when the 30-day goal matches.
- **Community skill vendoring** (RFC M1) — `byoh vendor add` fetches an external `SKILL.md` (local path or git URL), runs static validation + sha256, and commits it to `registry/vendored/`. The `build.rs` embeds vendored files into the binary preset catalog at build time — no network at runtime.

## Examples

`examples/` contains ready-to-load plugin trees — the output of `byoh render --target all` + `byoh install`, committed so you can load them without running the full pipeline yourself.

```
examples/
├── byoh-solo-creator/    # creator genre  — 9 skills, 2 agents (draft-writer, consistency-editor)
├── byoh-solo-developer/  # developer genre — 19 skills, 3 agents (code-reviewer, debugger, tech-debt-auditor)
├── byoh-solo-researcher/ # researcher genre — 13 skills, 1 agent (research-analyst)
└── byoh-solo-business/   # business genre  — 16 skills, 1 agent (decision-analyst)
```

Each directory is a **polyglot plugin tree** — load it directly into any supported host:

```bash
# Claude Code
claude plugin install ./examples/byoh-solo-developer

# agy (Antigravity)
agy plugin install ./examples/byoh-solo-developer
agy plugin enable byoh-solo-developer

# Codex
codex plugin marketplace add ./examples
codex plugin add byoh-solo-developer@local
```

These examples were generated from real profiles (30-day goals, Korean output, all three hosts) and passed the full `compile --dry-run → render → install → evolve` pipeline. They are the canonical reference for what BYOH produces.

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
