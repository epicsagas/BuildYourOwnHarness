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

    /// Language for interactive output (ko | en).
    #[arg(long, global = true, default_value = "ko")]
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
    /// Install a rendered harness plugin for a slug. Defaults to a safe
    /// project-local `dist/`; `--host` writes into the host's real plugin dir.
    Install {
        slug: String,
        /// Target host: claude | codex | agy | all.
        #[arg(long, default_value = "all")]
        target: String,
        /// Install into the host's real plugin directory (e.g. ~/.claude/plugins)
        /// instead of the safe project-local `dist/`.
        #[arg(long, default_value_t = false)]
        host: bool,
        /// Overwrite a non-BYOH directory of the same name (dangerous).
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Run an installed harness by slug (common entry point).
    Run { slug: String },
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
    /// Build a genre RAG index from a corpus directory (BYOH native RAG).
    Index {
        /// Slug whose data sources to index.
        slug: String,
        /// Genre index to build (developer|creator|researcher|business).
        #[arg(long)]
        genre: String,
        /// Corpus directory of text/markdown files to index.
        #[arg(long)]
        corpus: PathBuf,
        /// BYOH home root (indexes go under <root>/indexes/).
        #[arg(long, default_value = ".byoh")]
        home: PathBuf,
        /// Chunk max-tokens.
        #[arg(long, default_value_t = 256)]
        max_tokens: usize,
        /// Chunk overlap.
        #[arg(long, default_value_t = 32)]
        overlap: usize,
    },
    /// Hybrid search a genre RAG index (vector → BM25 → grep).
    Search {
        /// Slug to search.
        slug: String,
        /// Natural-language query.
        query: String,
        #[arg(long)]
        genre: String,
        #[arg(long, default_value = ".byoh")]
        home: PathBuf,
        /// Number of hits to return.
        #[arg(long, default_value_t = 5)]
        k: usize,
        /// Corpus directory (for BM25/grep fallback tiers).
        #[arg(long)]
        corpus: Option<PathBuf>,
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
    /// Start the BYOH MCP server over stdio (LLM agents drive BYOH via MCP tools).
    /// Requires the `mcp` cargo feature.
    #[cfg(feature = "mcp")]
    Serve,
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
