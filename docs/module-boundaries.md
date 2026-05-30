# Module Boundaries

TaskOtter starts with contracts as the shared boundary between product clients, the control-plane backend, runner runtimes, and gateway runtimes.

## Dependency Direction

- `contracts` is the canonical source for OpenAPI and JSON Schema contracts.
- `packages/api-client` is generated from `contracts/openapi`.
- `packages/schemas` is generated from `contracts/schemas`.
- Frontend app surfaces may import `packages/api-client` and `packages/schemas`.
- Frontend app surfaces must not import `services/api`, runner, or gateway internals.
- Runner and gateway repositories consume versioned fixtures and generated contract artifacts, not control-plane database models or policy engine implementation details.

## Guard

Run:

```sh
npm run check:module-boundaries
```

The guard scans `apps/web`, `apps/desktop`, `apps/mobile`, `packages/ui`, and `packages/workflow` when those directories exist. It fails on direct imports from `services/api`, `services/runner`, or `services/gateway`, while allowing generated contract imports.

This keeps first-party clients on the generated contract surface and leaves backend internals behind control-plane service boundaries.
