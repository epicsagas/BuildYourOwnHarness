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
        /// Profiles root dir.
        #[arg(long, default_value = ".byoh/profiles")]
        profiles_dir: PathBuf,
        /// Output dir for the bundle.
        #[arg(long, default_value = "./bundle")]
        out: PathBuf,
        /// Run the dry-run gate after compiling.
        #[arg(long, default_value_t = true)]
        dry_run: bool,
    },
    /// Run the doctor: verify dependency tools are installed.
    Doctor,
    /// Install a bundle by slug.
    Install { slug: String },
    /// Run an installed harness by slug (common entry point).
    Run { slug: String },
    /// Run one evolution cycle.
    Evolve { slug: String },
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
