> Dieses Dokument ist eine Übersetzung von [README.md](../../README.md). Die englische Version ist maßgeblich und kann aktueller sein.
>
# BuildYourOwnHarness (BYOH)

> Erfasst interaktiv das Wissen, die Daten, das Genre und die Ziele eines Nutzers — und **generiert, deployt, betreibt und entwickelt einen personalisierten KI-Agenten-Harness**.

BYOH ergänzt die validierten Bausteine des [epiccounty](https://github.com/epicsagas)-Workspace um eine **Generierungsschicht**. Statt ein fixes Skill-/Memory-/Pipeline-Set auszuliefern, kompiliert es auf Basis eines Interviews einen *einzigartigen* Harness pro Nutzer.

## Was es tut

Ein bestätigtes Nutzerprofil (Genre + Expertise + 30-Tage-Ziel) treibt eine Synthese-Engine an, die **Registry-Skills nach Stichwörtern rekombiniert** und in eine geordnete Pipeline überführt — das Ergebnis ist ein `HarnessBundle`, das *kein* festes Genre-Template ist. Die gesamte Pipeline ist geschlossen und durch drei Sicherheitsgates (Critic / Seesaw / Stagnation) abgesichert, die nie umgangen werden können.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## Build & Verifizierung

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                       # unit + e2e
./target/release/byoh --help
```

Hexagonale Architektur: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`.

## CLI

```bash
byoh profile init <slug> [--paths ...]   # S1 Autoscan (nicht-destruktiv)
byoh profile interview <slug>            # S2 Interview (Vorschlag + Council)
byoh profile confirm <slug> --genre <g>  # S3 Wizard-Bestätigung
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # statisches Gate + Dry-Run-Gate → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # sicheres dist/-Install (--host für live Plugin-Dir)
byoh run <slug>
byoh evolve <slug>                       # 3-Gate-Evolutionszyklus
byoh catalog index [--limit N]           # quemsah Top-100 README parsen → ~/.byoh/catalog.json
byoh catalog search "<Suchanfrage>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### Agent-gesteuerter Modus (MCP-Server)

`byoh serve` (`--features mcp`) startet einen stdio-MCP-Server, damit ein LLM-Agent **BYOH steuert** — das CLI wird sekundär (Kontrollumkehr). 14 Tools (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`) sind über `tools/list` abrufbar. Das Gespräch *ist* das Interview/der Wizard.

```bash
cargo build --release --features mcp
byoh serve
```

## Kern: Synthese, Vendoring und Katalog

- **Synthese-Engine** — `synthesize(profile)` gleicht Registry-Skills mit Profil-Tags ab, ordnet sie in eine Pipeline und erzwingt einen erneuten 3-Gate-Durchlauf (kein Bypass). Zielorientierte Pipelines (Produktlaunch / Entscheidung / Forschungsbericht / Secure-Ship / …) legen eine Skill-Leiter und ein Agenten-Set überlagert, wenn das 30-Tage-Ziel übereinstimmt.
- **Community-Skill-Vendoring** (RFC M3) — `byoh vendor add` lädt ein externes `SKILL.md` (lokaler Pfad oder Git-URL), führt statische Validierung + sha256 durch und bettet es zur Build-Zeit über `build.rs` in **Ring 3** (am stärksten eingeschränkt) ein. Externe Skills nehmen als nicht vertrauenswürdiger Code an der Synthese teil.
- **Plugin-Katalog** — `byoh catalog index` baut einen Offline-Cache unter `~/.byoh/catalog.json` aus dem kuratierten [quemsah/awesome-claude-plugins](https://github.com/quemsah/awesome-claude-plugins) README (Top 100 nach Stars, täglich aktualisiert). Ein einzelner Abruf + Parse (kein seitenweises Crawlen), und jeder Eintrag trägt echte `stars`. Standardmäßig lädt es zuerst ein **vorgefertigtes Bundle des Maintainers** (ein wöchentliches GitHub-Release-Asset — Sekunden) und parst das README nur selbst, wenn jenes unerreichbar ist. Nach der Indizierung funktionieren `catalog search` und `catalog vendor` vollständig offline. Während des S2-Wizard-Interviews fügt `profile_interview` automatisch `catalog_suggestions` hinzu — bis zu 5 genre-passende Plugins, die das LLM ohne zusätzliche Tool-Aufrufe empfehlen kann.

  ```bash
  # Einmalige Indexierung (Netzwerk; ~24 000 Seiten)
  byoh catalog index --limit 500          # klein anfangen; 0 = vollständiger Crawl

  # Offline-Suche — kein Netzwerk
  byoh catalog search "testgetriebene Entwicklung" --genre developer --limit 5

  # Gefundenes Plugin in registry/vendored/ einbetten
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust-Implementierung der Generierungsschicht: Profiler + Interview + Genre-Templates + Compiler (4-Ring, MCP-Tool-Codegen, statisches Gate) + Evolutions-Engine + selbstständige RAG (optionales `native-rag`-Feature) + MCP-Server (optionales `mcp`-Feature). Architekturübersicht in `AGENTS.md`.

Die RAG-Schicht ist eine **persistente Wissensbasis**: `byoh index` speichert den Genre-Index und einen Corpus-Sidecar unter `$BYOH_HOME/indexes/`, und ein späteres `byoh search` (oder das `rag_search`-MCP-Tool) ohne `--corpus` nutzt ihn via `load_index` — ohne erneutes Einbetten. Re-Indexierung ist **inkrementell** — ein Content-Hash-Manifest bettet nur hinzugefügte/geänderte Dokumente neu ein und verwirft entfernte (gemeldet als `+a ~c -r`); `--force` erzwingt einen vollständigen Neuaufbau.

## Lizenz

Apache-2.0.
