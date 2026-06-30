> Este documento es una traducción de [README.md](../../README.md). La versión en inglés es la fuente autorizada y puede estar más actualizada.
>
> ⚠️ Auto-translation pending — the English source below awaits translation via the i18n workflow.

# BuildYourOwnHarness (BYOH)

> Interactively collect a user's tacit knowledge, data, business genre, and goals — then **generate, deploy, operate, and evolve a personalized AI agent harness**.

BYOH adds a **generation layer** on top of the validated building blocks of the [epiccounty](https://github.com/epicsagas) workspace. Instead of shipping a fixed skill/memory/pipeline set, it compiles a *unique* harness per user from an interview.

## What it does

A confirmed user profile (genre + expertise + 30-day goal) drives a synthesis engine that **recombines registry skills by keyword** into an ordered pipeline, producing a `HarnessBundle` that is *not* a fixed genre template. The whole pipeline is closed-loop and gated by three safety gates (Critic / Seesaw / Stagnation) that can never be bypassed.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

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
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # static gate + dry-run gate → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # safe dist/ install (--host for live plugin dir)
byoh run <slug>
byoh evolve <slug>                       # ciclo de evolución con 3 puertas
byoh catalog index [--limit N]           # analizar el README top-100 de quemsah → ~/.byoh/catalog.json
byoh catalog search "<consulta>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### Modo dirigido por agente (servidor MCP)

`byoh serve` (`--features mcp`) inicia un servidor MCP sobre stdio para que un agente LLM **maneje BYOH** — la CLI se vuelve secundaria (inversión de control). 14 herramientas (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`) son descubribles via `tools/list`. La conversación *es* la entrevista/asistente.

```bash
cargo build --release --features mcp
byoh serve
```

## Núcleo: síntesis, vendoring y catálogo

- **Motor de síntesis** — `synthesize(profile)` combina habilidades del registro según las etiquetas del perfil, las ordena en un pipeline y fuerza un nuevo pase por las 3 puertas (sin bypass). Los pipelines orientados a objetivos (lanzamiento de producto / decisión / informe de investigación / entrega segura / …) superponen una escalera de habilidades + conjunto de agentes cuando el objetivo a 30 días coincide.
- **Vendoring de habilidades comunitarias** (RFC M3) — `byoh vendor add` obtiene un `SKILL.md` externo (ruta local o URL de git), ejecuta validación estática + sha256 y lo integra en **Ring 3** (más restringido) en tiempo de compilación via `build.rs`. Las habilidades externas se unen a la síntesis como código no confiable.
- **Catálogo de plugins** — `byoh catalog index` construye un caché offline en `~/.byoh/catalog.json` a partir del README curado [quemsah/awesome-claude-plugins](https://github.com/quemsah/awesome-claude-plugins) (top 100 por estrellas, actualizado a diario). Una sola descarga + análisis (sin rastreo página por página) y cada entrada incluye `stars` reales. Por defecto descarga primero un **bundle preconstruido por el maintainer** (un asset de GitHub Release semanal — segundos) y solo analiza el README directamente si aquel no está disponible. Tras la indexación, `catalog search` y `catalog vendor` funcionan completamente sin conexión. Durante la entrevista del asistente S2, `profile_interview` incluye automáticamente `catalog_suggestions` — hasta 5 plugins coincidentes por género que el LLM puede recomendar sin llamadas adicionales a herramientas.

  `catalog vendor` **enriquece el caché del catálogo en el momento de la importación**: tras clonar el repositorio del plugin, extrae `license` y `keywords` de `.claude-plugin/plugin.json` y registra el `genre` resuelto. Solo sobreescribe `catalog.json` cuando el valor en caché es `"unknown"` o está vacío, por lo que los resultados de `catalog search` se enriquecen con cada operación de vendoring. El agente LLM puede ejecutar de forma autónoma el flujo búsqueda → importación mediante las herramientas MCP `catalog_search` / `catalog_vendor`, o el usuario puede especificarlo directamente por CLI.

  ```bash
  # Indexación única — bundle primero, parseo de README como respaldo
  byoh catalog index                       # bundle primero, README como respaldo
  byoh catalog index --no-bundle           # parsear README directamente
  byoh catalog index --no-bundle --limit 20   # solo los 20 primeros

  # Anulación para pruebas con espejo local:
  #   BYOH_BUNDLE_URL=http://localhost:18099/catalog.json.gz byoh catalog index

  # Búsqueda sin conexión — sin red
  byoh catalog search "desarrollo orientado a pruebas" --genre developer --limit 5

  # Importar el plugin encontrado a registry/vendored/
  # license, keywords y genre se extraen automáticamente del repositorio clonado
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
