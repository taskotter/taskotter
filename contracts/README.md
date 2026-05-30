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
- Usage event fixture: `usage-event@0.1.0`.
- Audit event fixture: `audit-event@0.1.0`.

Runner and gateway consumers should reject unsupported major versions and tolerate unknown additive fields for compatible minor versions.
