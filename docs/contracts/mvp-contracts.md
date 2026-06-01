# TaskOtter MVP Contracts

The control plane OpenAPI document at `GET /openapi.json` is the source of
truth for MVP policy, usage, audit, and remote usage ingestion schemas. Gateway
and remote repositories may keep temporary local Rust structs while the shared
schema package is not published, but those local structs must match the OpenAPI
field names, enum values, and schema versions below.

## Policy Decision

`POST /v1/policy/decisions` accepts a policy subject, provider reference, and
operation such as `ai.relay`. It returns the canonical decision shape:

- `allowed`: boolean authorization result.
- `decision_id`: stable identifier for downstream audit and usage records.
- `reason`: optional denial or diagnostic reason.
- `max_tokens`: optional token budget for the authorized attempt.
- `max_cost_micro_usd`: optional cost budget for the authorized attempt.

Consumers must not infer authorization from `reason`; `allowed` is authoritative.

## Gateway Usage And Audit Events

Gateway relay attempts emit `usage_audit_event.v1` to
`POST /v1/usage/events`. The event covers success and failure paths with the same
schema:

- `request_id` and optional `correlation_id` connect the gateway response,
  provider attempt, and ingestion record.
- `decision_id` links the event to the policy response.
- `status` is one of `succeeded`, `denied`, `timeout`, `cancelled`, or `failed`.
- `prompt_tokens`, `completion_tokens`, and `estimated_cost_micro_usd` use zero
  until provider adapters report real usage.

Denied and timeout attempts still produce an event so cost, policy, and audit
pipelines can reconcile attempts without interpreting gateway error bodies.

## Remote Usage Reports

Remote runners report `remote_usage_report.v1` to
`POST /v1/remote/usage-reports`. The report has runner/job resource usage and
the same status enum as gateway usage events. Token and cost fields are included
for jobs that execute AI-provider work; non-provider jobs send zero values.

## Publication Path

For this MVP, schema publication is the generated control-plane OpenAPI
artifact. The next shared-schema step is a generated package or checked artifact
derived from that OpenAPI document, not hand-copied schemas between repositories.
