> Ce document est une traduction de [README.md](../../README.md). La version anglaise est la source faisant autorité et peut être plus à jour.

# BuildYourOwnHarness (BYOH)

> Collectez interactivement les connaissances tacites, les données, le genre métier et les objectifs d'un utilisateur — puis **générez, déployez, opérez et faites évoluer un harnais d'agents IA personnalisé**.

BYOH ajoute une **couche de génération** au-dessus des composants validés de l'espace de travail [epiccounty](https://github.com/epicsagas). Au lieu de livrer un ensemble fixe de compétences/mémoire/pipeline, il compile un harnais *unique* par utilisateur à partir d'un entretien.

## Ce qu'il fait

Un profil utilisateur confirmé (genre + expertise + objectif sur 30 jours) pilote un moteur de synthèse qui **recombine les compétences du registre par mot-clé** en un pipeline ordonné, produisant un `HarnessBundle` qui n'est *pas* un modèle de genre fixe. L'ensemble du pipeline est en boucle fermée et gardé par trois portes de sécurité (Critic / Seesaw / Stagnation) qui ne peuvent jamais être contournées.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## Construction et vérification

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                       # tests unitaires + e2e
./target/release/byoh --help
```

Architecture hexagonale : `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`.

## CLI

```bash
byoh profile init <slug> [--paths ...]   # S1 autoscan (non-destructif)
byoh profile interview <slug>            # S2 entretien (Suggest + Council)
byoh profile confirm <slug> --genre <g>  # S3 confirmation wizard
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # porte statique + porte dry-run → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (prêt pour git)
byoh install <slug>                      # installation sécurisée dans dist/ (--host pour le répertoire plugin actif)
byoh run <slug>
byoh evolve <slug>                       # cycle d'évolution à 3 portes
byoh catalog index [--limit N]           # analyser le README top-100 de quemsah → ~/.byoh/catalog.json
byoh catalog search "<requête>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### Mode piloté par agent (serveur MCP)

`byoh serve` (`--features mcp`) démarre un serveur MCP stdio pour qu'un agent LLM **pilote BYOH** — le CLI devient secondaire (inversion de contrôle). 14 outils (`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`) sont découvrables via `tools/list`. La conversation *est* l'entretien/wizard.

```bash
cargo build --release --features mcp
byoh serve
```

## Cœur : synthèse, vendoring et catalogue

- **Moteur de synthèse** — `synthesize(profile)` associe les compétences du registre aux balises du profil, les ordonne en pipeline, et force un nouveau passage à 3 portes (sans contournement). Les pipelines orientés objectif (lancement produit / décision / rapport de recherche / livraison sécurisée / …) superposent une échelle de compétences + un ensemble d'agents lorsque l'objectif sur 30 jours correspond.
- **Vendoring de compétences communautaires** (RFC M3) — `byoh vendor add` récupère un `SKILL.md` externe (chemin local ou URL git), effectue une validation statique + sha256, et l'intègre dans **Ring 3** (le plus restreint) lors de la compilation via `build.rs`. Les compétences externes rejoignent la synthèse en tant que code non approuvé.
- **Catalogue de plugins** — `byoh catalog index` construit un cache hors ligne dans `~/.byoh/catalog.json` à partir du README curaté [quemsah/awesome-claude-plugins](https://github.com/quemsah/awesome-claude-plugins) (top 100 par étoiles, mis à jour quotidiennement). Un seul téléchargement + analyse (pas de crawling page par page), et chaque entrée porte de vraies `stars`. Par défaut, il télécharge d'abord un **bundle préconstruit par le mainteneur** (un asset GitHub Release hebdomadaire — quelques secondes) et n'analyse le README lui-même que si celui-ci est indisponible. Après l'indexation, `catalog search` et `catalog vendor` fonctionnent entièrement hors ligne. Lors de l'entretien S2 du wizard, `profile_interview` inclut automatiquement `catalog_suggestions` — jusqu'à 5 plugins correspondant au genre que le LLM peut recommander sans appels d'outils supplémentaires.

  ```bash
  # Indexation unique (réseau ; ~24 000 pages)
  byoh catalog index --limit 500          # commencer petit ; 0 = exploration complète

  # Recherche hors ligne — sans réseau
  byoh catalog search "test driven development" --genre developer --limit 5

  # Intégrer un plugin trouvé dans registry/vendored/
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Statut

Implémentation Rust de la couche de génération : profileur + entretien + modèles de genre + compilateur (4-ring, génération de code d'outils MCP, porte statique) + moteur d'évolution + RAG autonome (fonctionnalité optionnelle `native-rag`) + serveur MCP (fonctionnalité optionnelle `mcp`). Voir `AGENTS.md` pour le guide d'architecture.

La couche RAG est une **base de connaissances persistante** : `byoh index` sauvegarde l'index de genre + un fichier annexe de corpus sous `$BYOH_HOME/indexes/`, et un `byoh search` ultérieur (ou l'outil MCP `rag_search`) sans `--corpus` le réutilise via `load_index` — sans ré-encodage. La réindexation est **incrémentale** — un manifeste de hachage de contenu ne ré-encode que les documents ajoutés/modifiés et supprime ceux retirés (rapporté comme `+a ~c -r`) ; `--force` effectue une reconstruction complète.

## Licence

Apache-2.0.
