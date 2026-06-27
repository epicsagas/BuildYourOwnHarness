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
    let lang = if cli.language == "auto" {
        byoh::i18n::detect_locale().to_string()
    } else {
        cli.language.clone()
    };
    match cli.command {
        Command::Profile { action } => run_profile(action, &lang)?,
        Command::Compile {
            slug,
            profiles_dir,
            out,
            dry_run,
        } => run_compile(&slug, profiles_dir.as_deref(), &out, dry_run, &lang)?,
        Command::Doctor => run_doctor(&lang)?,
        Command::Install {
            slug,
            target,
            host,
            force,
        } => run_install(&slug, &target, host, force, &lang)?,
        Command::Run { slug } => run_run(&slug, &lang)?,
        Command::Evolve {
            slug,
            genre,
            edit_type,
            score_with,
            score_without,
            samples,
        } => run_evolve(
            &slug,
            &genre,
            &edit_type,
            score_with,
            score_without,
            samples,
        )?,
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
        Command::Render { slug, target, out } => run_render(&slug, &target, &out)?,
        Command::Vendor { action } => run_vendor(action, &lang)?,
        #[cfg(feature = "mcp")]
        Command::Serve => run_serve(&lang)?,
    }
    Ok(())
}

/// Render a synthesized harness into a deployable plugin tree.
/// Loads the profile, synthesizes the bundle, then renders to the target host(s).
fn run_vendor(action: byoh::cli::VendorAction, _lang: &str) -> anyhow::Result<()> {
    use byoh::cli::VendorAction;
    use byoh::domain::genre::Genre;
    let repo_root = std::env::current_dir()?;
    match action {
        VendorAction::Add {
            source,
            genre,
            id,
            keywords,
            trust,
            sha,
        } => {
            let g: Genre = genre.parse()?;
            let kw: Vec<String> = keywords
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            let fetched_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
                .to_string();

            match byoh::deploy::resolve_source(&source.to_string_lossy()) {
                byoh::deploy::VendorSource::Local(p) => {
                    let entry = byoh::deploy::vendor_add(
                        &p,
                        g,
                        &id,
                        &kw,
                        "unknown",
                        &repo_root,
                        &fetched_at,
                    )?;
                    println!(
                        "vendored '{}' ({}) -> registry/vendored/{}/{}.md (sha256 {}...)",
                        id,
                        g.as_str(),
                        g.as_str(),
                        id,
                        &entry.sha256[..12]
                    );
                }
                byoh::deploy::VendorSource::GitSubdir { url, .. } => {
                    if !trust && !byoh::deploy::source_is_trusted(&url) {
                        anyhow::bail!(
                            "source not in allowlist: {url}\nPass --trust to vendor an untrusted source."
                        );
                    }
                    let dest = std::env::temp_dir().join(format!("byoh-vendor-{id}-{fetched_at}"));
                    let sha_actual = byoh::deploy::fetch_git(&url, "HEAD", sha.as_deref(), &dest)?;
                    let license = byoh::deploy::extract_license_from_dir(&dest)
                        .unwrap_or_else(|| "unknown".to_string());
                    let entry = byoh::deploy::vendor_add(
                        &dest,
                        g,
                        &id,
                        &kw,
                        &license,
                        &repo_root,
                        &fetched_at,
                    )?;
                    let sha_short: String = sha_actual.chars().take(12).collect();
                    println!(
                        "vendored '{}' ({}) <- {url} (commit {sha_short}, skill sha256 {}..., license {license})",
                        id,
                        g.as_str(),
                        &entry.sha256[..12]
                    );
                    let _ = std::fs::remove_dir_all(&dest);
                }
            }
        }
        VendorAction::List => {
            let entries = byoh::deploy::vendor_list(&repo_root)?;
            if entries.is_empty() {
                println!("(no vendored skills)");
            } else {
                println!(
                    "{:<24} {:<10} {:<10} sha256",
                    "skill_id", "genre", "license"
                );
                for e in entries {
                    let sha: String = e.sha256.chars().take(12).collect();
                    println!(
                        "{:<24} {:<10} {:<10} {}...",
                        e.skill_id, e.genre, e.license, sha
                    );
                }
            }
        }
        VendorAction::Remove { id, genre } => {
            let g: Genre = genre.parse()?;
            byoh::deploy::vendor_remove(&repo_root, g, &id)?;
            println!("removed vendored '{}' ({})", id, g.as_str());
        }
    }
    Ok(())
}

fn run_render(slug: &str, target: &str, out: &std::path::Path) -> anyhow::Result<()> {
    let target: byoh::domain::render_target::Target = target.parse()?;
    let profile = byoh::store::load_profile(slug)?;
    if profile.status != ProfileStatus::Confirmed {
        anyhow::bail!(
            "profile {slug} is not confirmed (status={}); run `byoh profile confirm`",
            profile.status
        );
    }
    // Synthesize (recombined bundle); falls back to the static template inside.
    let (bundle, _plan) = byoh::application::synthesize(&profile)?;
    let root = byoh::application::render_target(&bundle, target, out)?;
    println!(
        "[byoh] rendered '{slug}' → {} ({} skills, {} agents) at {}",
        target.as_str(),
        bundle.skills.len(),
        bundle.agents.len(),
        root.display()
    );
    println!(
        "[byoh] the output dir is git-ready: `cd {} && git init && git push`",
        root.display()
    );
    Ok(())
}

/// Start the BYOH stdio MCP server. Runtime env (BYOH_HOME, language,
/// native-rag) is fixed here and shared via `Arc` inside the server.
#[cfg(feature = "mcp")]
fn run_serve(lang: &str) -> anyhow::Result<()> {
    let ctx = byoh::mcp::server::ByohContext {
        home: byoh::store::byoh_home(),
        language: lang.to_string(),
        native_rag: cfg!(feature = "native-rag"),
    };
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(byoh::mcp::server::ByohServer::new(ctx).serve_stdio())
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn profile_path(slug: &str) -> PathBuf {
    byoh::store::profile_path(slug)
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
            // CLI confirm may run straight after `init` (Draft). stage3_confirm
            // requires Interviewed (Draft → Interviewed → Confirmed); advance
            // automatically so the minimal `init → confirm` path works.
            if p.status == ProfileStatus::Draft {
                p.advance(ProfileStatus::Interviewed)?;
            }
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
    profiles_dir: Option<&std::path::Path>,
    out: &std::path::Path,
    do_dry_run: bool,
    lang: &str,
) -> anyhow::Result<()> {
    let path = match profiles_dir {
        Some(dir) => dir.join(format!("{slug}.yaml")),
        None => profile_path(slug),
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
    std::fs::create_dir_all(out)?;
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
    let hooks_dir = out.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;
    std::fs::write(hooks_dir.join("hooks.json"), hooks_json)?;

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

/// Synthesize a confirmed profile then render it as a polyglot plugin into the
/// safe project-local `dist/`. `--host` additionally activates it so each host
/// (claude/codex/agy) discovers the tree. `--force` overwrites a non-BYOH dir.
fn run_install(
    slug: &str,
    target: &str,
    host: bool,
    force: bool,
    _lang: &str,
) -> anyhow::Result<()> {
    let target: byoh::domain::render_target::Target = target.parse()?;
    let profile = byoh::store::load_profile(slug)?;
    if profile.status != ProfileStatus::Confirmed {
        anyhow::bail!(
            "profile {slug} is not confirmed (status={}); run `byoh profile confirm`",
            profile.status
        );
    }
    let (bundle, _plan) = byoh::application::synthesize(&profile)?;
    let loc = byoh::deploy::InstallLocations::from_env();

    // Render ONE polyglot tree (all three hosts' manifests) to dist/.
    let path = byoh::deploy::install_plugin(&bundle, &loc, force)?;
    println!(
        "[byoh] installed '{slug}' → polyglot plugin ({} skills, {} agents) at {}",
        bundle.skills.len(),
        bundle.agents.len(),
        path.display()
    );

    if !host {
        println!(
            "[byoh] (polyglot tree in dist; pass --host to activate claude/codex/agy against it)"
        );
        return Ok(());
    }

    // --host: activate each selected host against the dist tree. `all` activates
    // all three; agy/codex copy the tree into their own root, claude links it.
    let commands = StdCommand::new();
    for t in target.concrete() {
        let report = byoh::deploy::activate_plugin(*t, &path, slug, &loc, &commands)?;
        let prefix = match report.status {
            byoh::deploy::ActivationStatus::Failed => "activation failed —",
            _ => "",
        };
        println!("[byoh] {}: {prefix}{}", t.as_str(), report.message);
    }
    Ok(())
}

/// Resolve and report what an installed harness would run — BYOH renders/installs
/// plugins; the host tool (Claude Code / agy / Codex) is what actually executes.
fn run_run(slug: &str, _lang: &str) -> anyhow::Result<()> {
    let slug = byoh::store::sanitize_slug(slug)?;
    let manifest = byoh::deploy::InstallLocations::from_env()
        .dist
        .join(format!("byoh-{slug}"));
    println!("[byoh] run '{slug}': BYOH installs plugins; the host tool executes them.");
    println!("[byoh] installed plugin (dist): {}", manifest.display());
    println!("[byoh] open your host (Claude Code / agy / Codex) in a project with this plugin to use it.");
    Ok(())
}

/// Run one evolution cycle, persisting seesaw/stagnation state across runs, and
/// report the HONEST decision. Exits non-zero on Rejected / RolledBack so the
/// 3 safety gates are not silently masked.
fn run_evolve(
    slug: &str,
    genre: &str,
    edit_type: &str,
    score_with: f64,
    score_without: f64,
    samples: u32,
) -> anyhow::Result<()> {
    use byoh::application::evolve_run::{decision_is_negative, decision_label, parse_edit_type};
    let genre: Genre = genre.parse()?;
    let edit = parse_edit_type(edit_type)?;
    let metric = byoh::domain::evidence::AbMetric {
        avg_score_with: score_with,
        avg_score_without: score_without,
        samples_with: samples,
        samples_without: samples,
    };
    let (decision, state) =
        byoh::application::evolve_one_cycle(&byoh::store::byoh_home(), slug, genre, edit, metric)?;
    let label = decision_label(&decision);
    println!("[byoh] evolve '{slug}' cycle #{}: {label}", state.cycle_n);
    match &decision {
        byoh::evolve::EvolutionDecision::Approved { critic } => {
            println!("[byoh]   critic: {critic:?}");
        }
        byoh::evolve::EvolutionDecision::Rejected { reason }
        | byoh::evolve::EvolutionDecision::RolledBack { reason } => {
            println!("[byoh]   reason: {reason}");
        }
        byoh::evolve::EvolutionDecision::AutoTuned => {}
    }
    if decision_is_negative(&decision) {
        // Honest non-zero exit: a gate rejected/rolled back the edit.
        std::process::exit(1);
    }
    Ok(())
}

/// Ring 0 hook dispatcher. Recognizes the known lifecycle hooks; an unknown
/// hook name is an explicit error (not a silent no-op).
fn run_hook(name: &str, _lang: &str) -> anyhow::Result<()> {
    const KNOWN: &[&str] = &[
        "session_start",
        "pre_tool_use",
        "post_tool_use",
        "pre_compact",
        "session_end",
    ];
    if !KNOWN.contains(&name) {
        anyhow::bail!("unknown hook '{name}' (known: {})", KNOWN.join(", "));
    }
    // BYOH itself has no runtime side effects to run here — the rendered plugin's
    // hooks.json wires real commands for the host. Report recognition honestly.
    println!(
        "[byoh] hook '{name}' recognized (no BYOH-side action; host runs the plugin's hooks.json)"
    );
    Ok(())
}

/// Collect text documents under `corpus` (.md/.txt/.rs/.py/...) into InputDocs.
fn collect_corpus(corpus: &std::path::Path) -> anyhow::Result<Vec<byoh::rag::InputDoc>> {
    Ok(byoh::store::collect_corpus(corpus)?)
}

fn make_embedder() -> anyhow::Result<Box<dyn byoh::ports::EmbedderProvider>> {
    Ok(byoh::store::make_embedder()?)
}

#[cfg(feature = "native-rag")]
fn make_embedder_native() -> anyhow::Result<Box<dyn byoh::ports::EmbedderProvider>> {
    Ok(byoh::store::make_embedder_native()?)
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
    Ok(byoh::store::load_profile(slug)?)
}

fn write_profile(p: &UserProfile) -> anyhow::Result<()> {
    Ok(byoh::store::write_profile(p)?)
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
