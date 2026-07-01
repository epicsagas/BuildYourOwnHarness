> Este documento es la versión en español de [README.md](../../../README.md). La versión en inglés es la fuente autorizada.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | **Español** | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Tu agente de IA, hecho a tu medida

*No una plantilla genérica — un harness compilado según tu rol, experiencia y objetivos.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

La mayoría de las herramientas de IA te dan un conjunto fijo de funciones y dicen "arréglate". BYOH hace lo contrario: te entrevista, aprende cómo trabajas de verdad y genera un harness de agente personalizado — habilidades, memoria, pipelines — que encaja con tu flujo de trabajo desde el primer momento.

## ¿Para quién es?

- **Desarrolladores** que quieren un agente que ya conoce su stack, estilo de pruebas y cadencia de entrega
- **Investigadores** que necesitan revisión bibliográfica, seguimiento de citas y síntesis conectados entre sí
- **Creadores** que quieren un compañero de escritura que se adapte a su voz y estructura de proyecto
- **Analistas de negocio** que necesitan marcos de decisión y pipelines de informes, no solo chat

Si alguna vez has pensado "ojalá mi IA supiera realmente mi contexto" — eso es exactamente lo que hace BYOH.

## Cómo funciona en 60 segundos

BYOH está diseñado para que lo gestione tu agente de IA — no para que tú escribas comandos. Instala el plugin y simplemente habla. La conversación *es* la entrevista, el asistente y la compilación.

```
1. Instala el plugin       # Claude Code / Codex / agy — instala el binario automáticamente
2. "Build me a harness"    # tu agente escanea tu repo y compila el resultado
```

En la siguiente sesión tu host carga el harness automáticamente — agentes, habilidades, memoria y pipelines ajustados a ti.

## Instala el plugin (recomendado)

¿Usas **Claude Code, Codex o agy**? Instala el plugin. Agrupa el servidor MCP e **instala el binario automáticamente en la primera carga** — sin toolchain de Rust, sin configuración manual:

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

BYOH habla MCP, así que Cursor, Zed, Continue y similares también funcionan. Instala el [binario](#installation) una vez y apunta tu host al servidor:

```bash
byoh serve   # servidor MCP por stdio
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Nota:** El repositorio es actualmente privado. Usa las rutas indicadas. Una vez público, aparecerá en el marketplace compartido `epicsagas/plugins`.

## Modo gestionado por el agente — el camino principal

Una vez que tu host está conectado, no escribes comandos — simplemente hablas. Tu agente llama directamente a las herramientas MCP de BYOH, y la conversación *es* la entrevista, la compilación y el ciclo de evolución:

> **Tú:** *Soy desarrollador backend de Go y este mes voy a sacar a producción una API de pagos. Constrúyeme un harness.*
>
> **Agente:** *(escanea tu repo vía `profile_scan`, te hace unas preguntas concretas vía `profile_interview`, fija el género en `developer`)* → compila un `HarnessBundle` → instala agentes, habilidades, memoria y un pipeline de entrega segura en Claude Code. Listo — en la próxima sesión, tu agente ya habla tu stack.

Ese mismo flujo, en el orden de herramientas sugerido:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → rag_index / rag_search → compile → compile_dry_run
           → (opcional) registry_clone_skill → (más tarde) evolve_cycle
```

Herramientas disponibles: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `compile_dry_run`, `evolve_cycle`, `genre_list`, `rag_index`, `rag_search`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` y más.

¿Quieres que el agente te guíe paso a paso? Solo di *"build my harness"* — el agente `byoh-guide` incluido orquesta todo el flujo.

## Catálogo de plugins

El catálogo ofrece una lista curada de los 100 mejores plugins de Claude (ordenados por estrellas, actualizada diariamente) para descubrir y añadir habilidades de la comunidad sin salir de la conversación.

```bash
# Indexación inicial — descarga un bundle precompilado en segundos
byoh catalog index

# Búsqueda offline — sin red tras la indexación
byoh catalog search "memoria" --genre developer --limit 5

# Añadir un plugin al harness
# licencia, keywords y género se detectan automáticamente del repo clonado
byoh catalog vendor obra/superpowers --genre developer
```

El agente LLM (vía herramientas MCP `catalog_search` / `catalog_vendor`) puede ejecutar todo este flujo de forma autónoma — *"añade un plugin de memoria a mi harness"* — o puedes manejarlo directamente desde la CLI.

## Usuarios avanzados: la CLI (opcional)

Todos los flujos anteriores también son accesibles desde el terminal. La CLI es **auxiliar** — útil para scripting, CI o cuando prefieres no chatear — pero el camino gestionado por el agente es el previsto.

### Tu primer harness — desde la CLI

```bash
byoh profile init yo --paths ./src ./docs   # escanea tu proyecto automáticamente
byoh profile interview yo                   # conversación de ~5 minutos
byoh profile confirm yo --genre developer   # fija tu género de trabajo

byoh compile yo                             # genera el HarnessBundle (validado y comprobado)
byoh render yo --target claude              # o: codex | agy | all
byoh install yo                             # instalación segura en dist/

byoh run yo                                 # lanza tu sesión con el harness activo
byoh evolve yo                              # mejora el harness según el feedback de tus sesiones
```

BYOH te pregunta sobre tu rol, nivel de experiencia, herramientas y objetivo a 30 días. La entrevista se adapta — un investigador recibe preguntas distintas a las de un desarrollador. `evolve` ejecuta un ciclo de 3 puertas (Critic / Seesaw / Stagnation) que nunca puede saltarse — la evolución es segura y auditable.

## Cómo funciona por dentro

El motor de síntesis de BYOH cruza tus etiquetas de perfil con el registro de habilidades, las ordena en un pipeline con dependencias resueltas y emite un `HarnessBundle` — un artefacto listo para git que se renderiza en el formato nativo de cada host compatible.

- **Modelo de seguridad de 4 anillos** — desde habilidades integradas (Anillo 1) hasta habilidades de la comunidad/no confiables (Anillo 4), cada una con validación progresiva
- **Evolución de 3 puertas** — cada ciclo `evolve` pasa por Critic (calidad), Seesaw (regresión) y Stagnation (estancamiento); sin atajos posibles
- **Pipelines orientados a objetivos** — declarar un objetivo a 30 días (lanzamiento de producto, informe de investigación, entrega segura…) superpone automáticamente una escalera de habilidades a medida

Arquitectura hexagonal: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Consulta `AGENTS.md` para la guía completa.

## Referencia CLI completa

```bash
# Perfil
byoh profile init <slug> [--paths ...]       # escaneo no destructivo del proyecto
byoh profile interview <slug>                # entrevista guiada
byoh profile confirm <slug> --genre <g>      # confirmar y fijar perfil

# Compilación
byoh compile <slug> [--dry-run]              # validar + generar HarnessBundle
byoh render <slug> --target <host>           # claude | codex | agy | all
byoh install <slug> [--host <dir>]           # desplegar en dist/ o directorio live

# Ejecución y evolución
byoh run <slug>
byoh evolve <slug>

# Habilidades de la comunidad
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catálogo
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<consulta>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Instalación

Solo necesaria si **no** usas el plugin (que instala el binario automáticamente) o si quieres BYOH en un host MCP sin plugin.

### Binario (no necesitas toolchain de Rust)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**Desde el código fuente:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verificar instalación
```

## Compilar y desarrollar

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unitarios + e2e
cvp                               # paralelo: check → clippy → test → fmt → build
```

La feature `mcp` (servidor MCP por stdio) está activada por defecto. BYOH no incluye ningún repositorio de documentos integrado — para recuperación, apunta tu harness generado a un servidor de documentación como [alcove](https://github.com/epicsagas/alcove).

## Licencia

Apache-2.0.
