//! Deployment subsystem — registry, bootstrappers, provider matching, i18n,
//! file-based state recovery.

pub mod agent_presets;
pub mod bootstrap;
pub mod genre_map;
pub mod install;
pub mod presets;
pub mod provider;
pub mod registry;
pub mod state;
pub mod vendor;

pub use agent_presets::{agent_catalog, agent_matches, inject_agent, AgentPresetMeta};
pub use bootstrap::{cargo_binstall_toml, install_script_posix, install_script_powershell};
pub use genre_map::infer_genre;
pub use install::{
    activate_plugin, install_plugin, ActivationReport, ActivationStatus, InstallDest,
    InstallLocations,
};
pub use provider::{match_provider, CapabilityProfile};
pub use registry::{Registry, RegistryEntry};
pub use state::{crash_check, BuildStore};
pub use vendor::{
    extract_license, extract_license_from_dir, fetch_git, git_available, load_manifest,
    resolve_source, save_manifest, source_is_trusted, static_validate, vendor_add, vendor_list,
    vendor_remove, vendored_body, VendorEntry, VendorManifest, VendorSource, TRUSTED_SOURCES,
};
