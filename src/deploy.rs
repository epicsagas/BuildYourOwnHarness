//! Deployment subsystem — registry, bootstrappers, provider matching, i18n,
//! file-based state recovery.

pub mod bootstrap;
pub mod presets;
pub mod provider;
pub mod registry;
pub mod state;

pub use bootstrap::{cargo_binstall_toml, install_script_posix, install_script_powershell};
pub use provider::{match_provider, CapabilityProfile};
pub use registry::{Registry, RegistryEntry};
pub use state::{crash_check, BuildStore};
