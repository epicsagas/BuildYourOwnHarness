# AGENTS.md — BuildYourOwnHarness (BYOH)

> Architecture guide for AI agents (Claude Code, etc.) working in this codebase.
> Korean design rationale (research report, project plan, architecture, roadmap, RFCs)
> lives in the alcove vault under `99-Archives/projects/BuildYourOwnHarness/`. This
> file is the implementation-facing operating guide in English.

## 1. Role

BYOH is a **generation layer**. It ingests a user profile (`truth` / `candidates` /
`derived`), combines it with genre templates to compile a `HarnessBundle`
(4-Ring), and renders it into a **static polyglot plugin** (Claude Code / Codex /
agy) that works on any machine with no extra binaries. Evolution runs under three
safety gates (Critic / Seesaw / Stagnation).

**The execution layer is delegated to external tools** — obsidian-forge (collection),
alcove (doc server), Episteme (knowledge graph), claudy (launcher) are separate
installed processes probed behind a `CommandPort`. BYOH does **not** reimplement
them (spec §Out).

## 2. Module map (hexagonal)

```
src/
├── domain/          pure types (no IO)
│   ├── profile      UserProfile schema + 4-state machine (draft→interviewed→confirmed→evolving)
│   ├── bundle       HarnessBundle, Ring, McpTool, HookSpec (declarative Ring-0 spec)
│   ├── genre        Genre, SafetyGate, GenreTemplate
│   ├── evidence     ObservationRecord, AbMetric
│   ├── scope        install scope (dist-only / local / global / publish)
│   └── synthesis    synthesis plan types
├── ports/           CommandPort, InterviewPort, LlmPort, ProfileSource (traits)
├── adapters/        FilesystemSource, RuleLlm, RuleInterview, StdCommand
├── application/     ProfileOrchestrator, synthesis, goal_pipelines, render_plugin, evolve_run
├── compiler/        compile_profile, static_gate (3 contracts), dry_run (real, failable checks)
├── evolve/          Critic / Seesaw / Stagnation gates + lifecycle + persisted state
├── templates/       genre template library (base inheritance + overrides)
├── deploy/          presets, agent_presets, vendor (sha256-pinned), install, provider
├── catalog/         quemsah top-100 index / search / vendor-from-catalog + curated companion-tool seeds
├── mcp/             stdio MCP server (`byoh serve`) — 10 tools, the primary interface
├── store.rs         profile persistence; sanitize_slug choke point; BYOH_HOME (~/.byoh)
├── security.rs      secret masking — applied to every rendered markdown artifact
├── i18n/            ko/en message catalog (10-language CLI flag, en fallback)
└── cli.rs / main.rs CLI + binary entry
```

## 3. Key invariants (do not break)

- **Slug/id sanitization at the choke point.** `store::profile_path/load/write`
  validate slugs; MCP and CLI both go through them. Vendored ids go through
  `sanitize_skill_id` before any path join.
- **Bundles are dependency-free by default.** `config.depends_on` starts empty —
  the compiler never hardcodes the epiccounty companion stack. Those tools
  surface as curated seeds in `catalog_search` (reference material) and as
  "e.g." hints in MCP tool descriptions; the user opts in via `catalog vendor`
  / `vendor add`.
- **Rendered plugins are static.** `render_plugin` emits skills/agents/manifests
  (+ `.claude-plugin/marketplace.json`) only — never an MCP server config or
  hook commands, which would require a `byoh` binary + this machine's profile on
  the consumer's machine. Bundle `hooks`/`mcp_tools` are an internal declarative
  spec.
- **sha256 pins are enforced, not just recorded.** `vendored_body` verifies the
  MANIFEST pin at read time; `build.rs` verifies it at embed time (build fails
  on mismatch). `fetch_git` requires https, passes `--` before the URL, and
  needs ≥7-char sha prefixes.
- **3 safety gates always present** for evolution (Critic + Seesaw + Stagnation);
  a Seesaw catastrophic rollback resets its counter (one-time correction, not a
  permanent brick).
- **Synthesis can never bypass the gates** — it re-runs `static_gate` after assembly.
- **External skills → Ring 3** (most-restricted) on both clone AND augment
  (id-collision) paths; a vendored body may never replace a safety-gate skill.
- **Build/render/install require a Confirmed profile** — enforced on the MCP
  surface too, not just the CLI.

## 4. Entry points

Primary interface: **MCP** (`byoh serve`, 10 tools — see README "Agent-led mode").
CLI (intentionally small): `profile init/confirm/show`,
`render --target <claude|codex|agy|all>` (synthesizes: compile + preset
injection + static gate), `install [--scope local|global|publish]`,
`vendor add/list/remove`, `catalog index/search/vendor`, `doctor`, `serve`.
Interview is MCP-only (`profile_interview`); `build` returns matched vs
skeleton skill classification so the agent decides install-readiness.

## 5. Conventions

- Edition 2024 (`rust-version = 1.85`), `clap` 4 derive, `anyhow` (binary) /
  `thiserror` `ByohError` (library), `#![forbid(unsafe_code)]`.
- Conventional Commits; lint with `cargo clippy -- -D warnings`.
- The `mcp` feature (tokio + rmcp) is **on by default** — the shipped plugin's
  mcp_config launches `byoh serve`. Build with `--no-default-features` for a
  CLI-only binary.
- Tests isolate state via thread-local overrides (`store::set_home_override`,
  `deploy::set_dist_override`, `vendor::set_vendor_root_override`) — never
  `std::env::set_var` (unsafe in Edition 2024).
