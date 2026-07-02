//! Per-harness stdio MCP server (`byoh harness-serve <slug>`).
//!
//! A rendered harness's `mcp_config.json` points its `command`/`args` back at
//! this binary (`byoh harness-serve <slug>`) instead of a fabricated
//! `byoh-<tool>` binary that never existed. The tool list is loaded at
//! startup from the harness's own `HarnessBundle.mcp_tools` (name/description/
//! input_schema), so it varies per harness — hence a manual `ServerHandler`
//! impl instead of the compile-time `#[tool_router]` macro used by
//! [`crate::mcp::server::ByohServer`].
//!
//! Each tool currently returns a stub `CallToolResult::error` explaining it is
//! not yet wired to a backend, rather than silently succeeding with no
//! effect or failing at the transport level with "command not found".

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::RoleServer;
use rmcp::ServerHandler;
use rmcp::ServiceExt;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ListToolsResult, PaginatedRequestParams, Tool,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;

use crate::domain::bundle::HarnessBundle;

/// The per-harness MCP server. Holds the compiled bundle for one slug.
#[derive(Clone)]
pub struct HarnessServer {
    bundle: Arc<HarnessBundle>,
}

impl HarnessServer {
    pub fn new(bundle: HarnessBundle) -> Self {
        Self {
            bundle: Arc::new(bundle),
        }
    }

    /// Run the stdio MCP server until the client disconnects.
    pub async fn serve_stdio(self) -> Result<(), String> {
        let service = self
            .serve(stdio())
            .await
            .map_err(|e| format!("MCP serve init failed: {e}"))?;
        service
            .waiting()
            .await
            .map_err(|e| format!("MCP serve stopped: {e}"))?;
        Ok(())
    }
}

impl ServerHandler for HarnessServer {
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = self
            .bundle
            .mcp_tools
            .iter()
            .map(|t| {
                let schema = match t.input_schema.as_object() {
                    Some(map) => map.clone(),
                    None => serde_json::Map::new(),
                };
                Tool::new(t.name.clone(), t.description.clone(), schema)
            })
            .collect();
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let Some(tool) = self
            .bundle
            .mcp_tools
            .iter()
            .find(|t| t.name == request.name)
        else {
            return Err(McpError::method_not_found::<
                rmcp::model::CallToolRequestMethod,
            >());
        };
        let message = format!(
            "'{}' is defined by the '{}' harness (genre: {}) but is not yet wired to a backend. \
             Implement its behavior in HarnessServer::call_tool (src/mcp/harness_server.rs) or \
             connect it to the tool your harness intends to call.",
            tool.name, self.bundle.slug, self.bundle.genre,
        );
        Ok(CallToolResult::error(vec![Content::text(message)]))
    }
}
