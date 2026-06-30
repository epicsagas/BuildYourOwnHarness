> Ce document est la version française de [README.md](../../../README.md). La version anglaise fait foi.

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | **Français** | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Votre agent IA, conçu pour vous

*Pas un template générique — un harness compilé selon votre rôle, votre expertise et vos objectifs.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

La plupart des outils IA vous donnent un ensemble fixe de fonctions et vous disent "débrouillez-vous". BYOH fait l'inverse : un court entretien apprend comment vous travaillez vraiment, puis génère un harness d'agent personnalisé — compétences, mémoire, pipelines — qui s'adapte à votre flux de travail dès le départ.

## Pour qui ?

- **Développeurs** qui veulent un agent qui connaît déjà leur stack, leur style de test et leur cadence de livraison
- **Chercheurs** qui ont besoin d'une revue de littérature, d'un suivi des citations et d'une synthèse bien câblés ensemble
- **Créatifs** qui souhaitent un partenaire d'écriture calqué sur leur voix et leur structure de projet
- **Analystes métier** qui ont besoin de frameworks de décision et de pipelines de reporting — pas d'un simple chat

Si vous vous êtes déjà dit "j'aimerais que mon IA connaisse vraiment mon contexte" — c'est exactement ce que fait BYOH.

## Démarrer en 60 secondes

```bash
byoh profile init moi        # analyse votre projet — lecture seule, non-destructif
byoh profile interview moi   # une courte conversation sur votre rôle et vos objectifs
byoh compile moi             # génère votre harness personnel
byoh install moi             # déploie vers Claude / Codex / agy
```

À la prochaine session, votre host charge automatiquement votre harness — agents, compétences, mémoire et pipelines adaptés à vous.

**Vous savez déjà ce qu'il vous faut ?** Parcourez le catalogue communautaire :
```bash
byoh catalog index                            # récupère la liste des 100 meilleurs plugins (secondes)
byoh catalog search "revue de code"           # trouvez des plugins pertinents
byoh catalog vendor anthropics/claude-code-review   # ajoutez-en un à votre harness
```

## Installation

### Binaire (recommandé — aucun toolchain Rust requis)

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

### Charger le plugin dans votre host IA

BYOH se présente comme un plugin polyglotte compatible avec Claude Code, Codex et agy — un seul dépôt, trois hosts.

**Claude Code :**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity) :**
```bash
agy plugin install /chemin/vers/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex :**
```bash
codex plugin marketplace add /chemin/vers/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

Le plugin installe automatiquement le binaire `byoh` au premier chargement — pas besoin de Rust de votre côté.

> **Note :** Le dépôt est actuellement privé. Utilisez les chemins ci-dessus. Une fois public, il apparaîtra dans le marketplace partagé `epicsagas/plugins`.

## Votre premier harness — étape par étape

### Étape 1 — Profil
```bash
byoh profile init moi --paths ./src ./docs   # analyse automatique du projet
byoh profile interview moi                   # conversation de ~5 min
byoh profile confirm moi --genre developer   # valider votre genre
```

BYOH vous interroge sur votre rôle, votre niveau d'expertise, vos outils et votre objectif à 30 jours. L'entretien s'adapte — un chercheur reçoit des questions différentes d'un développeur.

### Étape 2 — Compiler et installer
```bash
byoh compile moi                     # génère le HarnessBundle (validé et contrôlé)
byoh render moi --target claude      # ou : codex | agy | all
byoh install moi                     # installation sécurisée dans dist/
```

### Étape 3 — Exécuter et évoluer
```bash
byoh run moi       # lancer avec votre harness actif
byoh evolve moi    # améliorer le harness selon les retours de session
```

`evolve` exécute un cycle à 3 portes (Critic / Seesaw / Stagnation) qui ne peut jamais être contourné — l'évolution est donc sûre et auditable.

## Catalogue de plugins

Le catalogue vous propose une liste triée des 100 meilleurs plugins Claude (par nombre d'étoiles, actualisée quotidiennement) pour découvrir et ajouter des compétences communautaires sans quitter le terminal.

```bash
# Indexation unique — télécharge un bundle préconstruit en quelques secondes
byoh catalog index

# Recherche hors ligne — aucun réseau requis après l'indexation
byoh catalog search "mémoire" --genre developer --limit 5

# Ajouter un plugin à votre harness
# licence, mots-clés et genre sont auto-détectés depuis le dépôt cloné
byoh catalog vendor obra/superpowers --genre developer
```

L'agent LLM (via les outils MCP `catalog_search` / `catalog_vendor`) peut réaliser ce flux entièrement de manière autonome — ou vous pouvez le piloter directement depuis la CLI.

## Mode agent

`byoh serve` démarre un serveur MCP stdio. Au lieu de taper des commandes, votre host IA appelle directement les 14 outils de BYOH — la conversation *est* l'entretien, le wizard et l'exécution.

```bash
byoh serve   # Claude / Codex / agy se connecte et pilote tout
```

Outils disponibles : `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `rag_index`, `rag_search`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`, et d'autres.

## Référence CLI complète

```bash
# Profil
byoh profile init <slug> [--paths ...]      # analyse non-destructive du projet
byoh profile interview <slug>               # entretien guidé
byoh profile confirm <slug> --genre <g>     # confirmer et verrouiller le profil

# Build
byoh compile <slug> [--dry-run]             # valider + générer le HarnessBundle
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # déployer vers dist/ ou le répertoire du plugin

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

# Base de connaissances (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<requête>" [--genre <g>] [--k N]
```

## Comment ça marche

Le moteur de synthèse de BYOH fait correspondre vos tags de profil avec le registre de compétences, les ordonne dans un pipeline résolu par dépendances, et émet un `HarnessBundle` — un artefact prêt pour git qui se rend dans le format natif de chaque host supporté.

- **Modèle de sécurité à 4 anneaux** — des compétences intégrées (anneau 1) aux compétences communautaires/non fiables (anneau 4), avec une validation croissante à chaque niveau
- **Évolution à 3 portes** — chaque cycle `evolve` passe les portes Critic (qualité), Seesaw (régression) et Stagnation (plateau) ; aucun contournement possible
- **RAG persistant** — ré-embedding incrémental à chaque changement (`+ajouté ~modifié -supprimé`) ; la recherche réutilise l'index sauvegardé sans ré-embedding
- **Pipelines par objectif** — déclarer un objectif à 30 jours (lancement produit, rapport de recherche, livraison sécurisée…) superpose automatiquement une échelle de compétences adaptée

Architecture hexagonale : `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Voir `AGENTS.md` pour le guide complet.

## Compiler et développer

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # tests unitaires + e2e
cvp                               # parallèle : check → clippy → test → fmt → build
```

Fonctionnalités optionnelles : `--features mcp` (serveur MCP), `--features native-rag` (embeddings locaux), `--features rag-openai` (embeddings OpenAI). Les binaires de release incluent toutes les fonctionnalités.

## Licence

Apache-2.0.
