---
name: build-harness
description: Trigger when the user asks to build, create, generate, or customize their own AI agent harness ("make my harness", "build my harness", "내 하네스 만들어줘"). Routes the conversation to the BYOH MCP tools and drives the full profile → rag → compile → evolve flow. The conversation IS the interview/wizard.
---

# Build a Personalized Harness (BYOH)

When the user wants their own AI agent harness, drive the BYOH MCP tools
(`byoh` server) through this flow. **Do not shell out to the `byoh` CLI
directly** — call the MCP tools. The conversation with the user is the
interview/wizard; you collect answers in natural language and feed them to the
tools.

## Flow

1. **Collect (S1)** — ask what the user does and where their materials live.
   - `profile_create` (slug, optional `scan_paths`) to start a profile.
   - `profile_scan` to non-destructively gather derived candidates from paths.

2. **Index (S2)** — once you know the genre and corpus:
   - `rag_index` (genre, corpus path) to build a genre index.
   - `rag_search` (query, genre, optional corpus) to retrieve relevant material.

3. **Interview (S3, in-conversation)** — ask the user the open questions. Map
   their answers to `(answer, confidence)` and call `profile_interview`. Empty
   answers auto-accept rule-based suggestions. Use `genre_list` to show genres.

4. **Confirm (S3)** — once genre + goal are settled:
   - `profile_confirm` (slug, genre, goal_30d) → status becomes `confirmed`.

5. **Compile (S4)** — `compile` (slug, run_static_gate=true) to render the
   4-Ring HarnessBundle. Then `compile_dry_run` (slug) to validate gates.

6. **Clone vetted skills (optional)** — `registry_clone_skill` (genre, skill_id
   like `tdd`/`debug`, slug) to inject a verified preset skill into the bundle.
   Generate and clone coexist.

7. **Evolve (later)** — once the harness is running and you have observations,
   `evolve_cycle` (genre, edit_type, metric) runs one Ring-3 cycle under the
   3 safety gates.

## Rules

- The conversation is the UI. Never ask the user to run CLI commands.
- Mask anything that looks like a secret before echoing material back.
- If a tool returns `dependency_missing`, treat it as a graceful fallback, not
  an error — tell the user the optional tool isn't installed.
