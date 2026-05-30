# TaskOtter Shared Schemas

This package holds versioned contract schemas shared by TaskOtter product surfaces and service boundaries.

The schemas are intentionally narrow at this stage. They define stable naming and payload shape for backend, runner, and gateway implementation work without publishing private roadmap detail.

Current MVP contract artifacts:

- `v1/common.schema.json`: shared principal, resource, and action definitions for trust boundaries.
- `v1/policy-decision-request.schema.json`: policy evaluation input.
- `v1/policy-decision.schema.json`: scoped, expiring policy evaluation output.
- `v1/usage-event.schema.json`: idempotent usage event payload.
- `v1/runner-protocol.schema.json`: runner registration and job dispatch envelope shapes.
- `v1/gateway-protocol.schema.json`: gateway request envelope shape.
