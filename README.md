# AgentProxy (Rust)

Minimal Rust scaffold for an AgentProxy SQL preview/commit service.

## Run

```bash
cargo run -p agentproxy-cli -- --policy-file examples/policy.yaml --sqlite-path examples/puppyrestaurant/puppyrestaurant.db
```

The service listens on `http://127.0.0.1:3000` and loads policy config from `examples/policy.yaml` (override with `--policy-file`).

## Example workspace

With the workspace in place you can also run the PuppyRestaurant demo separately:

```bash
cd examples/puppyrestaurant
cargo run
```
## Policy engine

The policy engine evaluates SQL requests before preview/commit and enforces a mix of global rules (hard safety checks) and role-based table rules.

- **Parse + classify**: SQL is parsed into an AST and classified (SELECT/INSERT/UPDATE/DELETE).
- **Global guards**: Rejects destructive DDL and UPDATE/DELETE without a WHERE clause.
- **Role tables**: `context.role` selects a role section in the policy file. Table rules define allowed operations and required filters.
- **Column protection**: `deny_columns` blocks queries that reference sensitive columns.
- **Fallback**: If a table has no role rule, the global `tables` section is used.

Policy config lives in YAML/JSON (see `examples/puppyrestaurant/policy.yaml`) and is loaded at startup.

## Example requests

Preview:

```bash
curl -X POST http://127.0.0.1:3000/sql/preview \
  -H 'Content-Type: application/json' \
  -d '{"sql":"SELECT * FROM users WHERE tenant_id = \"acme\"","context":{"actor":"agent:gpt-4.1","tenant_id":"acme"}}'
```

Commit:

```bash
curl -X POST http://127.0.0.1:3000/sql/commit \
  -H 'Content-Type: application/json' \
  -d '{"sql":"UPDATE users SET name = \"Jane\" WHERE tenant_id = \"acme\"","context":{"actor":"agent:gpt-4.1","tenant_id":"acme"}}'
```

Query status:

```bash
curl http://127.0.0.1:3000/queries/<preview_id>
```
