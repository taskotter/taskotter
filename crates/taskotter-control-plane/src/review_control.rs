use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use crate::domain::IssueId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewControlWorkItem {
    pub id: IssueId,
    pub request: WorkItemRequestContext,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub risk_tier: RiskTier,
    pub autonomy_level: AutonomyLevel,
    pub plan_approval: PlanApproval,
    pub imported_result_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub review_decision: ReviewDecision,
    pub audit: AuditCorrelation,
    pub state: ReviewControlState,
    pub redaction_summary: ReviewControlRedactionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkItemRequestContext {
    pub source: WorkItemSource,
    pub summary: String,
    #[serde(default)]
    pub protected_operation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkItemSource {
    pub source_type: WorkItemSourceType,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemSourceType {
    MulticaIssue,
    GithubIssue,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub text: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    SuggestOnly,
    AgentCanPrepare,
    HumanApprovalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PlanApproval {
    pub state: PlanApprovalState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanApprovalState {
    Pending,
    Approved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReviewDecision {
    pub state: ReviewDecisionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<ReviewDecisionReasonCode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecisionState {
    Pending,
    Rework,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecisionReasonCode {
    AcceptanceCriteriaMet,
    ReviewerRequestedRework,
    MissingEvidence,
    ResidualRisk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuditCorrelation {
    pub correlation_id: String,
    pub request_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_event_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewControlState {
    Draft,
    PlanApproved,
    ReadyForReview,
    Rework,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReviewControlRedactionSummary {
    pub redacted: bool,
    pub redacted_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateReviewControlWorkItemRequest {
    pub schema_version: String,
    pub request: WorkItemRequestContext,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub risk_tier: RiskTier,
    pub autonomy_level: AutonomyLevel,
    pub audit: AuditCorrelation,
    #[serde(default)]
    pub imported_result_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateReviewControlWorkItemRequest {
    pub schema_version: String,
    #[serde(default)]
    pub request_summary: Option<String>,
    #[serde(default)]
    pub protected_operation: Option<bool>,
    #[serde(default)]
    pub acceptance_criteria: Option<Vec<AcceptanceCriterion>>,
    #[serde(default)]
    pub risk_tier: Option<RiskTier>,
    #[serde(default)]
    pub autonomy_level: Option<AutonomyLevel>,
    #[serde(default)]
    pub imported_result_refs: Option<Vec<String>>,
    #[serde(default)]
    pub evidence_refs: Option<Vec<String>>,
    #[serde(default)]
    pub audit_event_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApproveReviewControlPlanRequest {
    pub schema_version: String,
    pub decision_ref: String,
    pub audit_event_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CompleteReviewControlRequest {
    pub schema_version: String,
    pub decision_ref: String,
    pub reason_code: ReviewDecisionReasonCode,
    pub audit_event_ref: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReviewControlError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("schema_version must be review_control.work_item.v1")]
    UnsupportedSchemaVersion,
    #[error("{field} contains an unsafe reference")]
    UnsafeReference { field: &'static str },
    #[error("acceptance_criteria must not be empty")]
    MissingAcceptanceCriteria,
    #[error("high-risk, critical, or protected work items require human approval")]
    ApprovalRequiredInvariantViolation,
    #[error("invalid review control state transition")]
    InvalidTransition,
}

impl CreateReviewControlWorkItemRequest {
    pub fn into_work_item(self) -> Result<ReviewControlWorkItem, ReviewControlError> {
        require_schema_version(&self.schema_version)?;
        let mut redacted_fields = Vec::new();
        let request = sanitize_request(self.request, &mut redacted_fields)?;
        let acceptance_criteria = sanitize_acceptance_criteria(
            self.acceptance_criteria,
            "acceptance_criteria",
            &mut redacted_fields,
        )?;
        let imported_result_refs =
            sanitize_refs("imported_result_refs", self.imported_result_refs)?;
        let evidence_refs = sanitize_refs("evidence_refs", self.evidence_refs)?;
        let audit = sanitize_audit(self.audit)?;
        enforce_approval_required_invariant(
            &self.risk_tier,
            &self.autonomy_level,
            request.protected_operation,
        )?;
        let ready_for_review = !imported_result_refs.is_empty() || !evidence_refs.is_empty();

        Ok(ReviewControlWorkItem {
            id: IssueId::new(),
            request,
            acceptance_criteria,
            risk_tier: self.risk_tier,
            autonomy_level: self.autonomy_level,
            plan_approval: PlanApproval {
                state: PlanApprovalState::Pending,
                decision_ref: None,
            },
            imported_result_refs,
            evidence_refs,
            review_decision: ReviewDecision {
                state: ReviewDecisionState::Pending,
                decision_ref: None,
                reason_code: None,
            },
            audit,
            state: if ready_for_review {
                ReviewControlState::ReadyForReview
            } else {
                ReviewControlState::Draft
            },
            redaction_summary: ReviewControlRedactionSummary {
                redacted: !redacted_fields.is_empty(),
                redacted_fields,
            },
        })
    }
}

impl ReviewControlWorkItem {
    pub fn apply_update(
        &mut self,
        update: UpdateReviewControlWorkItemRequest,
    ) -> Result<(), ReviewControlError> {
        require_schema_version(&update.schema_version)?;
        self.ensure_mutable()?;
        let mut candidate = self.clone();
        candidate.apply_update_candidate(update)?;
        *self = candidate;
        Ok(())
    }

    fn apply_update_candidate(
        &mut self,
        update: UpdateReviewControlWorkItemRequest,
    ) -> Result<(), ReviewControlError> {
        let next_risk_tier = update
            .risk_tier
            .clone()
            .unwrap_or_else(|| self.risk_tier.clone());
        let next_autonomy_level = update
            .autonomy_level
            .clone()
            .unwrap_or_else(|| self.autonomy_level.clone());
        let next_protected_operation = update
            .protected_operation
            .unwrap_or(self.request.protected_operation);
        enforce_approval_required_invariant(
            &next_risk_tier,
            &next_autonomy_level,
            next_protected_operation,
        )?;

        if let Some(summary) = update.request_summary {
            self.request.summary = sanitize_text(
                "request.summary",
                summary,
                &mut self.redaction_summary.redacted_fields,
            );
        }
        if let Some(protected_operation) = update.protected_operation {
            self.request.protected_operation = protected_operation;
        }
        if let Some(acceptance_criteria) = update.acceptance_criteria {
            self.acceptance_criteria = sanitize_acceptance_criteria(
                acceptance_criteria,
                "acceptance_criteria",
                &mut self.redaction_summary.redacted_fields,
            )?;
        }
        if let Some(risk_tier) = update.risk_tier {
            self.risk_tier = risk_tier;
        }
        if let Some(autonomy_level) = update.autonomy_level {
            self.autonomy_level = autonomy_level;
        }
        if let Some(imported_result_refs) = update.imported_result_refs {
            self.imported_result_refs =
                sanitize_refs("imported_result_refs", imported_result_refs)?;
        }
        if let Some(evidence_refs) = update.evidence_refs {
            self.evidence_refs = sanitize_refs("evidence_refs", evidence_refs)?;
        }
        if let Some(audit_event_ref) = update.audit_event_ref {
            require_safe_ref("audit_event_ref", &audit_event_ref)?;
            self.audit.audit_event_ref = Some(audit_event_ref);
        }
        self.redaction_summary.redacted = !self.redaction_summary.redacted_fields.is_empty();
        if !self.imported_result_refs.is_empty() || !self.evidence_refs.is_empty() {
            self.state = ReviewControlState::ReadyForReview;
        }
        Ok(())
    }

    pub fn approve_plan(
        &mut self,
        request: ApproveReviewControlPlanRequest,
    ) -> Result<(), ReviewControlError> {
        require_schema_version(&request.schema_version)?;
        self.ensure_mutable()?;
        require_safe_ref("decision_ref", &request.decision_ref)?;
        require_safe_ref("audit_event_ref", &request.audit_event_ref)?;

        self.plan_approval = PlanApproval {
            state: PlanApprovalState::Approved,
            decision_ref: Some(request.decision_ref),
        };
        self.audit.audit_event_ref = Some(request.audit_event_ref);
        if self.state == ReviewControlState::Draft {
            self.state = ReviewControlState::PlanApproved;
        }
        Ok(())
    }

    pub fn mark_done(
        &mut self,
        request: CompleteReviewControlRequest,
    ) -> Result<(), ReviewControlError> {
        require_schema_version(&request.schema_version)?;
        self.ensure_ready_for_terminal_decision()?;
        require_safe_ref("decision_ref", &request.decision_ref)?;
        require_safe_ref("audit_event_ref", &request.audit_event_ref)?;
        self.review_decision = ReviewDecision {
            state: ReviewDecisionState::Done,
            decision_ref: Some(request.decision_ref),
            reason_code: Some(request.reason_code),
        };
        self.audit.audit_event_ref = Some(request.audit_event_ref);
        self.state = ReviewControlState::Done;
        Ok(())
    }

    pub fn request_rework(
        &mut self,
        request: CompleteReviewControlRequest,
    ) -> Result<(), ReviewControlError> {
        require_schema_version(&request.schema_version)?;
        self.ensure_ready_for_terminal_decision()?;
        require_safe_ref("decision_ref", &request.decision_ref)?;
        require_safe_ref("audit_event_ref", &request.audit_event_ref)?;
        self.review_decision = ReviewDecision {
            state: ReviewDecisionState::Rework,
            decision_ref: Some(request.decision_ref),
            reason_code: Some(request.reason_code),
        };
        self.audit.audit_event_ref = Some(request.audit_event_ref);
        self.state = ReviewControlState::Rework;
        Ok(())
    }

    fn ensure_mutable(&self) -> Result<(), ReviewControlError> {
        match self.state {
            ReviewControlState::Done | ReviewControlState::Rework => {
                Err(ReviewControlError::InvalidTransition)
            }
            ReviewControlState::Draft
            | ReviewControlState::PlanApproved
            | ReviewControlState::ReadyForReview => Ok(()),
        }
    }

    fn ensure_ready_for_terminal_decision(&self) -> Result<(), ReviewControlError> {
        if self.plan_approval.state != PlanApprovalState::Approved {
            return Err(ReviewControlError::InvalidTransition);
        }
        if self.imported_result_refs.is_empty() && self.evidence_refs.is_empty() {
            return Err(ReviewControlError::InvalidTransition);
        }
        self.ensure_mutable()
    }
}

fn require_schema_version(schema_version: &str) -> Result<(), ReviewControlError> {
    if schema_version != "review_control.work_item.v1" {
        return Err(ReviewControlError::UnsupportedSchemaVersion);
    }
    Ok(())
}

fn enforce_approval_required_invariant(
    risk_tier: &RiskTier,
    autonomy_level: &AutonomyLevel,
    protected_operation: bool,
) -> Result<(), ReviewControlError> {
    let approval_required =
        matches!(risk_tier, RiskTier::High | RiskTier::Critical) || protected_operation;
    if approval_required && autonomy_level != &AutonomyLevel::HumanApprovalRequired {
        return Err(ReviewControlError::ApprovalRequiredInvariantViolation);
    }
    Ok(())
}

fn sanitize_request(
    request: WorkItemRequestContext,
    redacted_fields: &mut Vec<String>,
) -> Result<WorkItemRequestContext, ReviewControlError> {
    require_safe_ref("request.source.source_ref", &request.source.source_ref)?;
    require_text("request.summary", &request.summary)?;
    Ok(WorkItemRequestContext {
        source: request.source,
        summary: sanitize_text("request.summary", request.summary, redacted_fields),
        protected_operation: request.protected_operation,
    })
}

fn sanitize_acceptance_criteria(
    criteria: Vec<AcceptanceCriterion>,
    field: &'static str,
    redacted_fields: &mut Vec<String>,
) -> Result<Vec<AcceptanceCriterion>, ReviewControlError> {
    if criteria.is_empty() {
        return Err(ReviewControlError::MissingAcceptanceCriteria);
    }
    criteria
        .into_iter()
        .enumerate()
        .map(|(index, criterion)| {
            require_safe_ref("acceptance_criteria.id", &criterion.id)?;
            require_text("acceptance_criteria.text", &criterion.text)?;
            Ok(AcceptanceCriterion {
                id: criterion.id,
                text: sanitize_text(
                    &format!("{field}[{index}].text"),
                    criterion.text,
                    redacted_fields,
                ),
                required: criterion.required,
            })
        })
        .collect()
}

fn sanitize_audit(audit: AuditCorrelation) -> Result<AuditCorrelation, ReviewControlError> {
    require_safe_ref("audit.correlation_id", &audit.correlation_id)?;
    require_safe_ref("audit.request_id", &audit.request_id)?;
    if let Some(audit_event_ref) = &audit.audit_event_ref {
        require_safe_ref("audit.audit_event_ref", audit_event_ref)?;
    }
    Ok(audit)
}

fn sanitize_refs(
    field: &'static str,
    refs: Vec<String>,
) -> Result<Vec<String>, ReviewControlError> {
    refs.into_iter()
        .map(|value| {
            require_safe_ref(field, &value)?;
            Ok(value)
        })
        .collect()
}

fn sanitize_text(field: &str, value: String, redacted_fields: &mut Vec<String>) -> String {
    if contains_sensitive_pattern(&value) {
        redacted_fields.push(field.to_owned());
        return "[redacted]".to_owned();
    }
    value
}

fn require_text(field: &'static str, value: &str) -> Result<(), ReviewControlError> {
    if value.trim().is_empty() {
        return Err(ReviewControlError::Required(field));
    }
    Ok(())
}

fn require_safe_ref(field: &'static str, value: &str) -> Result<(), ReviewControlError> {
    require_text(field, value)?;
    if contains_sensitive_pattern(value) {
        return Err(ReviewControlError::UnsafeReference { field });
    }
    Ok(())
}

fn contains_sensitive_pattern(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    let markers = [
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
    markers.iter().any(|marker| normalized.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_request() -> CreateReviewControlWorkItemRequest {
        CreateReviewControlWorkItemRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            request: WorkItemRequestContext {
                source: WorkItemSource {
                    source_type: WorkItemSourceType::MulticaIssue,
                    source_ref: "issue_BOG_571".to_owned(),
                },
                summary: "Implement review control contract.".to_owned(),
                protected_operation: false,
            },
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "ac_create".to_owned(),
                text: "Create stores safe work item state.".to_owned(),
                required: true,
            }],
            risk_tier: RiskTier::High,
            autonomy_level: AutonomyLevel::HumanApprovalRequired,
            audit: AuditCorrelation {
                correlation_id: "corr_BOG_571".to_owned(),
                request_id: "req_BOG_571".to_owned(),
                audit_event_ref: None,
            },
            imported_result_refs: Vec::new(),
            evidence_refs: Vec::new(),
        }
    }

    #[test]
    fn create_update_approve_done_transition_succeeds() -> Result<(), Box<dyn std::error::Error>> {
        let mut item = create_request().into_work_item()?;
        assert_eq!(item.state, ReviewControlState::Draft);

        item.apply_update(UpdateReviewControlWorkItemRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            request_summary: None,
            protected_operation: None,
            acceptance_criteria: None,
            risk_tier: None,
            autonomy_level: None,
            imported_result_refs: Some(vec!["import_agent_result_1".to_owned()]),
            evidence_refs: Some(vec!["review_packet_1".to_owned()]),
            audit_event_ref: Some("audit_update_1".to_owned()),
        })?;
        assert_eq!(item.state, ReviewControlState::ReadyForReview);

        item.approve_plan(ApproveReviewControlPlanRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            decision_ref: "plan_decision_1".to_owned(),
            audit_event_ref: "audit_plan_1".to_owned(),
        })?;
        item.mark_done(CompleteReviewControlRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            decision_ref: "done_decision_1".to_owned(),
            reason_code: ReviewDecisionReasonCode::AcceptanceCriteriaMet,
            audit_event_ref: "audit_done_1".to_owned(),
        })?;

        assert_eq!(item.state, ReviewControlState::Done);
        assert_eq!(item.review_decision.state, ReviewDecisionState::Done);
        Ok(())
    }

    #[test]
    fn approve_rework_transition_succeeds() -> Result<(), Box<dyn std::error::Error>> {
        let mut request = create_request();
        request.evidence_refs.push("review_packet_1".to_owned());
        let mut item = request.into_work_item()?;
        item.approve_plan(ApproveReviewControlPlanRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            decision_ref: "plan_decision_1".to_owned(),
            audit_event_ref: "audit_plan_1".to_owned(),
        })?;

        item.request_rework(CompleteReviewControlRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            decision_ref: "rework_decision_1".to_owned(),
            reason_code: ReviewDecisionReasonCode::ReviewerRequestedRework,
            audit_event_ref: "audit_rework_1".to_owned(),
        })?;

        assert_eq!(item.state, ReviewControlState::Rework);
        assert_eq!(item.review_decision.state, ReviewDecisionState::Rework);
        Ok(())
    }

    #[test]
    fn terminal_decision_requires_plan_approval_and_evidence() -> Result<(), ReviewControlError> {
        let mut item = create_request().into_work_item()?;
        assert_eq!(
            item.mark_done(CompleteReviewControlRequest {
                schema_version: "review_control.work_item.v1".to_owned(),
                decision_ref: "done_decision_1".to_owned(),
                reason_code: ReviewDecisionReasonCode::AcceptanceCriteriaMet,
                audit_event_ref: "audit_done_1".to_owned(),
            }),
            Err(ReviewControlError::InvalidTransition)
        );
        Ok(())
    }

    #[test]
    fn privacy_safe_contract_redacts_text_and_rejects_secret_refs()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut request = create_request();
        request.request.summary =
            "Implementation summary included bearer token by mistake.".to_owned();
        let item = request.into_work_item()?;
        assert_eq!(item.request.summary, "[redacted]");
        assert!(item.redaction_summary.redacted);

        let mut unsafe_request = create_request();
        unsafe_request
            .evidence_refs
            .push("raw_log_token_123".to_owned());
        assert_eq!(
            unsafe_request.into_work_item(),
            Err(ReviewControlError::UnsafeReference {
                field: "evidence_refs"
            })
        );
        Ok(())
    }

    #[test]
    fn high_and_critical_risk_require_human_approval() {
        for (risk_tier, autonomy_level) in [
            (RiskTier::High, AutonomyLevel::SuggestOnly),
            (RiskTier::High, AutonomyLevel::AgentCanPrepare),
            (RiskTier::Critical, AutonomyLevel::SuggestOnly),
            (RiskTier::Critical, AutonomyLevel::AgentCanPrepare),
        ] {
            let mut request = create_request();
            request.risk_tier = risk_tier;
            request.autonomy_level = autonomy_level;
            assert_eq!(
                request.into_work_item(),
                Err(ReviewControlError::ApprovalRequiredInvariantViolation)
            );
        }
    }

    #[test]
    fn protected_operation_requires_human_approval_even_when_risk_is_medium() {
        for autonomy_level in [AutonomyLevel::SuggestOnly, AutonomyLevel::AgentCanPrepare] {
            let mut request = create_request();
            request.risk_tier = RiskTier::Medium;
            request.autonomy_level = autonomy_level;
            request.request.protected_operation = true;
            assert_eq!(
                request.into_work_item(),
                Err(ReviewControlError::ApprovalRequiredInvariantViolation)
            );
        }
    }

    #[test]
    fn low_and_medium_unprotected_items_allow_non_human_autonomy()
    -> Result<(), Box<dyn std::error::Error>> {
        for (risk_tier, autonomy_level) in [
            (RiskTier::Low, AutonomyLevel::SuggestOnly),
            (RiskTier::Medium, AutonomyLevel::AgentCanPrepare),
        ] {
            let mut request = create_request();
            request.risk_tier = risk_tier.clone();
            request.autonomy_level = autonomy_level.clone();
            let item = request.into_work_item()?;
            assert_eq!(item.risk_tier, risk_tier);
            assert_eq!(item.autonomy_level, autonomy_level);
            assert!(!item.request.protected_operation);
        }
        Ok(())
    }

    #[test]
    fn update_rejects_policy_regression_to_non_human_autonomy()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut item = create_request().into_work_item()?;

        assert_eq!(
            item.apply_update(UpdateReviewControlWorkItemRequest {
                schema_version: "review_control.work_item.v1".to_owned(),
                request_summary: None,
                protected_operation: None,
                acceptance_criteria: None,
                risk_tier: None,
                autonomy_level: Some(AutonomyLevel::AgentCanPrepare),
                imported_result_refs: None,
                evidence_refs: None,
                audit_event_ref: None,
            }),
            Err(ReviewControlError::ApprovalRequiredInvariantViolation)
        );
        assert_eq!(item.autonomy_level, AutonomyLevel::HumanApprovalRequired);
        Ok(())
    }

    #[test]
    fn update_rejects_marking_agent_prepared_work_as_protected()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut request = create_request();
        request.risk_tier = RiskTier::Medium;
        request.autonomy_level = AutonomyLevel::AgentCanPrepare;
        let mut item = request.into_work_item()?;

        assert_eq!(
            item.apply_update(UpdateReviewControlWorkItemRequest {
                schema_version: "review_control.work_item.v1".to_owned(),
                request_summary: None,
                protected_operation: Some(true),
                acceptance_criteria: None,
                risk_tier: None,
                autonomy_level: None,
                imported_result_refs: None,
                evidence_refs: None,
                audit_event_ref: None,
            }),
            Err(ReviewControlError::ApprovalRequiredInvariantViolation)
        );
        assert!(!item.request.protected_operation);
        Ok(())
    }

    #[test]
    fn failed_update_does_not_partially_commit_security_sensitive_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut request = create_request();
        request.request.protected_operation = true;
        request.imported_result_refs = vec!["import_original".to_owned()];
        request.evidence_refs = vec!["evidence_original".to_owned()];
        request.audit.audit_event_ref = Some("audit_original".to_owned());
        let mut item = request.into_work_item()?;

        let result = item.apply_update(UpdateReviewControlWorkItemRequest {
            schema_version: "review_control.work_item.v1".to_owned(),
            request_summary: Some("Summary accidentally included bearer token.".to_owned()),
            protected_operation: Some(false),
            acceptance_criteria: None,
            risk_tier: Some(RiskTier::Low),
            autonomy_level: Some(AutonomyLevel::SuggestOnly),
            imported_result_refs: Some(vec!["import_new".to_owned()]),
            evidence_refs: Some(vec!["evidence_new".to_owned()]),
            audit_event_ref: Some("audit_access_token_unsafe".to_owned()),
        });

        assert_eq!(
            result,
            Err(ReviewControlError::UnsafeReference {
                field: "audit_event_ref"
            })
        );
        assert_eq!(item.risk_tier, RiskTier::High);
        assert_eq!(item.autonomy_level, AutonomyLevel::HumanApprovalRequired);
        assert!(item.request.protected_operation);
        assert_eq!(item.request.summary, "Implement review control contract.");
        assert_eq!(item.imported_result_refs, vec!["import_original"]);
        assert_eq!(item.evidence_refs, vec!["evidence_original"]);
        assert_eq!(
            item.audit.audit_event_ref,
            Some("audit_original".to_owned())
        );
        assert!(!item.redaction_summary.redacted);
        assert!(item.redaction_summary.redacted_fields.is_empty());
        Ok(())
    }
}
