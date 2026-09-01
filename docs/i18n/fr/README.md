> Ce document est la version française de [README.md](../../../README.md). La version anglaise fait foi.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | **Français** | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Votre agent IA, construit autour de vous

*Pas un modèle générique — un harness compilé à partir de votre rôle, de votre expertise et de vos objectifs.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

La plupart des configurations IA vous remettent un ensemble d'outils fixes et vous disent « bonne chance ». BYOH inverse le principe : il vous interroge, apprend ce que vous faites réellement, et génère un harness d'agent personnalisé — skills, agents, pipelines d'objectifs — qui s'intègre à votre flux de travail dès la sortie de la boîte.

## À qui cela s'adresse-t-il ?

- **Développeurs** qui veulent un agent connaissant déjà leur stack, leur style de tests et leur cadence de livraison
- **Chercheurs** qui ont besoin de revoir la littérature, suivre des citations et synthétiser le tout dans un flux cohérent
- **Créateurs** qui veulent un partenaire d'écriture correspondant à leur voix et à la structure de leurs projets
- **Analystes business** qui ont besoin de cadres décisionnels et de pipelines de reporting, pas d'un simple chat brut

Si vous avez déjà pensé « j'aimerais que mon IA connaisse vraiment mon contexte » — c'est exactement ce que fait BYOH.

## Comment ça marche en 60 secondes

BYOH est conçu pour être piloté par votre agent IA — pas par vous en tapant des commandes. Installez le plugin, puis parlez simplement. La conversation *est* l'interview, l'assistant et la compilation.

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # votre agent vous interviewe, construit, comble les lacunes
                           # lui-même et installe — le tout en conversation
```

Lors de la session suivante, votre hôte charge le harness automatiquement — agents, skills et pipelines d'objectifs réglés pour vous.

## Installer le plugin (recommandé)

Vous utilisez **Claude Code, Codex ou agy** ? Installez le plugin. Il regroupe le serveur MCP et **installe automatiquement le binaire au premier chargement** — pas de toolchain Rust, pas de configuration manuelle :

**Claude Code :**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@byoh
```

**agy (Antigravity) :**
```bash
agy plugin install https://github.com/epicsagas/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex :**
```bash
codex plugin marketplace add epicsagas/BuildYourOwnHarness
codex plugin add byoh@byoh
```

### Vous utilisez un autre hôte compatible MCP ?

BYOH parle MCP, donc Cursor, Zed, Continue et autres fonctionnent aussi. Installez le [binaire](#installer-le-binaire-directement) une fois, puis pointez votre hôte vers le serveur :

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Note :** Ce dépôt embarque son propre marketplace `byoh` (.claude-plugin/marketplace.json) — installation autonome, sans hub.

## Installer le binaire directement

Nécessaire uniquement si vous n'utilisez **pas** le plugin (qui installe automatiquement le binaire) ou si vous voulez BYOH sur un hôte MCP non-plugin.

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

### Depuis les sources

```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## Mode piloté par l'agent — le chemin principal

Une fois votre hôte connecté, vous ne tapez pas de commandes — vous parlez simplement. Votre agent appelle les outils MCP de BYOH directement, et la conversation *est* l'interview, la compilation et le cycle d'évolution :

> **Vous :** *Je suis un développeur backend Go qui livre une API de paiements ce mois-ci. Construis-moi un harness.*
>
> **Agent :** *(scanne votre dépôt via `profile_scan`, pose quelques questions ciblées via `profile_interview`, verrouille le genre sur `developer`)* → `build` synthétise un `HarnessBundle` et classe chaque skill comme `matched` / `authored` / `skeleton` → pour tout skeleton dont le profil a besoin (par ex. une skill de vérification spécifique aux paiements), l'agent le rédige à la volée via `author_skill`, puis relance un `build` pour confirmer → installe des agents, des skills et un pipeline d'objectifs secure-ship dans Claude Code. Le contenu rédigé persiste entre les reconstructions. Terminé — à la session suivante, votre agent parle déjà votre stack.

Le même flux, dans l'ordre d'outils suggéré :

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → (author_skill / author_doc to fill skeletons) → build → install_plugin
```

`build` synthétise le bundle (compile + injection de presets + static gate) et classe chaque skill en `matched` (corps de preset réel injecté) ou `skeleton` (placeholder du template de genre) — l'agent lit cela pour décider d'installer maintenant ou d'itérer d'abord sur le profil.

Outils disponibles : `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

Vous voulez que l'agent vous guide à travers tout ? Dites simplement *« build my harness »* — l'agent `byoh-guide` fourni orchestre l'ensemble du flux.

## Catalogue de plugins

Le catalogue est construit à partir du README [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — une liste maintenue par la communauté, classée par étoiles, des 100 meilleurs dépôts de plugins Claude. BYOH livre un bundle précompilé (reconstruit **chaque semaine**, le lundi à 03:17 UTC) pour que `byoh catalog index` résolve en quelques secondes ; passez `--no-bundle` pour parser la liste amont directement.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

L'agent LLM (via les outils MCP `catalog_search` / `catalog_vendor`) peut réaliser ce flux entier de manière autonome — *« add a memory plugin to my harness »* — ou vous pouvez le piloter directement depuis le CLI.

Quelques outils compagnons sont injectés dans les résultats de recherche comme **matériel de référence** (pas des dépendances) : les propres outils de la couche d'exécution de BYOH — [alcove](https://github.com/epicsagas/alcove) (serveur de docs), [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (automation de vault), [epic-harness](https://github.com/epicsagas/epic-harness) (runtime de hooks/skills) — apparaissent contextuellement (une requête « doc server » / « search backend » trouve alcove) pour qu'un agent puisse les recommander le cas échéant. N'en vendez (vendor) un que si vous le voulez réellement ; les bundles restent sans dépendance dans tous les cas.

## Utilisateurs avancés : le CLI (optionnel)

Chaque flux ci-dessus est également accessible depuis le terminal. Le CLI est **auxiliaire** — utile pour le scripting, la CI, ou quand vous préférez ne pas discuter — mais le chemin piloté par l'agent reste celui prévu.

### Votre premier harness — depuis le CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # synthesize (compile + preset injection + static gate) and write the HarnessBundle; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

L'interview elle-même est pilotée par l'agent (l'outil MCP `profile_interview`) — la conversation est l'interview, il n'y a donc pas d'interview interactive en CLI. Le static gate de `build` (présence des 3 gates de sécurité Critic / Seesaw / Stagnation, schéma MCP, entrées de hook) s'exécute toujours et ne peut jamais être contourné — le bundle est donc structurellement valide avant d'être livré. L'amélioration post-installation est une rétrospective conversationnelle lors de sessions ultérieures, pas un appel d'outil.

## Comment ça marche sous le capot

Le moteur de synthèse de BYOH fait correspondre vos tags de profil au registre de skills, les ordonne dans un pipeline résolu en termes de dépendances, et émet un `HarnessBundle` — un artefact prêt pour git qui se rend dans le format natif de tout hôte supporté.

- **Modèle de sécurité à 4 anneaux** — spec de cycle de vie (Ring 0) et skills de pipeline intégrés (Ring 1) jusqu'aux skills communautaires/non fiables (Ring 3), chacun avec une validation croissante ; les skills vendés sont épinglés en sha256 et vérifiés au moment de la lecture + l'embed
- **Socle de sécurité à 3 portes** — chaque build passe une static gate qui vérifie la présence des portes Critic (qualité), Seesaw (régression) et Stagnation (plateau) ; aucun contournement
- **Pipelines orientés objectifs** — déclarer un objectif à 30 jours (lancement produit, rapport de recherche, secure ship…) superpose automatiquement une échelle de skills adaptée

Architecture : hexagonale — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. Voir `AGENTS.md` pour le guide complet.

## Référence complète du CLI

Le CLI est volontairement restreint : points d'entrée machine (`serve`, `catalog index` en CI, `vendor` pour les mainteneurs) plus un miroir scriptable du flux de compilation principal. L'interview et l'évolution sont réservées au MCP (pilotées par l'agent).

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

Les profils et le cache du catalogue vivent sous `~/.byoh` par défaut (surchargeable via `BYOH_HOME`).

## Build & développement

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

La feature `mcp` (serveur MCP stdio) est activée par défaut. BYOH ne fournit aucune base de connaissances embarquée — pour la retrieval, pointez votre harness généré vers un serveur de docs comme [alcove](https://github.com/epicsagas/alcove).

## Remerciements

BYOH s'appuie sur plusieurs efforts communautaires :

- **Catalogue de plugins** — issu de [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), une liste communautaire classée par étoiles des 100 meilleurs dépôts de plugins Claude. Sans elle, le catalogue n'existerait pas.
- **Outils compagnons** — conçus pour interopérer avec [alcove](https://github.com/epicsagas/alcove) (serveur de docs / RAG), [Episteme](https://github.com/epicsagas/Episteme) (knowledge graph) et [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (automation de vault).
- **Stack open-source** — construit sur [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) et l'écosystème Rust.

Les entrées du catalogue et les skills communautaires vendés conservent leurs propres licences (détectées automatiquement au moment du vendor). BYOH lui-même est sous Apache-2.0.

## Licence

Apache-2.0.
