# Backend Contract Notes

This note records the current internal contract boundaries for the TaskOtter product repository.

## Owned Here

- `services/api`: Rust control-plane foundation for domain types, API surface registry, policy decisions, usage events, audit records, runner dispatch references, and gateway request references.
- `packages/schemas`: versioned JSON Schema documents for cross-repository payloads.

## Contract Rules

- Frontend, desktop, and future client packages call versioned backend APIs.
- `common.schema.json` owns shared trust-boundary references such as principal, resource, and action shapes.
- `/v1/policy/decisions` receives `policy-decision-request.schema.json` and returns `policy-decision.schema.json`.
- Runner-facing payloads are represented by `runner-protocol.schema.json`.
- Gateway-facing payloads are represented by `gateway-protocol.schema.json`.
- Policy decisions use AND semantics: every applicable constraint must allow a request, and issued decisions must carry request/decision identifiers, scope, and expiry so runners and gateways do not treat decisions as ambient authority.
- Usage events must carry an idempotency key so callers can safely retry.
- JSON Schema is the current canonical semantic contract for MVP REST/control-plane payloads. If runner or gateway streaming later uses protobuf/gRPC, the protobuf definitions must preserve these field names, identifiers, scope rules, and trust-boundary semantics rather than introducing a divergent contract.
- Public README content stays high-level; detailed product planning belongs outside public-facing docs.

## Known Gaps

- No HTTP framework is wired yet; `services/api/src/api.rs` only registers draft endpoint intent.
- JSON serialization derives are intentionally deferred until the project chooses the shared schema/codegen approach.
- Authentication provider integration, persistence, migrations, and generated API clients are outside this foundation pass.
- Audit event/outcome schemas are not defined yet; `services/api/src/audit.rs` is a Rust type placeholder only.
- Artifact upload/download metadata schemas are not defined yet.
- Runner or gateway log streaming envelopes, ordering, redaction, replay, and retention rules are not defined yet.
- Protobuf/gRPC artifacts are not defined yet; JSON Schema remains the interim source of truth until a streaming transport decision is made.
