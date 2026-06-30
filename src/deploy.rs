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

pub use agent_presets::{AgentPresetMeta, agent_catalog, agent_matches, inject_agent};
pub use bootstrap::{cargo_binstall_toml, install_script_posix, install_script_powershell};
pub use genre_map::infer_genre;
pub use install::{
    ActivationReport, ActivationStatus, InstallLocations, activate_plugin, install_plugin,
    set_dist_override,
};
pub use provider::{CapabilityProfile, match_provider};
pub use registry::{Registry, RegistryEntry};
pub use state::{BuildStore, crash_check};
pub use vendor::{
    TRUSTED_SOURCES, VendorEntry, VendorManifest, VendorSource, extract_license,
    extract_license_from_dir, fetch_git, git_available, load_manifest, resolve_source,
    sanitize_skill_id, save_manifest, source_is_trusted, static_validate, vendor_add, vendor_list,
    vendor_remove, vendored_body,
};
