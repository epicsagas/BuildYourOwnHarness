> Dieses Dokument ist die deutsche Version von [README.md](../../../README.md). Die englische Version ist die maßgebliche Quelle.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | **Deutsch** | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Dein KI-Agent, auf dich zugeschnitten

*Kein generisches Template — ein Harness, kompiliert nach deiner Rolle, Expertise und deinen Zielen.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Die meisten KI-Tools geben dir ein festes Funktionspaket und sagen „Viel Glück". BYOH dreht das um: Ein kurzes Interview lernt, wie du wirklich arbeitest, und generiert einen personalisierten Agenten-Harness — Skills, Memory, Pipelines — der von Anfang an zu deinem Workflow passt.

## Für wen ist das?

- **Entwickler**, die einen Agenten wollen, der ihren Stack, Teststil und Release-Rhythmus bereits kennt
- **Forscher**, die Literaturrecherche, Zitationsverfolgung und Synthese nahtlos verbunden brauchen
- **Kreative**, die einen Schreibpartner wollen, der ihren Stil und ihre Projektstruktur versteht
- **Business-Analysten**, die Entscheidungsframeworks und Reporting-Pipelines brauchen — kein reines Chat-Tool

Wenn du dir jemals gedacht hast „ich wünschte, meine KI würde meinen Kontext wirklich kennen" — genau das macht BYOH.

## In 60 Sekunden starten

```bash
byoh profile init ich        # scannt dein Projekt — nicht destruktiv, nur lesend
byoh profile interview ich   # ein kurzes Gespräch über deine Rolle und Ziele
byoh compile ich             # generiert deinen persönlichen Harness
byoh install ich             # deployt ihn in Claude / Codex / agy
```

Ab der nächsten Session lädt dein Host den Harness automatisch — Agenten, Skills, Memory und Pipelines, auf dich abgestimmt.

**Weißt du schon, was du brauchst?** Stöbere direkt im Community-Katalog:
```bash
byoh catalog index                                  # Top-100-Plugin-Liste laden (Sekunden)
byoh catalog search "code review"                   # passende Plugins finden
byoh catalog vendor anthropics/claude-code-review   # zum Harness hinzufügen
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

### Plugin in deinen KI-Host laden

BYOH ist ein polyglotter Plugin, der mit Claude Code, Codex und agy funktioniert — ein Repository, alle drei Hosts.

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

Das Plugin installiert das `byoh`-Binary beim ersten Laden automatisch — Rust ist nicht erforderlich.

> **Hinweis:** Das Repository ist aktuell privat. Nach der Veröffentlichung erscheint es auch im gemeinsamen `epicsagas/plugins`-Marketplace.

## Dein erster Harness — Schritt für Schritt

### Schritt 1 — Profil
```bash
byoh profile init ich --paths ./src ./docs   # Projekt automatisch analysieren
byoh profile interview ich                   # ~5-minütiges Gespräch
byoh profile confirm ich --genre developer   # Genre festlegen
```

BYOH fragt nach deiner Rolle, deinem Erfahrungsstand, deinen Tools und deinem 30-Tage-Ziel. Das Interview passt sich an — ein Forscher bekommt andere Fragen als ein Entwickler.

### Schritt 2 — Kompilieren & Installieren
```bash
byoh compile ich                    # HarnessBundle generieren (validiert und geprüft)
byoh render ich --target claude     # oder: codex | agy | all
byoh install ich                    # sicheres Installieren nach dist/
```

### Schritt 3 — Ausführen & Weiterentwickeln
```bash
byoh run ich       # Session mit aktivem Harness starten
byoh evolve ich    # Harness anhand von Session-Feedback verbessern
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

## Agenten-Modus

`byoh serve` startet einen stdio-MCP-Server. Statt selbst Befehle einzutippen, ruft dein KI-Host BYOHs 14 Tools direkt auf — das Gespräch *ist* Interview, Wizard und Ausführung in einem.

```bash
byoh serve   # Claude / Codex / agy verbindet sich und übernimmt alles
```

Verfügbare Tools: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `rag_index`, `rag_search`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` und weitere.

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

# Wissensbasis (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<Suchanfrage>" [--genre <g>] [--k N]
```

## Wie es funktioniert

BYOHs Synthese-Engine gleicht deine Profil-Tags mit dem Skill-Registry ab, ordnet sie in einer abhängigkeitsaufgelösten Pipeline und erzeugt ein `HarnessBundle` — ein git-fertiges Artefakt, das im nativen Format jedes unterstützten Hosts gerendert wird.

- **4-Ring-Sicherheitsmodell** — von integrierten Skills (Ring 1) bis zu Community-/nicht vertrauenswürdigen Skills (Ring 4), jeweils mit eskalierender Validierung
- **3-Gate-Evolution** — jeder `evolve`-Zyklus durchläuft Critic (Qualität), Seesaw (Regression) und Stagnation (Plateau); kein Bypass möglich
- **Persistentes RAG** — inkrementelles Re-Embedding bei Änderungen (`+hinzugefügt ~geändert -entfernt`); Suche nutzt den gespeicherten Index ohne Re-Embedding
- **Zielorientierte Pipelines** — ein 30-Tage-Ziel (Produktlaunch, Forschungsbericht, sicheres Deployment…) fügt automatisch eine passende Skill-Leiter hinzu

Architektur: hexagonal — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Vollständige Anleitung in `AGENTS.md`.

## Entwicklung

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # Unit- + E2E-Tests
cvp                               # parallel: check → clippy → test → fmt → build
```

Optionale Features: `--features mcp` (MCP-Server), `--features native-rag` (lokale Embeddings), `--features rag-openai` (OpenAI-Embeddings). Release-Binaries enthalten alle Features.

## Lizenz

Apache-2.0.
