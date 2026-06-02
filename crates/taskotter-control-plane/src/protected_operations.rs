use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};
use utoipa::ToSchema;

use crate::operations::{OperationsActorRef, OperationsResourceRef};

pub const FIXTURE_ONLY_CONSUMPTION_POLICY: &str =
    "fixture_smoke_only_do_not_use_for_executor_enforcement";
pub const CLIENT_SUPPLIED_RECORD_TRUSTED_FOR_EXECUTION: bool = false;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedOperationAction {
    Destructive,
    Credential,
    Deployment,
    Billing,
    Account,
    GlobalSharing,
    ProtectedBranch,
    PaidHostedMcp,
    HighCostAutomation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Pending,
    Approved,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApproverEligibility {
    Eligible,
    Ineligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalGateStatus {
    WaitingApproval,
    Approved,
    Denied,
    Expired,
    UnauthorizedApprover,
    AlreadyUsed,
    ScopeMismatch,
    InvalidTimestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalScope {
    pub action: ProtectedOperationAction,
    pub resource: OperationsResourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalDenial {
    pub reason_code: String,
    #[serde(default)]
    pub denied_by: Option<OperationsActorRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalRecord {
    pub id: String,
    pub actor: OperationsActorRef,
    #[serde(default)]
    pub delegated_actor: Option<OperationsActorRef>,
    pub action: ProtectedOperationAction,
    pub resource: OperationsResourceRef,
    pub risk_summary: String,
    pub expires_at: String,
    pub approver: OperationsActorRef,
    pub approver_eligibility: ApproverEligibility,
    pub decision: ApprovalDecision,
    #[serde(default)]
    pub scoped_permission: Option<ApprovalScope>,
    #[serde(default)]
    pub scoped_denial: Option<ApprovalDenial>,
    #[serde(default)]
    pub used_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalGateTrustBoundary {
    pub fixture_only: bool,
    pub client_supplied_record_trusted_for_execution: bool,
    pub consumption_policy: String,
    pub defense_reason: String,
}

/// Fixture/smoke-only request body. This client-supplied record is not an
/// authoritative enforcement source and must not be used by real executors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalGateRequest {
    pub actor: OperationsActorRef,
    #[serde(default)]
    pub delegated_actor: Option<OperationsActorRef>,
    pub action: ProtectedOperationAction,
    pub resource: OperationsResourceRef,
    pub risk_summary: String,
    pub evaluated_at: String,
    #[serde(default)]
    pub approval_record: Option<ApprovalRecord>,
}

/// Fixture/smoke-only decision. `side_effect_permitted` is only a regression
/// signal for this contract; real executors must fetch immutable approval
/// records from an authoritative server-side store before acting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalGateDecision {
    pub allowed: bool,
    pub status: ApprovalGateStatus,
    pub reason_code: String,
    pub actor: OperationsActorRef,
    #[serde(default)]
    pub delegated_actor: Option<OperationsActorRef>,
    pub action: ProtectedOperationAction,
    pub resource: OperationsResourceRef,
    pub risk_summary: String,
    #[serde(default)]
    pub approval_record: Option<ApprovalRecord>,
    pub side_effect_permitted: bool,
    pub trust_boundary: ApprovalGateTrustBoundary,
}

pub fn evaluate_approval_gate(request: ApprovalGateRequest) -> ApprovalGateDecision {
    let Some(record) = request.approval_record.clone() else {
        return denied_decision(
            request,
            ApprovalGateStatus::WaitingApproval,
            "approval_required",
        );
    };

    let Ok(expiry) = parse_rfc3339_utc(&record.expires_at) else {
        return denied_decision(
            request,
            ApprovalGateStatus::InvalidTimestamp,
            "invalid_approval_expiration",
        );
    };
    let Ok(evaluated_at) = parse_rfc3339_utc(&request.evaluated_at) else {
        return denied_decision(
            request,
            ApprovalGateStatus::InvalidTimestamp,
            "invalid_evaluation_timestamp",
        );
    };

    if record.used_at.is_some() {
        return denied_decision(
            request,
            ApprovalGateStatus::AlreadyUsed,
            "approval_record_already_used",
        );
    }

    if expiry <= evaluated_at {
        return denied_decision(
            request,
            ApprovalGateStatus::Expired,
            "approval_record_expired",
        );
    }

    if record.approver_eligibility != ApproverEligibility::Eligible {
        return denied_decision(
            request,
            ApprovalGateStatus::UnauthorizedApprover,
            "approver_not_eligible",
        );
    }

    if record.decision == ApprovalDecision::Denied {
        return denied_decision(
            request,
            ApprovalGateStatus::Denied,
            record
                .scoped_denial
                .as_ref()
                .map(|denial| denial.reason_code.as_str())
                .unwrap_or("approval_denied"),
        );
    }

    if record.decision == ApprovalDecision::Pending {
        return denied_decision(
            request,
            ApprovalGateStatus::WaitingApproval,
            "approval_pending",
        );
    }

    if !record_matches_request(&record, &request) {
        return denied_decision(
            request,
            ApprovalGateStatus::ScopeMismatch,
            "approval_scope_mismatch",
        );
    }

    ApprovalGateDecision {
        allowed: true,
        status: ApprovalGateStatus::Approved,
        reason_code: "approval_record_valid".to_owned(),
        actor: request.actor,
        delegated_actor: request.delegated_actor,
        action: request.action,
        resource: request.resource,
        risk_summary: request.risk_summary,
        approval_record: Some(record),
        side_effect_permitted: true,
        trust_boundary: trust_boundary_contract(),
    }
}

fn denied_decision(
    request: ApprovalGateRequest,
    status: ApprovalGateStatus,
    reason_code: &str,
) -> ApprovalGateDecision {
    ApprovalGateDecision {
        allowed: false,
        status,
        reason_code: reason_code.to_owned(),
        actor: request.actor,
        delegated_actor: request.delegated_actor,
        action: request.action,
        resource: request.resource,
        risk_summary: request.risk_summary,
        approval_record: request.approval_record,
        side_effect_permitted: false,
        trust_boundary: trust_boundary_contract(),
    }
}

fn parse_rfc3339_utc(timestamp: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(timestamp, &Rfc3339).map(|parsed| parsed.to_offset(UtcOffset::UTC))
}

fn trust_boundary_contract() -> ApprovalGateTrustBoundary {
    ApprovalGateTrustBoundary {
        fixture_only: true,
        client_supplied_record_trusted_for_execution: CLIENT_SUPPLIED_RECORD_TRUSTED_FOR_EXECUTION,
        consumption_policy: FIXTURE_ONLY_CONSUMPTION_POLICY.to_owned(),
        defense_reason: "Client-supplied approval records are regression fixtures only; real protected-operation executors must use authoritative server-side approval lookup and atomic consume semantics before side effects.".to_owned(),
    }
}

fn record_matches_request(record: &ApprovalRecord, request: &ApprovalGateRequest) -> bool {
    record.actor == request.actor
        && record.delegated_actor == request.delegated_actor
        && record.action == request.action
        && record.resource == request.resource
        && record.risk_summary == request.risk_summary
        && record.scoped_permission.as_ref().is_some_and(|scope| {
            scope.action == request.action && scope.resource == request.resource
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-06-01T00:00:00Z";
    const FUTURE: &str = "2026-06-01T00:05:00Z";
    const PAST: &str = "2026-05-31T23:55:00Z";

    #[test]
    fn protected_action_fixtures_stop_before_side_effect_until_valid_approval() {
        for action in protected_actions() {
            let mut executor = FakeProtectedExecutor::default();
            let pending = evaluate_approval_gate(gate_request(action.clone(), None));

            executor.maybe_execute(&pending);

            assert_eq!(pending.status, ApprovalGateStatus::WaitingApproval);
            assert!(!pending.allowed);
            assert!(!pending.side_effect_permitted);
            assert_eq!(executor.side_effect_count, 0, "{action:?} escaped gate");
        }
    }

    #[test]
    fn approval_record_state_fixtures_cover_gate_outcomes() {
        let cases = [
            (
                approval_record(ApprovalDecision::Pending),
                ApprovalGateStatus::WaitingApproval,
                "approval_pending",
            ),
            (
                approval_record(ApprovalDecision::Denied),
                ApprovalGateStatus::Denied,
                "operator_denied",
            ),
            (
                expired_approval_record(),
                ApprovalGateStatus::Expired,
                "approval_record_expired",
            ),
            (
                unauthorized_approval_record(),
                ApprovalGateStatus::UnauthorizedApprover,
                "approver_not_eligible",
            ),
            (
                already_used_approval_record(),
                ApprovalGateStatus::AlreadyUsed,
                "approval_record_already_used",
            ),
        ];

        for (record, status, reason_code) in cases {
            let decision =
                evaluate_approval_gate(gate_request(record.action.clone(), Some(record)));

            assert_eq!(decision.status, status);
            assert_eq!(decision.reason_code, reason_code);
            assert!(!decision.allowed);
            assert!(!decision.side_effect_permitted);
        }
    }

    #[test]
    fn approved_record_preserves_auditable_scope_and_allows_side_effect()
    -> Result<(), Box<dyn std::error::Error>> {
        let record = approval_record(ApprovalDecision::Approved);
        let decision = evaluate_approval_gate(gate_request(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
        ));

        let stored = decision
            .approval_record
            .as_ref()
            .ok_or_else(|| std::io::Error::other("approval record is echoed for audit"))?;
        assert!(decision.allowed);
        assert!(decision.side_effect_permitted);
        assert_eq!(decision.status, ApprovalGateStatus::Approved);
        assert_eq!(stored.actor.id, "user_approver_requester");
        assert_eq!(
            stored
                .delegated_actor
                .as_ref()
                .map(|actor| actor.id.as_str()),
            Some("agent_executor")
        );
        assert_eq!(stored.action, ProtectedOperationAction::PaidHostedMcp);
        assert_eq!(stored.resource.r#type, "mcp_server");
        assert_eq!(
            stored.risk_summary,
            "Starts paid hosted MCP runtime for a scoped server."
        );
        assert_eq!(stored.expires_at, FUTURE);
        assert_eq!(stored.approver_eligibility, ApproverEligibility::Eligible);
        assert_eq!(stored.decision, ApprovalDecision::Approved);
        assert!(stored.scoped_permission.is_some());
        assert!(stored.scoped_denial.is_none());
        Ok(())
    }

    #[test]
    fn approval_scope_mismatch_stops_before_side_effect() {
        let mut record = approval_record(ApprovalDecision::Approved);
        record.scoped_permission = Some(ApprovalScope {
            action: ProtectedOperationAction::Deployment,
            resource: resource("deployment", "deploy_prod"),
        });

        let mut executor = FakeProtectedExecutor::default();
        let decision = evaluate_approval_gate(gate_request(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
        ));

        executor.maybe_execute(&decision);

        assert_eq!(decision.status, ApprovalGateStatus::ScopeMismatch);
        assert_eq!(decision.reason_code, "approval_scope_mismatch");
        assert!(!decision.side_effect_permitted);
        assert_eq!(executor.side_effect_count, 0);
    }

    #[test]
    fn expiration_uses_rfc3339_utc_normalized_comparison_for_offset_equivalent_values() {
        let record = approval_record_with_expiration("2026-06-01T01:05:00+01:00");
        let decision = evaluate_approval_gate(gate_request_with_evaluated_at(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
            "2026-06-01T00:00:00Z",
        ));

        assert!(decision.allowed);
        assert!(decision.side_effect_permitted);
        assert_eq!(decision.status, ApprovalGateStatus::Approved);
    }

    #[test]
    fn fractional_second_expiration_is_parsed_before_comparison() {
        let record = approval_record_with_expiration("2026-06-01T00:00:00.500Z");
        let decision = evaluate_approval_gate(gate_request_with_evaluated_at(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
            "2026-06-01T00:00:00Z",
        ));

        assert!(decision.allowed);
        assert!(decision.side_effect_permitted);
        assert_eq!(decision.status, ApprovalGateStatus::Approved);
    }

    #[test]
    fn malformed_expiration_denies_before_side_effect() {
        let record = approval_record_with_expiration("not-a-timestamp");
        let mut executor = FakeProtectedExecutor::default();
        let decision = evaluate_approval_gate(gate_request(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
        ));

        executor.maybe_execute(&decision);

        assert_eq!(decision.status, ApprovalGateStatus::InvalidTimestamp);
        assert_eq!(decision.reason_code, "invalid_approval_expiration");
        assert!(!decision.side_effect_permitted);
        assert_eq!(executor.side_effect_count, 0);
    }

    #[test]
    fn malformed_evaluation_time_denies_before_side_effect() {
        let record = approval_record(ApprovalDecision::Approved);
        let decision = evaluate_approval_gate(gate_request_with_evaluated_at(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
            "June 1 2026",
        ));

        assert_eq!(decision.status, ApprovalGateStatus::InvalidTimestamp);
        assert_eq!(decision.reason_code, "invalid_evaluation_timestamp");
        assert!(!decision.side_effect_permitted);
    }

    #[test]
    fn exact_boundary_expiration_is_expired() {
        let record = approval_record_with_expiration(NOW);
        let decision = evaluate_approval_gate(gate_request(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
        ));

        assert_eq!(decision.status, ApprovalGateStatus::Expired);
        assert_eq!(decision.reason_code, "approval_record_expired");
        assert!(!decision.side_effect_permitted);
    }

    #[test]
    fn past_expiration_is_expired_after_normalization() {
        let record = approval_record_with_expiration("2026-06-01T00:59:59+01:00");
        let decision = evaluate_approval_gate(gate_request(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
        ));

        assert_eq!(decision.status, ApprovalGateStatus::Expired);
        assert_eq!(decision.reason_code, "approval_record_expired");
        assert!(!decision.side_effect_permitted);
    }

    #[test]
    fn fixture_api_contract_marks_client_supplied_records_as_not_executor_trusted() {
        let record = approval_record(ApprovalDecision::Approved);
        let decision = evaluate_approval_gate(gate_request(
            ProtectedOperationAction::PaidHostedMcp,
            Some(record),
        ));

        assert!(decision.trust_boundary.fixture_only);
        assert!(
            !decision
                .trust_boundary
                .client_supplied_record_trusted_for_execution
        );
        assert_eq!(
            decision.trust_boundary.consumption_policy,
            FIXTURE_ONLY_CONSUMPTION_POLICY
        );
        assert!(
            decision
                .trust_boundary
                .defense_reason
                .contains("authoritative server-side approval lookup")
        );
    }

    fn protected_actions() -> Vec<ProtectedOperationAction> {
        vec![
            ProtectedOperationAction::Destructive,
            ProtectedOperationAction::Credential,
            ProtectedOperationAction::Deployment,
            ProtectedOperationAction::Billing,
            ProtectedOperationAction::Account,
            ProtectedOperationAction::GlobalSharing,
            ProtectedOperationAction::ProtectedBranch,
            ProtectedOperationAction::PaidHostedMcp,
            ProtectedOperationAction::HighCostAutomation,
        ]
    }

    fn gate_request(
        action: ProtectedOperationAction,
        approval_record: Option<ApprovalRecord>,
    ) -> ApprovalGateRequest {
        gate_request_with_evaluated_at(action, approval_record, NOW)
    }

    fn gate_request_with_evaluated_at(
        action: ProtectedOperationAction,
        approval_record: Option<ApprovalRecord>,
        evaluated_at: &str,
    ) -> ApprovalGateRequest {
        ApprovalGateRequest {
            actor: actor("user", "user_approver_requester"),
            delegated_actor: Some(actor("agent", "agent_executor")),
            action,
            resource: resource("mcp_server", "mcp_paid_runtime"),
            risk_summary: "Starts paid hosted MCP runtime for a scoped server.".to_owned(),
            evaluated_at: evaluated_at.to_owned(),
            approval_record,
        }
    }

    fn approval_record(decision: ApprovalDecision) -> ApprovalRecord {
        ApprovalRecord {
            id: "approval_01J9Z4P4BS0M9P2QJ6T8Z6W2EE".to_owned(),
            actor: actor("user", "user_approver_requester"),
            delegated_actor: Some(actor("agent", "agent_executor")),
            action: ProtectedOperationAction::PaidHostedMcp,
            resource: resource("mcp_server", "mcp_paid_runtime"),
            risk_summary: "Starts paid hosted MCP runtime for a scoped server.".to_owned(),
            expires_at: FUTURE.to_owned(),
            approver: actor("user", "user_owner"),
            approver_eligibility: ApproverEligibility::Eligible,
            decision: decision.clone(),
            scoped_permission: (decision == ApprovalDecision::Approved).then(|| ApprovalScope {
                action: ProtectedOperationAction::PaidHostedMcp,
                resource: resource("mcp_server", "mcp_paid_runtime"),
            }),
            scoped_denial: (decision == ApprovalDecision::Denied).then(|| ApprovalDenial {
                reason_code: "operator_denied".to_owned(),
                denied_by: Some(actor("user", "user_owner")),
            }),
            used_at: None,
        }
    }

    fn expired_approval_record() -> ApprovalRecord {
        approval_record_with_expiration(PAST)
    }

    fn approval_record_with_expiration(expires_at: &str) -> ApprovalRecord {
        ApprovalRecord {
            expires_at: expires_at.to_owned(),
            decision: ApprovalDecision::Approved,
            scoped_permission: Some(ApprovalScope {
                action: ProtectedOperationAction::PaidHostedMcp,
                resource: resource("mcp_server", "mcp_paid_runtime"),
            }),
            ..approval_record(ApprovalDecision::Approved)
        }
    }

    fn unauthorized_approval_record() -> ApprovalRecord {
        ApprovalRecord {
            approver_eligibility: ApproverEligibility::Ineligible,
            decision: ApprovalDecision::Approved,
            scoped_permission: Some(ApprovalScope {
                action: ProtectedOperationAction::PaidHostedMcp,
                resource: resource("mcp_server", "mcp_paid_runtime"),
            }),
            ..approval_record(ApprovalDecision::Approved)
        }
    }

    fn already_used_approval_record() -> ApprovalRecord {
        ApprovalRecord {
            used_at: Some(NOW.to_owned()),
            decision: ApprovalDecision::Approved,
            scoped_permission: Some(ApprovalScope {
                action: ProtectedOperationAction::PaidHostedMcp,
                resource: resource("mcp_server", "mcp_paid_runtime"),
            }),
            ..approval_record(ApprovalDecision::Approved)
        }
    }

    fn actor(r#type: &str, id: &str) -> OperationsActorRef {
        OperationsActorRef {
            r#type: r#type.to_owned(),
            id: id.to_owned(),
        }
    }

    fn resource(r#type: &str, id: &str) -> OperationsResourceRef {
        OperationsResourceRef {
            r#type: r#type.to_owned(),
            id: id.to_owned(),
        }
    }

    #[derive(Default)]
    struct FakeProtectedExecutor {
        side_effect_count: usize,
    }

    impl FakeProtectedExecutor {
        fn maybe_execute(&mut self, decision: &ApprovalGateDecision) {
            if decision.side_effect_permitted {
                self.side_effect_count += 1;
            }
        }
    }
}
