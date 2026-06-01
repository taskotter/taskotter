# TaskOtter Contracts

This directory contains the canonical source for the first TaskOtter control-plane contracts.

Run:

```sh
npm run contracts:generate
```

The generator writes:

- `packages/api-client/src/index.ts`
- `packages/schemas/json/*.schema.json`
- `packages/schemas/src/index.ts`

Use `npm run contracts:check` in CI to verify generated artifacts are reproducible.

## Versions

- OpenAPI: `0.1.0`, served under `/api/v1`.
- Policy decision fixture: `policy-decision@0.1.0`.
- Event envelope: `0.1.0`, using canonical `id`, `type`, `version`, `occurred_at`, `resource`, and `payload` fields.
- Usage event fixture: `usage.gateway_request.recorded` payload version `0.1.0`.
- Audit event fixture: `audit.policy_decision.denied` payload version `0.1.0`.
- Workflow definition fixture: `workflow-definition@0.1.0`, representing the parsed form of portable workflow YAML.

Runner and gateway consumers should reject unsupported major versions and tolerate unknown additive fields for compatible minor versions.

Policy decisions, usage events, and audit events carry `correlation_id` and `request_id` so a user request, policy decision, runtime event, and audit record can be reconstructed as one chain. Usage and audit event payloads are nested under the common event envelope.

Workflow definitions describe triggers, conditions, jobs, steps, policy checks, approval gates, retry, timeout, concurrency, and audit event references. Webhook triggers require signature verification and replay protection fields. Protected side-effect steps must reference an approval gate that is required before execution. Secrets and external credentials are represented only by `secret_ref` and `integration_ref` pointers; raw secret values do not belong in workflow YAML, fixtures, generated schema artifacts, logs, or comments.

## Compatibility

`contracts/compatibility-matrix.json` declares the control-plane source of truth
for the currently supported first-party runner and gateway protocol versions.
`npm run test:compatibility` validates that generated contracts, canonical
fixtures, and the repo-local runner/gateway compatibility fixtures all reference
declared versions and preserve the policy, request, and credential boundaries
expected by `taskotter-remote` and `taskotter-gateway`.

`contracts/fixtures/audit-chain.synthetic-correlation-run.json` is a synthetic
correlation fixture for reconstructing one generated user request through
control-plane policy and approval evidence, runner dispatch, gateway request,
hosted MCP denial, usage, artifact/log summary, and final result events. It uses
only fake opaque references and redacted summaries; export, delete, legal hold,
and retention behavior remain out of scope for this fixture.
