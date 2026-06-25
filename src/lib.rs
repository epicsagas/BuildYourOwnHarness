//! BuildYourOwnHarness (BYOH) — the "generation layer".
//!
//! BYOH profiles a user's tacit knowledge / data / genre / goals via a 3-stage
//! hybrid aggregation pipeline (autoscan → interview → wizard), compiles a
//! `ConfirmedProfile` + genre template into an executable [`domain::HarnessBundle`]
//! (4-Ring skeleton), and evolves the installed harness under 3 mandatory safety
//! gates (Critic / Seesaw / Stagnation).
//!
//! Architecture follows hexagonal layout:
//! - [`domain`]  — pure types, no I/O
//! - [`ports`]   — trait boundaries (LlmPort, ProfileSource, EmbedderProvider, …)
//! - [`adapters`]— concrete implementations (rule-based LLM, filesystem scan, Dummy embedder, …)
//! - [`application`] — orchestrators wiring ports together
//! - [`compiler`] — profile → bundle (4-Ring) + validation gates
//! - [`evolve`]  — Ring 3 evolution + 3 safety gates
//! - [`templates`] — genre template library (base inheritance + overrides)
//! - [`rag`]     — self-contained RAG (chunk → embed → index → hybrid search).
//!   `native-rag` cargo feature switches to llm-kernel's TurbovecIndex.
//! - [`deploy`]  — registry + bootstrappers + provider matching + i18n
//! - [`obs`]     — file-based state + 45-min crash recovery
//! - [`i18n`]    — B17 ko/en message catalog
//! - [`security`]— secret masking (PAN / OC keys)
//! - [`cli`]     — clap command tree

#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]

pub mod adapters;
pub mod application;
pub mod cli;
pub mod compiler;
pub mod deploy;
pub mod domain;
pub mod evolve;
pub mod i18n;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod obs;
pub mod ports;
pub mod rag;
pub mod security;
pub mod store;
pub mod templates;

pub use domain::error::{ByohError, Result};
