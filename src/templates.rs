//! Genre template library — base inheritance + 4 child templates (ARCH §6).
//!
//! `base` holds the immutable Ring 0-3 skeleton + 3 safety gates. Children
//! (`developer`, `creator`, `researcher`, `business`) extend it and override
//! skill bodies / tool blueprints / domain entity types only (ARCH §6.1).

pub mod agents;
pub mod base;
pub mod inherit;
pub mod library;

pub use base::base_template;
pub use inherit::merge_child_into_base;
pub use library::TemplateLibrary;
