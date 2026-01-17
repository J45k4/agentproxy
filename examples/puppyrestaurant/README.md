# PuppyRestaurant Example

This example uses the AgentProxy library with a local SQLite database to model restaurant reservations.

## Setup

```bash
cargo run -- --sqlite-path examples/puppyrestaurant/puppyrestaurant.db \
  --policy-file examples/puppyrestaurant/policy.yaml
```

The first run initializes the schema and seed data.

## Sample requests

```bash
curl -X POST http://127.0.0.1:3000/sql/preview \
  -H 'Content-Type: application/json' \
  -d '{"sql":"SELECT * FROM reservations WHERE tenant_id = \"puppyrestaurant\"","context":{"actor":"agent:test","tenant_id":"puppyrestaurant"}}'
```
