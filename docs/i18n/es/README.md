> Este documento es la versión en español de [README.md](../../../README.md). La versión en inglés es la fuente autorizada.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | **Español** | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Tu agente de IA, construido en torno a ti

*No es una plantilla genérica — un harness compilado a partir de tu rol, tu experiencia y tus objetivos.*

<img src="assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

La mayoría de las configuraciones de IA te entregan un conjunto fijo de herramientas y te dicen "buena suerte". BYOH invierte eso: te entrevista, aprende lo que realmente haces y genera un harness de agente personalizado — skills, agents, goal pipelines — que encaja con tu flujo de trabajo desde el primer momento.

## ¿Para quién es esto?

- **Developers** que quieren un agente que ya conoce su stack, su estilo de tests y su cadencia de entrega
- **Researchers** que necesitan revisión de literatura, seguimiento de citas y síntesis integrados
- **Creators** que quieren un compañero de escritura que coincida con su voz y la estructura de su proyecto
- **Business analysts** que necesitan marcos de decisión y pipelines de reporting, no un chat genérico

Si alguna vez has pensado "Ojalá mi IA realmente conociera mi contexto" — esto es lo que hace BYOH.

## Cómo funciona en 60 segundos

BYOH está diseñado para ser conducido por tu agente de IA — no para que tú escribas comandos. Instala el plugin y luego simplemente habla. La conversación *es* la entrevista, el asistente y la construcción.

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # your agent scans your repo and compiles the result
```

En la siguiente sesión, tu host carga el harness automáticamente — agents, skills y goal pipelines ajustados a ti.

## Instalar el plugin (recomendado)

¿Usas **Claude Code, Codex o agy**? Instala el plugin. Incluye el MCP server y **auto-instala el binario en la primera carga** — sin toolchain de Rust, sin configuración manual:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

### ¿Usas cualquier otro host compatible con MCP?

BYOH habla MCP, así que Cursor, Zed, Continue y similares también funcionan. Instala el [binary](#installation) una vez y luego apunta tu host al server:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Nota:** El repo es actualmente privado. Usa las rutas anteriores. Una vez público aparecerá en el marketplace compartido `epicsagas/plugins`.

## Modo agent-led — el camino principal

Una vez que tu host está conectado, no escribes comandos — simplemente hablas. Tu agente llama a las MCP tools de BYOH directamente, y la conversación *es* la entrevista, la construcción y el ciclo de evolve:

> **Tú:** *Soy un desarrollador backend de Go que envía una API de pagos este mes. Constrúyeme un harness.*
>
> **Agent:** *(escanea tu repo vía `profile_scan`, hace algunas preguntas dirigidas vía `profile_interview`, fija el genre en `developer`)* → compila un `HarnessBundle` → instala agents, skills y un goal pipeline de secure-ship en Claude Code. Listo — en la próxima sesión, tu agente ya habla tu stack.

El mismo flujo, en el orden de herramientas sugerido:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → install_plugin
```

`build` sintetiza el bundle (compile + inyección de presets + static gate) y
clasifica cada skill como `matched` (cuerpo de preset real) o `skeleton`
(placeholder de la plantilla de género) — el agente lo lee para decidir si
instalar ahora o iterar el perfil primero.

Herramientas disponibles: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

¿Quieres que el agente te guíe a través del proceso? Solo di *"build my harness"* — el agent `byoh-guide` incluido orquesta todo el flujo.

## Catálogo de plugins

El catálogo se construye a partir del README de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — una lista mantenida por la comunidad, ordenada por estrellas, con los 100 mejores repositorios de plugins de Claude. BYOH distribuye un bundle preconstruido (reconstruido **semanalmente**, cada lunes a las 03:17 UTC) para que `byoh catalog index` se resuelva en segundos; pasa `--no-bundle` para parsear la lista directamente.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

El LLM agent (vía MCP tools `catalog_search` / `catalog_vendor`) puede realizar todo este flujo de forma autónoma — *"add a memory plugin to my harness"* — o puedes conducirlo directamente desde la CLI.

Algunas herramientas complementarias se incluyen en los resultados de búsqueda como **material de referencia** (no dependencias): las propias herramientas de capa de ejecución de BYOH — [alcove](https://github.com/epicsagas/alcove) (doc server), [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (vault automation), [epic-harness](https://github.com/epicsagas/epic-harness) (hook/skill runtime) — aparecen contextualmente (una consulta sobre "doc server" / "search backend" encuentra alcove) para que un agente pueda recomendarlas cuando sea relevante. Vendoriza una solo si realmente la quieres; los bundles se distribuyen sin dependencias de cualquier manera.

## Power users: la CLI (opcional)

Todos los flujos anteriores también son accesibles desde la terminal. La CLI es **auxiliar** — útil para scripting, CI o cuando prefieres no chatear — pero el camino agent-led es el previsto.

### Tu primer harness — desde la CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # sintetiza (compile + inyección de presets + static gate) y escribe el HarnessBundle; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

La entrevista en sí es agent-led (la MCP tool `profile_interview`) — la conversación es la entrevista, así que no hay entrevista interactiva en la CLI. El static gate del build (presencia de safety gates Critic / Seesaw / Stagnation, esquema MCP, hook input) siempre se ejecuta y nunca puede omitirse — así el bundle es estructuralmente válido antes de publicarse. La mejora post-instalación es una retrospectiva conversacional en sesiones posteriores, no una llamada a una herramienta.

## Cómo funciona por dentro

El motor de síntesis de BYOH compara tus profile tags contra el skill registry, los ordena en un pipeline con resolución de dependencias y emite un `HarnessBundle` — un artifact listo para git que se renderiza al formato nativo de cualquier host soportado.

- **Modelo de seguridad de 4 anillos** — lifecycle spec (Ring 0) y skills de pipeline integrados (Ring 1) hasta skills comunitarios/no confiables (Ring 3), cada uno con validación creciente; las skills vendorizadas se fijan con sha256 y se verifican al leer + embeber
- **Base de seguridad de 3 gates** — el static gate de cada build confirma que los gates Critic (calidad), Seesaw (regresión) y Stagnation (meseta) están presentes; sin omisión posible
- **Pipelines orientados a objetivos** — declarar un objetivo de 30 días (lanzamiento de producto, reporte de investigación, secure ship…) superpone automáticamente una skill ladder coincidente

Arquitectura: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. Consulta `AGENTS.md` para la guía completa.

## Referencia completa de la CLI

La CLI es intencionalmente pequeña: puntos de entrada para máquinas (`serve`, `catalog index` en CI, `vendor` para maintainers) más un espejo scriptable del flujo principal de construcción. La entrevista y la evolución son solo MCP (agent-led).

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

Los profiles y la caché del catálogo viven bajo `~/.byoh` por defecto (sobrescribe con `BYOH_HOME`).

## Instalación

Solo necesaria si **no** usas el plugin (que auto-instala el binario) o si quieres BYOH en un host MCP sin plugin.

### Binario (sin toolchain de Rust)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**Desde fuente:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## Construir y desarrollar

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

La feature `mcp` (stdio MCP server) está activa por defecto. BYOH no distribuye ninguna knowledge base embebida — para retrieval, apunta tu harness generado a un doc server como [alcove](https://github.com/epicsagas/alcove).

## Agradecimientos

BYOH se apoya en los hombros de varios esfuerzos comunitarios:

- **Catálogo de plugins** — proviene de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), una lista comunitaria ordenada por estrellas con los 100 mejores repositorios de plugins de Claude. Sin ella, el catálogo no existiría.
- **Herramientas complementarias** — diseñadas para interoperar con [alcove](https://github.com/epicsagas/alcove) (doc server / RAG), [Episteme](https://github.com/epicsagas/Episteme) (knowledge graph) y [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (vault automation).
- **Stack de open-source** — construido sobre [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) y el ecosistema de Rust.

Las entradas del catálogo y las skills comunitarias vendorizadas conservan sus propias licencias (detectadas automáticamente al vendorizar). BYOH en sí es Apache-2.0.

## Licencia

Apache-2.0.
