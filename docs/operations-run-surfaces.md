# Operations Run Surfaces Design Spec

Status: draft for implementation planning.

Product/UX mode: standard.

Audience: frontend, control-plane API, runner, gateway, provider, and QA owners implementing TaskOtter operational visibility after the initial contract scaffold.

## Purpose

TaskOtter needs operational surfaces that let Working Group administrators, reviewers, and assigned users understand delegated agent work without reading raw service logs. These surfaces must preserve the quiet, dense, desktop SaaS design direction while exposing enough evidence to diagnose queue delays, policy or usage denials, approval waits, runner failures, gateway issues, and provider degradation.

This spec covers three repo-local product surfaces:

- Per-run timeline in issue, chat, automation, and run detail contexts.
- Admin operations overview for Working Group administrators.
- Runner, gateway, and provider health list with drill-down.

It does not define production observability infrastructure, backend persistence, external monitoring vendors, or the final API schema.

## Source Boundaries

Use the control plane as the only frontend data authority. First-party clients should read generated API client and schema artifacts from `packages/api-client` and `packages/schemas` when contracts exist. Frontend surfaces must not call runner or gateway runtimes directly for ordinary product behavior.

Runtime evidence should flow back to the control plane as scoped structured events, then be exposed to authorized clients through control-plane APIs. Runner and gateway repositories remain implementation owners for local diagnostics, heartbeats, and runtime-specific event production.

Do not include private roadmap details, credentials, raw logs, prompts, provider payloads, customer payloads, private hostnames, stack traces, or raw secret values in public docs, fixtures, UI fixtures, comments, screenshots, or generated examples. Use stable IDs, resource references, summary labels, and redacted diagnostic snippets.

## Primary Users

- Working Group admin: needs tenant-scoped operational confidence, setup gaps, degraded runtime visibility, usage or policy pressure, and recovery paths.
- Issue owner or reviewer: needs to understand why a run is waiting, failed, denied, retried, cancelled, or ready for review.
- Agent operator or support user: needs enough correlation evidence to route a problem to frontend, control plane, runner, gateway, provider, integration, policy, or QA without exposing sensitive payloads.

## Information Architecture

### Navigation Placement

Operations should appear as an admin surface under the existing app shell rather than a marketing dashboard. Recommended entry points:

- `Issues` focus panel: per-run timeline for the selected issue run.
- `Chat` run detail: same timeline component filtered to the selected chat run.
- `Automations` run detail: same timeline component filtered to workflow run evidence.
- `Settings` or `Admin > Operations`: Working Group operations overview.
- `Settings > Runners`, `Settings > Gateways`, and `Settings > Providers`: health tables and resource drill-downs.

### Layout Model

Follow the existing three-zone design direction:

- Navigation zone: global product/admin destinations.
- Workspace zone: dense lists, filters, grouped operations sections, and health tables.
- Focus zone: run timeline, event detail, resource drill-down, or recovery action panel.

Avoid decorative cards, hero-style dashboards, large single-metric panels, or colorful marketing visuals. Status color should be limited to dots, badges, icons, and compact alerts with text labels.

## Status Taxonomy

All run and operations surfaces should support this product taxonomy, even if initial contracts expose only a subset:

| Family     | Status               | User-facing meaning                                                                               |
| ---------- | -------------------- | ------------------------------------------------------------------------------------------------- |
| Queue      | `queued`             | Accepted and waiting for dispatch.                                                                |
| Queue      | `dispatching`        | Control plane is selecting a runner, gateway, provider, or workflow route.                        |
| Execution  | `starting`           | Runtime accepted the job and is preparing work.                                                   |
| Execution  | `running`            | Work is actively progressing.                                                                     |
| Execution  | `progress`           | A safe progress update or milestone was recorded.                                                 |
| Waiting    | `waiting_approval`   | A human approval is required before protected work continues.                                     |
| Waiting    | `waiting_policy`     | Policy or usage evaluation is pending.                                                            |
| Waiting    | `waiting_dependency` | Runtime, provider, integration, or workflow dependency is unavailable or delayed.                 |
| Retry      | `retrying`           | A retry is scheduled or in progress.                                                              |
| Stop       | `cancelling`         | Cancellation was requested and cleanup is in progress.                                            |
| Stop       | `cancelled`          | Work stopped by user, policy, or system cancellation.                                             |
| Stop       | `timed_out`          | Work exceeded its timeout.                                                                        |
| Failure    | `failed`             | Work ended unsuccessfully.                                                                        |
| Success    | `completed`          | Work finished and result evidence is available.                                                   |
| Governance | `policy_denied`      | Policy rejected the requested action.                                                             |
| Governance | `usage_limited`      | Usage or cost limit blocked the action.                                                           |
| Governance | `approval_denied`    | Required approval was rejected or expired.                                                        |
| Health     | `runner_degraded`    | Runner is reachable but cannot satisfy all expected capability or capacity needs.                 |
| Health     | `gateway_degraded`   | Gateway is reachable but dependency, adapter, streaming, or usage export health is impaired.      |
| Health     | `provider_degraded`  | Provider route is enabled but degraded by latency, errors, limit pressure, or model availability. |

Initial UI can map these to fewer visual severities:

- Neutral: queued, dispatching, waiting_dependency.
- Info: starting, running, progress.
- Warning: waiting_approval, waiting_policy, retrying, cancelling, degraded states.
- Danger: policy_denied, usage_limited, approval_denied, timed_out, failed.
- Success: completed.

Do not rely on color alone. Every state needs visible text and an icon or dot shape.

## Surface 1: Per-Run Timeline

### User Job

As an issue owner, reviewer, or admin, I need to inspect one run and answer:

- What triggered this work?
- Which agent, skill, runner, gateway, provider, and integration participated?
- What is happening now?
- Why is work blocked, waiting, failed, retried, cancelled, or complete?
- What can I safely do next?

### Entry Points

- Issue focus panel run section.
- Chat run detail.
- Automation run detail.
- Admin operations overview linked run row.
- Health drill-down linked affected run.

### Timeline Structure

The timeline should render as a compact chronological list with stable row height and expandable details.

Each item should include:

- Status dot or icon plus status label.
- Short event title.
- Timestamp with timezone-aware formatting.
- Actor or source label.
- Safe resource references.
- Summary text.
- Optional action affordance when allowed.
- Expandable metadata drawer for correlation evidence.

Recommended grouping:

- Trigger and context.
- Policy and usage checks.
- Queue and dispatch.
- Runtime execution.
- Gateway/provider calls.
- Approval waits.
- Retry, cancellation, timeout, or failure.
- Completion and handoff.

### Minimum Data Fields

Per run:

- `run_id`
- `working_group_id`
- `subject_type` such as `issue`, `chat`, or `workflow`
- `subject_id`
- `trigger_type`
- `triggered_by_actor_id`
- `agent_id`
- `status`
- `started_at`
- `updated_at`
- `completed_at`
- `correlation_id`
- `request_id`
- `policy_decision_ids`
- `usage_event_ids`
- `audit_event_ids`

Per timeline item:

- `event_id`
- `run_id`
- `sequence`
- `occurred_at`
- `source` such as `control_plane`, `runner`, `gateway`, or `provider`
- `status`
- `title`
- `summary`
- `severity`
- `resource_type`
- `resource_id`
- `safe_detail`
- `correlation_id`
- `request_id`
- `redaction_state`
- `action_kind`
- `action_authorization`

Optional drill-down references:

- `runner_id`
- `gateway_id`
- `provider_id`
- `integration_id`
- `workflow_id`
- `approval_request_id`
- `artifact_ids`
- `retry_of_event_id`
- `cancelled_by_actor_id`
- `timeout_policy_id`

### Redaction Rules

The timeline may show:

- Resource names visible to the caller.
- Stable public or scoped IDs.
- Policy decision outcome, reason code, and safe summary.
- Usage measurements and estimated cost ranges when authorized.
- Tool or integration names.
- Artifact metadata and links when authorized.
- Safe diagnostic details already allowlisted by the control plane.

The timeline must not show:

- Raw prompts or model responses unless explicitly stored as user-visible run output.
- Raw service logs.
- Provider request or response payloads.
- Credentials, tokens, cookies, private keys, or secret names that reveal sensitive context.
- Private host paths, private network names, local usernames, environment variables, or stack traces.
- Customer-sensitive input payloads copied from tools, integrations, or artifacts.

### Actions

Actions should appear only when authorized and applicable:

- Retry run.
- Cancel run.
- Approve or deny protected operation.
- Open artifact.
- Open policy decision.
- Open health detail.
- Copy safe correlation ID.
- Create follow-up issue.

Protected actions require a visible confirmation step and must not be hidden behind a primary-only affordance.

### Empty, Loading, and Error States

- Empty: show a stable timeline shell with "No run events yet" and the expected first event type when known.
- Loading: keep the timeline structure visible and use skeleton rows, not a full-screen spinner.
- Partial data: render available events and mark missing event families as "not reported".
- Authorization gap: explain that the caller lacks permission for details; do not imply the event is absent.
- Event ingestion delay: show last received time and retry refresh affordance.

## Surface 2: Admin Operations Overview

### User Job

As a Working Group admin, I need a dense operations overview that shows whether current agentic work can run, what needs attention, and which owner boundary should handle each problem.

### Overview Sections

The overview should be a scan-first workspace, not a KPI dashboard.

Recommended sections:

- Active runs: queued, running, waiting approval, retrying, and failing runs.
- Attention required: policy denials, usage-limited runs, failed dispatches, degraded runtimes, approval waits, and repeated failures.
- Runtime health summary: runner, gateway, and provider state counts.
- Usage and cost pressure: current Working Group limits, recent blocks, and projected risk where authorized.
- Recent audit and policy events: only safe summaries with links to details.
- Setup gaps: missing provider route, no compatible runner, disabled integration, missing policy, or failed usage export.

### Minimum Data Fields

- `working_group_id`
- `window_start`
- `window_end`
- `active_run_count`
- `queued_run_count`
- `waiting_approval_count`
- `failed_run_count`
- `policy_denied_count`
- `usage_limited_count`
- `runner_health_counts`
- `gateway_health_counts`
- `provider_health_counts`
- `oldest_queue_age_ms`
- `median_dispatch_latency_ms`
- `p95_run_duration_ms`
- `usage_limit_state`
- `estimated_cost_state`
- `last_audit_event_at`
- `last_usage_event_at`
- `last_runtime_heartbeat_at`

### Filters

Filters should support:

- Time window.
- Status family.
- Agent.
- Runner.
- Gateway.
- Provider.
- Integration.
- Workflow.
- Issue or chat context.
- Policy decision outcome.
- Usage limit state.

Default filter should be the current Working Group and a recent operational window. Tenant-wide views are admin-only and must not leak cross-tenant counts to Working Group admins.

### Recovery and Routing

Each attention row should name one likely owner boundary:

- Frontend: rendering, stale client state, inaccessible action, visual or accessibility bug.
- Control-plane API: missing run event, incorrect authorization, dispatch state mismatch, usage or audit event not surfaced.
- Runner: heartbeat, capability, lease, cancellation, workspace cleanup, local tool, MCP host, or local LLM issue.
- Gateway: provider adapter, hosted MCP, streaming relay, policy hook, usage export, or route cache issue.
- Provider: model availability, provider latency, provider errors, quota, or account-level block.
- QA: test coverage, regression verification, visual evidence, or fixture completeness.

The UI can suggest these boundaries but should avoid claiming root cause unless evidence supports it.

## Surface 3: Runner, Gateway, and Provider Health

### User Job

As an admin or operator, I need to see whether each execution dependency is usable for my Working Group and which runs are affected by degraded components.

### Health List

Use a dense table or grouped list. Avoid nested cards.

Columns:

- Resource name.
- Resource type: runner, gateway, provider.
- Status: online, offline, limited, degraded, disabled, incompatible, unknown.
- Scope: Working Group, tenant, or environment label visible to caller.
- Version or protocol compatibility.
- Last heartbeat or check time.
- Capacity or concurrency.
- Error or denial rate summary.
- Usage or quota pressure.
- Affected active runs.
- Next action.

### Drill-Down

The drill-down should open in the focus zone and include:

- Resource identity and scope.
- Health status history.
- Capabilities or supported models.
- Protocol and contract version.
- Last safe diagnostic summary.
- Recent linked runs.
- Recent policy or usage blocks.
- Safe audit event links.
- Owner boundary and escalation path.

### Minimum Health Fields

Common:

- `resource_id`
- `resource_type`
- `display_name`
- `working_group_id`
- `tenant_id`
- `status`
- `status_reason_code`
- `status_summary`
- `version`
- `protocol_version`
- `last_seen_at`
- `last_check_at`
- `capability_summary`
- `compatibility_state`
- `affected_run_count`
- `safe_diagnostic_detail`
- `redaction_state`

Runner-specific:

- `heartbeat_freshness_ms`
- `available_concurrency`
- `max_concurrency`
- `capability_inventory_updated_at`
- `workspace_cleanup_state`
- `mcp_host_state`
- `local_llm_state`

Gateway-specific:

- `readiness_state`
- `provider_adapter_state`
- `mcp_supervisor_state`
- `streaming_relay_state`
- `policy_hook_state`
- `usage_export_state`

Provider-specific:

- `provider_route_state`
- `enabled_model_count`
- `latency_state`
- `error_rate_state`
- `quota_state`
- `fallback_state`

### Health Status Semantics

- `online`: available and compatible.
- `offline`: no fresh heartbeat or readiness signal.
- `limited`: available with reduced capacity or missing optional capability.
- `degraded`: available but failing or slow enough to affect user work.
- `disabled`: intentionally unavailable by admin, policy, or configuration.
- `incompatible`: version or protocol cannot satisfy current control-plane contract.
- `unknown`: no current evidence yet; treat as warning until resolved.

## Authorization and Policy Boundary

Every operations surface must be scoped by the caller's Working Group, tenant role, and resource permissions. Authorization should apply before aggregation when possible so counts do not reveal hidden resources.

Required behavior:

- Member users see only runs and resources they can already access.
- Working Group admins see Working Group operational state.
- Tenant admins can see tenant-wide aggregation when the product role allows it.
- Support or platform roles need explicit authorization and audit events for tenant access.
- Policy-denied and usage-limited states can show safe reason codes, not hidden policy internals.

## API and Contract Candidates

The final endpoints may differ, but implementation planning should separate these contracts:

- `GET /runs/{run_id}` for run summary and current status.
- `GET /runs/{run_id}/timeline` for paginated timeline events.
- `GET /operations/overview?working_group_id=...` for admin overview aggregates.
- `GET /operations/health?working_group_id=...` for health list.
- `GET /operations/health/{resource_type}/{resource_id}` for health drill-down.
- `POST /runs/{run_id}/retry` for authorized retry.
- `POST /runs/{run_id}/cancel` for authorized cancellation.
- `POST /approval-requests/{approval_request_id}/decision` for approval or denial.

Compatibility expectations:

- Additive fields should be tolerated by clients.
- Unknown status values should render as `unknown` with safe text.
- Event IDs and correlation IDs should remain stable.
- Pagination should preserve event order using `sequence` plus `occurred_at`.
- Timeline event payloads should be display-oriented and already redacted by the server.

## Frontend Boundary

Frontend owns:

- App shell placement, routing, tables, timeline presentation, filters, focus-panel drill-downs, action affordances, empty/loading/error states, keyboard behavior, and visual density.
- Mapping status taxonomy to labels, icons, severity, and copy.
- Safe display of control-plane redacted fields.
- Playwright and visual checks for critical surfaces after implementation.

Frontend must not own:

- Authorization decisions.
- Redaction decisions.
- Runtime health truth.
- Policy or usage evaluation.
- Direct runner, gateway, or provider calls.

## Backend and API Boundary

Control-plane backend/API owns:

- Run and event persistence.
- Authorization and aggregation scoping.
- Redaction before data reaches clients.
- Status normalization.
- Correlation across policy, usage, audit, dispatch, runner, gateway, and provider events.
- Retry, cancel, and approval decision command validation.
- Pagination, ordering, and compatibility behavior.

## Runner, Gateway, and Provider Boundary

Runner and gateway owners provide:

- Heartbeat and readiness evidence.
- Capability and protocol version evidence.
- Safe diagnostic summaries.
- Runtime event production.
- Usage and audit event production.
- Error classification that can be normalized by the control plane.

Provider integrations provide:

- Route availability.
- Model availability.
- Latency and error summaries.
- Quota or limit state where available.
- Fallback route state when configured.

## QA and Verification Boundary

QA should verify requirements at the smallest reliable layer:

- Contract tests for required fields, status taxonomy, additive-field tolerance, unknown status fallback, event ordering, pagination, and redaction fields.
- API tests for Working Group scoping, tenant/admin authorization, retry/cancel/approval command permissions, and aggregation boundaries.
- Unit tests for frontend status mapping, filtering, empty states, error states, and redaction display.
- Playwright tests for issue run timeline, admin operations overview, health list filters, drill-down, retry/cancel/approval affordance visibility, keyboard navigation, and responsive layout.
- Visual verification for dense table scanability, focus-panel layout, badges, timeline ordering, degraded states, and no overlapping text at desktop and narrow viewports.
- Accessibility smoke checks for semantic headings, table/list structure, focus order, visible focus, status text not color-only, button labels, and reduced-motion behavior for live progress.

Use generated or fixture-backed test data only. Do not use live credentials, paid provider calls, production logs, real customer payloads, or private host details.

## Implementation Split

Suggested follow-up slices:

1. Frontend IA and fixture slice: add routes/components for timeline, overview, and health list using redacted fixtures and current adapter boundaries.
2. Control-plane contract slice: define run, timeline, overview, health, and action contracts with authorization and redaction semantics.
3. Runtime evidence slice: runner and gateway emit normalized heartbeat, health, usage, audit, and run event evidence.
4. Action command slice: retry, cancel, and approval decisions with control-plane authorization and audit events.
5. QA slice: contract, API, Playwright, visual, and accessibility smoke coverage using safe fixtures.

## Acceptance Criteria

- Per-run timeline IA covers trigger, policy and usage checks, queue, dispatch, start, progress, wait, retry, cancel, timeout, fail, and completion states.
- Admin operations overview covers active runs, attention required, runtime health summary, usage or cost pressure, policy or audit summaries, and setup gaps.
- Runner, gateway, and provider health surface defines rows, filters, drill-down content, health status semantics, and affected-run linking.
- Minimum fields include event IDs, correlation IDs, request IDs, source events, status, timestamps, resource references, authorization state, and redaction state.
- Policy denial, approval wait, usage limit, and degraded runner/gateway/provider states are included in the taxonomy.
- Authorization and redaction boundaries are explicit and keep private payloads, credentials, raw logs, and customer-sensitive data out of public docs and fixtures.
- Frontend, backend/API, runner/gateway/provider, and QA ownership boundaries are separated for follow-up implementation issues.
- Playwright or visual verification is required for implemented UI surfaces, while this design-only artifact requires markdown review and acceptance criteria traceability.
