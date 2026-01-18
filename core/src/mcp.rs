use crate::query_engine::SqlRequest;
use crate::query_executor::QueryExecutor;
use crate::service::AppState;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ErrorCode, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AgentProxyMcp {
    state: Arc<RwLock<AppState>>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct QueryIdRequest {
    id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct EmptyRequest {}

impl AgentProxyMcp {
    pub fn new(state: AppState) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            tool_router: Self::tool_router(),
        }
    }

    async fn preview_internal(
        &self,
        payload: SqlRequest,
    ) -> Result<crate::query_engine::PreviewResponse, McpError> {
        let state = self.state.read().await;
        let executor = QueryExecutor::new(state.engine.clone());
        let executed = executor
            .preview(&payload, &state.policy, state.db.as_ref())
            .map_err(|message| McpError::new(ErrorCode::INVALID_REQUEST, message, None))?;

        Ok(executed.preview)
    }

    async fn commit_internal(&self, payload: SqlRequest) -> Result<CallToolResult, McpError> {
        let state = self.state.read().await;
        let executor = QueryExecutor::new(state.engine.clone());
        let executed = executor
            .commit(&payload, &state.policy, state.db.as_ref())
            .map_err(|message| McpError::new(ErrorCode::INTERNAL_ERROR, message, None))?;

        Ok(CallToolResult::success(vec![Content::json(
            executed.preview,
        )?]))
    }

    async fn get_query_internal(&self, id: String) -> Result<CallToolResult, McpError> {
        let state = self.state.read().await;
        let store = state.store.read().await;
        let record = store
            .entries
            .get(&id)
            .map(|entry| &entry.record)
            .ok_or_else(|| McpError::new(ErrorCode::RESOURCE_NOT_FOUND, "Query not found", None))?;
        Ok(CallToolResult::success(vec![Content::json(record)?]))
    }

    async fn policy_internal(&self) -> Result<CallToolResult, McpError> {
        let state = self.state.read().await;
        Ok(CallToolResult::success(vec![Content::json(&state.policy)?]))
    }
}

#[tool_router]
impl AgentProxyMcp {
    #[tool(description = "Preview SQL with policy enforcement")]
    async fn sql_preview(
        &self,
        Parameters(payload): Parameters<SqlRequest>,
    ) -> Result<CallToolResult, McpError> {
        let preview = self.preview_internal(payload).await?;
        Ok(CallToolResult::success(vec![Content::json(preview)?]))
    }

    #[tool(description = "Commit SQL after preview (stub)")]
    async fn sql_commit(
        &self,
        Parameters(payload): Parameters<SqlRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.commit_internal(payload).await
    }

    #[tool(description = "Get stored query metadata")]
    async fn queries_get(
        &self,
        Parameters(payload): Parameters<QueryIdRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.get_query_internal(payload.id).await
    }

    #[tool(description = "Describe active policy config")]
    async fn policy_describe(
        &self,
        Parameters(_): Parameters<EmptyRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.policy_internal().await
    }

    #[tool(description = "Describe schema tables (placeholder)")]
    async fn schema_describe(
        &self,
        Parameters(_): Parameters<EmptyRequest>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            "Schema introspection not wired yet",
        )]))
    }
}

#[tool_handler]
impl ServerHandler for AgentProxyMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "AgentProxy MCP server exposing SQL preview/commit with policy enforcement"
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
