---
name: build-harness
description: Trigger when the user asks to build, create, generate, or customize their own AI agent harness ("make my harness", "build my harness", "내 하네스 만들어줘"). Routes the conversation to the BYOH MCP tools and drives the full profile → compile → install → evolve flow. The conversation IS the interview/wizard.
---

# Build a Personalized Harness (BYOH)

When the user wants their own AI agent harness, drive the BYOH MCP tools
(`byoh` server) through this flow. **Do not shell out to the `byoh` CLI
directly** — call the MCP tools. The conversation with the user is the
interview/wizard; you collect answers in natural language and feed them to the
tools.

## Flow

Follow the steps in this order — compile/render/install require a **confirmed**
profile and will refuse a draft.

1. **Collect (S1)** — ask what the user does and where their materials live.
   - `profile_create` (slug, optional `scan_paths`, optional `language`) to start a profile.
   - `profile_scan` (slug, `paths`) to non-destructively gather derived candidates.

2. **Interview (S2, in-conversation)** — ask the user the open questions. Map
   their answers to `(answer, confidence)` and call `profile_interview`. Only
   explicit answers are applied — unanswered questions stay open, so you can
   ask one question per turn and call the tool again. Use `genre_list` to show
   genres.

3. **Confirm (S3)** — once genre + goal are settled:
   - `profile_confirm` (slug, genre, goal_30d) → status becomes `confirmed`.

4. **Compile (S4)** — `compile` (slug, run_static_gate=true) to render the
   4-Ring HarnessBundle. Then `compile_dry_run` (slug) to validate gates.

5. **Clone vetted skills (optional)** — `registry_clone_skill` (genre, skill_id
   like `tdd`/`debug`, slug) to inject a verified preset skill into the bundle.
   Generate and clone coexist.

6. **Render & install (S5)** — produce and deploy the harness:
   - `render_plugin` (slug, target) to render the host-native plugin tree. The
     output is a static polyglot plugin (skills/agents/manifests, incl.
     `.claude-plugin/marketplace.json`) — push it to GitHub and it installs via
     `claude plugin marketplace add`.
   - **Ask the user for the install scope** before installing:
     - `local` — this project only (`./.claude/skills/`); HOME untouched.
     - `global` — the user's HOME (`~/.claude`, `~/.codex`, `~/.gemini`).
     - `publish` — add `LICENSE` + `.gitignore`, no activation, return git steps.
     If they don't pick one, omit `scope` (writes `dist/` only, activates nothing).
   - `install_plugin` (slug, target, scope) to write it to `dist/` and, per the
     scope, activate on the host. BYOH ships no embedded knowledge base — if the
     user needs retrieval, point the generated harness at a doc server like `alcove`.

7. **Evolve (later)** — once the harness is running and you have observations,
   `evolve_cycle` (genre, edit_type, metric) runs one Ring-3 cycle under the
   3 safety gates.

## Rules

- The conversation is the UI. Never ask the user to run CLI commands.
- Mask anything that looks like a secret before echoing material back.
- If a tool returns `dependency_missing`, treat it as a graceful fallback, not
  an error — tell the user the optional tool isn't installed.
