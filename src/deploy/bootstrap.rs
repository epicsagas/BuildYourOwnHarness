//! Bootstrappers — install.sh / install.ps1 / cargo-binstall (ARCH §9.1, B16).
//!
//! These are generators: they produce the installer *content* for a registry
//! entry. The single common entry point is `byoh run <slug>`.

use crate::deploy::registry::RegistryEntry;

/// Generate the POSIX install.sh (macOS/Linux). Verifies dependency versions
/// via `byoh doctor`, then activates the bundle.
pub fn install_script_posix(entry: &RegistryEntry) -> String {
    let slug = &entry.slug;
    let mut deps = String::new();
    for d in &entry.depends_on {
        deps.push_str(&format!(
            "  byoh doctor --require {id}@{min} || warn \"dependency {id} missing\"\n",
            id = d.id,
            min = d.min_version
        ));
    }
    format!(
        "#!/usr/bin/env sh\n\
         # BYOH bootstrap for {slug} (POSIX)\n\
         set -e\n\
         warn() {{ echo \"[byoh] WARN: $*\" >&2; }}\n\
         echo \"[byoh] installing bundle {slug}\"\n\
         {deps}\
         # common entrypoint — install method does not change runtime\n\
         exec byoh run {slug}\n",
        slug = slug,
        deps = deps
    )
}

/// Generate the Windows install.ps1.
pub fn install_script_powershell(entry: &RegistryEntry) -> String {
    let slug = &entry.slug;
    let mut deps = String::new();
    for d in &entry.depends_on {
        deps.push_str(&format!(
            "byoh doctor --require {id}@{min}; if ($LASTEXITCODE -ne 0) {{ Write-Warning \"dependency {id} missing\" }}\n",
            id = d.id,
            min = d.min_version
        ));
    }
    format!(
        "# BYOH bootstrap for {slug} (PowerShell)\n\
         $ErrorActionPreference = 'Stop'\n\
         Write-Host \"[byoh] installing bundle {slug}\"\n\
         {deps}\
         & byoh run {slug}\n",
        slug = slug,
        deps = deps
    )
}

/// Generate the cargo-binstall TOML snippet (Rust toolchain fast path).
pub fn cargo_binstall_toml(entry: &RegistryEntry) -> String {
    format!(
        "[package]\nname = \"byoh-{slug}\"\nversion = \"{ver}\"\n\n\
         [binstall]\n\
         # fetches the prebuilt bundle and runs `byoh run {slug}`\n",
        slug = entry.slug,
        ver = entry.bundle_version
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deploy::registry::DependencyRef;
    use crate::domain::genre::Genre;

    fn entry() -> RegistryEntry {
        RegistryEntry {
            id: "byoh-dev1".into(),
            slug: "dev1".into(),
            genre: Genre::Developer,
            bundle_version: "1.0.0".into(),
            source_profile_hash: "sha256:abc".into(),
            install_methods: vec!["script".into()],
            depends_on: vec![
                DependencyRef {
                    id: "alcove".into(),
                    min_version: "0.1.0".into(),
                },
                DependencyRef {
                    id: "epic-harness".into(),
                    min_version: "0.1.0".into(),
                },
            ],
        }
    }

    #[test]
    fn posix_script_checks_deps_and_runs() {
        let s = install_script_posix(&entry());
        assert!(s.contains("byoh doctor --require alcove@0.1.0"));
        assert!(s.contains("exec byoh run dev1"));
        assert!(s.starts_with("#!/usr/bin/env sh"));
    }

    #[test]
    fn powershell_script_runs() {
        let s = install_script_powershell(&entry());
        assert!(s.contains("byoh run dev1"));
        assert!(s.contains("alcove@0.1.0"));
    }

    #[test]
    fn cargo_binstall_has_package_name() {
        let t = cargo_binstall_toml(&entry());
        assert!(t.contains("name = \"byoh-dev1\""));
    }
}
