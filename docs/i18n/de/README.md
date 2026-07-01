> Dieses Dokument ist die deutsche Version von [README.md](../../../README.md). Die englische Version ist die maßgebliche Quelle.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | **Deutsch** | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Dein KI-Agent, auf dich zugeschnitten

*Kein generisches Template — ein Harness, kompiliert nach deiner Rolle, Expertise und deinen Zielen.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Die meisten KI-Setups geben dir ein festes Funktionspaket und sagen „Viel Glück." BYOH dreht das um: Es interviewt dich, lernt, wie du wirklich arbeitest, und generiert einen personalisierten Agenten-Harness — Skills, Memory, Pipelines — der von Anfang an zu deinem Workflow passt.

## Für wen ist das?

- **Entwickler**, die einen Agenten wollen, der ihren Stack, Teststil und Release-Rhythmus bereits kennt
- **Forscher**, die Literaturrecherche, Zitationsverfolgung und Synthese als integriertes Ganzes benötigen
- **Kreative**, die einen Schreibpartner wollen, der ihren Stil und ihre Projektstruktur versteht
- **Business-Analysten**, die Entscheidungsframeworks und Reporting-Pipelines brauchen — kein reines Chat-Tool

Wenn du dir jemals gedacht hast „ich wünschte, meine KI würde meinen Kontext wirklich kennen" — genau das macht BYOH.

## In 60 Sekunden starten

BYOH ist darauf ausgelegt, von deinem KI-Agenten gesteuert zu werden — nicht von dir, die Befehle eintippt. Installiere das Plugin und sprich einfach. Das Gespräch *ist* zugleich Interview, Assistent und Build-Vorgang.

```
1. Install the plugin      # Claude Code / Codex / agy — installiert das Binary automatisch
2. „Build me a harness"    # dein Agent scannt dein Repo und kompiliert das Ergebnis
```

Ab der nächsten Session lädt dein Host den Harness automatisch — Agenten, Skills, Memory und Pipelines, auf dich abgestimmt.

## Plugin installieren (empfohlen)

Nutzt du **Claude Code, Codex oder agy**? Installiere das Plugin. Es bündelt den MCP-Server und **installiert das Binary beim ersten Laden automatisch** — kein Rust-Toolchain, kein manuelles Setup:

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

### Du nutzt einen anderen MCP-kompatiblen Host?

BYOH spricht MCP, sodass auch Cursor, Zed, Continue und andere funktionieren. Installiere die [Binärdatei](#installation) einmal und verweise deinen Host auf den Server:

```bash
byoh serve   # stdio-MCP-Server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Hinweis:** Das Repository ist aktuell privat. Verwende die Pfade oben. Nach der Veröffentlichung erscheint es auch im gemeinsamen `epicsagas/plugins`-Marketplace.

## Agentengesteuerter Modus — der Hauptpfad

Sobald dein Host verbunden ist, tippst du keine Befehle mehr ein — du sprichst einfach. Dein Agent ruft die MCP-Tools von BYOH direkt auf, und das Gespräch *ist* Interview, Build und Evolve-Zyklus in einem:

> **Du:** *Ich bin Backend-Go-Entwickler und liefere diesen Monat eine Payments-API aus. Bau mir einen Harness.*
>
> **Agent:** *(scannt dein Repo via `profile_scan`, stellt ein paar gezielte Fragen via `profile_interview`, sperrt das Genre auf `developer`)* → kompiliert ein `HarnessBundle` → installiert Agenten, Skills, Memory und eine Secure-Ship-Pipeline in Claude Code. Fertig — in der nächsten Session spricht dein Agent bereits deinen Stack.

Derselbe Ablauf, in der vorgeschlagenen Werkzeug-Reihenfolge:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → compile → compile_dry_run → render_plugin → install_plugin
           → (optional) registry_clone_skill → (später) evolve_cycle
```

Verfügbare Tools: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `genre_list`, `compile`, `compile_dry_run`, `render_plugin`, `install_plugin`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`.

Möchtest du, dass dich der Agent Schritt für Schritt begleitet? Sag einfach *„build my harness"* — der mitgelieferte `byoh-guide`-Agent orchestriert den gesamten Ablauf.

## Plugin-Katalog

Der Katalog wird aus dem README von [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) erstellt — einer von der Community gepflegten, nach Sternen geordneten Liste der Top-100-Claude-Plugin-Repositories. BYOH liefert ein vorgefertigtes Bundle (**wöchentlich** neu erstellt, jeden Montag 03:17 UTC), sodass `byoh catalog index` in Sekunden fertig ist; mit `--no-bundle` wird die Upstream-Liste direkt geparst.

```bash
# Einmalige Indexierung — lädt ein vorgefertigtes Bundle in Sekunden
byoh catalog index

# Offline-Suche — nach der Indexierung kein Netz nötig
byoh catalog search "memory" --genre developer --limit 5

# Plugin zum Harness hinzufügen
# Lizenz, Keywords und Genre werden automatisch aus dem geklonten Repo erkannt
byoh catalog vendor obra/superpowers --genre developer
```

Der LLM-Agent (über die MCP-Tools `catalog_search` / `catalog_vendor`) kann diesen gesamten Ablauf autonom ausführen — *„füge ein Memory-Plugin zu meinem Harness hinzu"* — oder du steuerst ihn direkt per CLI.

## Power-User: die CLI (optional)

Jeder der obigen Abläufe lässt sich auch vom Terminal aus erreichen. Die CLI ist **eine Ergänzung** — nützlich für Skripte, CI oder wenn du lieber nicht chattest — aber der agentengesteuerte Weg ist der vorgesehene.

### Dein erster Harness — über die CLI

```bash
byoh profile init me --paths ./src ./docs   # scannt dein Projekt automatisch
byoh profile interview me                   # ~5-minütiges Gespräch
byoh profile confirm me --genre developer   # Genre festlegen

byoh compile me --no-dry-run                # validiert + schreibt das HarnessBundle (dry-run ist Standard)
byoh render me --target claude              # oder: codex | agy | all (Standard: all)
byoh install me --scope local               # rendert nach dist/, aktiviert nur im .claude/ dieses Projekts
byoh install me --scope global              # ...oder ~/.claude + ~/.codex + ~/.gemini (früher --host)
byoh install me --scope publish             # ...oder LICENSE + .gitignore hinzufügen und git-Anweisungen ausgeben

byoh run me                                 # Session mit aktivem Harness starten
byoh evolve me                              # Harness anhand von Session-Feedback verbessern
```

BYOH fragt nach deiner Rolle, deinem Erfahrungsstand, deinen Tools und deinem 30-Tage-Ziel. Das Interview passt sich an — ein Forscher bekommt andere Fragen als ein Entwickler. `evolve` durchläuft einen 3-Gate-Zyklus (Critic / Seesaw / Stagnation), der nicht umgangen werden kann — Weiterentwicklung ist sicher und nachvollziehbar.

## Wie es unter der Haube funktioniert

BYOHs Synthese-Engine gleicht deine Profil-Tags mit der Skill-Registry ab, ordnet sie in einer abhängigkeitsaufgelösten Pipeline an und erzeugt ein `HarnessBundle` — ein git-fertiges Artefakt, das im nativen Format jedes unterstützten Hosts gerendert wird.

- **4-Ring-Sicherheitsmodell** — von integrierten Skills (Ring 1) bis zu Community-/nicht vertrauenswürdigen Skills (Ring 4), jeweils mit eskalierender Validierung
- **3-Gate-Evolution** — jeder `evolve`-Zyklus durchläuft Critic (Qualität), Seesaw (Regression) und Stagnation (Plateau); kein Bypass möglich
- **Zielorientierte Pipelines** — ein 30-Tage-Ziel (Produktlaunch, Forschungsbericht, sicheres Deployment…) legt automatisch eine passende Skill-Leiter darüber

Architektur: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Vollständige Anleitung in `AGENTS.md`.

## Vollständige CLI-Referenz

```bash
# Profil
byoh profile init <slug> [--paths ...]       # nicht-destruktiver Projekt-Scan
byoh profile interview <slug>                # geführtes Interview
byoh profile confirm <slug> --genre <g>      # Profil bestätigen und sperren

# Build
byoh compile <slug> [--no-dry-run]          # dry-run ist Standard; --no-dry-run schreibt das Bundle
byoh render <slug> [--target <host>]        # claude | codex | agy | all (Standard: all)
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # dist/-Baum; --scope bestimmt das Ziel (local=dieses Projekt, global=HOME, publish=+LICENSE/.gitignore+git-Schritte). --host ist veraltet für --scope global.

# Ausführen & Weiterentwickeln
byoh run <slug>
byoh evolve <slug>

# Community-Skills
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Katalog
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<Suchanfrage>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Installation

Nur nötig, wenn du **nicht** das Plugin verwendest (das das Binary automatisch installiert) oder BYOH auf einem MCP-Host ohne Plugin-Unterstützung nutzen willst.

### Binärdatei (kein Rust-Toolchain nötig)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**Aus dem Quellcode:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # Installation prüfen
```

## Bauen & Entwickeln

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # Unit- + E2E-Tests
cvp                               # parallel: check → clippy → test → fmt → build
```

Das `mcp`-Feature (stdio-MCP-Server) ist standardmäßig aktiviert. BYOH wird ohne eingebettete Dokumentbasis ausgeliefert — für den Abruf kannst du deinen generierten Harness auf einen Doc-Server wie [alcove](https://github.com/epicsagas/alcove) zeigen lassen.

## Danksagung

BYOH steht auf den Schultern mehrerer Community-Projekte:

- **Plugin-Katalog** — stammt aus [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), einer nach Sternen geordneten Community-Liste der Top-100-Claude-Plugin-Repositories. Ohne sie gäbe es den Katalog nicht.
- **Begleit-Tools** — darauf ausgelegt, mit [alcove](https://github.com/epicsagas/alcove) (Doc-Server / RAG), [Episteme](https://github.com/epicsagas/Episteme) (Knowledge Graph) und [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (Vault-Automatisierung) zusammenzuarbeiten.
- **Open-Source-Stack** — gebaut auf [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) und dem Rust-Ökosystem.

Katalog-Einträge und integrierte Community-Skills behalten ihre eigenen Lizenzen (beim Integrieren automatisch erkannt). BYOH selbst ist Apache-2.0.

## Lizenz

Apache-2.0.
