> Dieses Dokument ist die deutsche Version von [README.md](../../../README.md). Die englische Version ist die maßgebliche Quelle.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | **Deutsch** | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Dein KI-Agent, auf dich zugeschnitten

*Kein generisches Template — ein Harness, kompiliert nach deiner Rolle, Expertise und deinen Zielen.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Die meisten KI-Setups geben dir ein festes Funktionspaket und sagen „Viel Glück". BYOH dreht das um: Es interviewt dich, lernt, wie du wirklich arbeitest, und generiert einen personalisierten Agenten-Harness — Skills, Memory, Pipelines — der von Anfang an zu deinem Workflow passt.

## Für wen ist das?

- **Entwickler**, die einen Agenten wollen, der ihren Stack, Teststil und Release-Rhythmus bereits kennt
- **Forscher**, die Literaturrecherche, Zitationsverfolgung und Synthese nahtlos verbunden brauchen
- **Kreative**, die einen Schreibpartner wollen, der ihren Stil und ihre Projektstruktur versteht
- **Business-Analysten**, die Entscheidungsframeworks und Reporting-Pipelines brauchen — kein reines Chat-Tool

Wenn du dir jemals gedacht hast „ich wünschte, meine KI würde meinen Kontext wirklich kennen" — genau das macht BYOH.

## In 60 Sekunden starten

BYOH ist darauf ausgelegt, von deinem KI-Agenten gesteuert zu werden. Installiere es, verbinde deinen Host über MCP und sprich einfach — das Gespräch *ist* Interview, Wizard und Build in einem.

```
1. Install byoh              # Ein-Zeilen-Installation (siehe unten)
2. Connect your host via MCP # byoh serve — jeder MCP-kompatible Agent
3. "Build me a harness"      # dein Agent scannt dein Repo und kompiliert das Ergebnis
```

Ab der nächsten Session lädt dein Host den Harness automatisch — Agenten, Skills, Memory und Pipelines, auf dich abgestimmt.

**Lieber im Terminal?** Derselbe Ablauf über die CLI:
```
byoh profile init me        # scannt dein Projekt — nicht destruktiv, nur lesend
byoh profile interview me   # ein kurzes Gespräch über deine Rolle und Ziele
byoh compile me             # generiert deinen persönlichen Harness
byoh install me             # deployt ihn in Claude / Codex / agy
```

**Weißt du schon, was du brauchst?** Stöbere im Community-Katalog:
```bash
byoh catalog index                                 # Top-100-Plugin-Liste laden (Sekunden)
byoh catalog search "code review"                  # passende Plugins finden
byoh catalog vendor anthropics/claude-code-review  # zum Harness hinzufügen
```

## Installation

### Binary (empfohlen — kein Rust nötig)

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

### Verbinde deinen KI-Host

BYOH spricht MCP, sodass jeder MCP-kompatible Agent es steuern kann. Installiere das Binary oben, starte den Server und dein Host ruft jedes BYOH-Tool direkt auf:

```bash
byoh serve   # stdio-MCP-Server
```

Für **andere Agenten** (Cursor, Zed, Continue, …) füge `byoh` zur MCP-Konfiguration deines Hosts hinzu:
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

Nutzt du **Claude Code, Codex oder agy**? Installiere stattdessen das Plugin — es bündelt den MCP-Server und installiert das Binary beim ersten Laden automatisch (kein Rust erforderlich):

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /pfad/zu/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /pfad/zu/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

> **Hinweis:** Das Repository ist aktuell privat. Verwende die Pfade oben. Nach der Veröffentlichung erscheint es auch im gemeinsamen `epicsagas/plugins`-Marketplace.

## Agentengesteuerter Modus

Sobald dein Host verbunden ist, tippst du keine Befehle mehr ein — du sprichst einfach. Dein Agent ruft BYOHs 14 Tools direkt auf, und das Gespräch *ist* Interview, Build und Evolve-Zyklus in einem:

Verfügbare Tools: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` und weitere.

## Dein erster Harness — über die CLI

Dieselben Schritte, gesteuert vom Terminal:

### Schritt 1 — Profil
```bash
byoh profile init me --paths ./src ./docs   # Projekt automatisch analysieren
byoh profile interview me                   # ~5-minütiges Gespräch
byoh profile confirm me --genre developer   # Genre festlegen
```

BYOH fragt nach deiner Rolle, deinem Erfahrungsstand, deinen Tools und deinem 30-Tage-Ziel. Das Interview passt sich an — ein Forscher bekommt andere Fragen als ein Entwickler.

### Schritt 2 — Kompilieren & Installieren
```bash
byoh compile me          # HarnessBundle generieren (validiert und geprüft)
byoh render me --target claude   # oder: codex | agy | all
byoh install me          # sicheres Installieren nach dist/
```

### Schritt 3 — Ausführen & Weiterentwickeln
```bash
byoh run me              # Session mit aktivem Harness starten
byoh evolve me           # Harness anhand von Session-Feedback verbessern
```

`evolve` durchläuft einen 3-Gate-Zyklus (Critic / Seesaw / Stagnation), der nicht umgangen werden kann — Weiterentwicklung ist sicher und nachvollziehbar.

## Plugin-Katalog

Der Katalog bietet eine kuratierte Liste der 100 besten Claude-Plugins (nach Sternen sortiert, täglich aktualisiert), damit du Community-Skills direkt im Terminal entdecken und hinzufügen kannst.

```bash
# Einmalige Indexierung — lädt ein vorgefertigtes Bundle in Sekunden
byoh catalog index

# Offline-Suche — nach der Indexierung kein Netz nötig
byoh catalog search "memory" --genre developer --limit 5

# Plugin zum Harness hinzufügen
# Lizenz, Keywords und Genre werden automatisch aus dem geklonten Repo erkannt
byoh catalog vendor obra/superpowers --genre developer
```

Der LLM-Agent (über MCP-Tools `catalog_search` / `catalog_vendor`) kann diesen gesamten Ablauf autonom ausführen — oder du steuerst ihn direkt per CLI.

## Vollständige CLI-Referenz

```bash
# Profil
byoh profile init <slug> [--paths ...]       # nicht-destruktiver Projekt-Scan
byoh profile interview <slug>                # geführtes Interview
byoh profile confirm <slug> --genre <g>      # Profil bestätigen und sperren

# Build
byoh compile <slug> [--dry-run]              # validieren + HarnessBundle generieren
byoh render <slug> --target <host>           # claude | codex | agy | all
byoh install <slug> [--host <dir>]           # nach dist/ oder Live-Plugin-Verzeichnis deployen

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

## Wie es unter der Haube funktioniert

BYOHs Synthese-Engine gleicht deine Profil-Tags mit dem Skill-Registry ab, ordnet sie in einer abhängigkeitsaufgelösten Pipeline und erzeugt ein `HarnessBundle` — ein git-fertiges Artefakt, das im nativen Format jedes unterstützten Hosts gerendert wird.

- **4-Ring-Sicherheitsmodell** — von integrierten Skills (Ring 1) bis zu Community-/nicht vertrauenswürdigen Skills (Ring 4), jeweils mit eskalierender Validierung
- **3-Gate-Evolution** — jeder `evolve`-Zyklus durchläuft Critic (Qualität), Seesaw (Regression) und Stagnation (Plateau); kein Bypass möglich
- **Zielorientierte Pipelines** — ein 30-Tage-Ziel (Produktlaunch, Forschungsbericht, sicheres Deployment…) fügt automatisch eine passende Skill-Leiter hinzu

Architektur: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Vollständige Anleitung in `AGENTS.md`.

## Entwickeln & Bauen

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # Unit- + E2E-Tests
cvp                               # parallel: check → clippy → test → fmt → build
```

Das `mcp`-Feature (stdio-MCP-Server) ist standardmäßig aktiviert. BYOH wird ohne eingebettete Dokumentbasis ausgeliefert — für den Abruf kannst du deinen generierten Harness auf einen Doc-Server wie [alcove](https://github.com/epicsagas/alcove) zeigen lassen.

## Lizenz

Apache-2.0.
