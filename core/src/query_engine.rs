use serde::{Deserialize, Serialize};
use sqlparser::{
    ast::{ObjectName, Statement, TableFactor},
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::policy::{PolicyConfig, RequiredFilter, TablePolicy};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryContext {
    pub actor: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SqlRequest {
    pub sql: String,
    pub context: QueryContext,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreviewResponse {
    pub ok: bool,
    pub preview_id: String,
    pub operation: String,
    pub tables: Vec<String>,
    pub rows_affected: u64,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommitResponse {
    pub ok: bool,
    pub preview_id: String,
    pub committed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueryRecord {
    pub id: String,
    pub actor: String,
    pub tenant_id: String,
    pub sql: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub tables: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ParsedQuery {
    pub operation: String,
    pub tables: Vec<String>,
    pub has_where: bool,
}

#[derive(Clone, Debug, Default)]
pub struct QueryEngine;

impl QueryEngine {
    pub fn evaluate_sql(&self, payload: &SqlRequest) -> Result<ParsedQuery, String> {
        let dialect = PostgreSqlDialect {};
        let statements = Parser::parse_sql(&dialect, &payload.sql)
            .map_err(|err| format!("SQL parse error: {err}"))?;

        if statements.len() != 1 {
            return Err("Only single-statement SQL is supported".to_string());
        }

        let statement = statements
            .into_iter()
            .next()
            .ok_or_else(|| "Missing SQL statement".to_string())?;

        let (operation, tables, has_where) = analyze_statement(statement)?;

        Ok(ParsedQuery {
            operation,
            tables,
            has_where,
        })
    }

    pub fn enforce_rules(&self, payload: &SqlRequest, parsed: &ParsedQuery) -> Result<(), String> {
        if (parsed.operation == "update" || parsed.operation == "delete") && !parsed.has_where {
            return Err("UPDATE/DELETE requires a WHERE clause".to_string());
        }

        if !parsed.tables.is_empty()
            && !payload.sql.contains("tenant_id")
            && payload.context.tenant_id != "*"
        {
            return Err("Tenant filter missing; tenant_id must be enforced".to_string());
        }

        Ok(())
    }

    pub fn enforce_policy(
        &self,
        payload: &SqlRequest,
        parsed: &ParsedQuery,
        policy: &PolicyConfig,
    ) -> Result<(), String> {
        for table in &parsed.tables {
            if let Some(table_policy) = policy.tables.get(table) {
                self.validate_table_policy(payload, parsed, table_policy)?;
            }
        }

        Ok(())
    }

    fn validate_table_policy(
        &self,
        payload: &SqlRequest,
        parsed: &ParsedQuery,
        table_policy: &TablePolicy,
    ) -> Result<(), String> {
        if !table_policy.allow_ops.is_empty()
            && !table_policy.allow_ops.iter().any(|op| op == &parsed.operation)
        {
            return Err(format!(
                "Operation '{}' is not allowed for table",
                parsed.operation
            ));
        }

        for required in &table_policy.required_filters {
            ensure_required_filter(payload, required)?;
        }

        if !table_policy.deny_columns.is_empty()
            && table_policy
                .deny_columns
                .iter()
                .any(|column| payload.sql.contains(column))
        {
            return Err("Query references denied columns".to_string());
        }

        Ok(())
    }
}

fn ensure_required_filter(payload: &SqlRequest, required: &RequiredFilter) -> Result<(), String> {
    if payload.sql.contains(&required.column) {
        Ok(())
    } else {
        Err(format!(
            "Missing required filter on column '{}'",
            required.column
        ))
    }
}

fn analyze_statement(statement: Statement) -> Result<(String, Vec<String>, bool), String> {
    match statement {
        Statement::Query(query) => {
            let tables = extract_tables_from_query(&query.to_string());
            Ok(("select".to_string(), tables, true))
        }
        Statement::Insert { table_name, .. } => Ok((
            "insert".to_string(),
            vec![table_name_to_string(&table_name)],
            true,
        )),
        Statement::Update { table, selection, .. } => Ok((
            "update".to_string(),
            vec![table_name_from_table_with_joins(&table)],
            selection.is_some(),
        )),
        Statement::Delete { from, selection, .. } => {
            let mut tables = Vec::new();
            if let Some(table) = from.first() {
                tables.push(table_name_from_table_with_joins(table));
            }
            Ok(("delete".to_string(), tables, selection.is_some()))
        }
        Statement::Drop { .. } | Statement::AlterTable { .. } | Statement::Truncate { .. } => {
            Err("Destructive DDL statements are not allowed".to_string())
        }
        _ => Err("Statement type not supported".to_string()),
    }
}

fn extract_tables_from_query(query: &str) -> Vec<String> {
    let mut tables = Vec::new();
    let lower = query.to_lowercase();
    if let Some(from_pos) = lower.find(" from ") {
        let remainder = &query[from_pos + 6..];
        if let Some(table) = remainder.split_whitespace().next() {
            tables.push(table.trim_matches('"').to_string());
        }
    }
    tables
}

fn table_name_from_table_with_joins(table: &sqlparser::ast::TableWithJoins) -> String {
    match &table.relation {
        TableFactor::Table { name, .. } => table_name_to_string(name),
        _ => "unknown".to_string(),
    }
}

fn table_name_to_string(name: &ObjectName) -> String {
    name.0
        .iter()
        .map(|ident| ident.value.clone())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(sql: &str) -> SqlRequest {
        SqlRequest {
            sql: sql.to_string(),
            context: QueryContext {
                actor: "agent:test".to_string(),
                tenant_id: "acme".to_string(),
            },
        }
    }

    #[test]
    fn rejects_multiple_statements() {
        let engine = QueryEngine::default();
        let payload = request("SELECT 1; SELECT 2");
        let error = engine.evaluate_sql(&payload).unwrap_err();
        assert!(error.contains("single-statement"));
    }

    #[test]
    fn blocks_delete_without_where() {
        let engine = QueryEngine::default();
        let payload = request("DELETE FROM users");
        let parsed = engine.evaluate_sql(&payload).unwrap();
        let error = engine.enforce_rules(&payload, &parsed).unwrap_err();
        assert!(error.contains("WHERE clause"));
    }

    #[test]
    fn allows_delete_with_where() {
        let engine = QueryEngine::default();
        let payload = request("DELETE FROM users WHERE tenant_id = 'acme'");
        let parsed = engine.evaluate_sql(&payload).unwrap();
        engine.enforce_rules(&payload, &parsed).unwrap();
    }

    #[test]
    fn blocks_missing_tenant_filter() {
        let engine = QueryEngine::default();
        let payload = request("SELECT * FROM users");
        let parsed = engine.evaluate_sql(&payload).unwrap();
        let error = engine.enforce_rules(&payload, &parsed).unwrap_err();
        assert!(error.contains("tenant_id"));
    }

    #[test]
    fn extracts_table_for_update() {
        let engine = QueryEngine::default();
        let payload = request("UPDATE users SET name = 'Jane' WHERE tenant_id = 'acme'");
        let parsed = engine.evaluate_sql(&payload).unwrap();
        assert_eq!(parsed.tables, vec!["users".to_string()]);
        assert!(parsed.has_where);
    }
}
