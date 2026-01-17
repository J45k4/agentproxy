use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

pub fn connect(path: impl AsRef<Path>) -> SqlResult<Connection> {
    Connection::open(path)
}
