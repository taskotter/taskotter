# Backend Contract Notes

This note records the current internal contract boundaries for the TaskOtter product repository.

## Owned Here

- `services/api`: Rust control-plane foundation for domain types, API surface registry, policy decisions, usage events, audit records, runner dispatch references, and gateway request references.
- `packages/schemas`: versioned JSON Schema documents for cross-repository payloads.

## Contract Rules

- Frontend, desktop, and future client packages call versioned backend APIs.
- Runner-facing payloads are represented by `runner-protocol.schema.json`.
- Gateway-facing payloads are represented by `gateway-protocol.schema.json`.
- Policy decisions use AND semantics: every applicable constraint must allow a request.
- Usage events must carry an idempotency key so callers can safely retry.
- Public README content stays high-level; detailed product planning belongs outside public-facing docs.

## Known Gaps

- No HTTP framework is wired yet; `services/api/src/api.rs` only registers draft endpoint intent.
- JSON serialization derives are intentionally deferred until the project chooses the shared schema/codegen approach.
- Authentication provider integration, persistence, migrations, and generated API clients are outside this foundation pass.

