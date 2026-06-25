# registry/presets/

Vetted skill presets that the `registry_clone_skill` MCP tool injects into a
genre bundle. This is the **clone** path of BYOH's "generate or clone" model:
instead of (or in addition to) letting the compiler *generate* a skill from a
genre template, a verified skill body is *cloned* into the bundle.

## Scope (local-only)

- Presets are **embedded at compile time** (`include_str!`) — zero runtime file
  or network dependency.
- There is **no `git clone`** and no network fetch. BYOH pulls nothing from the
  internet to assemble a harness (spec §Out). Network/remote registries are
  explicitly out of scope for PR #3.
- Adding a preset = add a `.md` under `<genre>/` and wire it in
  `src/deploy/presets.rs::preset_body`.

## Layout

```
registry/presets/
├── developer/
│   ├── tdd.md
│   └── debug.md
└── creator/
    └── continuity.md
```

## Preset format

Each `.md` follows the SKILL.md convention the compiler emits — YAML frontmatter
(`name`, `description`) + a 4-section body: **Process / Anti-Rationalization /
Evidence / Red Flags**. See any file for the template.

## Injection semantics

`inject_preset(bundle, genre, skill_id)` dedupes by skill `id`: if the bundle
already contains a skill with that id (e.g. the base template's `tdd`), the
preset **augments** it (replaces the body); otherwise it **clones** a new
`SkillSpec` into Ring 2 (quality). Generate and clone coexist — they never
duplicate.
