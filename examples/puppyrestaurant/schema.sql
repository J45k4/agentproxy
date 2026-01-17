CREATE TABLE reservations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guest_name TEXT NOT NULL,
    party_size INTEGER NOT NULL,
    reserved_at TEXT NOT NULL,
    table_number INTEGER NOT NULL,
    tenant_id TEXT NOT NULL
);

CREATE TABLE diners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    vip_level TEXT NOT NULL,
    tenant_id TEXT NOT NULL
);
