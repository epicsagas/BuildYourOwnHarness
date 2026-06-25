//! `byoh` binary entry point.

use std::path::PathBuf;

use byoh::adapters::{FilesystemSource, RuleInterview, RuleLlm, StaticWizard, StdCommand};
use byoh::application::ProfileOrchestrator;
use byoh::cli::{Cli, Command, ProfileAction};
use byoh::compiler::{compile_profile, dry_run, static_gate};
use byoh::deploy::registry::Registry;
use byoh::domain::genre::Genre;
use byoh::domain::profile::{ProfileStatus, UserProfile};
use byoh::i18n::{t, Msg};
use byoh::ports::{CommandPort, InterviewPort};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.verbose {
        eprintln!("[byoh] verbose mode");
    }
    let lang = cli.language.clone();
    match cli.command {
        Command::Profile { action } => run_profile(action, &lang)?,
        Command::Compile {
            slug,
            profiles_dir,
            out,
            dry_run,
        } => run_compile(&slug, &profiles_dir, &out, dry_run, &lang)?,
        Command::Doctor => run_doctor(&lang)?,
        Command::Install { slug } => run_install(&slug, &lang)?,
        Command::Run { slug } => run_run(&slug, &lang)?,
        Command::Evolve { slug } => run_evolve(&slug, &lang)?,
        Command::Index {
            slug,
            genre,
            corpus,
            home,
            max_tokens,
            overlap,
        } => run_index(&slug, &genre, &corpus, &home, max_tokens, overlap)?,
        Command::Search {
            slug,
            query,
            genre,
            home,
            k,
            corpus,
        } => run_search(&slug, &query, &genre, &home, k, corpus.as_deref())?,
        Command::Hook { name } => run_hook(&name, &lang)?,
    }
    Ok(())
}

fn profiles_root() -> PathBuf {
    PathBuf::from(std::env::var("BYOH_HOME").unwrap_or_else(|_| ".byoh".to_string()))
}

fn profile_path(slug: &str) -> PathBuf {
    profiles_root()
        .join("profiles")
        .join(format!("{slug}.yaml"))
}

fn run_profile(action: ProfileAction, lang: &str) -> anyhow::Result<()> {
    match action {
        ProfileAction::Init { slug, paths } => {
            let p = UserProfile::new_draft(&slug, lang);
            write_profile(&p)?;
            // M1 autoscan if paths given.
            if !paths.is_empty() {
                let src = FilesystemSource::new();
                let llm = RuleLlm::new();
                let iv = RuleInterview::new(RuleLlm::new());
                let wz = StaticWizard::new();
                let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
                let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
                let mut loaded = load_profile(&slug)?;
                orch.stage1_scan(&mut loaded, &path_refs)?;
                write_profile(&loaded)?;
            }
            println!("{}", t(Msg::Welcome, lang));
            println!("created draft profile: {}", profile_path(&slug).display());
        }
        ProfileAction::Interview { slug } => {
            let llm = RuleLlm::new();
            let iv = RuleInterview::new(RuleLlm::new());
            let mut p = load_profile(&slug)?;
            let _ = iv.next_questions(&p);
            // Non-interactive default: accept suggestions, then mark interviewed.
            let answers = std::collections::HashMap::new();
            let wz = StaticWizard::new();
            let src = FilesystemSource::new();
            let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
            orch.stage2_interview(&mut p, &answers)?;
            write_profile(&p)?;
            println!(
                "interview complete (suggestions applied); status: {}",
                p.status
            );
        }
        ProfileAction::Confirm { slug, genre, goal } => {
            let llm = RuleLlm::new();
            let iv = RuleInterview::new(RuleLlm::new());
            let wz = StaticWizard::new();
            let src = FilesystemSource::new();
            let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
            let g: Genre = genre.parse()?;
            let mut p = load_profile(&slug)?;
            orch.stage3_confirm(&mut p, g, goal.as_deref())?;
            write_profile(&p)?;
            println!("{} — status: {}", t(Msg::Confirm, lang), p.status);
        }
        ProfileAction::Show { slug } => {
            let p = load_profile(&slug)?;
            println!("{}", serde_yaml::to_string(&p)?);
        }
    }
    Ok(())
}

fn run_compile(
    slug: &str,
    profiles_dir: &std::path::Path,
    out: &std::path::Path,
    do_dry_run: bool,
    lang: &str,
) -> anyhow::Result<()> {
    let path = if profiles_dir.ends_with("profiles") {
        profiles_dir.join(format!("{slug}.yaml"))
    } else {
        profile_path(slug)
    };
    let body = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("reading profile {}: {e}", path.display()))?;
    let profile: UserProfile = serde_yaml::from_str(&body)?;
    if profile.status != ProfileStatus::Confirmed {
        anyhow::bail!(
            "profile {slug} is not confirmed (status={}); run `byoh profile confirm`",
            profile.status
        );
    }
    let bundle = compile_profile(&profile)?;

    // Static gate.
    let report = static_gate(&bundle)?;
    if !report.passed() {
        anyhow::bail!("static gate failed: {}", report.errors.join("; "));
    }

    // Dry-run gate.
    if do_dry_run {
        let cmd = StdCommand::new();
        let dr = dry_run(&bundle, &cmd)?;
        if dr.passed() {
            println!("{}", t(Msg::DryRunPassed, lang));
        } else {
            println!("{}", t(Msg::DryRunFailed, lang));
        }
        for fb in &dr.fallbacks {
            eprintln!("[byoh] {fb}");
        }
    }

    // Materialize the bundle on disk.
    std::fs::create_dir_all(out)?;
    materialize_bundle(&bundle, out)?;
    println!("bundle written to {}", out.display());

    // Register.
    let mut reg = Registry::new();
    let entry = reg.register(&bundle);
    println!("registered: {}", entry.id);
    Ok(())
}

fn materialize_bundle(
    bundle: &byoh::domain::bundle::HarnessBundle,
    out: &std::path::Path,
) -> anyhow::Result<()> {
    use byoh::domain::bundle::Ring;
    let cfg = toml::to_string(&bundle.config())?;
    std::fs::write(out.join("config").with_file_name("harness.toml"), cfg)?;

    for skill in &bundle.skills {
        let ring_dir = out.join("skills").join(skill.ring.as_str());
        std::fs::create_dir_all(&ring_dir)?;
        std::fs::write(
            ring_dir.join(format!("{}.md", skill.id)),
            &skill.body_markdown,
        )?;
    }
    for hook in &bundle.hooks {
        let _ = hook; // hooks.json aggregated below
    }
    let hooks_json = serde_json::to_string_pretty(&serde_json::json!({
        "hooks": bundle.hooks.iter().map(|h| serde_json::json!({
            "event": h.event,
            "command": h.command,
            "reads": h.reads,
        })).collect::<Vec<_>>()
    }))?;
    std::fs::write(out.join("hooks").join("hooks.json"), hooks_json)?;

    std::fs::create_dir_all(out.join("mcp").join("tools"))?;
    for tool in &bundle.mcp_tools {
        std::fs::write(
            out.join("mcp")
                .join("tools")
                .join(format!("{}.json", tool.name)),
            serde_json::to_string_pretty(tool)?,
        )?;
    }

    let policy = toml::to_string(&serde_json::json!({
        "enabled": true,
        "safety_gates": bundle.safety_gates,
        "stagnation_limit": bundle.stagnation_limit,
        "improvement_threshold": bundle.improvement_threshold,
    }))?;
    std::fs::write(out.join("evolution_policy.toml"), policy)?;

    let _ = Ring::all(); // touch to avoid dead-code on Ring import
    Ok(())
}

fn run_doctor(lang: &str) -> anyhow::Result<()> {
    let cmd = StdCommand::new();
    for tool in ["obsidian-forge", "alcove", "epic-harness", "claudy"] {
        let installed = cmd.is_installed(tool);
        println!(
            "{tool}: {}",
            if installed { "installed" } else { "MISSING" }
        );
    }
    if !lang.is_empty() {
        println!("language: {lang}");
    }
    Ok(())
}

fn run_install(slug: &str, lang: &str) -> anyhow::Result<()> {
    println!("{}", t(Msg::Installed, lang).replace("<slug>", slug));
    Ok(())
}

fn run_run(slug: &str, _lang: &str) -> anyhow::Result<()> {
    println!("[byoh] run {slug} — delegating to execution-layer tools");
    Ok(())
}

fn run_evolve(slug: &str, lang: &str) -> anyhow::Result<()> {
    println!("[byoh] evolve {slug}: {} ", t(Msg::EvolveApproved, lang));
    Ok(())
}

fn run_hook(name: &str, _lang: &str) -> anyhow::Result<()> {
    println!("[byoh] hook {name} (no-op)");
    Ok(())
}

/// Collect text documents under `corpus` (.md/.txt/.rs/.py/...) into InputDocs.
fn collect_corpus(corpus: &std::path::Path) -> anyhow::Result<Vec<byoh::rag::InputDoc>> {
    let mut docs = Vec::new();
    if !corpus.exists() {
        anyhow::bail!("corpus path does not exist: {}", corpus.display());
    }
    if corpus.is_file() {
        let text = std::fs::read_to_string(corpus)?;
        docs.push(byoh::rag::InputDoc {
            id: corpus
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("doc")
                .to_string(),
            text,
        });
        return Ok(docs);
    }
    for entry in walkdir::WalkDir::new(corpus)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let is_text = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                matches!(
                    e,
                    "md" | "txt" | "rs" | "py" | "ts" | "js" | "toml" | "yaml" | "yml" | "json"
                )
            })
            .unwrap_or(false);
        if !is_text {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(p) {
            let id = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("doc")
                .to_string();
            docs.push(byoh::rag::InputDoc { id, text });
        }
    }
    Ok(docs)
}

fn make_embedder() -> anyhow::Result<Box<dyn byoh::ports::EmbedderProvider>> {
    // Default build: DummyEmbedder (no model download). With native-rag at run
    // time we would construct FastembedEmbedder; selection is a runtime concern.
    Ok(Box::new(byoh::adapters::DummyEmbedder::new()))
}

#[cfg(feature = "native-rag")]
fn make_embedder_native() -> anyhow::Result<Box<dyn byoh::ports::EmbedderProvider>> {
    match byoh::adapters::embedder::FastembedEmbedder::new() {
        Ok(fe) => Ok(Box::new(fe)),
        Err(e) => {
            eprintln!("[byoh] fastembed unavailable ({e}); falling back to dummy");
            Ok(Box::new(byoh::adapters::DummyEmbedder::new()))
        }
    }
}

fn run_index(
    slug: &str,
    genre: &str,
    corpus: &std::path::Path,
    home: &std::path::Path,
    max_tokens: usize,
    overlap: usize,
) -> anyhow::Result<()> {
    let genre_v: Genre = genre.parse()?;
    let docs = collect_corpus(corpus)?;
    let opts = byoh::rag::ChunkOptions::new(max_tokens, overlap);
    let report = index_build(&genre_v, &docs, &opts, home)?;

    println!(
        "[byoh] indexed slug={slug} genre={genre}: {} docs / {} chunks / dim={} / backend={}",
        report.docs, report.chunks, report.dim, report.backend
    );
    Ok(())
}

#[cfg(feature = "native-rag")]
fn index_build(
    genre_v: &Genre,
    docs: &[byoh::rag::InputDoc],
    opts: &byoh::rag::ChunkOptions,
    home: &std::path::Path,
) -> anyhow::Result<byoh::rag::BuildReport> {
    let embedder = make_embedder_native()?;
    let (report, handle) =
        byoh::rag::pipeline::native::build_index_native(&*embedder, *genre_v, docs, opts, 4)?;
    byoh::rag::pipeline::native::save_index_native(&handle, home)?;
    Ok(report)
}

#[cfg(not(feature = "native-rag"))]
fn index_build(
    genre_v: &Genre,
    docs: &[byoh::rag::InputDoc],
    opts: &byoh::rag::ChunkOptions,
    home: &std::path::Path,
) -> anyhow::Result<byoh::rag::BuildReport> {
    let embedder = make_embedder()?;
    let (report, handle) = byoh::rag::build_index(&*embedder, *genre_v, docs, opts)?;
    byoh::rag::save_index(&handle, home)?;
    Ok(report)
}

fn run_search(
    slug: &str,
    query: &str,
    genre: &str,
    _home: &std::path::Path,
    k: usize,
    corpus: Option<&std::path::Path>,
) -> anyhow::Result<()> {
    // NOTE: `home` would select a persisted genre index under <home>/indexes/.
    // For now we build an ephemeral index from the supplied corpus (or run the
    // grep tier). Loading a persisted TurbovecIndex is a native-rag path.
    let genre_v: Genre = genre.parse()?;

    // Mask any secret in the query before logging/output (R10/AC10).
    let masked_query_log = byoh::security::mask(query);
    let embedder = make_embedder()?;

    // Build an ephemeral index from the corpus (if provided) for search.
    if let Some(corpus_dir) = corpus {
        let docs = collect_corpus(corpus_dir)?;
        let opts = byoh::rag::ChunkOptions::default();
        let (_report, handle) = byoh::rag::build_index(&*embedder, genre_v, &docs, &opts)?;
        let hits = handle.search(&*embedder, query, k)?;
        println!(
            "[byoh] search slug={slug} genre={genre} q=\"{masked_query_log}\" → {} hits",
            hits.len()
        );
        for h in hits {
            let text = byoh::security::mask(&h.text);
            println!(
                "[{}] id={} score={:.4} :: {}",
                h.mode,
                h.id,
                h.score,
                truncate_str(&text, 120)
            );
        }
        return Ok(());
    }

    // No corpus supplied: run grep-only fallback against an empty corpus.
    let empty: Vec<(String, String)> = Vec::new();
    let qe = embedder.embed(query)?;
    let hits = byoh::rag::hybrid_search(None, Some(&qe), &empty, query, k, genre_v);
    println!(
        "[byoh] search slug={slug} genre={genre} q=\"{masked_query_log}\" → {} hits (no corpus; grep tier only)",
        hits.len()
    );
    for h in hits {
        let text = byoh::security::mask(&h.text);
        println!(
            "[{}] id={} score={:.4} :: {}",
            h.mode.as_str(),
            h.id,
            h.score,
            truncate_str(&text, 120)
        );
    }
    Ok(())
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

fn load_profile(slug: &str) -> anyhow::Result<UserProfile> {
    let path = profile_path(slug);
    let body = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("reading profile {}: {e}", path.display()))?;
    Ok(serde_yaml::from_str(&body)?)
}

fn write_profile(p: &UserProfile) -> anyhow::Result<()> {
    let path = profile_path(&p.slug);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_yaml::to_string(p)?)?;
    Ok(())
}

// Helper trait to convert BundleConfig into a TOML-serializable form via serde.
trait ConfigTomlExt {
    fn config(&self) -> serde_json::Value;
}

impl ConfigTomlExt for byoh::domain::bundle::HarnessBundle {
    fn config(&self) -> serde_json::Value {
        serde_json::to_value(&self.config).unwrap_or(serde_json::json!({}))
    }
}
