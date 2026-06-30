> Este documento es la versión en español de [README.md](../../../README.md). La versión en inglés es la fuente autorizada.

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | **Español** | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

> **Tu agente de IA, hecho a tu medida** — no una plantilla genérica, sino un harness compilado según tu rol, experiencia y objetivos.

La mayoría de las herramientas de IA te dan un conjunto fijo de funciones y dicen "arréglate". BYOH hace lo contrario: te hace una breve entrevista, aprende cómo trabajas de verdad y genera un harness de agente personalizado — habilidades, memoria, pipelines — que encaja con tu flujo de trabajo desde el primer momento.

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

## ¿Para quién es?

- **Desarrolladores** que quieren un agente que ya conoce su stack, estilo de pruebas y cadencia de entrega
- **Investigadores** que necesitan revisión bibliográfica, seguimiento de citas y síntesis conectados entre sí
- **Creadores** que quieren un compañero de escritura que se adapte a su voz y estructura de proyecto
- **Analistas de negocio** que necesitan marcos de decisión y pipelines de informes, no solo chat

Si alguna vez has pensado "ojalá mi IA supiera realmente mi contexto" — eso es exactamente lo que hace BYOH.

## Empieza en 60 segundos

```bash
byoh profile init yo        # escanea tu proyecto — no destructivo, solo lectura
byoh profile interview yo   # una breve conversación sobre tu rol y objetivos
byoh compile yo             # genera tu harness personal
byoh install yo             # lo despliega en Claude / Codex / agy
```

En la siguiente sesión tu host carga el harness automáticamente — agentes, habilidades, memoria y pipelines ajustados a ti.

**¿Ya sabes lo que necesitas?** Explora el catálogo de la comunidad:
```bash
byoh catalog index                                  # descarga la lista top-100 (segundos)
byoh catalog search "revisión de código"            # encuentra plugins relevantes
byoh catalog vendor anthropics/claude-code-review   # añade uno a tu harness
```

## Instalación

### Binario (recomendado — no necesitas Rust)

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

### Cargar el plugin en tu host de IA

BYOH se distribuye como un plugin políglota compatible con Claude Code, Codex y agy — un solo repositorio, los tres hosts.

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /ruta/a/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /ruta/a/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

El plugin instala el binario `byoh` automáticamente en la primera carga — sin necesidad de Rust.

> **Nota:** El repositorio es actualmente privado. Una vez público, aparecerá en el marketplace compartido `epicsagas/plugins`.

## Tu primer harness — paso a paso

### Paso 1 — Perfil
```bash
byoh profile init yo --paths ./src ./docs   # escanea tu proyecto automáticamente
byoh profile interview yo                   # conversación de ~5 minutos
byoh profile confirm yo --genre developer   # confirma tu género de trabajo
```

BYOH te pregunta sobre tu rol, nivel de experiencia, herramientas y objetivo a 30 días. La entrevista se adapta — un investigador recibe preguntas distintas a las de un desarrollador.

### Paso 2 — Compilar e instalar
```bash
byoh compile yo                    # genera el HarnessBundle (validado y comprobado)
byoh render yo --target claude     # o: codex | agy | all
byoh install yo                    # instalación segura en dist/
```

### Paso 3 — Ejecutar y evolucionar
```bash
byoh run yo       # lanza tu sesión con el harness activo
byoh evolve yo    # mejora el harness según el feedback de tus sesiones
```

`evolve` ejecuta un ciclo de 3 puertas (Critic / Seesaw / Stagnation) que nunca puede saltarse — la evolución es segura y auditable.

## Catálogo de plugins

El catálogo ofrece una lista curada de los 100 mejores plugins de Claude (ordenados por estrellas, actualizada diariamente) para descubrir y añadir habilidades de la comunidad sin salir del terminal.

```bash
# Indexación inicial — descarga un bundle precompilado en segundos
byoh catalog index

# Búsqueda offline — sin red tras la indexación
byoh catalog search "memoria" --genre developer --limit 5

# Añadir un plugin al harness
# licencia, keywords y género se detectan automáticamente del repo clonado
byoh catalog vendor obra/superpowers --genre developer
```

El agente LLM (vía herramientas MCP `catalog_search` / `catalog_vendor`) puede ejecutar todo este flujo de forma autónoma, o puedes manejarlo directamente desde la CLI.

## Modo agente

`byoh serve` inicia un servidor MCP por stdio. En lugar de escribir comandos tú mismo, tu host de IA llama directamente a las 14 herramientas de BYOH — la conversación *es* la entrevista, el asistente y la ejecución.

```bash
byoh serve   # Claude / Codex / agy se conecta y gestiona todo
```

Herramientas disponibles: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `rag_index`, `rag_search`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` y más.

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

# Base de conocimiento (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<consulta>" [--genre <g>] [--k N]
```

## Cómo funciona por dentro

El motor de síntesis de BYOH cruza tus etiquetas de perfil con el registro de habilidades, las ordena en un pipeline con dependencias resueltas y emite un `HarnessBundle` — un artefacto listo para git que se renderiza en el formato nativo de cada host compatible.

- **Modelo de seguridad de 4 anillos** — desde habilidades integradas (Anillo 1) hasta habilidades de la comunidad/no confiables (Anillo 4), cada una con validación progresiva
- **Evolución de 3 puertas** — cada ciclo `evolve` pasa por Critic (calidad), Seesaw (regresión) y Stagnation (estancamiento); sin atajos posibles
- **RAG persistente** — re-embedding incremental ante cambios (`+añadidos ~modificados -eliminados`); la búsqueda reutiliza el índice guardado sin re-embeber
- **Pipelines orientados a objetivos** — declarar un objetivo a 30 días (lanzamiento de producto, informe de investigación, entrega segura…) superpone automáticamente una escalera de habilidades a medida

Arquitectura hexagonal: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Consulta `AGENTS.md` para la guía completa.

## Compilar y desarrollar

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unitarios + e2e
cvp                               # paralelo: check → clippy → test → fmt → build
```

Features opcionales: `--features mcp` (servidor MCP), `--features native-rag` (embeddings locales), `--features rag-openai` (embeddings de OpenAI). Los binarios de release incluyen todas las features.

## Licencia

Apache-2.0.
