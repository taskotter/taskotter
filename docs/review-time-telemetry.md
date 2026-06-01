# Review Time Telemetry Scaffold

TaskOtter prototype review-time telemetry measures `human minutes per completed agent task` for internal evaluation of the review control workflow.

## Events

The scaffold records these coarse workflow events through `ReviewTelemetryEventV1`:

- `review_started`
- `review_stopped`
- `packet_opened`
- `checklist_reviewed`
- `decision_made`
- `rework_requested`
- `done_approved`

The calculation sorts events by `occurred_at`, sums closed review start/stop intervals, counts `done_approved` events as completed agent tasks, and counts `rework_requested` events as rework loops. Open review intervals are reported as `missing_stop_events` and are not converted into minutes.

## Baseline

`ReviewBaselineComparison` is intentionally a fixture/import field. It supports comparing the prototype against a Claude Code + GitHub/Linear manual baseline without adding a real external integration.

## Privacy Boundary

This telemetry is prototype/internal evaluation data. It is not a billing meter, team performance dashboard, employee productivity surveillance feed, or process-mining platform.

Only opaque references and coarse event names belong in review-time telemetry. Do not store raw prompts, diffs, comments, private logs, reviewer names, customer data, access tokens, billing records, or generated artifact bodies in these events or fixtures.
