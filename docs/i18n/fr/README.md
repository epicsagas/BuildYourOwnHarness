> Ce document est la version française de [README.md](../../../README.md). La version anglaise fait foi.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | **Français** | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Votre agent IA, conçu autour de vous

*Pas un template générique — un harness compilé selon votre rôle, votre expertise et vos objectifs.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

La plupart des setups IA vous remettent un ensemble d'outils figé et vous disent « bonne chance ». BYOH **renverse la tendance** : il vous interroge, apprend ce que vous faites réellement, puis génère un harness d'agent personnalisé — compétences, mémoire, pipelines — qui s'intègre à votre flux de travail dès le départ.

## Pour qui ?

- **Développeurs** qui veulent un agent connaissant déjà leur stack, leur style de tests et leur cadence de livraison
- **Chercheurs** qui ont besoin d'une revue de littérature, d'un suivi des citations et d'une synthèse câblés ensemble
- **Créatifs** qui veulent un partenaire d'écriture calqué sur leur voix et la structure de leur projet
- **Analystes métier** qui ont besoin de frameworks de décision et de pipelines de reporting — pas d'un simple chat

Si vous vous êtes déjà dit « j'aimerais que mon IA connaisse vraiment mon contexte » — c'est exactement ce que fait BYOH.

## Comment ça marche en 60 secondes

BYOH est conçu pour être piloté par votre agent IA — pas par vous en tapant des commandes. Installez le plugin, puis discutez simplement. La conversation *est* l'entretien, **l'assistant** et la compilation.

```
1. Install the plugin      # Claude Code / Codex / agy — installation automatique du binaire
2. "Build me a harness"    # votre agent scanne votre dépôt et compile le résultat
```

À la session suivante, votre host charge automatiquement le harness — agents, compétences, mémoire et pipelines réglés pour vous.

## Installer le plugin (recommandé)

Vous utilisez **Claude Code, Codex ou agy** ? Installez le plugin. Il regroupe le serveur MCP et **installe le binaire automatiquement au premier chargement** — pas de toolchain Rust, pas de configuration manuelle :

**Claude Code :**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity) :**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex :**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

### Vous utilisez un autre host compatible MCP ?

BYOH parle MCP, donc Cursor, Zed, Continue et consœurs fonctionnent aussi. Installez le [binaire](#installation) une fois, puis pointez votre host vers le serveur :

```bash
byoh serve   # serveur MCP stdio
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Note :** Le dépôt est actuellement privé. Utilisez les chemins ci-dessus. Une fois public, il apparaîtra dans le marketplace partagé `epicsagas/plugins`.

## Mode piloté par l'agent — le chemin principal

Une fois votre host connecté, vous ne tapez plus de commandes — vous discutez simplement. Votre agent appelle directement les outils MCP de BYOH, et la conversation *est* l'entretien, la compilation et le cycle d'évolution :

> **Vous :** *Je suis un développeur backend Go qui met en production une API de paiements ce mois-ci. Construis-moi un harness.*
>
> **Agent :** *(scanne votre dépôt via `profile_scan`, pose quelques questions ciblées via `profile_interview`, verrouille le genre sur `developer`)* → compile un `HarnessBundle` → installe les agents, compétences, mémoire et un pipeline de livraison sécurisée dans Claude Code. Terminé — à la prochaine session, votre agent parle déjà votre stack.

Le même flux, dans l'ordre d'outils suggéré :

```
profile_create → profile_scan → profile_interview → profile_confirm
           → compile → compile_dry_run → render_plugin → install_plugin
           → (optionnel) registry_clone_skill → (plus tard) evolve_cycle
```

Outils disponibles : `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `genre_list`, `compile`, `compile_dry_run`, `render_plugin`, `install_plugin`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`.

Vous voulez que l'agent vous guide ? Dites simplement *« build my harness »* — l'agent `byoh-guide` fourni avec **orchestre** l'ensemble du flux.

## Catalogue de plugins

Le catalogue est construit à partir du README de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — une liste communautaire, classée par étoiles, des 100 principaux dépôts de plugins Claude. BYOH fournit un bundle précompilé (reconstruit **chaque semaine**, tous les lundis 03:17 UTC) pour que `byoh catalog index` se résolve en quelques secondes ; passez `--no-bundle` pour analyser directement la liste en amont.

```bash
# Indexation unique — télécharge un bundle préconstruit en quelques secondes
byoh catalog index

# Recherche hors ligne — aucun réseau requis après l'indexation
byoh catalog search "memory" --genre developer --limit 5

# Ajouter un plugin à votre harness
# licence, mots-clés et genre sont auto-détectés depuis le dépôt cloné
byoh catalog vendor obra/superpowers --genre developer
```

L'agent LLM (via les outils MCP `catalog_search` / `catalog_vendor`) peut réaliser ce flux entièrement de manière autonome — *« ajoute un plugin mémoire à mon harness »* — ou vous pouvez le piloter directement depuis la CLI.

## Utilisateurs avancés : la CLI (optionnel)

Chaque flux ci-dessus est également accessible depuis le terminal. La CLI est **auxiliaire** — pratique pour le scripting, la CI, ou quand vous préférez ne pas discuter — mais le chemin piloté par l'agent reste la voie **prévue**.

### Votre premier harness — depuis la CLI

```bash
byoh profile init me --paths ./src ./docs   # analyse automatique de votre projet
byoh profile interview me                   # ~5 min d'entretien
byoh profile confirm me --genre developer   # verrouiller votre genre

byoh compile me --no-dry-run                # valide + écrit le HarnessBundle (dry-run par défaut)
byoh render me --target claude              # ou : codex | agy | all (défaut : all)
byoh install me                             # rend vers dist/, puis --host l'active

byoh run me                                 # lancer avec votre harness actif
byoh evolve me                              # améliorer le harness selon les retours de session
```

BYOH vous interroge sur votre rôle, votre niveau d'expertise, vos outils et votre objectif à 30 jours. L'entretien s'adapte — un chercheur reçoit des questions différentes d'un développeur. `evolve` exécute un cycle à 3 portes (Critic / Seesaw / Stagnation) qui ne peut jamais être contourné — l'évolution est donc sûre et auditable.

## Comment ça marche sous le capot

Le moteur de synthèse de BYOH fait correspondre les tags de votre profil avec le registre de compétences, les ordonne dans un pipeline résolu par dépendances, et émet un `HarnessBundle` — un artefact prêt pour git qui se rend dans le format natif de chaque host supporté.

- **Modèle de sécurité à 4 anneaux** — des compétences intégrées (anneau 1) jusqu'aux compétences communautaires/non fiables (anneau 4), chacune avec une validation croissante
- **Évolution à 3 portes** — chaque cycle `evolve` passe les portes Critic (qualité), Seesaw (régression) et Stagnation (plateau) ; aucun contournement possible
- **Pipelines orientés objectif** — déclarer un objectif à 30 jours (lancement produit, rapport de recherche, livraison sécurisée…) superpose automatiquement l'échelle de compétences correspondante

Architecture : hexagonale — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Voir `AGENTS.md` pour le guide complet.

## Référence CLI complète

```bash
# Profil
byoh profile init <slug> [--paths ...]      # analyse non-destructive du projet
byoh profile interview <slug>               # entretien guidé
byoh profile confirm <slug> --genre <g>     # confirmer et verrouiller le profil

# Build
byoh compile <slug> [--no-dry-run]          # dry-run par défaut ; --no-dry-run pour écrire le bundle
byoh render <slug> [--target <host>]        # claude | codex | agy | all (défaut : all)
byoh install <slug> [--target <host>] [--host] [--force]  # rend l'arbre polyglotte vers dist/ ; --host l'active par hôte

# Exécuter et évoluer
byoh run <slug>
byoh evolve <slug>

# Compétences communautaires
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catalogue
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<requête>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Installation

Nécessaire uniquement si vous n'utilisez **pas** le plugin (qui installe le binaire automatiquement) ou si vous voulez BYOH sur un host MCP non plugin.

### Binaire (aucun toolchain Rust requis)

**macOS / Linux :**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell) :**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**Depuis les sources :**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # vérification
```

## Compiler et développer

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # tests unitaires + e2e
cvp                               # parallèle : check → clippy → test → fmt → build
```

La fonctionnalité `mcp` (serveur MCP stdio) est activée par défaut. BYOH n'embarque aucun corpus de connaissances — pour la récupération documentaire, pointez votre harness généré vers un serveur de docs comme [alcove](https://github.com/epicsagas/alcove).

## Remerciements

BYOH s'appuie sur plusieurs efforts communautaires :

- **Catalogue de plugins** — provient de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), une liste communautaire classée par étoiles des 100 principaux dépôts de plugins Claude. Sans elle, le catalogue n'existerait pas.
- **Outils compagnons** — conçu pour interopérer avec [alcove](https://github.com/epicsagas/alcove) (serveur de docs / RAG), [Episteme](https://github.com/epicsagas/Episteme) (graphe de connaissances) et [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (automatisation de vaults).
- **Stack open source** — bâti sur [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) et l'écosystème Rust.

Les entrées du catalogue et les compétences communautaires intégrées conservent leurs propres licences (détectées automatiquement à l'intégration). BYOH lui-même est sous Apache-2.0.

## Licence

Apache-2.0.
