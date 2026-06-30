//! CLI command tree (clap derive). Every entry point from the spec is here.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// BYOH — build your own harness.
#[derive(Debug, Parser)]
#[command(
    name = "byoh",
    version,
    about = "BuildYourOwnHarness — profile → compile → install → evolve a personalized AI agent harness"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Language for interactive output. `auto` (default) detects from LC_ALL/LANG;
    /// otherwise en | ko | ja | zh-hans | es | de | fr | pt | ru | ar.
    #[arg(long, global = true, default_value = "auto")]
    pub language: String,

    /// Verbosity.
    #[arg(long, global = true, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a new draft profile.
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Compile a confirmed profile into a harness bundle.
    Compile {
        /// Profile slug.
        slug: String,
        /// Profiles root dir (overrides BYOH_HOME). Defaults to BYOH_HOME/profiles.
        #[arg(long)]
        profiles_dir: Option<PathBuf>,
        /// Output dir for the bundle.
        #[arg(long, default_value = "./bundle")]
        out: PathBuf,
        /// Run the dry-run gate after compiling.
        #[arg(long, default_value_t = true)]
        dry_run: bool,
    },
    /// Run the doctor: verify dependency tools are installed.
    Doctor,
    /// Render a harness plugin for a slug as a polyglot tree into a safe
    /// project-local `dist/`; `--host` additionally activates it so each host
    /// (claude/codex/agy) discovers the tree.
    Install {
        slug: String,
        /// Which hosts to activate with `--host`: claude | codex | agy | all.
        /// The dist render is always polyglot (all three manifests).
        #[arg(long, default_value = "all")]
        target: String,
        /// Activate the polyglot tree so each host loads it — Claude links
        /// @skills-dir, Codex registers via its marketplace, agy via
        /// `agy plugin install` + `enable`. Without this, only `dist/` is written.
        #[arg(long, default_value_t = false)]
        host: bool,
        /// Overwrite a non-BYOH directory of the same name (dangerous).
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Run an installed harness by slug (common entry point).
    Run { slug: String },
    /// Vendor a community skill into registry/vendored/ (offline; RFC
    /// community-skill-fetch). Vendored files are committed to the repo.
    Vendor {
        #[command(subcommand)]
        action: VendorAction,
    },
    /// Run one evolution cycle under the 3 safety gates (Critic/Seesaw/Stagnation).
    Evolve {
        slug: String,
        /// Genre: developer | creator | researcher | business.
        #[arg(long, default_value = "developer")]
        genre: String,
        /// Proposed edit: AddSkill|ModifyInstinct|ModifyConfig|AddGuardRule|ModifyPrompt|RemoveSkill.
        #[arg(long, default_value = "AddSkill")]
        edit_type: String,
        /// A/B metric: avg score WITH the evolved edit.
        #[arg(long, default_value_t = 0.0)]
        score_with: f64,
        /// A/B metric: avg score WITHOUT the edit (baseline).
        #[arg(long, default_value_t = 0.0)]
        score_without: f64,
        /// Sample count for each arm.
        #[arg(long, default_value_t = 1)]
        samples: u32,
    },
    /// Hook dispatcher (called by Ring 0 hooks).
    Hook { name: String },
    /// Render a synthesized harness into a deployable plugin tree.
    /// The output dir is `git init`-ready: push it and others can use the plugin.
    Render {
        /// Profile slug to render.
        slug: String,
        /// Target host: claude | codex | agy | all.
        #[arg(long, default_value = "all")]
        target: String,
        /// Output directory for the plugin tree.
        #[arg(long, default_value = "./harness-plugin")]
        out: PathBuf,
    },
    /// Search or index the plugin catalog (quemsah top-100 by stars).
    Catalog {
        #[command(subcommand)]
        action: CatalogAction,
    },
    /// Start the BYOH MCP server over stdio (LLM agents drive BYOH via MCP tools).
    /// Requires the `mcp` cargo feature.
    #[cfg(feature = "mcp")]
    Serve,
}

#[derive(Debug, Subcommand)]
pub enum VendorAction {
    /// Vendor an external SKILL.md into registry/vendored/<genre>/<id>.md.
    Add {
        /// Source: local path (file or skills/<id>/ dir) OR a git URL (https://...).
        source: PathBuf,
        /// Genre: developer | creator | researcher | business.
        #[arg(long)]
        genre: String,
        /// Skill id to vendor as.
        #[arg(long)]
        id: String,
        /// Comma-separated keyword tags for synthesis matching (vendored catalog).
        #[arg(long)]
        keywords: Option<String>,
        /// Allow vendoring a git URL not in the default trusted-source allowlist.
        #[arg(long)]
        trust: bool,
        /// Expected commit sha (prefix match) for a git source. Mismatch aborts.
        #[arg(long)]
        sha: Option<String>,
    },
    /// List vendored skills (from registry/vendored/MANIFEST.toml).
    List,
    /// Remove a vendored skill (deletes its .md and drops the MANIFEST row).
    Remove {
        /// Skill id to remove.
        id: String,
        /// Genre the skill was vendored under.
        #[arg(long)]
        genre: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum CatalogAction {
    /// Parse the `quemsah/awesome-claude-plugins` README (top 100 by stars) →
    /// rebuild `~/.byoh/catalog.json`.
    Index {
        /// Max entries to keep (the README lists 100, sorted by stars). The
        /// default keeps all of them; pass a smaller N for just the top N.
        /// `--limit 0` also means all.
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// Cache TTL in hours (existing cache is reused if still fresh).
        #[arg(long, default_value_t = 24)]
        ttl_hours: u64,
        /// Skip the maintainer-built remote bundle and parse the README
        /// directly. By default a stale cache first tries the bundle (seconds);
        /// set this to force a fresh parse (e.g. when debugging the parser).
        #[arg(long, default_value_t = false)]
        no_bundle: bool,
    },
    /// Keyword search the local `~/.byoh/catalog.json` — no network required.
    Search {
        /// Natural-language query (searches name, id, keywords, description).
        query: String,
        /// Filter by genre: developer | creator | researcher | business.
        #[arg(long)]
        genre: Option<String>,
        /// AND-filter tags (comma-separated); entry must contain ALL listed tags.
        #[arg(long)]
        tags: Option<String>,
        /// Max results to show.
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Vendor a catalog plugin into `registry/vendored/` — fetches its GitHub repo
    /// and delegates to `byoh vendor add`.
    Vendor {
        /// Plugin id as listed in the catalog (owner/repo slug).
        plugin_id: String,
        /// Genre override: developer | creator | researcher | business.
        /// Required when the catalog entry has no inferred genre.
        #[arg(long)]
        genre: Option<String>,
        /// Extra keywords to merge (comma-separated).
        #[arg(long)]
        keywords: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProfileAction {
    /// Create a new draft profile.
    Init {
        slug: String,
        /// Existing resources to scan (M1 autoscan).
        #[arg(long, num_args = 0..,)]
        paths: Vec<PathBuf>,
    },
    /// Run the interview + wizard interactively (M0 path).
    Interview { slug: String },
    /// Confirm the genre/goal via the wizard.
    Confirm {
        slug: String,
        #[arg(long)]
        genre: String,
        #[arg(long)]
        goal: Option<String>,
    },
    /// Show a profile.
    Show { slug: String },
}
