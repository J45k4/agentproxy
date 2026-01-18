use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

const SCHEMA_SQL: &str = include_str!("../schema.sql");
const SEED_SQL: &str = include_str!("../seed.sql");

pub fn ensure_schema(path: impl AsRef<Path>) -> SqlResult<()> {
    let conn = Connection::open(path)?;
    let has_schema = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'reservations'",
        [],
        |row| row.get::<_, i64>(0),
    )? > 0;

    if !has_schema {
        conn.execute_batch(SCHEMA_SQL)?;
        conn.execute_batch(SEED_SQL)?;
    }

    Ok(())
}
