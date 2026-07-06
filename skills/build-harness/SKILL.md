---
name: build-harness
description: Trigger when the user asks to build, create, generate, or customize their own AI agent harness ("make my harness", "build my harness"). Routes the conversation to the BYOH MCP tools and drives the full profile → build → install flow. The conversation IS the interview/wizard.
---

# Build a Personalized Harness (BYOH)

When the user wants their own AI agent harness, drive the BYOH MCP tools
(`byoh` server) through this flow. **Do not shell out to the `byoh` CLI
directly** — call the MCP tools. The conversation with the user is the
interview/wizard; you collect answers in natural language and feed them to the
tools.

## Tools

`profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`,
`build`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.
(`profile_read` to inspect state.)

## Flow

Drive these phases in order. `build` and `install_plugin` require a **confirmed**
profile and will refuse a draft.

1. **Collect (S1)** — ask what the user does and where their materials live.
   - `profile_create` (slug, optional `scan_paths`, optional `language`) to start a profile.
   - `profile_scan` (slug, `paths`) to non-destructively gather derived candidates.

2. **Interview (S2, in-conversation)** — ask the user the open questions surfaced
   by `profile_interview`. Map their answers to `(answer, confidence)` and call
   `profile_interview` again. Only explicit answers are applied — unanswered
   questions stay open, so you can ask one question per turn and call again.
   The four genres are `developer | creator | researcher | business` (pass the
   one the user wants to `profile_confirm` — no separate listing tool).

3. **Confirm (S3)** — once genre + goal are settled:
   - `profile_confirm` (slug, genre, goal_30d) → status becomes `confirmed`.

4. **Build** — `build` (slug, `run_dry_run`) synthesizes the bundle (compile +
   preset injection + static gate) and classifies every skill:
   - **`matched_skills`** — got real preset bodies.
   - **`skeleton_skills`** — still genre-template placeholders.
   This is where you learn whether the harness is content-complete or hollow.
   `run_dry_run: true` also probes dependency tools (missing → graceful fallback).

5. **Iterate or install** — read `skeleton_skills` and decide with the user:
   - Empty / acceptable skeletons → go to step 6.
   - Needed skills still skeletons → go back to step 2 with more expertise/goal
     detail so synthesis matches richer presets, then `build` again. If the local
     preset catalog doesn't cover the domain, `catalog_search` then
     `catalog_vendor`.

6. **Render & install (S4)** — produce and deploy the harness:
   - `render_plugin` (slug, target, out) to render the host-native plugin tree.
     The output is a static polyglot plugin (skills/agents/manifests, incl.
     `.claude-plugin/marketplace.json`) — push it to GitHub and it installs via
     `claude plugin marketplace add`.
   - **Ask the user for the install scope** before installing:
     - `local` — this project only (`./.claude/skills/`); HOME untouched.
     - `global` — the user's HOME (`~/.claude`, `~/.codex`, `~/.gemini`).
     - `publish` — add `LICENSE` + `.gitignore`, no activation, return git steps.
     If they don't pick one, omit `scope` (writes `dist/` only, activates nothing).
   - `install_plugin` (slug, scope, target) to write it to `dist/` and, per the
     scope, activate on the host. BYOH ships no embedded knowledge base — if the
     user needs retrieval, point the generated harness at a doc server like `alcove`.

## Rules

- The conversation is the UI. Never ask the user to run CLI commands.
- Mask anything that looks like a secret before echoing material back.
- If a tool returns `dependency_missing`, treat it as a graceful fallback, not
  an error — tell the user the optional tool isn't installed.

## After install

There is no `evolve` tool. Improvement is a conversational retrospective in later
sessions: observe how the skills perform, note where they stay hollow, then
re-interview the profile and rebuild. The loop is human-in-the-loop, not a tool
call.
