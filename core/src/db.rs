use rusqlite::{Connection, Result as SqlResult};
use std::{path::Path, sync::Mutex};

pub trait SQLDB: Send + Sync {
    fn execute(&self, sql: &str) -> Result<u64, String>;
}

pub struct SqliteDb {
    connection: Mutex<Connection>,
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
}

pub fn connect(path: impl AsRef<Path>) -> SqlResult<Connection> {
    Connection::open(path)
}
