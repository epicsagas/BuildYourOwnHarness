//! Deployment subsystem — provider matching, presets, install, vendoring.

pub mod agent_presets;
pub mod genre_map;
pub mod install;
pub mod presets;
pub mod provider;
pub mod vendor;

pub use agent_presets::{AgentPresetMeta, agent_catalog, agent_matches, inject_agent};
pub use genre_map::infer_genre;
pub use install::{
    ActivationReport, ActivationStatus, InstallLocations, activate_plugin, install_plugin,
    resolve_scope, set_dist_override,
};
pub use provider::{CapabilityProfile, match_provider};
pub use vendor::{
    TRUSTED_SOURCES, VendorEntry, VendorManifest, VendorSource, extract_keywords_from_dir,
    extract_license, extract_license_from_dir, fetch_git, git_available, load_manifest,
    resolve_source, sanitize_skill_id, save_manifest, source_is_trusted, static_validate,
    vendor_add, vendor_list, vendor_remove, vendored_body,
};
