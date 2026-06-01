# Review Control Prototype QA Scenario Pack

Status: draft QA pack for BOG-568 prototype wave.

QA mode: standard. This wave changes user-visible workflow, API/domain
contracts, evidence handling, review decisions, audit reconstruction, and
telemetry. Strict release certification is out of scope until the parent final
PR exists.

## Source Scope

- Parent wave: BOG-568, review control prototype.
- QA child: BOG-577.
- Target repo: `taskotter/taskotter`.
- Intended integration branch: `integration/BOG-TBD-review-control-prototype`.
- Current blocker: the intended integration branch was not resolvable at QA
  drafting time, and several implementation children were still not complete.

## Acceptance Trace

| Parent workflow capability                                  | Owning child signal           | QA evidence required                                                                                                                   |
| ----------------------------------------------------------- | ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| New work item with acceptance criteria and risk tier        | BOG-571, BOG-575              | Contract fixture/API test plus UI smoke showing title, acceptance checklist, risk tier, owner, and current state.                      |
| Plan approval and rework request                            | BOG-570/569, BOG-571, BOG-575 | Contract/UI test for approve and request-rework decisions, including required rationale and state transition.                          |
| Agent result and evidence import                            | BOG-572                       | Contract fixture/API test for imported agent result, changed files, test output, artifact refs, and redaction state.                   |
| Review packet with missing evidence and failed test signals | BOG-573, BOG-575              | Packet generation unit/contract test and UI smoke for pass, missing evidence, failed test, uncertainty, and rollback guidance.         |
| Done vs rework decision                                     | BOG-571, BOG-575              | E2E state transition checks from review packet to `done` and from review packet to rework.                                             |
| Audit/event timeline reconstruction                         | BOG-576                       | Contract or integration test proving timeline order, actor/source, correlation IDs, decisions, and evidence refs.                      |
| Human review time metric calculation                        | BOG-574                       | Unit/contract test for review start, pause/continue if supported, decision timestamp, baseline comparison, and timezone-safe duration. |

## Scenario Matrix

| ID              | Scenario                                                                                                                                                   | Level                                            | Required data                                                                                                                                                          | Expected result                                                                                                                                                                                | Current status                                                                                                       |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| RC-SMOKE-001    | App shell still exposes issue workspace and focus panel after prototype changes.                                                                           | Playwright smoke                                 | Existing fixture data plus review prototype fixture once available.                                                                                                    | Navigation, issue list, selected issue, run/evidence focus panel, and composer render with accessible names.                                                                                   | Executable now through `npm run test:ui`; extend once BOG-575 lands.                                                 |
| RC-E2E-001      | Happy path: request -> plan approval -> agent result import -> review packet -> done -> audit timeline.                                                    | Playwright e2e plus contract fixtures            | One work item with acceptance criteria, medium/high risk tier, approved plan, imported result evidence, passing tests, review packet, done decision, and audit events. | Reviewer can inspect packet evidence, choose Done, see final status, audit timeline reconstructs every decision, and review-time metric is recorded.                                           | Blocked until BOG-571, BOG-572, BOG-573, BOG-574, BOG-575, and BOG-576 produce integrated fixtures/UI.               |
| RC-E2E-002      | Rework path: failed test or missing evidence -> request rework -> packet/timeline update.                                                                  | Playwright e2e plus contract fixtures            | One work item with failed test signal or missing required evidence, reviewer rationale, rework decision, and regenerated packet/timeline.                              | Reviewer cannot accidentally mark Done without acknowledging failed/missing evidence; Rework records rationale, routes the item back, and timeline shows decision and imported failure signal. | Blocked until same implementation children produce integrated fixtures/UI.                                           |
| RC-CONTRACT-001 | Work item and review decision contracts cover acceptance criteria, risk tier, plan state, review state, and decision actor.                                | Contract/unit                                    | Canonical JSON/OpenAPI fixtures or generated client payloads.                                                                                                          | Schema validation rejects missing acceptance/risk/decision fields and accepts representative happy/rework payloads.                                                                            | Blocked until BOG-571 contract artifacts exist.                                                                      |
| RC-CONTRACT-002 | Agent result import stores safe evidence refs instead of raw logs or secrets.                                                                              | Contract/unit                                    | Agent result fixture with changed files, command result summaries, artifact IDs, redacted details, and a negative fixture with raw sensitive data.                     | Positive fixture validates; negative fixture is rejected or redacted according to the contract.                                                                                                | Blocked until BOG-572 artifacts exist.                                                                               |
| RC-CONTRACT-003 | Review packet generation summarizes changed files, acceptance checklist, failed/missing evidence, risk signals, uncertainty, and rollback/rework guidance. | Unit/contract                                    | Packet input fixture with pass/fail/missing evidence permutations.                                                                                                     | Packet output is deterministic and marks missing evidence and failed tests as review blockers.                                                                                                 | Blocked until BOG-573 artifacts exist.                                                                               |
| RC-CONTRACT-004 | Audit/event timeline reconstructs plan approval, import, packet generation, review decision, and rework/done outcome in order.                             | Unit/contract                                    | Timeline event fixtures with correlation IDs and actor/source fields.                                                                                                  | Events are ordered by sequence/time, correlate to the same work item/run, and hide raw sensitive payloads.                                                                                     | Partially covered by existing operations timeline contracts; review-control-specific fixtures blocked until BOG-576. |
| RC-CONTRACT-005 | Human review time metric handles baseline comparison and timezone-safe duration.                                                                           | Unit/contract                                    | Review started/decision timestamps, reviewer id, baseline duration, and optional paused intervals if supported.                                                        | Duration and baseline delta are deterministic across timezone formatting, and missing timestamps fail validation.                                                                              | Blocked until BOG-574 artifacts exist.                                                                               |
| RC-VISUAL-001   | Review packet UI remains scannable at desktop and narrow viewport.                                                                                         | Screenshot/manual visual + Playwright assertions | Happy packet, failed test packet, missing evidence packet, and rework decision state.                                                                                  | No overlapping text, evidence badges remain visible, decision controls fit, status is not color-only, and focus order reaches decision controls.                                               | Blocked until BOG-575 UI exists.                                                                                     |

## Executable Baseline Checks

Run these on every implementation PR targeting the prototype integration branch:

```sh
npm run contracts:check
npm run check:module-boundaries
npm run check:i18n
npm run test:fixtures
npm run test:compatibility
npm run test:unit
npm run build
npm run test:ui
```

For child PR triage, use the smallest relevant subset first:

| Child area                      | Minimum commands before QA handoff                                                                    |
| ------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Domain/API contracts            | `npm run contracts:check`, `npm run test:fixtures`, `npm run test:compatibility`, `npm run test:unit` |
| Frontend flow                   | `npm run check:i18n`, `npm run test:unit`, `npm run build`, `npm run test:ui`                         |
| Evidence import/packet/timeline | `npm run test:fixtures`, `npm run test:compatibility`, `npm run test:unit`                            |
| Review time telemetry           | `npm run test:unit`, plus any new telemetry fixture/contract command added by BOG-574                 |

## Screenshot And Visual Verification Expectations

Capture or inspect the following states once BOG-575 implements the prototype UI:

- New work item detail with acceptance checklist and risk tier visible.
- Plan approval pending, approved, and rework-requested states.
- Imported agent result with changed files, artifacts, test evidence, and redaction state.
- Review packet happy path with all acceptance checks passing.
- Review packet warning path with missing evidence.
- Review packet failure path with failed test signals.
- Done decision confirmation and final done state.
- Rework decision form with required rationale and returned-to-work state.
- Audit timeline reconstruction after done and after rework.
- Human review time metric and baseline comparison.

Each screenshot pass should confirm:

- Text and controls do not overlap at desktop and narrow viewport.
- Status labels are visible text, not color-only.
- Decision controls have accessible names and keyboard focus.
- Missing evidence and failed tests remain visible before Done/Rework action.
- Raw logs, secrets, private paths, prompts, provider payloads, and credentials are not rendered.

## Defect Routing Rules

- Contract/schema failures route to the child that owns the schema or generated client.
- Missing imported result/evidence behavior routes to BOG-572.
- Packet content, checklist, failed-test, missing-evidence, uncertainty, or rework guidance defects route to BOG-573.
- UI rendering, accessibility, keyboard, responsive, and screenshot defects route to BOG-575.
- Timeline ordering, correlation, actor/source, or redaction defects route to BOG-576.
- Review-time duration, baseline, or timestamp defects route to BOG-574.
- Cross-child integration failures on the parent branch should be held on BOG-568 until the conflicting child owners resolve the integration surface.

## Current QA Result

At drafting time, the full happy path and rework path are not executable because
the review-control-specific implementation PRs and integration branch are not
available. Existing repo checks can still verify the current baseline app shell,
contract fixture validation, module boundaries, i18n guard, unit tests, build,
and Playwright app-shell smoke.
