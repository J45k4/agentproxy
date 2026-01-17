# AgentProxy (Rust)

Minimal Rust scaffold for an AgentProxy SQL preview/commit service.

## Run

```bash
cargo run -p agentproxy -- --policy-file examples/policy.yaml --sqlite-path examples/puppyrestaurant/puppyrestaurant.db
```

The service listens on `http://127.0.0.1:3000` and loads policy config from `examples/policy.yaml` (override with `--policy-file`).

## Example workspace

With the workspace in place you can also run the PuppyRestaurant demo separately:

```bash
cd examples/puppyrestaurant
cargo run
```
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
