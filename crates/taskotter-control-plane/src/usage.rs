use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::WorkingGroupId;
use crate::policy::{PolicySubject, ProviderRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageSnapshot {
    pub working_group_id: WorkingGroupId,
    pub monthly_cost_cents: u64,
    pub daily_tokens: u64,
    pub hourly_actions: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageLimit {
    pub name: String,
    pub max_monthly_cost_cents: Option<u64>,
    pub max_daily_tokens: Option<u64>,
    pub max_hourly_actions: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsagePolicySet {
    pub limits: Vec<UsageLimit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageEvaluationRequest {
    pub snapshot: UsageSnapshot,
    pub policy_set: UsagePolicySet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageEvaluation {
    pub allowed: bool,
    pub failed_limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageAttemptStatus {
    Succeeded,
    Denied,
    Timeout,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageAuditEventV1 {
    pub schema_version: String,
    pub event_id: Uuid,
    pub request_id: Uuid,
    #[serde(default)]
    pub correlation_id: Option<String>,
    pub subject: PolicySubject,
    pub provider: ProviderRef,
    pub decision_id: String,
    pub status: UsageAttemptStatus,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub estimated_cost_micro_usd: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RemoteUsageReportV1 {
    pub schema_version: String,
    pub job_id: String,
    pub runner_id: String,
    #[serde(default)]
    pub request_id: Option<Uuid>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    pub status: UsageAttemptStatus,
    pub duration_ms: u64,
    #[serde(default)]
    pub cpu_time_ms: Option<u64>,
    #[serde(default)]
    pub peak_memory_bytes: Option<u64>,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub estimated_cost_micro_usd: u64,
}

impl UsagePolicySet {
    pub fn evaluate(&self, snapshot: &UsageSnapshot) -> UsageEvaluation {
        let failed_limits = self
            .limits
            .iter()
            .filter(|limit| !limit.allows(snapshot))
            .map(|limit| limit.name.clone())
            .collect::<Vec<_>>();

        UsageEvaluation {
            allowed: failed_limits.is_empty(),
            failed_limits,
        }
    }
}

impl UsageLimit {
    fn allows(&self, snapshot: &UsageSnapshot) -> bool {
        self.max_monthly_cost_cents
            .is_none_or(|max| snapshot.monthly_cost_cents <= max)
            && self
                .max_daily_tokens
                .is_none_or(|max| snapshot.daily_tokens <= max)
            && self
                .max_hourly_actions
                .is_none_or(|max| snapshot.hourly_actions <= max)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn snapshot() -> UsageSnapshot {
        UsageSnapshot {
            working_group_id: WorkingGroupId(Uuid::new_v4()),
            monthly_cost_cents: 2_500,
            daily_tokens: 100_000,
            hourly_actions: 42,
        }
    }

    #[test]
    fn usage_limits_compose_with_and_semantics() {
        let policy_set = UsagePolicySet {
            limits: vec![
                UsageLimit {
                    name: "monthly-cost".to_owned(),
                    max_monthly_cost_cents: Some(3_000),
                    max_daily_tokens: None,
                    max_hourly_actions: None,
                },
                UsageLimit {
                    name: "daily-tokens".to_owned(),
                    max_monthly_cost_cents: None,
                    max_daily_tokens: Some(10_000),
                    max_hourly_actions: None,
                },
            ],
        };

        let evaluation = policy_set.evaluate(&snapshot());

        assert!(!evaluation.allowed);
        assert_eq!(evaluation.failed_limits, vec!["daily-tokens"]);
    }

    #[test]
    fn usage_policy_allows_only_when_all_limits_pass() {
        let policy_set = UsagePolicySet {
            limits: vec![
                UsageLimit {
                    name: "monthly-cost".to_owned(),
                    max_monthly_cost_cents: Some(3_000),
                    max_daily_tokens: None,
                    max_hourly_actions: None,
                },
                UsageLimit {
                    name: "daily-tokens".to_owned(),
                    max_monthly_cost_cents: None,
                    max_daily_tokens: Some(150_000),
                    max_hourly_actions: None,
                },
            ],
        };

        let evaluation = policy_set.evaluate(&snapshot());

        assert!(evaluation.allowed);
        assert!(evaluation.failed_limits.is_empty());
    }
}
