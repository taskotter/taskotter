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
- Review packet fixture: `review-packet@0.1.0`, representing the prototype review packet assembled from work item, plan approval, acceptance criteria status, evidence references, risk signals, uncertainty, rollback path, decision outcome, and audit event references.

Runner and gateway consumers should reject unsupported major versions and tolerate unknown additive fields for compatible minor versions.
Review packet consumers follow the same backward-compatible strategy: additive
minor fields are compatible, while renamed fields, removed fields, or changed
semantics require a new major schema version.

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
only fake opaque references and redacted summaries. The fixture is validated by
`contracts/schemas/audit-chain-fixture.schema.json`; each stage carries the
canonical event envelope fields `id`, `type`, `version`, `occurred_at`, `source`,
`working_group_id`, `actor`, `resource`, `correlation_id`, `request_id`, and
`payload`.

The `usage_event` and `audit_event` stages are canonical event references, not
parallel summary records. They point to the existing usage and audit event
fixtures and schemas so changes to those envelopes are still caught by the
normal schema and compatibility checks.

The remote runner dispatch evidence is intentionally modeled as a dispatch
fragment. `remote.v1alpha1` `job.dispatch` carries only part of the full lineage;
request id and policy-decision lineage must be reconstructed by joining the
runner dispatch fragment to the control-plane chain record. Export, delete,
legal hold, and retention behavior remain out of scope for this fixture.

`contracts/fixtures/review-packet.prototype.json` is the canonical
review-control packet fixture for this prototype slice. It stores only
reference fields and redacted summary references for source context, evidence,
diffs, transcripts, and audit events. Raw transcripts, full diffs, secret
values, customer data, and private roadmap bodies must remain outside review
packet payloads.
