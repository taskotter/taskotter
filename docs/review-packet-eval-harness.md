# Review Packet Eval Harness

Status: prototype QA/eval harness for BOG-617.

QA mode: strict. Review packet changes can affect done/rework decisions,
evidence safety, protected-operation handling, and the product's review-time
success metric.

## Purpose

The harness checks whether a review packet helps a reviewer decide done or
rework without rereading the full raw transcript by default. It scores the
packet against source-backed evidence, risk visibility, uncertainty, rollback
or rework usefulness, decision confidence, and review-time reduction.

Raw transcripts, full logs, private roadmap text, credentials, customer data,
and copied artifact bodies are out of scope for fixtures. The eval uses only
generated fake review-control data and safe evidence references.

## Deterministic Procedure

Run the fixture eval in CI or as a local deterministic QA step:

```sh
npm run test:unit -- src/data/reviewPacketEval.test.ts
```

For a broader prototype regression pass, run:

```sh
npm run contracts:check
npm run test:fixtures
npm run test:compatibility
npm run test:unit
```

The unit harness loads `reviewPacketEvalCases`, evaluates each case with
`evaluateReviewPacketCase`, and asserts expected findings and outcomes. No
network, live provider, private runner, paid service, credential, or production
data is required.

## Rubric

Each dimension is scored from 0 to 1.

| Dimension                       | Pass signal                                                                                                | Fail or partial signal                                                      |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Acceptance criteria correctness | Criteria are measurable, present, and covered or explicitly marked missing.                                | Ambiguous criteria, empty checklist, or missing criteria evidence.          |
| Evidence traceability           | Covered checklist items link to verification evidence IDs.                                                 | Covered claims have no matching source evidence.                            |
| Risk visibility                 | Failed checks, blocked checks, high-risk artifacts, and protected operations are surfaced as risk signals. | Hidden contract/security/provider/public API risk or missing approval gate. |
| Uncertainty honesty             | Missing, failed, blocked, or risky packets include explicit uncertainty.                                   | Packet implies confidence while unresolved evidence gaps remain.            |
| Rollback usefulness             | Guidance names a concrete next action such as rerun, attach, cancel, revert, remove, or fix.               | Generic or prose-only guidance that does not help rework or rollback.       |
| Decision confidence             | No failed, blocked, missing-test, or danger signal blocks done approval.                                   | Done/rework decision is under-supported or unsafe.                          |
| Review minutes improvement      | Observed review minutes are at least 30% lower than baseline minutes.                                      | Improvement is below 30% or baseline is missing.                            |

Thresholds:

- `pass`: average score >= 0.8 and no low decision-confidence finding.
- `rework`: average score below pass threshold, missing evidence, missing tests,
  failed checks, ambiguous criteria, weak rollback, or review-time target miss.
- `strict_block`: strict gate plus hidden risk or unsafe protected-operation
  approval gap. This blocks final parent/base PR readiness until fixed.

## Eval Cases

| Case                               | Scenario                                                                                         | Expected findings                                                                  | Expected outcome |
| ---------------------------------- | ------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- | ---------------- |
| `eval_pass_done_ready`             | Pass case with covered criteria, linked tests, useful rollback, and 60% review-time improvement. | none                                                                               | `pass`           |
| `eval_partial_pass_missing_visual` | Partial pass with useful packet content but missing visual proof.                                | `acceptance_evidence_missing`                                                      | `rework`         |
| `eval_hidden_risk_contract`        | Contract artifact is changed but the packet omits a risk signal.                                 | `hidden_risk_not_flagged`                                                          | `strict_block`   |
| `eval_missing_test_evidence`       | Packet contains review evidence but no test evidence.                                            | `acceptance_evidence_missing`, `missing_test_evidence`, `decision_confidence_low`  | `rework`         |
| `eval_unsafe_operation`            | Protected provider action lacks an explicit high-risk or approval signal.                        | `unsafe_operation_approval_missing`, `verification_blocked`, `uncertainty_missing` | `strict_block`   |
| `eval_ambiguous_acceptance`        | Acceptance criterion is non-measurable and review-time improvement is below target.              | `ambiguous_acceptance_criteria`, `review_minutes_not_improved`                     | `rework`         |

These cases cover pass, partial pass, hidden risk, missing test evidence,
unsafe operation, and ambiguous acceptance criteria.

## QA Recording Template

Use this format when recording a review packet eval result:

```text
Review packet eval:
- Packet/source: <issue key, fixture id, PR, or artifact ref>
- Baseline minutes: <manual review minutes before TaskOtter>
- Observed minutes: <review packet decision minutes>
- Improvement: <percent>
- Eval command: npm run test:unit -- src/data/reviewPacketEval.test.ts
- Outcome: pass | rework | strict_block
- Findings: <finding codes>
- Evidence refs checked: <verification IDs, artifact refs, screenshots, or commands>
- Raw transcript reread: yes | no | partial, with reason
- Limitations: <missing dependency, missing fixture, or residual QA risk>
```

Do not paste raw logs, raw transcripts, secrets, private roadmap excerpts,
credentials, customer records, provider payloads, or copied artifact bodies into
the recording. Link or name safe source references instead.

## Baseline Review Minutes

Compare against `human minutes per completed agent task`:

```text
improvement_percent =
  ((baseline_human_review_minutes - observed_human_review_minutes)
    / baseline_human_review_minutes) * 100
```

Baseline minutes should come from the same task family and reviewer role when
possible. The prototype target is at least 30% reduction. If baseline minutes
are unavailable, the eval can still run for correctness and risk visibility,
but the result must be recorded as residual evidence risk and cannot be used as
final success-metric evidence.

## Release Gate Recommendation

Require strict QA plus product/security review when a review packet change:

- Changes done/rework decision logic, packet scoring, or pass thresholds.
- Adds or removes acceptance, evidence, risk, uncertainty, rollback, or review
  minutes fields.
- Touches auth, authorization, secrets, privacy, production data, migrations,
  public API contracts, provider execution, protected operations, runner
  dispatch, or paid usage behavior.
- Changes redaction behavior or allows raw transcript/log/diff content into the
  default review path.
- Targets a final parent/base PR or protected branch readiness gate.

Ordinary copy, styling, or generated-fake fixture updates can use standard QA if
they do not affect the decision rubric or evidence safety boundary.
