//! MCP server — exposes BYOH capabilities as stdio MCP tools (PR #3).
//!
//! This is the "empty seat" that inverts control: instead of the `byoh` CLI
//! driving an LLM, an LLM agent discovers and drives BYOH via these tools.
//! Gated behind the `mcp` cargo feature so default builds stay light (no
//! async runtime).

pub mod harness_server;
pub mod params;
pub mod server;
