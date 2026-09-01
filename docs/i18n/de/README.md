> Dieses Dokument ist die deutsche Version von [README.md](../../../README.md). Die englische Version ist die maßgebliche Quelle.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | **Deutsch** | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Dein KI-Agent, um dich herum gebaut

*Keine generische Vorlage — ein Harness, das aus deiner Rolle, deiner Expertise und deinen Zielen kompiliert wird.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Die meisten KI-Setups händigen dir ein festes Set an Tools aus und wünschen „viel Glück." BYOH dreht das um: Es befragt dich, lernt, was du tatsächlich tust, und generiert ein personalisiertes Agent-Harness — Skills, Agents, Goal-Pipelines — das von Anfang an zu deinem Workflow passt.

## Für wen ist das?

- **Entwickler**, die einen Agenten wollen, der ihren Stack, ihren Test-Stil und ihr Auslieferungstempo bereits kennt
- **Forscher**, die Literaturrecherche, Zitationsverfolgung und Synthese miteinander verknüpft benötigen
- **Content-Ersteller**, die einen Schreibpartner wollen, der zu ihrer Stimme und Projektstruktur passt
- **Business-Analysten**, die Entscheidungsrahmen und Reporting-Pipelines brauchen, keinen reinen Chat

Wenn du je gedacht hast „Ich wünschte, meine KI würde meinen Kontext wirklich kennen" — genau das macht BYOH.

## Wie es in 60 Sekunden funktioniert

BYOH ist dafür gebaut, von deinem KI-Agent gesteuert zu werden — nicht von dir, die Befehle eintippt. Installiere das Plugin und sprich einfach. Die Konversation *ist* das Interview, der Wizard und der Build.

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # dein Agent interviewt dich, baut, füllt Lücken selbst
                           # aus und installiert — alles im Gespräch
```

In der nächsten Sitzung lädt dein Host das Harness automatisch — Agents, Skills und Goal-Pipelines, die auf dich abgestimmt sind.

## Plugin installieren (empfohlen)

Du nutzt **Claude Code, Codex oder agy**? Installiere das Plugin. Es bündelt den MCP-Server und **installiert die Binary beim ersten Laden automatisch** — keine Rust-Toolchain, kein manuelles Setup:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@byoh
```

**agy (Antigravity):**
```bash
agy plugin install https://github.com/epicsagas/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add epicsagas/BuildYourOwnHarness
codex plugin add byoh@byoh
```

### Du nutzt einen anderen MCP-kompatiblen Host?

BYOH spricht MCP, daher funktionieren auch Cursor, Zed, Continue und andere. Installiere die [Binary](#binärdatei-direkt-installieren) einmal und zeige deinen Host auf den Server:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Hinweis:** Dieses Repository bringt einen eigenen `byoh`-Marktplatz mit (.claude-plugin/marketplace.json) — eigenständig installierbar, ohne Hub.

## Binärdatei direkt installieren

Nur nötig, wenn du **nicht** das Plugin verwendest (das die Binary automatisch installiert) oder BYOH auf einem Nicht-Plugin-MCP-Host betreiben willst.

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

### Aus dem Quellcode

```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## Agent-geführter Modus — der Hauptpfad

Sobald dein Host verbunden ist, tippst du keine Befehle ein — du sprichst einfach. Dein Agent ruft BYOHs MCP-Tools direkt auf, und die Konversation *ist* das Interview, der Build und der Evolve-Zyklus:

> **Du:** *Ich bin ein Backend-Go-Entwickler, der diesen Monat eine Payments-API ausliefert. Bau mir ein Harness.*
>
> **Agent:** *(scannt dein Repo via `profile_scan`, stellt ein paar gezielte Fragen via `profile_interview`, sperrt das Genre auf `developer`)* → `build` synthetisiert ein `HarnessBundle` und klassifiziert jeden Skill als `matched` / `authored` / `skeleton` → für jedes Skeleton, das das Profil braucht (z. B. einen zahlungsspezifischen Verifikations-Skill), schreibt der Agent es direkt via `author_skill` und baut dann erneut mit `build`, um zu bestätigen → installiert Agents, Skills und eine Secure-Ship-Goal-Pipeline in Claude Code. Verfasste Inhalte bleiben über Rebuilds hinweg erhalten. Fertig — in der nächsten Sitzung spricht dein Agent bereits deinen Stack.

Derselbe Ablauf, in der vorgeschlagenen Tool-Reihenfolge:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → (author_skill / author_doc to fill skeletons) → build → install_plugin
```

`build` synthetisiert das Bundle (Compile + Preset-Injection + Static Gate) und
klassifiziert jeden Skill als `matched` (echter Preset-Body) oder `skeleton`
(Genre-Template-Platzhalter) — der Agent liest das, um zu entscheiden, ob jetzt
installiert oder das Profil zuerst iteriert wird.

Verfügbare Tools: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

Möchtest du, dass dich der Agent Schritt für Schritt durchführt? Sag einfach *„build my harness"* — der mitgelieferte `byoh-guide`-Agent orchestriert den gesamten Ablauf.

## Plugin-Katalog

Der Katalog wird aus dem README von [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) aufgebaut — einer von der Community gepflegten, nach Sternen gerankten Liste der Top-100-Claude-Plugin-Repositories. BYOH liefert ein vorgefertigtes Bundle aus (jeden **Montag 03:17 UTC** neu erstellt), sodass `byoh catalog index` in Sekunden auflöst; übergib `--no-bundle`, um die Upstream-Liste direkt zu parsen.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

Der LLM-Agent (über die MCP-Tools `catalog_search` / `catalog_vendor`) kann diesen gesamten Ablauf autonom erledigen — *„füge ein Memory-Plugin zu meinem Harness hinzu"* — oder du steuerst ihn direkt über die CLI.

Einige Begleit-Tools werden als **Referenzmaterial** (nicht als Abhängigkeiten) in die Suchergebnisse eingestreut: BYOHs eigene Execution-Layer-Tools — [alcove](https://github.com/epicsagas/alcove) (Doc-Server), [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (Vault-Automatisierung), [epic-harness](https://github.com/epicsagas/epic-harness) (Hook/Skill-Runtime) — erscheinen kontextuell (eine „doc server"- / „search backend"-Anfrage findet alcove), sodass ein Agent sie empfehlen kann, wenn relevant. Vendor eines nur, wenn du es wirklich willst; Bundles bleiben entweder Weise abhängigkeitsfrei.

## Power-User: die CLI (optional)

Jeder der obigen Abläufe ist auch vom Terminal aus erreichbar. Die CLI ist **hilfsmittelartig** — nützlich für Skripte, CI oder wenn du lieber nicht chattest — aber der agent-geführte Pfad ist der vorgesehene.

### Dein erstes Harness — über die CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # synthesize (compile + preset injection + static gate) and write the HarnessBundle; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

Das Interview selbst ist agent-geführt (das MCP-Tool `profile_interview`) — die Konversation ist das Interview, daher gibt es kein interaktives CLI-Interview. Das Static Gate von `build` (Vorhandensein der 3 Safety-Gates Critic / Seesaw / Stagnation, MCP-Schema, Hook-Eingaben) läuft immer und kann niemals umgangen werden — das Bundle ist daher vor der Auslieferung strukturell gültig. Verbesserungen nach der Installation sind eine gesprächsbasierte Retrospektive in späteren Sitzungen, kein Tool-Aufruf.

## Wie es unter der Haube funktioniert

BYOHs Synthese-Engine gleicht deine Profil-Tags gegen die Skill-Registry ab, ordnet sie in eine abhängigkeitsaufgelöste Pipeline ein und emittiert ein `HarnessBundle` — ein git-bereites Artefakt, das in das native Format jedes unterstützten Hosts gerendert wird.

- **4-Ring-Sicherheitsmodell** — Lifecycle-Spec (Ring 0) und eingebaute Pipeline-Skills (Ring 1) bis hin zu Community-/unausgewiesenen Skills (Ring 3), jeweils mit zunehmender Validierung; vendorte Skills sind sha256-gepinnt und werden zum Lese- + Embed-Zeitpunkt verifiziert
- **3-Gate-Sicherheitsfundament** — jedes Build durchläuft ein Static Gate, das die Gates Critic (Qualität), Seesaw (Regression) und Stagnation (Plateau) auf Vorhandensein prüft; kein Bypass
- **Zielorientierte Pipelines** — das Deklarieren eines 30-Tage-Ziels (Produktlaunch, Forschungsbericht, Secure Ship…) legt automatisch eine passende Skill-Leiter darüber

Architektur: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. Siehe `AGENTS.md` für die vollständige Anleitung.

## Vollständige CLI-Referenz

Die CLI ist absichtlich klein: Maschinen-Einstiegspunkte (`serve`, `catalog index` in CI, `vendor` für Maintainer) plus eine skriptbare Spiegelung des Kern-Build-Ablaufs. Interview und Evolution sind MCP-only (agent-geführt).

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

Profile und der Katalog-Cache liegen standardmäßig unter `~/.byoh` (überschreibbar mit `BYOH_HOME`).

## Bauen & Entwickeln

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

Das `mcp`-Feature (stdio-MCP-Server) ist standardmäßig aktiviert. BYOH liefert keine eingebettete Knowledge-Base — für Retrieval zeige dein generiertes Harness auf einen Doc-Server wie [alcove](https://github.com/epicsagas/alcove).

## Danksagung

BYOH steht auf den Schultern mehrerer Community-Bemühungen:

- **Plugin-Katalog** — bezogen aus [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), einer nach Sternen gerankten Community-Liste der Top-100-Claude-Plugin-Repositories. Ohne sie gäbe es den Katalog nicht.
- **Begleit-Tools** — so gestaltet, dass sie mit [alcove](https://github.com/epicsagas/alcove) (Doc-Server / RAG), [Episteme](https://github.com/epicsagas/Episteme) (Knowledge-Graph) und [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (Vault-Automatisierung) zusammenarbeiten.
- **Open-Source-Stack** — gebaut auf [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) und dem Rust-Ökosystem.

Katalog-Einträge und vendorte Community-Skills behalten ihre eigenen Lizenzen (beim Vendor-Vorgang automatisch erkannt). BYOH selbst ist Apache-2.0.

## Lizenz

Apache-2.0.
