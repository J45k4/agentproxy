use crate::{
    db::SQLDB,
    policy::PolicyConfig,
    query_engine::{PreviewResponse, QueryEngine, SqlRequest},
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct QueryExecutor {
    engine: QueryEngine,
}

#[derive(Clone, Debug)]
pub struct ExecutedQuery {
    pub preview: PreviewResponse,
    pub rewritten_sql: String,
}

impl QueryExecutor {
    pub fn new(engine: QueryEngine) -> Self {
        Self { engine }
    }

    pub fn preview(
        &self,
        payload: &SqlRequest,
        policy: &PolicyConfig,
        db: Option<&Arc<dyn SQLDB>>,
    ) -> Result<ExecutedQuery, String> {
        let (parsed, rewritten_sql) = self.engine.evaluate_sql(payload)?;
        self.engine.enforce_rules(payload, &parsed)?;
        self.engine.enforce_policy(payload, &parsed, policy)?;

        let preview_id = Uuid::new_v4().to_string();
        let preview = PreviewResponse {
            ok: true,
            preview_id,
            operation: parsed.operation,
            tables: parsed.tables,
            rows_affected: 0,
            rewritten_sql: rewritten_sql.clone(),
            warnings: if db.is_some() {
                Vec::new()
            } else {
                vec!["Preview executed in dry-run mode; no database configured".to_string()]
            },
        };

        Ok(ExecutedQuery {
            preview,
            rewritten_sql,
        })
    }

    pub fn commit(
        &self,
        payload: &SqlRequest,
        policy: &PolicyConfig,
        db: Option<&Arc<dyn SQLDB>>,
    ) -> Result<ExecutedQuery, String> {
        let mut executed = self.preview(payload, policy, db)?;
        if let Some(db) = db {
            let rows = db.execute(&executed.rewritten_sql)?;
            executed.preview.rows_affected = rows;
        }

        Ok(executed)
    }
}
