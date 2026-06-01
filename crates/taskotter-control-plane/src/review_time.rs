use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

// Prototype telemetry boundary: this module records only opaque references and
// coarse review workflow events. It must not store raw prompts, diffs, comments,
// reviewer names, private logs, billing records, or employee performance data.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewTelemetryEventV1 {
    pub id: String,
    pub r#type: String,
    pub version: String,
    pub occurred_at: String,
    pub working_group_id: String,
    pub task_id: String,
    pub reviewer_ref: String,
    pub event: ReviewTelemetryEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTelemetryEventKind {
    ReviewStarted,
    ReviewStopped,
    PacketOpened,
    ChecklistReviewed,
    DecisionMade,
    ReworkRequested,
    DoneApproved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewBaselineComparison {
    pub source: ReviewBaselineSource,
    pub completed_agent_tasks: u64,
    pub human_review_minutes: u64,
    #[serde(default)]
    pub notes_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewBaselineSource {
    ClaudeCodeGithubLinearManual,
    ImportedFixture,
    ManualEstimate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewTelemetryEvaluationRequest {
    pub events: Vec<ReviewTelemetryEventV1>,
    #[serde(default)]
    pub baseline: Option<ReviewBaselineComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReviewTimeMetrics {
    pub completed_agent_tasks: u64,
    pub human_review_minutes: u64,
    #[serde(default)]
    pub human_minutes_per_completed_agent_task: Option<u64>,
    pub rework_loops: u64,
    pub missing_stop_events: u64,
    #[serde(default)]
    pub baseline: Option<ReviewBaselineComparison>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReviewTelemetryError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("type must be telemetry.review_time.recorded")]
    UnsupportedType,
    #[error("version must be 0.1.0")]
    UnsupportedVersion,
    #[error("{field} must use an allowed opaque reference prefix")]
    InvalidReference { field: &'static str },
    #[error("{field} contains a sensitive pattern")]
    SensitivePattern { field: &'static str },
    #[error("occurred_at must be a UTC RFC3339 timestamp")]
    InvalidTimestamp,
    #[error("review completion requires a decision before done approval")]
    MissingDecisionBeforeDone,
}

pub fn calculate_review_time_metrics(
    request: &ReviewTelemetryEvaluationRequest,
) -> Result<ReviewTimeMetrics, ReviewTelemetryError> {
    for event in &request.events {
        event.validate_for_ingestion()?;
    }
    if let Some(baseline) = &request.baseline {
        baseline.validate()?;
    }

    let mut events = request
        .events
        .iter()
        .map(|event| Ok((parse_utc_seconds(&event.occurred_at)?, event)))
        .collect::<Result<Vec<_>, ReviewTelemetryError>>()?;
    events.sort_by_key(|(occurred_at, _)| *occurred_at);

    let mut open_review_started_at = None;
    let mut human_review_seconds = 0u64;
    let mut completed_agent_tasks = 0u64;
    let mut rework_loops = 0u64;
    let mut missing_stop_events = 0u64;
    let mut has_decision_since_start = false;

    for (occurred_at, event) in events {
        match event.event {
            ReviewTelemetryEventKind::ReviewStarted => {
                if open_review_started_at.is_some() {
                    missing_stop_events += 1;
                }
                open_review_started_at = Some(occurred_at);
                has_decision_since_start = false;
            }
            ReviewTelemetryEventKind::ReviewStopped => {
                if let Some(started_at) = open_review_started_at.take() {
                    human_review_seconds += occurred_at.saturating_sub(started_at);
                }
            }
            ReviewTelemetryEventKind::DecisionMade => {
                has_decision_since_start = true;
            }
            ReviewTelemetryEventKind::ReworkRequested => {
                rework_loops += 1;
                has_decision_since_start = true;
            }
            ReviewTelemetryEventKind::DoneApproved => {
                if !has_decision_since_start {
                    return Err(ReviewTelemetryError::MissingDecisionBeforeDone);
                }
                completed_agent_tasks += 1;
            }
            ReviewTelemetryEventKind::PacketOpened
            | ReviewTelemetryEventKind::ChecklistReviewed => {}
        }
    }

    if open_review_started_at.is_some() {
        missing_stop_events += 1;
    }

    let human_review_minutes = human_review_seconds.div_ceil(60);
    Ok(ReviewTimeMetrics {
        completed_agent_tasks,
        human_review_minutes,
        human_minutes_per_completed_agent_task: (completed_agent_tasks > 0)
            .then(|| human_review_minutes / completed_agent_tasks),
        rework_loops,
        missing_stop_events,
        baseline: request.baseline.clone(),
    })
}

impl ReviewTelemetryEventV1 {
    pub fn validate_for_ingestion(&self) -> Result<(), ReviewTelemetryError> {
        require_opaque_ref("id", &self.id, &["evt_"])?;
        if self.r#type != "telemetry.review_time.recorded" {
            return Err(ReviewTelemetryError::UnsupportedType);
        }
        if self.version != "0.1.0" {
            return Err(ReviewTelemetryError::UnsupportedVersion);
        }
        parse_utc_seconds(&self.occurred_at)?;
        require_opaque_ref("working_group_id", &self.working_group_id, &["wg_"])?;
        require_opaque_ref("task_id", &self.task_id, &["task_", "issue_", "run_"])?;
        require_opaque_ref("reviewer_ref", &self.reviewer_ref, &["usr_", "svc_"])?;
        Ok(())
    }
}

impl ReviewBaselineComparison {
    fn validate(&self) -> Result<(), ReviewTelemetryError> {
        if let Some(notes_ref) = &self.notes_ref {
            require_opaque_ref("baseline.notes_ref", notes_ref, &["doc_", "fixture_"])?;
        }
        Ok(())
    }
}

fn require_text(field: &'static str, value: &str) -> Result<(), ReviewTelemetryError> {
    if value.trim().is_empty() {
        return Err(ReviewTelemetryError::Required(field));
    }
    reject_sensitive_pattern(field, value)?;
    Ok(())
}

fn require_opaque_ref(
    field: &'static str,
    value: &str,
    prefixes: &[&str],
) -> Result<(), ReviewTelemetryError> {
    require_text(field, value)?;
    if value.len() > 80 || !prefixes.iter().any(|prefix| value.starts_with(prefix)) {
        return Err(ReviewTelemetryError::InvalidReference { field });
    }
    Ok(())
}

fn reject_sensitive_pattern(field: &'static str, value: &str) -> Result<(), ReviewTelemetryError> {
    let normalized = value.to_ascii_lowercase();
    let sensitive_patterns = [
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "private_key",
        "client_secret",
        "bearer ",
        "password",
        "raw_prompt",
        "raw_log",
        "artifact_body",
        "diff_body",
        "employee",
        "productivity",
        "-----begin",
    ];

    if sensitive_patterns
        .iter()
        .any(|pattern| normalized.contains(pattern))
    {
        return Err(ReviewTelemetryError::SensitivePattern { field });
    }
    Ok(())
}

fn parse_utc_seconds(value: &str) -> Result<u64, ReviewTelemetryError> {
    require_text("occurred_at", value)?;
    let timestamp = value
        .strip_suffix('Z')
        .ok_or(ReviewTelemetryError::InvalidTimestamp)?;
    let (date, time) = timestamp
        .split_once('T')
        .ok_or(ReviewTelemetryError::InvalidTimestamp)?;
    let mut date_parts = date.split('-').map(parse_u64);
    let year = date_parts
        .next()
        .ok_or(ReviewTelemetryError::InvalidTimestamp)??;
    let month = date_parts
        .next()
        .ok_or(ReviewTelemetryError::InvalidTimestamp)??;
    let day = date_parts
        .next()
        .ok_or(ReviewTelemetryError::InvalidTimestamp)??;
    if date_parts.next().is_some() {
        return Err(ReviewTelemetryError::InvalidTimestamp);
    }

    let time = time.split_once('.').map_or(time, |(seconds, _)| seconds);
    let mut time_parts = time.split(':').map(parse_u64);
    let hour = time_parts
        .next()
        .ok_or(ReviewTelemetryError::InvalidTimestamp)??;
    let minute = time_parts
        .next()
        .ok_or(ReviewTelemetryError::InvalidTimestamp)??;
    let second = time_parts
        .next()
        .ok_or(ReviewTelemetryError::InvalidTimestamp)??;
    if time_parts.next().is_some()
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(ReviewTelemetryError::InvalidTimestamp);
    }

    Ok(days_before_year(year) * 86_400
        + days_before_month(year, month) * 86_400
        + (day - 1) * 86_400
        + hour * 3_600
        + minute * 60
        + second)
}

fn parse_u64(value: &str) -> Result<u64, ReviewTelemetryError> {
    value
        .parse::<u64>()
        .map_err(|_| ReviewTelemetryError::InvalidTimestamp)
}

fn days_before_year(year: u64) -> u64 {
    let prior_year = year - 1;
    prior_year * 365 + prior_year / 4 - prior_year / 100 + prior_year / 400
}

fn days_before_month(year: u64, month: u64) -> u64 {
    let month_days = [31, february_days(year), 31, 30, 31, 30, 31, 31, 30, 31, 30];
    month_days.iter().take((month - 1) as usize).copied().sum()
}

fn february_days(year: u64) -> u64 {
    if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) {
        29
    } else {
        28
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: ReviewTelemetryEventKind, occurred_at: &str) -> ReviewTelemetryEventV1 {
        ReviewTelemetryEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            r#type: "telemetry.review_time.recorded".to_owned(),
            version: "0.1.0".to_owned(),
            occurred_at: occurred_at.to_owned(),
            working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            task_id: "task_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            reviewer_ref: "usr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            event: kind,
        }
    }

    fn calculate(
        events: Vec<ReviewTelemetryEventV1>,
    ) -> Result<ReviewTimeMetrics, ReviewTelemetryError> {
        calculate_review_time_metrics(&ReviewTelemetryEvaluationRequest {
            events,
            baseline: None,
        })
    }

    #[test]
    fn event_ordering_sorts_before_calculating_review_minutes() -> Result<(), ReviewTelemetryError>
    {
        let metrics = calculate(vec![
            event(
                ReviewTelemetryEventKind::DoneApproved,
                "2026-06-01T00:08:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ReviewStopped,
                "2026-06-01T00:05:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ReviewStarted,
                "2026-06-01T00:00:00Z",
            ),
            event(
                ReviewTelemetryEventKind::DecisionMade,
                "2026-06-01T00:07:00Z",
            ),
        ])?;

        assert_eq!(metrics.human_review_minutes, 5);
        assert_eq!(metrics.completed_agent_tasks, 1);
        assert_eq!(metrics.human_minutes_per_completed_agent_task, Some(5));
        Ok(())
    }

    #[test]
    fn missing_stop_event_is_reported_and_not_counted_as_minutes()
    -> Result<(), ReviewTelemetryError> {
        let metrics = calculate(vec![
            event(
                ReviewTelemetryEventKind::ReviewStarted,
                "2026-06-01T00:00:00Z",
            ),
            event(
                ReviewTelemetryEventKind::DecisionMade,
                "2026-06-01T00:03:00Z",
            ),
            event(
                ReviewTelemetryEventKind::DoneApproved,
                "2026-06-01T00:04:00Z",
            ),
        ])?;

        assert_eq!(metrics.human_review_minutes, 0);
        assert_eq!(metrics.missing_stop_events, 1);
        assert_eq!(metrics.completed_agent_tasks, 1);
        Ok(())
    }

    #[test]
    fn rework_loop_accumulates_review_cycles_for_one_completed_task()
    -> Result<(), ReviewTelemetryError> {
        let metrics = calculate(vec![
            event(
                ReviewTelemetryEventKind::ReviewStarted,
                "2026-06-01T00:00:00Z",
            ),
            event(
                ReviewTelemetryEventKind::PacketOpened,
                "2026-06-01T00:01:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ChecklistReviewed,
                "2026-06-01T00:02:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ReworkRequested,
                "2026-06-01T00:03:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ReviewStopped,
                "2026-06-01T00:05:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ReviewStarted,
                "2026-06-01T00:20:00Z",
            ),
            event(
                ReviewTelemetryEventKind::DecisionMade,
                "2026-06-01T00:24:00Z",
            ),
            event(
                ReviewTelemetryEventKind::DoneApproved,
                "2026-06-01T00:25:00Z",
            ),
            event(
                ReviewTelemetryEventKind::ReviewStopped,
                "2026-06-01T00:26:00Z",
            ),
        ])?;

        assert_eq!(metrics.rework_loops, 1);
        assert_eq!(metrics.human_review_minutes, 11);
        assert_eq!(metrics.completed_agent_tasks, 1);
        assert_eq!(metrics.human_minutes_per_completed_agent_task, Some(11));
        Ok(())
    }

    #[test]
    fn completed_task_calculation_includes_baseline_fixture_without_integration()
    -> Result<(), Box<dyn std::error::Error>> {
        let request: ReviewTelemetryEvaluationRequest = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/review-time-telemetry.prototype.json"
        ))?;

        let metrics = calculate_review_time_metrics(&request)?;

        assert_eq!(metrics.completed_agent_tasks, 1);
        assert_eq!(metrics.human_review_minutes, 11);
        assert_eq!(metrics.human_minutes_per_completed_agent_task, Some(11));
        assert_eq!(metrics.rework_loops, 1);
        assert_eq!(
            metrics
                .baseline
                .map(|baseline| baseline.human_review_minutes),
            Some(38)
        );
        Ok(())
    }

    #[test]
    fn done_approval_requires_a_prior_decision_event() {
        let result = calculate_review_time_metrics(&ReviewTelemetryEvaluationRequest {
            events: vec![
                event(
                    ReviewTelemetryEventKind::ReviewStarted,
                    "2026-06-01T00:00:00Z",
                ),
                event(
                    ReviewTelemetryEventKind::DoneApproved,
                    "2026-06-01T00:02:00Z",
                ),
            ],
            baseline: None,
        });

        assert_eq!(result, Err(ReviewTelemetryError::MissingDecisionBeforeDone));
    }

    #[test]
    fn privacy_boundary_rejects_sensitive_references() {
        let mut unsafe_event = event(
            ReviewTelemetryEventKind::ReviewStarted,
            "2026-06-01T00:00:00Z",
        );
        unsafe_event.task_id = "task_raw_prompt_body".to_owned();

        assert_eq!(
            unsafe_event.validate_for_ingestion(),
            Err(ReviewTelemetryError::SensitivePattern { field: "task_id" })
        );
    }
}
