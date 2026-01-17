use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::policy::PolicyConfig;
use crate::query_engine::{
    CommitResponse, ErrorResponse, PreviewResponse, QueryEngine, QueryRecord, SqlRequest,
};

#[derive(Clone, Debug)]
struct StoredQuery {
    record: QueryRecord,
}

#[derive(Default)]
struct QueryStore {
    entries: HashMap<String, StoredQuery>,
}

#[derive(Clone)]
pub struct AppState {
    store: Arc<RwLock<QueryStore>>,
    engine: QueryEngine,
    policy: PolicyConfig,
}

impl AppState {
    pub fn new(policy: PolicyConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(QueryStore::default())),
            engine: QueryEngine::default(),
            policy,
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/sql/preview", post(preview_sql))
        .route("/sql/commit", post(commit_sql))
        .route("/queries/:id", get(get_query))
        .with_state(state)
}

async fn preview_sql(State(state): State<AppState>, Json(payload): Json<SqlRequest>) -> Response {
    match state.engine.evaluate_sql(&payload) {
        Ok(parsed) => match state.engine.enforce_rules(&payload, &parsed) {
            Ok(()) => match state.engine.enforce_policy(&payload, &parsed, &state.policy) {
                Ok(()) => {
                    let preview_id = Uuid::new_v4().to_string();
                    let response = PreviewResponse {
                        ok: true,
                        preview_id: preview_id.clone(),
                        operation: parsed.operation.clone(),
                        tables: parsed.tables.clone(),
                        rows_affected: 0,
                        warnings: vec![
                            "Preview executed in dry-run mode; no database configured".to_string(),
                        ],
                    };

                    let record = QueryRecord {
                        id: preview_id.clone(),
                        actor: payload.context.actor.clone(),
                        tenant_id: payload.context.tenant_id.clone(),
                        sql: payload.sql.clone(),
                        status: "previewed".to_string(),
                        created_at: Utc::now(),
                        operation: parsed.operation,
                        tables: response.tables.clone(),
                    };

                    let stored = StoredQuery { record };

                    let mut store = state.store.write().await;
                    store.entries.insert(preview_id, stored);

                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(message) => error_response(StatusCode::FORBIDDEN, message),
            },
            Err(message) => error_response(StatusCode::BAD_REQUEST, message),
        },
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn commit_sql(State(state): State<AppState>, Json(payload): Json<SqlRequest>) -> Response {
    let preview = preview_sql(State(state.clone()), Json(payload)).await;
    let response = preview.into_response();
    let status = response.status();
    if status != StatusCode::OK {
        return response;
    }

    let body = response.into_body();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Failed to parse preview"),
    };

    let preview_result: Result<PreviewResponse, _> = serde_json::from_slice(&bytes);
    let preview = match preview_result {
        Ok(preview) if preview.ok => preview,
        _ => return error_response(StatusCode::BAD_REQUEST, "Preview failed"),
    };

    let mut store = state.store.write().await;
    let stored = match store.entries.get_mut(&preview.preview_id) {
        Some(stored) => stored,
        None => return error_response(StatusCode::NOT_FOUND, "Preview not found"),
    };

    stored.record.status = "committed".to_string();

    let response = CommitResponse {
        ok: true,
        preview_id: preview.preview_id,
        committed_at: Utc::now(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn get_query(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let store = state.store.read().await;
    match store.entries.get(&id) {
        Some(stored) => (StatusCode::OK, Json(&stored.record)).into_response(),
        None => error_response(StatusCode::NOT_FOUND, "Query not found"),
    }
}

fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    let response = ErrorResponse {
        ok: false,
        error: message.into(),
    };
    (status, Json(response)).into_response()
}
