use rusqlite::{Connection, Result as SqlResult};
use std::{path::Path, sync::Mutex};

pub trait SQLDB: Send + Sync {
    fn execute(&self, sql: &str) -> Result<u64, String>;
    fn describe_schema(&self) -> Result<SchemaSnapshot, String>;
}

pub struct SqliteDb {
    connection: Mutex<Connection>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaSnapshot {
    pub tables: Vec<TableSchema>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl SqliteDb {
    pub fn new(path: impl AsRef<Path>) -> SqlResult<Self> {
        Ok(Self {
            connection: Mutex::new(Connection::open(path)?),
        })
    }
}

impl SQLDB for SqliteDb {
    fn execute(&self, sql: &str) -> Result<u64, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "DB connection lock poisoned".to_string())?;
        connection
            .execute(sql, [])
            .map(|rows| rows as u64)
            .map_err(|err| err.to_string())
    }

    fn describe_schema(&self) -> Result<SchemaSnapshot, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "DB connection lock poisoned".to_string())?;
        let mut tables_stmt = connection
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .map_err(|err| err.to_string())?;
        let table_rows = tables_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|err| err.to_string())?;

        let mut tables = Vec::new();
        for table in table_rows {
            let table_name = table.map_err(|err| err.to_string())?;
            let mut columns_stmt = connection
                .prepare(&format!("PRAGMA table_info('{table_name}')"))
                .map_err(|err| err.to_string())?;
            let column_rows = columns_stmt
                .query_map([], |row| {
                    Ok(ColumnSchema {
                        name: row.get::<_, String>(1)?,
                        data_type: row.get::<_, String>(2)?,
                        nullable: row.get::<_, i32>(3)? == 0,
                    })
                })
                .map_err(|err| err.to_string())?;

            let mut columns = Vec::new();
            for column in column_rows {
                columns.push(column.map_err(|err| err.to_string())?);
            }

            tables.push(TableSchema {
                name: table_name,
                columns,
            });
        }

        Ok(SchemaSnapshot { tables })
    }
}

pub fn connect(path: impl AsRef<Path>) -> SqlResult<Connection> {
    Connection::open(path)
}
