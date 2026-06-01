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

Runner and gateway consumers should reject unsupported major versions and tolerate unknown additive fields for compatible minor versions.

Policy decisions, usage events, and audit events carry `correlation_id` and `request_id` so a user request, policy decision, runtime event, and audit record can be reconstructed as one chain. Usage and audit event payloads are nested under the common event envelope.

## Compatibility

`contracts/compatibility-matrix.json` declares the control-plane source of truth
for the currently supported first-party runner and gateway protocol versions.
`npm run test:compatibility` validates that generated contracts, canonical
fixtures, and the repo-local runner/gateway compatibility fixtures all reference
declared versions and preserve the policy, request, and credential boundaries
expected by `taskotter-remote` and `taskotter-gateway`.
