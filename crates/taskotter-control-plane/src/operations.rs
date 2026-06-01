use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationsEventEnvelope {
    pub version: String,
    pub occurred_at: String,
    pub working_group_id: String,
    #[serde(default)]
    pub tenant_id: Option<String>,
    pub correlation_id: String,
    pub request_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub job_id: Option<String>,
    pub source: OperationsSourceSurface,
    pub actor: OperationsActorRef,
    pub resource: OperationsResourceRef,
    pub redaction: RedactionClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperationsSourceSurface {
    ControlPlane,
    Gateway,
    Runner,
    Provider,
    AdminApi,
    PolicyEngine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedactionClassification {
    PublicMetadata,
    InternalReferenceOnly,
    RedactedSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationsActorRef {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationsResourceRef {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageContractLink {
    #[serde(default)]
    pub usage_event_id: Option<String>,
    #[serde(default)]
    pub ledger_entry_id: Option<String>,
    #[serde(default)]
    pub reservation_id: Option<String>,
    #[serde(default)]
    pub policy_decision_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RunTimelineEventV1 {
    pub id: String,
    pub r#type: String,
    pub envelope: OperationsEventEnvelope,
    pub stage: RunTimelineStage,
    #[serde(default)]
    pub status_reason: Option<RunTimelineStatusReason>,
    #[serde(default)]
    pub usage_link: Option<UsageContractLink>,
    #[serde(default)]
    pub health_signal: Option<HealthSignalRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunTimelineStage {
    Queued,
    Dispatching,
    Started,
    Progress,
    Waiting,
    RetryScheduled,
    Cancelled,
    TimedOut,
    Failed,
    Completed,
    PolicyDenied,
    ApprovalWaiting,
    RunnerDegraded,
    GatewayDegraded,
    ProviderDegraded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunTimelineStatusReason {
    DispatchAccepted,
    PolicyDenied,
    ApprovalRequired,
    RetryBackoff,
    UserCancelled,
    TimeoutExceeded,
    RunnerUnavailable,
    GatewayUnavailable,
    ProviderUnavailable,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct HealthSignalRef {
    pub health_event_id: String,
    pub target_kind: HealthTargetKind,
    pub target_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationsAuditEventV1 {
    pub id: String,
    pub r#type: String,
    pub envelope: OperationsEventEnvelope,
    pub action: OperationsAuditAction,
    pub outcome: OperationsAuditOutcome,
    pub evidence: OperationsAuditEvidence,
    #[serde(default)]
    pub usage_link: Option<UsageContractLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperationsAuditAction {
    CredentialAccess,
    PolicyDecision,
    GlobalSharingChanged,
    HighCostUsage,
    RunnerRegistration,
    GatewayRegistration,
    PrivateRunnerDispatch,
    ProtectedOperationRequested,
    ProtectedOperationApproved,
    ProtectedOperationDenied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperationsAuditOutcome {
    Succeeded,
    Denied,
    Failed,
    PendingApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationsAuditEvidence {
    pub evidence_id: String,
    #[serde(default)]
    pub policy_decision_id: Option<String>,
    #[serde(default)]
    pub approval_id: Option<String>,
    #[serde(default)]
    pub capability_ref: Option<String>,
    #[serde(default)]
    pub secret_ref: Option<String>,
    #[serde(default)]
    pub integration_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationsHealthEventV1 {
    pub id: String,
    pub r#type: String,
    pub envelope: OperationsEventEnvelope,
    pub target: HealthTargetRef,
    pub availability: HealthAvailability,
    #[serde(default)]
    pub capability_snapshot_ref: Option<String>,
    pub last_seen_at: String,
    #[serde(default)]
    pub degraded_reason: Option<HealthDegradedReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct HealthTargetRef {
    pub kind: HealthTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthTargetKind {
    Runner,
    Gateway,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthAvailability {
    Available,
    Degraded,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthDegradedReason {
    CapacityLimited,
    HeartbeatMissed,
    ProviderError,
    CapabilityMismatch,
    Maintenance,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OperationsValidationError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("version must be 0.1.0")]
    UnsupportedVersion,
    #[error("{field} must use an allowed opaque reference prefix and length")]
    InvalidReference { field: &'static str },
    #[error("{field} contains a sensitive pattern")]
    SensitivePattern { field: &'static str },
    #[error("{0} must use internal_reference_only or redacted_summary redaction")]
    UnsafeRedaction(&'static str),
    #[error("type must be {0}")]
    UnsupportedType(&'static str),
    #[error("{action} requires {field}")]
    MissingEvidence {
        action: &'static str,
        field: &'static str,
    },
    #[error("{action} has an invalid outcome")]
    InvalidOutcome { action: &'static str },
}

impl RunTimelineEventV1 {
    pub fn validate_for_ingestion(&self) -> Result<(), OperationsValidationError> {
        require_type("operations.timeline.recorded", &self.r#type)?;
        require_opaque_ref("id", &self.id, &["evt_"])?;
        self.envelope.validate()?;
        if let Some(usage_link) = &self.usage_link {
            usage_link.validate()?;
        }
        if let Some(health_signal) = &self.health_signal {
            health_signal.validate()?;
        }
        Ok(())
    }
}

impl OperationsAuditEventV1 {
    pub fn validate_for_ingestion(&self) -> Result<(), OperationsValidationError> {
        require_type("operations.audit.recorded", &self.r#type)?;
        require_opaque_ref("id", &self.id, &["evt_"])?;
        self.envelope.validate()?;
        self.evidence.validate()?;
        self.evidence
            .validate_for_action(&self.action, &self.outcome)?;
        if let Some(usage_link) = &self.usage_link {
            usage_link.validate()?;
        }
        if matches!(
            self.envelope.redaction,
            RedactionClassification::PublicMetadata
        ) {
            return Err(OperationsValidationError::UnsafeRedaction("audit event"));
        }
        Ok(())
    }
}

impl OperationsHealthEventV1 {
    pub fn validate_for_ingestion(&self) -> Result<(), OperationsValidationError> {
        require_type("operations.health.recorded", &self.r#type)?;
        require_opaque_ref("id", &self.id, &["evt_"])?;
        self.envelope.validate()?;
        self.target.validate()?;
        require_text("last_seen_at", &self.last_seen_at)?;
        if let Some(capability_snapshot_ref) = &self.capability_snapshot_ref {
            require_opaque_ref(
                "capability_snapshot_ref",
                capability_snapshot_ref,
                &["cap_"],
            )?;
        }
        Ok(())
    }
}

impl OperationsEventEnvelope {
    pub fn validate(&self) -> Result<(), OperationsValidationError> {
        if self.version != "0.1.0" {
            return Err(OperationsValidationError::UnsupportedVersion);
        }
        require_text("occurred_at", &self.occurred_at)?;
        require_opaque_ref("working_group_id", &self.working_group_id, &["wg_"])?;
        let tenant_id = self
            .tenant_id
            .as_ref()
            .ok_or(OperationsValidationError::Required("tenant_id"))?;
        require_opaque_ref("tenant_id", tenant_id, &["tenant_"])?;
        require_opaque_ref("correlation_id", &self.correlation_id, &["corr_"])?;
        require_opaque_ref("request_id", &self.request_id, &["req_"])?;
        if let Some(run_id) = &self.run_id {
            require_opaque_ref("run_id", run_id, &["run_"])?;
        }
        if let Some(job_id) = &self.job_id {
            require_opaque_ref("job_id", job_id, &["job_"])?;
        }
        self.actor.validate()?;
        self.resource.validate()?;
        Ok(())
    }
}

impl UsageContractLink {
    pub fn validate(&self) -> Result<(), OperationsValidationError> {
        if let Some(usage_event_id) = &self.usage_event_id {
            require_opaque_ref("usage_event_id", usage_event_id, &["evt_"])?;
        }
        if let Some(ledger_entry_id) = &self.ledger_entry_id {
            require_uuid_ref("ledger_entry_id", ledger_entry_id)?;
        }
        if let Some(reservation_id) = &self.reservation_id {
            require_opaque_ref("reservation_id", reservation_id, &["resv_"])?;
        }
        if let Some(policy_decision_id) = &self.policy_decision_id {
            require_opaque_ref("policy_decision_id", policy_decision_id, &["poldec_"])?;
        }
        Ok(())
    }
}

impl OperationsActorRef {
    fn validate(&self) -> Result<(), OperationsValidationError> {
        require_text("actor.type", &self.r#type)?;
        require_opaque_ref(
            "actor.id",
            &self.id,
            &["user_", "agent_", "team_", "service_"],
        )
    }
}

impl OperationsResourceRef {
    fn validate(&self) -> Result<(), OperationsValidationError> {
        require_text("resource.type", &self.r#type)?;
        require_opaque_ref(
            "resource.id",
            &self.id,
            &[
                "run_",
                "job_",
                "issue_",
                "workflow_",
                "runner_",
                "gateway_",
                "provider_",
            ],
        )
    }
}

impl HealthSignalRef {
    fn validate(&self) -> Result<(), OperationsValidationError> {
        require_opaque_ref("health_event_id", &self.health_event_id, &["evt_"])?;
        require_health_target_ref(
            "health_signal.target_id",
            &self.target_kind,
            &self.target_id,
        )
    }
}

impl OperationsAuditEvidence {
    fn validate(&self) -> Result<(), OperationsValidationError> {
        require_opaque_ref("evidence_id", &self.evidence_id, &["evidence_"])?;
        if let Some(policy_decision_id) = &self.policy_decision_id {
            require_opaque_ref("policy_decision_id", policy_decision_id, &["poldec_"])?;
        }
        if let Some(approval_id) = &self.approval_id {
            require_opaque_ref("approval_id", approval_id, &["approval_"])?;
        }
        if let Some(capability_ref) = &self.capability_ref {
            require_opaque_ref("capability_ref", capability_ref, &["cap_"])?;
        }
        if let Some(secret_ref) = &self.secret_ref {
            require_opaque_ref("secret_ref", secret_ref, &["secret_"])?;
        }
        if let Some(integration_ref) = &self.integration_ref {
            require_opaque_ref("integration_ref", integration_ref, &["integration_"])?;
        }
        Ok(())
    }

    fn validate_for_action(
        &self,
        action: &OperationsAuditAction,
        outcome: &OperationsAuditOutcome,
    ) -> Result<(), OperationsValidationError> {
        match action {
            OperationsAuditAction::ProtectedOperationRequested => {
                require_outcome(
                    "protected_operation_requested",
                    outcome,
                    &[OperationsAuditOutcome::PendingApproval],
                )?;
                self.require_policy_decision_id("protected_operation_requested")?;
                self.require_capability_ref("protected_operation_requested")?;
            }
            OperationsAuditAction::ProtectedOperationApproved => {
                require_outcome(
                    "protected_operation_approved",
                    outcome,
                    &[OperationsAuditOutcome::Succeeded],
                )?;
                self.require_approval_id("protected_operation_approved")?;
                self.require_policy_decision_id("protected_operation_approved")?;
                self.require_capability_ref("protected_operation_approved")?;
            }
            OperationsAuditAction::ProtectedOperationDenied => {
                require_outcome(
                    "protected_operation_denied",
                    outcome,
                    &[OperationsAuditOutcome::Denied],
                )?;
                self.require_policy_decision_id("protected_operation_denied")?;
                self.require_capability_ref("protected_operation_denied")?;
            }
            OperationsAuditAction::CredentialAccess => {
                self.require_policy_decision_id("credential_access")?;
                self.require_secret_ref("credential_access")?;
            }
            OperationsAuditAction::PolicyDecision | OperationsAuditAction::HighCostUsage => {
                self.require_policy_decision_id(action.as_static_str())?;
            }
            OperationsAuditAction::RunnerRegistration
            | OperationsAuditAction::GatewayRegistration
            | OperationsAuditAction::PrivateRunnerDispatch => {
                self.require_capability_ref(action.as_static_str())?;
            }
            OperationsAuditAction::GlobalSharingChanged => {}
        }
        Ok(())
    }

    fn require_approval_id(&self, action: &'static str) -> Result<(), OperationsValidationError> {
        require_evidence_field(action, "approval_id", self.approval_id.as_deref())
    }

    fn require_policy_decision_id(
        &self,
        action: &'static str,
    ) -> Result<(), OperationsValidationError> {
        require_evidence_field(
            action,
            "policy_decision_id",
            self.policy_decision_id.as_deref(),
        )
    }

    fn require_capability_ref(
        &self,
        action: &'static str,
    ) -> Result<(), OperationsValidationError> {
        require_evidence_field(action, "capability_ref", self.capability_ref.as_deref())
    }

    fn require_secret_ref(&self, action: &'static str) -> Result<(), OperationsValidationError> {
        require_evidence_field(action, "secret_ref", self.secret_ref.as_deref())
    }
}

impl HealthTargetRef {
    fn validate(&self) -> Result<(), OperationsValidationError> {
        require_health_target_ref("target.id", &self.kind, &self.id)
    }
}

fn require_type(expected: &'static str, actual: &str) -> Result<(), OperationsValidationError> {
    if actual != expected {
        return Err(OperationsValidationError::UnsupportedType(expected));
    }
    Ok(())
}

fn require_text(field: &'static str, value: &str) -> Result<(), OperationsValidationError> {
    if value.trim().is_empty() {
        return Err(OperationsValidationError::Required(field));
    }
    reject_sensitive_pattern(field, value)
}

fn require_opaque_ref(
    field: &'static str,
    value: &str,
    allowed_prefixes: &[&str],
) -> Result<(), OperationsValidationError> {
    if value.trim().is_empty() {
        return Err(OperationsValidationError::Required(field));
    }
    reject_sensitive_pattern(field, value)?;
    if value.len() > 96
        || !allowed_prefixes
            .iter()
            .any(|prefix| value.starts_with(prefix))
    {
        return Err(OperationsValidationError::InvalidReference { field });
    }
    Ok(())
}

fn require_uuid_ref(field: &'static str, value: &str) -> Result<(), OperationsValidationError> {
    if value.trim().is_empty() {
        return Err(OperationsValidationError::Required(field));
    }
    reject_sensitive_pattern(field, value)?;
    Uuid::parse_str(value).map_err(|_| OperationsValidationError::InvalidReference { field })?;
    Ok(())
}

fn require_health_target_ref(
    field: &'static str,
    kind: &HealthTargetKind,
    value: &str,
) -> Result<(), OperationsValidationError> {
    let prefix = match kind {
        HealthTargetKind::Runner => "runner_",
        HealthTargetKind::Gateway => "gateway_",
        HealthTargetKind::Provider => "provider_",
    };
    require_opaque_ref(field, value, &[prefix])
}

fn reject_sensitive_pattern(
    field: &'static str,
    value: &str,
) -> Result<(), OperationsValidationError> {
    let normalized = value.to_ascii_lowercase();
    let sensitive_markers = [
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
        "authorization:",
        "cookie:",
        "secret=",
        "token",
    ];
    if sensitive_markers
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Err(OperationsValidationError::SensitivePattern { field });
    }
    Ok(())
}

fn require_outcome(
    action: &'static str,
    actual: &OperationsAuditOutcome,
    allowed: &[OperationsAuditOutcome],
) -> Result<(), OperationsValidationError> {
    if !allowed.contains(actual) {
        return Err(OperationsValidationError::InvalidOutcome { action });
    }
    Ok(())
}

fn require_evidence_field(
    action: &'static str,
    field: &'static str,
    value: Option<&str>,
) -> Result<(), OperationsValidationError> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(()),
        _ => Err(OperationsValidationError::MissingEvidence { action, field }),
    }
}

impl OperationsAuditAction {
    fn as_static_str(&self) -> &'static str {
        match self {
            OperationsAuditAction::CredentialAccess => "credential_access",
            OperationsAuditAction::PolicyDecision => "policy_decision",
            OperationsAuditAction::GlobalSharingChanged => "global_sharing_changed",
            OperationsAuditAction::HighCostUsage => "high_cost_usage",
            OperationsAuditAction::RunnerRegistration => "runner_registration",
            OperationsAuditAction::GatewayRegistration => "gateway_registration",
            OperationsAuditAction::PrivateRunnerDispatch => "private_runner_dispatch",
            OperationsAuditAction::ProtectedOperationRequested => "protected_operation_requested",
            OperationsAuditAction::ProtectedOperationApproved => "protected_operation_approved",
            OperationsAuditAction::ProtectedOperationDenied => "protected_operation_denied",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::usage::UsageAuditEventV1;

    use super::*;

    fn envelope() -> OperationsEventEnvelope {
        OperationsEventEnvelope {
            version: "0.1.0".to_owned(),
            occurred_at: "2026-06-01T04:30:00Z".to_owned(),
            working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            tenant_id: Some("tenant_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned()),
            correlation_id: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            request_id: "req_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            run_id: Some("run_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned()),
            job_id: Some("job_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned()),
            source: OperationsSourceSurface::ControlPlane,
            actor: OperationsActorRef {
                r#type: "user".to_owned(),
                id: "user_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            },
            resource: OperationsResourceRef {
                r#type: "run".to_owned(),
                id: "run_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            },
            redaction: RedactionClassification::InternalReferenceOnly,
        }
    }

    #[test]
    fn timeline_event_covers_lifecycle_and_usage_link_without_redefining_usage() {
        let event = RunTimelineEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            r#type: "operations.timeline.recorded".to_owned(),
            envelope: envelope(),
            stage: RunTimelineStage::PolicyDenied,
            status_reason: Some(RunTimelineStatusReason::PolicyDenied),
            usage_link: Some(UsageContractLink {
                usage_event_id: Some("evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned()),
                ledger_entry_id: Some("018f30d5-9471-7c4c-85c4-0e14c3f76c03".to_owned()),
                reservation_id: Some("resv_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned()),
                policy_decision_id: Some("poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned()),
            }),
            health_signal: None,
        };

        assert_eq!(event.validate_for_ingestion(), Ok(()));
    }

    #[test]
    fn operations_link_accepts_actual_usage_ledger_uuid_losslessly()
    -> Result<(), Box<dyn std::error::Error>> {
        let usage_event: UsageAuditEventV1 = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;
        let ledger_entry = usage_event.to_ledger_entry()?;
        let ledger_entry_id = ledger_entry.ledger_entry_id.to_string();
        let event = RunTimelineEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EF".to_owned(),
            r#type: "operations.timeline.recorded".to_owned(),
            envelope: envelope(),
            stage: RunTimelineStage::Completed,
            status_reason: Some(RunTimelineStatusReason::Completed),
            usage_link: Some(UsageContractLink {
                usage_event_id: Some(ledger_entry.event_id),
                ledger_entry_id: Some(ledger_entry_id.clone()),
                reservation_id: ledger_entry.reservation_id,
                policy_decision_id: Some(ledger_entry.policy_decision_id),
            }),
            health_signal: None,
        };

        assert_eq!(event.validate_for_ingestion(), Ok(()));
        assert_eq!(
            event
                .usage_link
                .as_ref()
                .and_then(|link| link.ledger_entry_id.as_ref()),
            Some(&ledger_entry_id)
        );

        let mut invalid = event.clone();
        if let Some(usage_link) = invalid.usage_link.as_mut() {
            usage_link.ledger_entry_id = Some("ledger_01J9Z4P4BS0M9P2QJ6T8Z6W2EP".to_owned());
        }
        assert_eq!(
            invalid.validate_for_ingestion(),
            Err(OperationsValidationError::InvalidReference {
                field: "ledger_entry_id"
            })
        );
        Ok(())
    }

    #[test]
    fn envelope_rejects_malformed_scope_and_missing_correlation_id() {
        let mut invalid_scope = envelope();
        invalid_scope.working_group_id = "workspace_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned();
        assert_eq!(
            invalid_scope.validate(),
            Err(OperationsValidationError::InvalidReference {
                field: "working_group_id"
            })
        );

        let mut missing_correlation = envelope();
        missing_correlation.correlation_id = "".to_owned();
        assert_eq!(
            missing_correlation.validate(),
            Err(OperationsValidationError::Required("correlation_id"))
        );
    }

    #[test]
    fn envelope_requires_tenant_scope_invariant() {
        let mut missing_tenant = envelope();
        missing_tenant.tenant_id = None;

        assert_eq!(
            missing_tenant.validate(),
            Err(OperationsValidationError::Required("tenant_id"))
        );
    }

    #[test]
    fn audit_event_rejects_public_or_sensitive_evidence() {
        let mut event = OperationsAuditEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EB".to_owned(),
            r#type: "operations.audit.recorded".to_owned(),
            envelope: envelope(),
            action: OperationsAuditAction::CredentialAccess,
            outcome: OperationsAuditOutcome::Denied,
            evidence: OperationsAuditEvidence {
                evidence_id: "evidence_01J9Z4P4BS0M9P2QJ6T8Z6W2EB".to_owned(),
                policy_decision_id: Some("poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EB".to_owned()),
                approval_id: None,
                capability_ref: Some("cap_high_risk_runtime".to_owned()),
                secret_ref: Some("secret_01J9Z4P4BS0M9P2QJ6T8Z6W2EB".to_owned()),
                integration_ref: Some("integration_01J9Z4P4BS0M9P2QJ6T8Z6W2EB".to_owned()),
            },
            usage_link: None,
        };

        assert_eq!(event.validate_for_ingestion(), Ok(()));

        event.envelope.redaction = RedactionClassification::PublicMetadata;
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::UnsafeRedaction("audit event"))
        );

        event.envelope.redaction = RedactionClassification::InternalReferenceOnly;
        event.evidence.secret_ref = Some("secret_raw_prompt_body".to_owned());
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::SensitivePattern {
                field: "secret_ref"
            })
        );
    }

    #[test]
    fn audit_evidence_rejects_usage_level_sensitive_patterns() {
        let mut event = OperationsAuditEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2ED".to_owned(),
            r#type: "operations.audit.recorded".to_owned(),
            envelope: envelope(),
            action: OperationsAuditAction::CredentialAccess,
            outcome: OperationsAuditOutcome::Denied,
            evidence: OperationsAuditEvidence {
                evidence_id: "evidence_01J9Z4P4BS0M9P2QJ6T8Z6W2ED".to_owned(),
                policy_decision_id: Some("poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2ED".to_owned()),
                approval_id: None,
                capability_ref: Some("cap_high_risk_runtime".to_owned()),
                secret_ref: Some("secret_01J9Z4P4BS0M9P2QJ6T8Z6W2ED".to_owned()),
                integration_ref: Some("integration_01J9Z4P4BS0M9P2QJ6T8Z6W2ED".to_owned()),
            },
            usage_link: None,
        };

        event.evidence.capability_ref = Some("cap_client_secret_runtime".to_owned());
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::SensitivePattern {
                field: "capability_ref"
            })
        );

        event.evidence.capability_ref = Some("cap_high_risk_runtime".to_owned());
        event.evidence.integration_ref = Some("integration_artifact_body_ref".to_owned());
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::SensitivePattern {
                field: "integration_ref"
            })
        );

        event.evidence.integration_ref = Some("integration_01J9Z4P4BS0M9P2QJ6T8Z6W2ED".to_owned());
        event.envelope.occurred_at = "-----BEGIN PRIVATE KEY-----".to_owned();
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::SensitivePattern {
                field: "occurred_at"
            })
        );
    }

    #[test]
    fn protected_operation_evidence_is_action_aware() {
        let mut event = OperationsAuditEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EE".to_owned(),
            r#type: "operations.audit.recorded".to_owned(),
            envelope: envelope(),
            action: OperationsAuditAction::ProtectedOperationRequested,
            outcome: OperationsAuditOutcome::PendingApproval,
            evidence: OperationsAuditEvidence {
                evidence_id: "evidence_01J9Z4P4BS0M9P2QJ6T8Z6W2EE".to_owned(),
                policy_decision_id: Some("poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EE".to_owned()),
                approval_id: None,
                capability_ref: Some("cap_high_risk_runtime".to_owned()),
                secret_ref: None,
                integration_ref: None,
            },
            usage_link: None,
        };

        assert_eq!(event.validate_for_ingestion(), Ok(()));

        event.evidence.capability_ref = None;
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::MissingEvidence {
                action: "protected_operation_requested",
                field: "capability_ref"
            })
        );

        event.evidence.capability_ref = Some("cap_high_risk_runtime".to_owned());
        event.action = OperationsAuditAction::ProtectedOperationApproved;
        event.outcome = OperationsAuditOutcome::PendingApproval;
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::InvalidOutcome {
                action: "protected_operation_approved"
            })
        );

        event.outcome = OperationsAuditOutcome::Succeeded;
        assert_eq!(
            event.validate_for_ingestion(),
            Err(OperationsValidationError::MissingEvidence {
                action: "protected_operation_approved",
                field: "approval_id"
            })
        );

        event.evidence.approval_id = Some("approval_01J9Z4P4BS0M9P2QJ6T8Z6W2EE".to_owned());
        assert_eq!(event.validate_for_ingestion(), Ok(()));
    }

    #[test]
    fn health_event_models_heartbeat_and_degraded_reason() {
        let event = OperationsHealthEventV1 {
            id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EC".to_owned(),
            r#type: "operations.health.recorded".to_owned(),
            envelope: OperationsEventEnvelope {
                source: OperationsSourceSurface::Runner,
                resource: OperationsResourceRef {
                    r#type: "runner".to_owned(),
                    id: "runner_01J9Z4P4BS0M9P2QJ6T8Z6W2EC".to_owned(),
                },
                ..envelope()
            },
            target: HealthTargetRef {
                kind: HealthTargetKind::Runner,
                id: "runner_01J9Z4P4BS0M9P2QJ6T8Z6W2EC".to_owned(),
            },
            availability: HealthAvailability::Degraded,
            capability_snapshot_ref: Some("cap_01J9Z4P4BS0M9P2QJ6T8Z6W2EC".to_owned()),
            last_seen_at: "2026-06-01T04:30:03Z".to_owned(),
            degraded_reason: Some(HealthDegradedReason::HeartbeatMissed),
        };

        assert_eq!(event.validate_for_ingestion(), Ok(()));
    }
}
