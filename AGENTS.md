# AGENTS.md — BuildYourOwnHarness (BYOH)

> Architecture guide for AI agents (Claude Code, etc.) working in this codebase.
> Korean design rationale (research report, project plan, architecture, roadmap, RFCs)
> lives in the alcove vault under `99-Archives/projects/BuildYourOwnHarness/`. This
> file is the implementation-facing operating guide in English.

## 1. Role

BYOH is a **generation layer**. It ingests a user profile (`truth` / `candidates` /
`derived`), combines it with genre templates to compile an executable `HarnessBundle`
(4-Ring), and evolves it under three safety gates (Critic / Seesaw / Stagnation) after
install.

**The execution layer is delegated to external tools** — obsidian-forge (collection),
alcove (RAG), Episteme (knowledge graph), epic-harness (execution/evolution prototype),
claudy (launcher) are separate installed processes invoked behind a `CommandPort`. BYOH
does **not** reimplement them (spec §Out).

## 2. Module map (hexagonal)

```
src/
├── domain/          pure types (no IO)
│   ├── profile      UserProfile schema + 4-state machine
│   ├── bundle       HarnessBundle, Ring, McpTool, HookSpec
│   ├── genre        Genre, SafetyGate, GenreTemplate
│   ├── evidence     ObservationRecord, AbMetric
│   └── state        BuildState, 45-min crash threshold
├── ports/           CommandPort, InterviewPort, LlmPort, ... (traits)
├── adapters/        FilesystemSource, RuleLlm, RuleInterview, StaticWizard, ...
├── application/     ProfileOrchestrator, synthesis, goal_pipelines
├── compiler/        compile_profile, static_gate (3 contracts), dry_run
├── evolve/          Critic / Seesaw / Stagnation gates
├── templates/       genre template library
├── deploy/          presets, agent_presets, vendor, install, registry, state
├── i18n/            ko/en message catalog
└── cli.rs / main.rs CLI + binary entry
```

## 3. Key invariants (do not break)

- **Compile-time embed.** Skill/agent bodies are `include_str!`'d into the binary — no
  runtime remote registry (offline + reproducible + auditable). Vendored skills are
  embedded by `build.rs` from `registry/vendored/`.
- **3 safety gates always present** for evolution (Critic + Seesaw + Stagnation).
- **Synthesis can never bypass the gates** — it re-runs `static_gate` after assembly.
- **External skills → Ring 3** (most-restricted), static-validated + sha256-pinned.

## 4. CLI entry points

`profile init/interview/confirm`, `vendor add/list/remove`, `compile [--dry-run]`,
`render --target <claude|codex|agy|all>`, `install [--host]`, `run`, `evolve`,
`serve` (MCP, `--features mcp`).

## 5. Conventions

- Edition 2021, `clap` 4 derive, `anyhow` (binary) / `thiserror` `ByohError` (library).
- Conventional Commits; lint with `cargo clippy -- -D warnings`.
- Optional features: `mcp` (stdio MCP server). Default build is light (no async runtime).
