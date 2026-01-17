use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

const SCHEMA_SQL: &str = include_str!("../schema.sql");
const SEED_SQL: &str = include_str!("../seed.sql");

pub fn ensure_schema(path: impl AsRef<Path>) -> SqlResult<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA_SQL)?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reservations",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        conn.execute_batch(SEED_SQL)?;
    }

    Ok(())
}
