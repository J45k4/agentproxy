use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::db::SQLDB;
use crate::policy::PolicyConfig;
use crate::query_engine::{CommitResponse, ErrorResponse, QueryEngine, QueryRecord, SqlRequest};
use crate::query_executor::QueryExecutor;

#[derive(Clone, Debug)]
pub(crate) struct StoredQuery {
    pub(crate) record: QueryRecord,
}

#[derive(Default)]
pub(crate) struct QueryStore {
    pub(crate) entries: HashMap<String, StoredQuery>,
}

#[derive(Clone)]
pub struct AppState {
    pub(crate) store: Arc<RwLock<QueryStore>>,
    pub(crate) engine: QueryEngine,
    pub(crate) executor: QueryExecutor,
    pub(crate) policy: PolicyConfig,
    pub(crate) db: Option<Arc<dyn SQLDB>>,
}

impl AppState {
    pub fn new(policy: PolicyConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(QueryStore::default())),
            engine: QueryEngine::default(),
            executor: QueryExecutor::default(),
            policy,
            db: None,
        }
    }

    pub fn with_db(mut self, db: Arc<dyn SQLDB>) -> Self {
        self.db = Some(db);
        self
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
    match state
        .executor
        .preview(&payload, &state.policy, state.db.as_ref())
    {
        Ok(executed) => {
            let record = QueryRecord {
                id: executed.preview.preview_id.clone(),
                actor: payload.context.actor.clone(),
                tenant_id: payload.context.tenant_id.clone(),
                sql: executed.rewritten_sql,
                status: "previewed".to_string(),
                created_at: Utc::now(),
                operation: executed.preview.operation.clone(),
                tables: executed.preview.tables.clone(),
            };

            let stored = StoredQuery { record };

            let mut store = state.store.write().await;
            store
                .entries
                .insert(executed.preview.preview_id.clone(), stored);

            (StatusCode::OK, Json(executed.preview)).into_response()
        }
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn commit_sql(State(state): State<AppState>, Json(payload): Json<SqlRequest>) -> Response {
    let preview = match state
        .executor
        .commit(&payload, &state.policy, state.db.as_ref())
    {
        Ok(executed) => executed.preview,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
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
        rows_affected: preview.rows_affected,
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
