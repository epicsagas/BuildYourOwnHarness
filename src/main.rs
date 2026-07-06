//! `byoh` binary entry point.

use std::path::PathBuf;

use byoh::adapters::{FilesystemSource, RuleInterview, RuleLlm, StdCommand};
use byoh::application::ProfileOrchestrator;
use byoh::catalog::search::{SearchOptions, catalog_search};
use byoh::cli::{CatalogAction, Cli, Command, ProfileAction};
use byoh::domain::genre::Genre;
use byoh::domain::profile::{ProfileStatus, UserProfile};
use byoh::i18n::{Msg, t};
use byoh::ports::CommandPort;
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
        Command::Doctor => run_doctor(&lang)?,
        Command::Install {
            slug,
            target,
            host,
            scope,
            force,
        } => run_install(&slug, &target, host, scope, force, &lang)?,
        Command::Render { slug, target, out } => run_render(&slug, &target, &out)?,
        Command::Vendor { action } => run_vendor(action, &lang)?,
        Command::Catalog { action } => run_catalog(action)?,
        #[cfg(feature = "mcp")]
        Command::Serve => run_serve(&lang)?,
    }
    Ok(())
}

fn run_catalog(action: CatalogAction) -> anyhow::Result<()> {
    let home = byoh::store::byoh_home();
    match action {
        CatalogAction::Index {
            limit,
            ttl_hours,
            no_bundle,
        } => {
            let cache = byoh::catalog::load_cache(&home)?;
            if byoh::catalog::cache_is_fresh(&cache, ttl_hours) {
                println!(
                    "[byoh catalog] cache is fresh ({} entries, TTL {}h) — skip re-index. Use --ttl-hours 0 to force.",
                    cache.entries.len(),
                    ttl_hours
                );
                return Ok(());
            }
            // 1. Prefer the maintainer-built remote bundle (seconds) unless the
            //    user opted out. On any failure `try_remote_bundle` returns
            //    `None` after logging, and we fall through to a full crawl.
            if !no_bundle {
                if let Some(bundle) = byoh::catalog::index::try_remote_bundle()? {
                    let built = chrono::DateTime::from_timestamp(bundle.built_at as i64, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "?".to_string());
                    byoh::catalog::save_cache(&home, &bundle)?;
                    println!(
                        "[byoh catalog] loaded remote bundle ({} entries, built {built}) → {}",
                        bundle.entries.len(),
                        byoh::catalog::catalog_path(&home).display()
                    );
                    return Ok(());
                }
            }
            // 2. Fallback: crawl directly. `limit == 0` means "all" (bounded by
            //    the sitemap size, ~24k pages). The CLI default is 500 (cli.rs)
            //    so a cold crawl never runs unbounded for hours; `--limit 0` is
            //    the explicit opt-in to a full crawl.
            let cache =
                byoh::catalog::index::catalog_index(&home, limit, ttl_hours, |fetched, total| {
                    if fetched % 100 == 0 || fetched == total {
                        eprint!("\r[byoh catalog] indexing {fetched}/{total}...");
                    }
                })?;
            eprintln!();
            println!(
                "[byoh catalog] indexed {} entries → {}",
                cache.entries.len(),
                byoh::catalog::catalog_path(&home).display()
            );
        }
        CatalogAction::Search {
            query,
            genre,
            tags,
            limit,
        } => {
            let genre_parsed = genre.as_deref().map(|g| g.parse::<Genre>()).transpose()?;
            let tag_list: Vec<String> = tags
                .map(|t| {
                    t.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            let opts = SearchOptions {
                query: &query,
                genre: genre_parsed,
                tags: &tag_list,
                limit,
            };
            let results = catalog_search(&home, &opts)?;
            if results.is_empty() {
                println!("(no results for \"{query}\")");
            } else {
                println!("{:<40} {:<12} {:<6} description", "id", "genre", "stars");
                for e in &results {
                    let g = e
                        .byoh_genre
                        .map(|g| g.as_str().to_string())
                        .unwrap_or_else(|| "?".into());
                    let s = e.stars.map(|n| n.to_string()).unwrap_or_else(|| "-".into());
                    let desc = truncate_str(&e.description, 60);
                    println!("{:<40} {:<12} {:<6} {}", e.id, g, s, desc);
                }
                println!("({} results)", results.len());
            }
        }
        CatalogAction::Vendor {
            plugin_id,
            genre,
            keywords,
        } => {
            let genre_parsed = genre.as_deref().map(|g| g.parse::<Genre>()).transpose()?;
            let extra_kw: Vec<String> = keywords
                .map(|k| {
                    k.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            // Look up the plugin in the catalog cache.
            let mut cache = byoh::catalog::load_cache(&home)?;
            let entry = cache
                .entries
                .iter()
                .find(|e| e.id == plugin_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "plugin '{plugin_id}' not found in catalog cache. Run `byoh catalog index` first."
                    )
                })?
                .clone();
            let repo_root = std::env::current_dir()?;
            let (vendor_entry, enrichment) = byoh::catalog::vendor_from_catalog::catalog_vendor(
                &entry,
                genre_parsed,
                &extra_kw,
                &repo_root,
            )?;
            // Write enriched metadata (license, keywords, genre) back to the
            // catalog cache so subsequent `catalog search` results are richer.
            if let Some(cached) = cache.entries.iter_mut().find(|e| e.id == plugin_id) {
                if cached.license == "unknown" || cached.license.is_empty() {
                    cached.license = enrichment.license.clone();
                }
                if cached.keywords.is_empty() && !enrichment.keywords.is_empty() {
                    cached.keywords = enrichment.keywords.clone();
                }
                if cached.byoh_genre.is_none() {
                    cached.byoh_genre = Some(enrichment.genre);
                }
            }
            byoh::catalog::save_cache(&home, &cache)?;
            println!(
                "vendored '{}' ({}) → registry/vendored/{}/{}.md (sha256 {}...)",
                plugin_id,
                vendor_entry.genre,
                vendor_entry.genre,
                vendor_entry.skill_id,
                &vendor_entry.sha256[..12.min(vendor_entry.sha256.len())]
            );
        }
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
    let slug = byoh::store::sanitize_slug(slug)?;
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

/// Start the BYOH stdio MCP server. Runtime env (BYOH_HOME, language) is fixed
/// here and shared via `Arc` inside the server.
#[cfg(feature = "mcp")]
fn run_serve(lang: &str) -> anyhow::Result<()> {
    let ctx = byoh::mcp::server::ByohContext {
        home: byoh::store::byoh_home(),
        language: lang.to_string(),
    };
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(byoh::mcp::server::ByohServer::new(ctx).serve_stdio())
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn profile_path(slug: &str) -> anyhow::Result<PathBuf> {
    Ok(byoh::store::profile_path(slug)?)
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
                let iv = RuleInterview::new();
                let orch = ProfileOrchestrator::new(&src, &llm, &iv);
                let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
                let mut loaded = load_profile(&slug)?;
                orch.stage1_scan(&mut loaded, &path_refs)?;
                write_profile(&loaded)?;
            }
            println!("{}", t(Msg::Welcome, lang));
            println!("created draft profile: {}", profile_path(&slug)?.display());
        }
        ProfileAction::Confirm { slug, genre, goal } => {
            let llm = RuleLlm::new();
            let iv = RuleInterview::new();
            let src = FilesystemSource::new();
            let orch = ProfileOrchestrator::new(&src, &llm, &iv);
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
    scope: Option<String>,
    force: bool,
    _lang: &str,
) -> anyhow::Result<()> {
    let slug = byoh::store::sanitize_slug(slug)?;
    let target: byoh::domain::render_target::Target = target.parse()?;
    let scope = byoh::deploy::resolve_scope(scope, host)?;
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

    match scope {
        byoh::domain::scope::Scope::DistOnly => println!(
            "[byoh] (polyglot tree in dist; pass --scope local|global|publish, or --host, to activate)"
        ),
        byoh::domain::scope::Scope::Publish => {
            byoh::application::render_plugin::write_publish_extras(&path)?;
            println!(
                "[byoh] publish: added LICENSE + .gitignore to {}",
                path.display()
            );
            println!(
                "[byoh]   cd {} && git init && git add -A && git commit -m 'publish byoh-{slug}'",
                path.display()
            );
            println!("[byoh]   then: git remote add origin <url> && git push -u origin main");
            println!("[byoh]   (auto-push is intentionally not performed; review the tree first.)");
        }
        byoh::domain::scope::Scope::Global => activate_all(target, &path, slug, &loc),
        byoh::domain::scope::Scope::Local => {
            // Point Claude at the project-local .claude/ instead of HOME.
            let local_claude = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".claude");
            let loc_local = loc.with_claude_config(local_claude.clone());
            let commands = StdCommand::new();
            for t in target.concrete() {
                match t {
                    byoh::domain::render_target::Target::Claude => {
                        let report = byoh::deploy::activate_plugin(
                            byoh::domain::render_target::Target::Claude,
                            &path,
                            slug,
                            &loc_local,
                            &commands,
                        )?;
                        println!("[byoh] local: claude → {}", report.message);
                    }
                    other => println!(
                        "[byoh] local: {name} has no project-local plugin scope (its CLI only knows \
                         HOME), so it's not activated here. To use this harness with {name}, either: \
                         (a) re-run with --scope global, or (b) point {name} at the dist tree \
                         directly: {dist}",
                        name = other.as_str(),
                        dist = path.display()
                    ),
                }
            }
        }
    }
    Ok(())
}

/// `--scope global` / legacy `--host`: activate each selected host from HOME.
fn activate_all(
    target: byoh::domain::render_target::Target,
    path: &std::path::Path,
    slug: &str,
    loc: &byoh::deploy::InstallLocations,
) {
    let commands = StdCommand::new();
    for t in target.concrete() {
        match byoh::deploy::activate_plugin(*t, path, slug, loc, &commands) {
            Ok(report) => {
                let prefix = match report.status {
                    byoh::deploy::ActivationStatus::Failed => "activation failed —",
                    _ => "",
                };
                println!("[byoh] {}: {prefix}{}", t.as_str(), report.message);
            }
            Err(e) => println!("[byoh] {}: error: {e}", t.as_str()),
        }
    }
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
