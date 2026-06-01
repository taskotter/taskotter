use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::WorkingGroupId;

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
    pub enforcement: QuotaEnforcement,
    #[serde(default)]
    pub denial_reason: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageAttemptStatus {
    #[default]
    Succeeded,
    Denied,
    Timeout,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuotaEnforcement {
    Allow,
    HardDeny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageSourceSurface {
    ControlPlane,
    Gateway,
    Runner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeteringUnit {
    Request,
    ToolInvocation,
    RuntimeMs,
    LocalComputeMs,
    HostedMcpRuntimeMs,
    Token,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CostReconciliationStatus {
    Estimated,
    Actual,
    PendingProviderReconciliation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReservationStatus {
    Open,
    Consumed,
    Released,
    Expired,
    Overrun,
    Settled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageSecuritySignalCode {
    IdempotencyPayloadMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageSecuritySignal {
    pub signal_id: Uuid,
    pub idempotency_key: String,
    pub existing_event_id: String,
    pub rejected_event_id: String,
    pub reason_code: UsageSecuritySignalCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageActorRef {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageResourceRef {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageSubjectRef {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageEventPayload {
    pub subject: UsageSubjectRef,
    pub measurements: UsageMeasurements,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageMeasurements {
    pub duration_ms: u64,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub tool_invocations: Option<u64>,
    #[serde(default)]
    pub estimated_cost_micros: Option<u64>,
    #[serde(default)]
    pub actual_cost_micros: Option<u64>,
    #[serde(default)]
    pub metering_unit: Option<MeteringUnit>,
    #[serde(default)]
    pub runtime_capability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageAuditEventV1 {
    pub id: String,
    pub r#type: String,
    pub version: String,
    pub occurred_at: String,
    pub source: UsageSourceSurface,
    pub working_group_id: String,
    pub actor: UsageActorRef,
    pub resource: UsageResourceRef,
    pub correlation_id: String,
    pub request_id: String,
    pub policy_decision_id: String,
    pub idempotency_key: String,
    pub payload: UsageEventPayload,
    #[serde(default)]
    pub reservation_id: Option<String>,
    #[serde(default)]
    pub status: UsageAttemptStatus,
    #[serde(default)]
    pub denial_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageLedgerEntry {
    pub ledger_entry_id: Uuid,
    pub event_id: String,
    pub idempotency_key: String,
    pub working_group_id: String,
    pub source: UsageSourceSurface,
    pub actor: UsageActorRef,
    pub resource: UsageResourceRef,
    pub subject: UsageSubjectRef,
    pub policy_decision_id: String,
    pub request_id: String,
    pub correlation_id: String,
    pub metered_quantity: u64,
    pub metering_unit: MeteringUnit,
    pub estimated_cost_micros: u64,
    #[serde(default)]
    pub actual_cost_micros: Option<u64>,
    pub cost_reconciliation_status: CostReconciliationStatus,
    #[serde(default)]
    pub reservation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageReservation {
    pub reservation_id: String,
    pub working_group_id: String,
    pub policy_decision_id: String,
    pub idempotency_key: String,
    pub reserved_quantity: u64,
    pub metering_unit: MeteringUnit,
    pub estimated_cost_micros: u64,
    pub consumed_quantity: u64,
    pub status: ReservationStatus,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UsageValidationError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("version must be 0.1.0")]
    UnsupportedVersion,
    #[error("type must be usage.gateway_request.recorded")]
    UnsupportedType,
    #[error("usage event denied before dispatch requires denial_reason")]
    MissingDenialReason,
    #[error("{field} must use an allowed opaque reference prefix and length")]
    InvalidReference { field: &'static str },
    #[error("{field} contains a sensitive pattern")]
    SensitivePattern { field: &'static str },
    #[error("denial_reason must be an allowlisted safe reason code")]
    UnsafeDenialReason,
    #[error("runtime_capability must be an allowlisted capability reference")]
    InvalidRuntimeCapability,
}

impl UsagePolicySet {
    pub fn evaluate(&self, snapshot: &UsageSnapshot) -> UsageEvaluation {
        let failed_limits = self
            .limits
            .iter()
            .filter(|limit| !limit.allows(snapshot))
            .map(|limit| limit.name.clone())
            .collect::<Vec<_>>();

        let allowed = failed_limits.is_empty();
        UsageEvaluation {
            allowed,
            enforcement: if allowed {
                QuotaEnforcement::Allow
            } else {
                QuotaEnforcement::HardDeny
            },
            denial_reason: (!allowed).then(|| "usage_limit_reached".to_owned()),
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

impl UsageAuditEventV1 {
    pub fn validate_for_ingestion(&self) -> Result<(), UsageValidationError> {
        require_opaque_ref("id", &self.id, &["evt_"])?;
        require_text("occurred_at", &self.occurred_at)?;
        require_opaque_ref("working_group_id", &self.working_group_id, &["wg_"])?;
        require_opaque_ref("correlation_id", &self.correlation_id, &["corr_"])?;
        require_opaque_ref("request_id", &self.request_id, &["req_"])?;
        require_opaque_ref("policy_decision_id", &self.policy_decision_id, &["poldec_"])?;
        require_opaque_ref("idempotency_key", &self.idempotency_key, &["usage_"])?;
        require_allowed_value(
            "actor.type",
            &self.actor.r#type,
            &["user", "agent", "service"],
        )?;
        require_opaque_ref(
            "actor.id",
            &self.actor.id,
            &actor_prefixes(&self.actor.r#type),
        )?;
        require_allowed_value(
            "resource.type",
            &self.resource.r#type,
            &["provider", "mcp_server", "runner", "job", "integration"],
        )?;
        require_opaque_ref(
            "resource.id",
            &self.resource.id,
            &resource_prefixes(&self.resource.r#type),
        )?;
        require_allowed_value(
            "payload.subject.type",
            &self.payload.subject.r#type,
            &["agent_run", "gateway_request", "workflow_run", "runner_job"],
        )?;
        require_opaque_ref(
            "payload.subject.id",
            &self.payload.subject.id,
            &subject_prefixes(&self.payload.subject.r#type),
        )?;

        if self.version != "0.1.0" {
            return Err(UsageValidationError::UnsupportedVersion);
        }
        if self.r#type != "usage.gateway_request.recorded" {
            return Err(UsageValidationError::UnsupportedType);
        }
        if self.status == UsageAttemptStatus::Denied
            && self
                .denial_reason
                .as_ref()
                .is_none_or(|reason| reason.trim().is_empty())
        {
            return Err(UsageValidationError::MissingDenialReason);
        }
        if let Some(reason) = &self.denial_reason {
            require_safe_denial_reason(reason)?;
        }
        if let Some(reservation_id) = &self.reservation_id {
            require_opaque_ref("reservation_id", reservation_id, &["resv_"])?;
        }
        if let Some(runtime_capability) = &self.payload.measurements.runtime_capability {
            require_runtime_capability(runtime_capability)?;
        }

        Ok(())
    }

    pub fn has_same_billing_fingerprint(&self, other: &Self) -> bool {
        self.working_group_id == other.working_group_id
            && self.source == other.source
            && self.actor == other.actor
            && self.resource == other.resource
            && self.policy_decision_id == other.policy_decision_id
            && self.payload == other.payload
            && self.reservation_id == other.reservation_id
            && self.status == other.status
            && self.denial_reason == other.denial_reason
    }

    pub fn to_ledger_entry(&self) -> Result<UsageLedgerEntry, UsageValidationError> {
        self.validate_for_ingestion()?;
        let measurements = &self.payload.measurements;
        let metering_unit = measurements
            .metering_unit
            .clone()
            .unwrap_or(MeteringUnit::Request);
        let metered_quantity = measurements.quantity_for(&metering_unit);
        let estimated_cost_micros = measurements.estimated_cost_micros.unwrap_or(0);
        let actual_cost_micros = measurements.actual_cost_micros;

        Ok(UsageLedgerEntry {
            ledger_entry_id: Uuid::new_v4(),
            event_id: self.id.clone(),
            idempotency_key: self.idempotency_key.clone(),
            working_group_id: self.working_group_id.clone(),
            source: self.source.clone(),
            actor: self.actor.clone(),
            resource: self.resource.clone(),
            subject: self.payload.subject.clone(),
            policy_decision_id: self.policy_decision_id.clone(),
            request_id: self.request_id.clone(),
            correlation_id: self.correlation_id.clone(),
            metered_quantity,
            metering_unit,
            estimated_cost_micros,
            actual_cost_micros,
            cost_reconciliation_status: if actual_cost_micros.is_some() {
                CostReconciliationStatus::Actual
            } else {
                CostReconciliationStatus::PendingProviderReconciliation
            },
            reservation_id: self.reservation_id.clone(),
        })
    }
}

impl UsageMeasurements {
    fn quantity_for(&self, metering_unit: &MeteringUnit) -> u64 {
        match metering_unit {
            MeteringUnit::Request => 1,
            MeteringUnit::ToolInvocation => self.tool_invocations.unwrap_or(0),
            MeteringUnit::RuntimeMs
            | MeteringUnit::LocalComputeMs
            | MeteringUnit::HostedMcpRuntimeMs => self.duration_ms,
            MeteringUnit::Token => self.input_tokens.unwrap_or(0) + self.output_tokens.unwrap_or(0),
        }
    }
}

impl UsageReservation {
    pub fn open(
        reservation_id: String,
        working_group_id: String,
        policy_decision_id: String,
        idempotency_key: String,
        reserved_quantity: u64,
        metering_unit: MeteringUnit,
        estimated_cost_micros: u64,
    ) -> Self {
        Self {
            reservation_id,
            working_group_id,
            policy_decision_id,
            idempotency_key,
            reserved_quantity,
            metering_unit,
            estimated_cost_micros,
            consumed_quantity: 0,
            status: ReservationStatus::Open,
        }
    }

    pub fn consume(&mut self, quantity: u64) {
        self.consumed_quantity = quantity;
        self.status = if quantity > self.reserved_quantity {
            ReservationStatus::Overrun
        } else {
            ReservationStatus::Consumed
        };
    }

    pub fn release(&mut self) {
        self.status = ReservationStatus::Released;
    }

    pub fn expire(&mut self) {
        self.status = ReservationStatus::Expired;
    }

    pub fn settle(&mut self) {
        self.status = ReservationStatus::Settled;
    }
}

fn require_text(field: &'static str, value: &str) -> Result<(), UsageValidationError> {
    if value.trim().is_empty() {
        return Err(UsageValidationError::Required(field));
    }
    reject_sensitive_pattern(field, value)?;

    Ok(())
}

fn require_opaque_ref(
    field: &'static str,
    value: &str,
    prefixes: &[&str],
) -> Result<(), UsageValidationError> {
    require_text(field, value)?;
    if value.len() > 80 || !prefixes.iter().any(|prefix| value.starts_with(prefix)) {
        return Err(UsageValidationError::InvalidReference { field });
    }

    Ok(())
}

fn require_allowed_value(
    field: &'static str,
    value: &str,
    allowed: &[&str],
) -> Result<(), UsageValidationError> {
    require_text(field, value)?;
    if !allowed.contains(&value) {
        return Err(UsageValidationError::InvalidReference { field });
    }

    Ok(())
}

fn actor_prefixes(actor_type: &str) -> [&'static str; 1] {
    match actor_type {
        "user" => ["usr_"],
        "agent" => ["agt_"],
        "service" => ["svc_"],
        _ => [""],
    }
}

fn resource_prefixes(resource_type: &str) -> [&'static str; 1] {
    match resource_type {
        "provider" => ["prv_"],
        "mcp_server" => ["mcp_"],
        "runner" => ["runr_"],
        "job" => ["job_"],
        "integration" => ["int_"],
        _ => [""],
    }
}

fn subject_prefixes(subject_type: &str) -> [&'static str; 1] {
    match subject_type {
        "agent_run" => ["run_"],
        "gateway_request" => ["gwreq_"],
        "workflow_run" => ["wfrun_"],
        "runner_job" => ["job_"],
        _ => [""],
    }
}

fn require_safe_denial_reason(reason: &str) -> Result<(), UsageValidationError> {
    require_text("denial_reason", reason)?;
    if reason.len() > 80
        || ![
            "policy_denied",
            "usage_limit_reached",
            "protected_capability_disabled",
            "quota_exceeded",
        ]
        .contains(&reason)
    {
        return Err(UsageValidationError::UnsafeDenialReason);
    }

    Ok(())
}

fn require_runtime_capability(value: &str) -> Result<(), UsageValidationError> {
    require_text("runtime_capability", value)?;
    if value.len() > 80
        || ![
            "remote.local_tool_execution",
            "remote.local_llm_exposure",
            "remote.external_agent_runtime_adapter",
            "remote.computer_use_execution",
            "gateway.hosted_mcp_billing",
            "gateway.sensitive_provider_routing",
        ]
        .contains(&value)
    {
        return Err(UsageValidationError::InvalidRuntimeCapability);
    }

    Ok(())
}

fn reject_sensitive_pattern(field: &'static str, value: &str) -> Result<(), UsageValidationError> {
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
        "-----begin",
    ];

    if sensitive_patterns
        .iter()
        .any(|pattern| normalized.contains(pattern))
    {
        return Err(UsageValidationError::SensitivePattern { field });
    }

    Ok(())
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
        assert_eq!(evaluation.enforcement, QuotaEnforcement::HardDeny);
        assert_eq!(evaluation.failed_limits, vec!["daily-tokens"]);
        assert_eq!(
            evaluation.denial_reason.as_deref(),
            Some("usage_limit_reached")
        );
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
        assert_eq!(evaluation.enforcement, QuotaEnforcement::Allow);
        assert!(evaluation.failed_limits.is_empty());
        assert!(evaluation.denial_reason.is_none());
    }

    #[test]
    fn usage_event_maps_to_immutable_ledger_entry() -> Result<(), Box<dyn std::error::Error>> {
        let event = UsageAuditEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            r#type: "usage.gateway_request.recorded".to_owned(),
            version: "0.1.0".to_owned(),
            occurred_at: "2026-01-01T00:00:01.000Z".to_owned(),
            source: UsageSourceSurface::Gateway,
            working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            actor: UsageActorRef {
                r#type: "agent".to_owned(),
                id: "agt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            },
            resource: UsageResourceRef {
                r#type: "provider".to_owned(),
                id: "prv_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            },
            correlation_id: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            request_id: "req_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            policy_decision_id: "poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            idempotency_key: "usage_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
            payload: UsageEventPayload {
                subject: UsageSubjectRef {
                    r#type: "gateway_request".to_owned(),
                    id: "gwreq_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned(),
                },
                measurements: UsageMeasurements {
                    duration_ms: 840,
                    input_tokens: Some(1_200),
                    output_tokens: Some(320),
                    tool_invocations: Some(1),
                    estimated_cost_micros: Some(2_300),
                    actual_cost_micros: None,
                    metering_unit: Some(MeteringUnit::Token),
                    runtime_capability: None,
                },
            },
            reservation_id: Some("resv_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned()),
            status: UsageAttemptStatus::Succeeded,
            denial_reason: None,
        };

        let entry = event.to_ledger_entry()?;

        assert_eq!(entry.event_id, event.id);
        assert_eq!(entry.metering_unit, MeteringUnit::Token);
        assert_eq!(entry.metered_quantity, 1_520);
        assert_eq!(entry.estimated_cost_micros, 2_300);
        assert_eq!(
            entry.cost_reconciliation_status,
            CostReconciliationStatus::PendingProviderReconciliation
        );
        assert_eq!(entry.reservation_id, event.reservation_id);
        Ok(())
    }

    #[test]
    fn denied_usage_events_require_a_clear_reason() -> Result<(), Box<dyn std::error::Error>> {
        let mut event: UsageAuditEventV1 = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.high-risk-runtime-denied.json"
        ))?;
        event.status = UsageAttemptStatus::Denied;
        event.denial_reason = None;

        assert_eq!(
            event.validate_for_ingestion(),
            Err(UsageValidationError::MissingDenialReason)
        );
        Ok(())
    }

    #[test]
    fn reservation_lifecycle_covers_settlement_states() {
        let mut reservation = UsageReservation::open(
            "resv_1".to_owned(),
            "wg_1".to_owned(),
            "poldec_1".to_owned(),
            "usage_1".to_owned(),
            10,
            MeteringUnit::Request,
            1_000,
        );
        assert_eq!(reservation.status, ReservationStatus::Open);

        reservation.consume(12);
        assert_eq!(reservation.status, ReservationStatus::Overrun);

        reservation.settle();
        assert_eq!(reservation.status, ReservationStatus::Settled);

        let mut released = reservation.clone();
        released.release();
        assert_eq!(released.status, ReservationStatus::Released);

        let mut expired = reservation;
        expired.expire();
        assert_eq!(expired.status, ReservationStatus::Expired);
    }
}
