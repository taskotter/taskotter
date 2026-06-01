use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{FromRequest, Path, Request, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    agent_result::{
        ArtifactEvidence, ChangedFileEvidence, ChangedFileKind, CommandEvidence, CommandOutcome,
        ImportAgentResultRequest, ImportedAgentResultEvidence, RedactionSummary,
        ReviewPacketEvidence,
    },
    audit::{AuditEvent, AuditEventId, CreateAuditEventRequest},
    authorization::{
        AuthorizationContext, DelegatedAuthority, RoleBinding, WorkingGroupMembership,
        WorkingGroupRole,
    },
    domain::{
        AgentId, Comment, CommentId, CreateCommentRequest, CreateIssueRequest,
        CreateRegistryEntryRequest, CreateWorkingGroupRequest, Issue, IssueId, IssueStatus,
        RegistryEntry, WorkingGroup, WorkingGroupId,
    },
    operations::{
        HealthAvailability, HealthDegradedReason, HealthSignalRef, HealthTargetKind,
        HealthTargetRef, OperationsActorRef, OperationsAuditAction, OperationsAuditEventV1,
        OperationsAuditEvidence, OperationsAuditOutcome, OperationsEventEnvelope,
        OperationsHealthEventV1, OperationsResourceRef, OperationsSourceSurface,
        RedactionClassification, RunTimelineEventV1, RunTimelineStage, RunTimelineStatusReason,
        UsageContractLink,
    },
    policy::{
        BaselinePolicyEvaluator, PolicyActorRef, PolicyDecision, PolicyDecisionRequest,
        PolicyEvaluator,
    },
    review_control::{
        AcceptanceCriterion, ApproveReviewControlPlanRequest, AuditCorrelation, AutonomyLevel,
        CompleteReviewControlRequest, CreateReviewControlWorkItemRequest, PlanApproval,
        PlanApprovalState, ReviewControlError, ReviewControlRedactionSummary, ReviewControlState,
        ReviewControlWorkItem, ReviewDecision, ReviewDecisionReasonCode, ReviewDecisionState,
        RiskTier, UpdateReviewControlWorkItemRequest, WorkItemRequestContext, WorkItemSource,
        WorkItemSourceType,
    },
    review_time::{
        ReviewTelemetryEvaluationRequest, ReviewTimeMetrics, calculate_review_time_metrics,
    },
    usage::{
        CostReconciliationStatus, MeteringUnit, QuotaEnforcement, RemoteUsageReportV1,
        ReservationStatus, UsageActorRef, UsageAuditEventV1, UsageEvaluation,
        UsageEvaluationRequest, UsageEventPayload, UsageLedgerEntry, UsageMeasurements,
        UsageReservation, UsageResourceRef, UsageSecuritySignal, UsageSecuritySignalCode,
        UsageSourceSurface, UsageSubjectRef,
    },
};

#[derive(Clone, Default)]
pub struct AppState {
    store: Arc<Mutex<Store>>,
    policy_evaluator: Arc<BaselinePolicyEvaluator>,
}

#[derive(Debug, Default)]
struct Store {
    working_groups: Vec<WorkingGroup>,
    issues: Vec<Issue>,
    comments: Vec<Comment>,
    registry_entries: Vec<RegistryEntry>,
    memberships: Vec<WorkingGroupMembership>,
    role_bindings: Vec<RoleBinding>,
    delegated_authorities: Vec<DelegatedAuthority>,
    policy_decisions: Vec<PolicyDecision>,
    audit_events: Vec<AuditEvent>,
    usage_events: Vec<UsageAuditEventV1>,
    usage_ledger_entries: Vec<UsageLedgerEntry>,
    usage_reservations: Vec<UsageReservation>,
    usage_security_signals: Vec<UsageSecuritySignal>,
    remote_usage_reports: Vec<RemoteUsageReportV1>,
    imported_agent_result_evidence: Vec<ImportedAgentResultEvidence>,
    review_packet_refs: Vec<ReviewPacketRef>,
    review_control_work_items: Vec<ReviewControlWorkItem>,
    run_timeline_events: Vec<RunTimelineEventV1>,
    operations_audit_events: Vec<OperationsAuditEventV1>,
    operations_health_events: Vec<OperationsHealthEventV1>,
}

#[derive(Debug, Clone)]
struct ReviewPacketRef {
    work_item_id: IssueId,
    evidence_id: String,
    timeline_event_id: String,
    audit_event_id: String,
}

impl Store {
    fn authorization_context(&self) -> AuthorizationContext {
        AuthorizationContext {
            memberships: self.memberships.clone(),
            role_bindings: self.role_bindings.clone(),
            delegated_authorities: self.delegated_authorities.clone(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorEnvelope,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message_key: String,
    pub severity: ErrorSeverity,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_errors: Vec<FieldError>,
    pub support: ErrorSupportMetadata,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ValidationFailed,
    Conflict,
    NotFound,
    InternalError,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Error,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FieldError {
    pub field: String,
    pub code: FieldErrorCode,
    pub message_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldErrorCode {
    Required,
    Invalid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorSupportMetadata {
    pub redacted: bool,
}

pub struct SafeJson<T>(T);

impl<S, T> FromRequest<S> for SafeJson<T>
where
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|_| ApiError::invalid_payload())?;
        Ok(Self(value))
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi))
        .route("/v1/working-groups", post(create_working_group))
        .route("/v1/issues", post(create_issue))
        .route("/v1/comments", post(create_comment))
        .route("/v1/registry", post(create_registry_entry))
        .route("/v1/policy/decisions", post(evaluate_policy))
        .route("/v1/usage/evaluate", post(evaluate_usage))
        .route("/v1/usage/events", post(create_usage_event))
        .route("/v1/remote/usage-reports", post(create_remote_usage_report))
        // Internal-only prototype evaluator. It is intentionally omitted from
        // ApiDoc/canonical OpenAPI until the review-time contract graduates.
        .route("/v1/review-time/evaluate", post(evaluate_review_time))
        .route("/v1/agent-result-imports", post(import_agent_result))
        .route("/v1/review-packets/{work_item_id}", get(get_review_packet))
        .route(
            "/v1/review-control/work-items",
            post(create_review_control_work_item),
        )
        .route(
            "/v1/review-control/work-items/{work_item_id}",
            get(get_review_control_work_item).patch(update_review_control_work_item),
        )
        .route(
            "/v1/review-control/work-items/{work_item_id}/approve-plan",
            post(approve_review_control_plan),
        )
        .route(
            "/v1/review-control/work-items/{work_item_id}/done",
            post(mark_review_control_done),
        )
        .route(
            "/v1/review-control/work-items/{work_item_id}/rework",
            post(request_review_control_rework),
        )
        .route("/v1/audit/events", post(create_audit_event))
        .route(
            "/v1/operations/timeline/events",
            post(create_run_timeline_event),
        )
        .route(
            "/v1/operations/audit/events",
            post(create_operations_audit_event),
        )
        .route(
            "/v1/operations/health/events",
            post(create_operations_health_event),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Control plane is healthy", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[utoipa::path(
    post,
    path = "/v1/working-groups",
    request_body = CreateWorkingGroupRequest,
    responses((status = 201, body = WorkingGroup), (status = 400, body = ErrorResponse))
)]
async fn create_working_group(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<CreateWorkingGroupRequest>,
) -> Result<(StatusCode, Json<WorkingGroup>), ApiError> {
    require_non_empty("slug", &request.slug)?;
    require_non_empty("name", &request.name)?;

    let working_group = WorkingGroup {
        id: WorkingGroupId::new(),
        slug: request.slug,
        name: request.name,
    };

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .working_groups
        .push(working_group.clone());

    Ok((StatusCode::CREATED, Json(working_group)))
}

#[utoipa::path(
    post,
    path = "/v1/issues",
    request_body = CreateIssueRequest,
    responses((status = 201, body = Issue), (status = 400, body = ErrorResponse))
)]
async fn create_issue(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<CreateIssueRequest>,
) -> Result<(StatusCode, Json<Issue>), ApiError> {
    require_non_empty("title", &request.title)?;

    let issue = Issue {
        id: IssueId::new(),
        working_group_id: request.working_group_id,
        title: request.title,
        status: IssueStatus::Todo,
    };

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .issues
        .push(issue.clone());

    Ok((StatusCode::CREATED, Json(issue)))
}

#[utoipa::path(
    post,
    path = "/v1/comments",
    request_body = CreateCommentRequest,
    responses((status = 201, body = Comment), (status = 400, body = ErrorResponse))
)]
async fn create_comment(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<CreateCommentRequest>,
) -> Result<(StatusCode, Json<Comment>), ApiError> {
    require_non_empty("body", &request.body)?;

    let comment = Comment {
        id: CommentId::new(),
        issue_id: request.issue_id,
        body: request.body,
    };

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .comments
        .push(comment.clone());

    Ok((StatusCode::CREATED, Json(comment)))
}

#[utoipa::path(
    post,
    path = "/v1/registry",
    request_body = CreateRegistryEntryRequest,
    responses((status = 201, body = RegistryEntry), (status = 400, body = ErrorResponse))
)]
async fn create_registry_entry(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<CreateRegistryEntryRequest>,
) -> Result<(StatusCode, Json<RegistryEntry>), ApiError> {
    require_non_empty("display_name", &request.display_name)?;

    let entry = RegistryEntry {
        kind: request.kind,
        id: Uuid::new_v4(),
        working_group_id: request.working_group_id,
        display_name: request.display_name,
        enabled: request.enabled,
    };

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .registry_entries
        .push(entry.clone());

    Ok((StatusCode::CREATED, Json(entry)))
}

#[utoipa::path(
    post,
    path = "/v1/policy/decisions",
    request_body = PolicyDecisionRequest,
    responses((status = 200, body = PolicyDecision))
)]
async fn evaluate_policy(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<PolicyDecisionRequest>,
) -> Result<Json<PolicyDecision>, ApiError> {
    let mut store = state.store.lock().map_err(|_| ApiError::internal())?;
    let context = store.authorization_context();
    let decision = state.policy_evaluator.evaluate(&request, &context);
    store.policy_decisions.push(decision.clone());
    Ok(Json(decision))
}

#[utoipa::path(
    post,
    path = "/v1/usage/evaluate",
    request_body = UsageEvaluationRequest,
    responses((status = 200, body = UsageEvaluation))
)]
async fn evaluate_usage(
    SafeJson(request): SafeJson<UsageEvaluationRequest>,
) -> Json<UsageEvaluation> {
    Json(request.policy_set.evaluate(&request.snapshot))
}

#[utoipa::path(
    post,
    path = "/v1/usage/events",
    request_body = UsageAuditEventV1,
    responses(
        (status = 202, body = UsageAuditEventV1),
        (status = 400, body = ErrorResponse),
        (status = 409, body = ErrorResponse)
    )
)]
async fn create_usage_event(
    State(state): State<AppState>,
    SafeJson(event): SafeJson<UsageAuditEventV1>,
) -> Result<(StatusCode, Json<UsageAuditEventV1>), ApiError> {
    let ledger_entry = event
        .to_ledger_entry()
        .map_err(|error| ApiError::invalid_payload_with_reason(&error.to_string()))?;

    let mut store = state.store.lock().map_err(|_| ApiError::internal())?;
    if let Some(existing) = store
        .usage_events
        .iter()
        .find(|stored| stored.idempotency_key == event.idempotency_key)
        .cloned()
    {
        if !existing.has_same_billing_fingerprint(&event) {
            store.usage_security_signals.push(UsageSecuritySignal {
                signal_id: Uuid::new_v4(),
                idempotency_key: event.idempotency_key.clone(),
                existing_event_id: existing.id,
                rejected_event_id: event.id,
                reason_code: UsageSecuritySignalCode::IdempotencyPayloadMismatch,
            });
            return Err(ApiError::conflict());
        }
        return Ok((StatusCode::ACCEPTED, Json(existing)));
    }

    if let Some(reservation_id) = &event.reservation_id {
        store.usage_reservations.push(UsageReservation::open(
            reservation_id.clone(),
            event.working_group_id.clone(),
            event.policy_decision_id.clone(),
            event.idempotency_key.clone(),
            ledger_entry.metered_quantity,
            ledger_entry.metering_unit.clone(),
            ledger_entry.estimated_cost_micros,
        ));
    }
    store.usage_ledger_entries.push(ledger_entry);
    store.usage_events.push(event.clone());

    Ok((StatusCode::ACCEPTED, Json(event)))
}

#[utoipa::path(
    post,
    path = "/v1/remote/usage-reports",
    request_body = RemoteUsageReportV1,
    responses((status = 202, body = RemoteUsageReportV1), (status = 400, body = ErrorResponse))
)]
async fn create_remote_usage_report(
    State(state): State<AppState>,
    SafeJson(report): SafeJson<RemoteUsageReportV1>,
) -> Result<(StatusCode, Json<RemoteUsageReportV1>), ApiError> {
    require_schema_version("remote_usage_report.v1", &report.schema_version)?;
    require_non_empty("job_id", &report.job_id)?;
    require_non_empty("runner_id", &report.runner_id)?;

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .remote_usage_reports
        .push(report.clone());

    Ok((StatusCode::ACCEPTED, Json(report)))
}

#[utoipa::path(
    post,
    path = "/v1/review-time/evaluate",
    request_body = ReviewTelemetryEvaluationRequest,
    responses((status = 200, body = ReviewTimeMetrics), (status = 400, body = ErrorResponse))
)]
async fn evaluate_review_time(
    SafeJson(request): SafeJson<ReviewTelemetryEvaluationRequest>,
) -> Result<Json<ReviewTimeMetrics>, ApiError> {
    let metrics = calculate_review_time_metrics(&request)
        .map_err(|error| ApiError::invalid_payload_with_reason(&error.to_string()))?;
    Ok(Json(metrics))
}

#[utoipa::path(
    post,
    path = "/v1/agent-result-imports",
    request_body = ImportAgentResultRequest,
    responses((status = 202, body = ReviewPacketEvidence), (status = 400, body = ErrorResponse))
)]
async fn import_agent_result(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<ImportAgentResultRequest>,
) -> Result<(StatusCode, Json<ReviewPacketEvidence>), ApiError> {
    let (evidence, timeline_event_id, audit_event_id) = request
        .into_evidence()
        .map_err(|error| ApiError::invalid_payload_with_reason(&error.to_string()))?;
    let packet = evidence.to_review_packet(timeline_event_id.clone(), audit_event_id.clone());
    let timeline_event =
        imported_result_timeline_event(timeline_event_id.clone(), evidence.work_item_id);
    timeline_event
        .validate_for_ingestion()
        .map_err(|_| ApiError::internal())?;
    let audit_event = imported_result_audit_event(
        audit_event_id.clone(),
        evidence.work_item_id,
        evidence.evidence_id.clone(),
    )?;
    let review_packet_ref = ReviewPacketRef {
        work_item_id: evidence.work_item_id,
        evidence_id: evidence.evidence_id.clone(),
        timeline_event_id,
        audit_event_id,
    };

    let mut store = state.store.lock().map_err(|_| ApiError::internal())?;
    store.run_timeline_events.push(timeline_event);
    store.audit_events.push(audit_event);
    store.review_packet_refs.push(review_packet_ref);
    store.imported_agent_result_evidence.push(evidence);

    Ok((StatusCode::ACCEPTED, Json(packet)))
}

#[utoipa::path(
    get,
    path = "/v1/review-packets/{work_item_id}",
    params(("work_item_id" = Uuid, Path, description = "Prototype work item identifier")),
    responses((status = 200, body = ReviewPacketEvidence), (status = 404, body = ErrorResponse))
)]
async fn get_review_packet(
    State(state): State<AppState>,
    Path(work_item_id): Path<IssueId>,
) -> Result<Json<ReviewPacketEvidence>, ApiError> {
    let store = state.store.lock().map_err(|_| ApiError::internal())?;
    let packet_ref = store
        .review_packet_refs
        .iter()
        .rev()
        .find(|packet_ref| packet_ref.work_item_id == work_item_id)
        .ok_or_else(ApiError::not_found)?;
    let evidence = store
        .imported_agent_result_evidence
        .iter()
        .find(|evidence| evidence.evidence_id == packet_ref.evidence_id)
        .ok_or_else(ApiError::not_found)?;

    Ok(Json(evidence.to_review_packet(
        packet_ref.timeline_event_id.clone(),
        packet_ref.audit_event_id.clone(),
    )))
}

#[utoipa::path(
    post,
    path = "/v1/review-control/work-items",
    request_body = CreateReviewControlWorkItemRequest,
    responses((status = 201, body = ReviewControlWorkItem), (status = 400, body = ErrorResponse))
)]
async fn create_review_control_work_item(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<CreateReviewControlWorkItemRequest>,
) -> Result<(StatusCode, Json<ReviewControlWorkItem>), ApiError> {
    let work_item = request.into_work_item().map_err(ApiError::from)?;
    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .review_control_work_items
        .push(work_item.clone());

    Ok((StatusCode::CREATED, Json(work_item)))
}

#[utoipa::path(
    get,
    path = "/v1/review-control/work-items/{work_item_id}",
    params(("work_item_id" = Uuid, Path, description = "Review control work item identifier")),
    responses((status = 200, body = ReviewControlWorkItem), (status = 404, body = ErrorResponse))
)]
async fn get_review_control_work_item(
    State(state): State<AppState>,
    Path(work_item_id): Path<IssueId>,
) -> Result<Json<ReviewControlWorkItem>, ApiError> {
    let store = state.store.lock().map_err(|_| ApiError::internal())?;
    let work_item = find_review_control_work_item(&store, work_item_id)?;
    Ok(Json(work_item.clone()))
}

#[utoipa::path(
    patch,
    path = "/v1/review-control/work-items/{work_item_id}",
    params(("work_item_id" = Uuid, Path, description = "Review control work item identifier")),
    request_body = UpdateReviewControlWorkItemRequest,
    responses((status = 200, body = ReviewControlWorkItem), (status = 400, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 404, body = ErrorResponse))
)]
async fn update_review_control_work_item(
    State(state): State<AppState>,
    Path(work_item_id): Path<IssueId>,
    SafeJson(request): SafeJson<UpdateReviewControlWorkItemRequest>,
) -> Result<Json<ReviewControlWorkItem>, ApiError> {
    mutate_review_control_work_item(state, work_item_id, |work_item| {
        work_item.apply_update(request)
    })
}

#[utoipa::path(
    post,
    path = "/v1/review-control/work-items/{work_item_id}/approve-plan",
    params(("work_item_id" = Uuid, Path, description = "Review control work item identifier")),
    request_body = ApproveReviewControlPlanRequest,
    responses((status = 200, body = ReviewControlWorkItem), (status = 400, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 404, body = ErrorResponse))
)]
async fn approve_review_control_plan(
    State(state): State<AppState>,
    Path(work_item_id): Path<IssueId>,
    SafeJson(request): SafeJson<ApproveReviewControlPlanRequest>,
) -> Result<Json<ReviewControlWorkItem>, ApiError> {
    mutate_review_control_work_item(state, work_item_id, |work_item| {
        work_item.approve_plan(request)
    })
}

#[utoipa::path(
    post,
    path = "/v1/review-control/work-items/{work_item_id}/done",
    params(("work_item_id" = Uuid, Path, description = "Review control work item identifier")),
    request_body = CompleteReviewControlRequest,
    responses((status = 200, body = ReviewControlWorkItem), (status = 400, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 404, body = ErrorResponse))
)]
async fn mark_review_control_done(
    State(state): State<AppState>,
    Path(work_item_id): Path<IssueId>,
    SafeJson(request): SafeJson<CompleteReviewControlRequest>,
) -> Result<Json<ReviewControlWorkItem>, ApiError> {
    mutate_review_control_work_item(state, work_item_id, |work_item| {
        work_item.mark_done(request)
    })
}

#[utoipa::path(
    post,
    path = "/v1/review-control/work-items/{work_item_id}/rework",
    params(("work_item_id" = Uuid, Path, description = "Review control work item identifier")),
    request_body = CompleteReviewControlRequest,
    responses((status = 200, body = ReviewControlWorkItem), (status = 400, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 404, body = ErrorResponse))
)]
async fn request_review_control_rework(
    State(state): State<AppState>,
    Path(work_item_id): Path<IssueId>,
    SafeJson(request): SafeJson<CompleteReviewControlRequest>,
) -> Result<Json<ReviewControlWorkItem>, ApiError> {
    mutate_review_control_work_item(state, work_item_id, |work_item| {
        work_item.request_rework(request)
    })
}

fn find_review_control_work_item(
    store: &Store,
    work_item_id: IssueId,
) -> Result<&ReviewControlWorkItem, ApiError> {
    store
        .review_control_work_items
        .iter()
        .find(|work_item| work_item.id == work_item_id)
        .ok_or_else(ApiError::not_found)
}

fn mutate_review_control_work_item(
    state: AppState,
    work_item_id: IssueId,
    update: impl FnOnce(&mut ReviewControlWorkItem) -> Result<(), ReviewControlError>,
) -> Result<Json<ReviewControlWorkItem>, ApiError> {
    let mut store = state.store.lock().map_err(|_| ApiError::internal())?;
    let work_item = store
        .review_control_work_items
        .iter_mut()
        .find(|work_item| work_item.id == work_item_id)
        .ok_or_else(ApiError::not_found)?;
    update(work_item).map_err(ApiError::from)?;
    Ok(Json(work_item.clone()))
}

fn imported_result_timeline_event(event_id: String, work_item_id: IssueId) -> RunTimelineEventV1 {
    RunTimelineEventV1 {
        id: event_id,
        r#type: "operations.timeline.recorded".to_owned(),
        envelope: imported_result_envelope(work_item_id),
        stage: RunTimelineStage::Completed,
        status_reason: Some(RunTimelineStatusReason::Completed),
        usage_link: None,
        health_signal: None,
    }
}

fn imported_result_audit_event(
    event_id: String,
    work_item_id: IssueId,
    evidence_id: String,
) -> Result<AuditEvent, ApiError> {
    Ok(AuditEvent {
        id: AuditEventId(Uuid::parse_str(&event_id).map_err(|_| ApiError::internal())?),
        working_group_id: WorkingGroupId::new(),
        actor: crate::domain::Actor::Agent {
            id: AgentId(Uuid::new_v4()),
        },
        action: "agent_result.imported".to_owned(),
        resource: format!("issue:{}#{}", work_item_id.0, evidence_id),
        outcome: crate::audit::AuditOutcome::Succeeded,
    })
}

fn imported_result_envelope(work_item_id: IssueId) -> OperationsEventEnvelope {
    let short_id = work_item_id.0.to_string();
    OperationsEventEnvelope {
        version: "0.1.0".to_owned(),
        occurred_at: "2026-06-01T00:00:00Z".to_owned(),
        working_group_id: format!("wg_{short_id}"),
        tenant_id: Some(format!("tenant_{short_id}")),
        correlation_id: format!("corr_{short_id}"),
        request_id: format!("req_{short_id}"),
        run_id: None,
        job_id: None,
        source: OperationsSourceSurface::ControlPlane,
        actor: OperationsActorRef {
            r#type: "agent".to_owned(),
            id: format!("agent_{short_id}"),
        },
        resource: OperationsResourceRef {
            r#type: "issue".to_owned(),
            id: format!("issue_{short_id}"),
        },
        redaction: RedactionClassification::RedactedSummary,
    }
}

#[utoipa::path(
    post,
    path = "/v1/audit/events",
    request_body = CreateAuditEventRequest,
    responses((status = 201, body = AuditEvent), (status = 400, body = ErrorResponse))
)]
async fn create_audit_event(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<CreateAuditEventRequest>,
) -> Result<(StatusCode, Json<AuditEvent>), ApiError> {
    require_non_empty("action", &request.action)?;
    require_non_empty("resource", &request.resource)?;

    let event = AuditEvent {
        id: AuditEventId::new(),
        working_group_id: request.working_group_id,
        actor: request.actor,
        action: request.action,
        resource: request.resource,
        outcome: request.outcome,
    };

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .audit_events
        .push(event.clone());

    Ok((StatusCode::CREATED, Json(event)))
}

#[utoipa::path(
    post,
    path = "/v1/operations/timeline/events",
    request_body = RunTimelineEventV1,
    responses((status = 202, body = RunTimelineEventV1), (status = 400, body = ErrorResponse))
)]
async fn create_run_timeline_event(
    State(state): State<AppState>,
    SafeJson(event): SafeJson<RunTimelineEventV1>,
) -> Result<(StatusCode, Json<RunTimelineEventV1>), ApiError> {
    event
        .validate_for_ingestion()
        .map_err(|error| ApiError::invalid_payload_with_reason(&error.to_string()))?;

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .run_timeline_events
        .push(event.clone());

    Ok((StatusCode::ACCEPTED, Json(event)))
}

#[utoipa::path(
    post,
    path = "/v1/operations/audit/events",
    request_body = OperationsAuditEventV1,
    responses((status = 202, body = OperationsAuditEventV1), (status = 400, body = ErrorResponse))
)]
async fn create_operations_audit_event(
    State(state): State<AppState>,
    SafeJson(event): SafeJson<OperationsAuditEventV1>,
) -> Result<(StatusCode, Json<OperationsAuditEventV1>), ApiError> {
    event
        .validate_for_ingestion()
        .map_err(|error| ApiError::invalid_payload_with_reason(&error.to_string()))?;

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .operations_audit_events
        .push(event.clone());

    Ok((StatusCode::ACCEPTED, Json(event)))
}

#[utoipa::path(
    post,
    path = "/v1/operations/health/events",
    request_body = OperationsHealthEventV1,
    responses((status = 202, body = OperationsHealthEventV1), (status = 400, body = ErrorResponse))
)]
async fn create_operations_health_event(
    State(state): State<AppState>,
    SafeJson(event): SafeJson<OperationsHealthEventV1>,
) -> Result<(StatusCode, Json<OperationsHealthEventV1>), ApiError> {
    event
        .validate_for_ingestion()
        .map_err(|error| ApiError::invalid_payload_with_reason(&error.to_string()))?;

    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .operations_health_events
        .push(event.clone());

    Ok((StatusCode::ACCEPTED, Json(event)))
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::required_field(field));
    }

    Ok(())
}

fn require_schema_version(expected: &'static str, actual: &str) -> Result<(), ApiError> {
    if actual != expected {
        return Err(ApiError::invalid_field(
            "schema_version",
            &format!("expected {expected}, received {actual}"),
        ));
    }

    Ok(())
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ErrorEnvelope,
}

impl ApiError {
    fn required_field(field: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ErrorEnvelope {
                code: ErrorCode::ValidationFailed,
                message_key: "errors.validation_failed".to_owned(),
                severity: ErrorSeverity::Error,
                retryable: false,
                field_errors: vec![FieldError {
                    field: field.to_owned(),
                    code: FieldErrorCode::Required,
                    message_key: "errors.field.required".to_owned(),
                }],
                support: ErrorSupportMetadata { redacted: true },
            },
        }
    }

    fn invalid_field(field: &'static str, _safe_reason: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ErrorEnvelope {
                code: ErrorCode::ValidationFailed,
                message_key: "errors.validation_failed".to_owned(),
                severity: ErrorSeverity::Error,
                retryable: false,
                field_errors: vec![FieldError {
                    field: field.to_owned(),
                    code: FieldErrorCode::Invalid,
                    message_key: "errors.field.invalid".to_owned(),
                }],
                support: ErrorSupportMetadata { redacted: true },
            },
        }
    }

    fn invalid_payload() -> Self {
        Self::invalid_field("body", "invalid payload")
    }

    fn invalid_payload_with_reason(_safe_reason: &str) -> Self {
        Self::invalid_payload()
    }

    fn conflict() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorEnvelope {
                code: ErrorCode::Conflict,
                message_key: "errors.conflict".to_owned(),
                severity: ErrorSeverity::Error,
                retryable: false,
                field_errors: Vec::new(),
                support: ErrorSupportMetadata { redacted: true },
            },
        }
    }

    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ErrorEnvelope {
                code: ErrorCode::NotFound,
                message_key: "errors.not_found".to_owned(),
                severity: ErrorSeverity::Error,
                retryable: false,
                field_errors: Vec::new(),
                support: ErrorSupportMetadata { redacted: true },
            },
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorEnvelope {
                code: ErrorCode::InternalError,
                message_key: "errors.internal".to_owned(),
                severity: ErrorSeverity::Error,
                retryable: true,
                field_errors: Vec::new(),
                support: ErrorSupportMetadata { redacted: true },
            },
        }
    }
}

impl From<ReviewControlError> for ApiError {
    fn from(error: ReviewControlError) -> Self {
        match error {
            ReviewControlError::InvalidTransition => Self::conflict(),
            ReviewControlError::Required(_)
            | ReviewControlError::UnsupportedSchemaVersion
            | ReviewControlError::UnsafeReference { .. }
            | ReviewControlError::MissingAcceptanceCriteria
            | ReviewControlError::ApprovalRequiredInvariantViolation => Self::invalid_payload(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorResponse { error: self.body })).into_response()
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        create_working_group,
        create_issue,
        create_comment,
        create_registry_entry,
        evaluate_policy,
        evaluate_usage,
        create_usage_event,
        create_remote_usage_report,
        import_agent_result,
        get_review_packet,
        create_review_control_work_item,
        get_review_control_work_item,
        update_review_control_work_item,
        approve_review_control_plan,
        mark_review_control_done,
        request_review_control_rework,
        create_audit_event,
        create_run_timeline_event,
        create_operations_audit_event,
        create_operations_health_event
    ),
    components(schemas(
        AcceptanceCriterion,
        ApproveReviewControlPlanRequest,
        ArtifactEvidence,
        AuditCorrelation,
        AutonomyLevel,
        AuditEvent,
        ChangedFileEvidence,
        ChangedFileKind,
        CommandEvidence,
        CommandOutcome,
        CompleteReviewControlRequest,
        CreateAuditEventRequest,
        Comment,
        CreateCommentRequest,
        CreateIssueRequest,
        CreateRegistryEntryRequest,
        CreateReviewControlWorkItemRequest,
        CreateWorkingGroupRequest,
        ErrorCode,
        ErrorEnvelope,
        ErrorSeverity,
        ErrorResponse,
        ErrorSupportMetadata,
        FieldError,
        FieldErrorCode,
        HealthResponse,
        Issue,
        ImportAgentResultRequest,
        ImportedAgentResultEvidence,
        HealthAvailability,
        HealthDegradedReason,
        HealthSignalRef,
        HealthTargetKind,
        HealthTargetRef,
        OperationsActorRef,
        OperationsAuditAction,
        OperationsAuditEventV1,
        OperationsAuditEvidence,
        OperationsAuditOutcome,
        OperationsEventEnvelope,
        OperationsHealthEventV1,
        OperationsResourceRef,
        OperationsSourceSurface,
        PlanApproval,
        PlanApprovalState,
        PolicyDecision,
        PolicyDecisionRequest,
        PolicyActorRef,
        RedactionClassification,
        RedactionSummary,
        RegistryEntry,
        ReviewControlRedactionSummary,
        ReviewControlState,
        ReviewControlWorkItem,
        ReviewDecision,
        ReviewDecisionReasonCode,
        ReviewDecisionState,
        ReviewPacketEvidence,
        RoleBinding,
        RemoteUsageReportV1,
        RiskTier,
        RunTimelineEventV1,
        RunTimelineStage,
        RunTimelineStatusReason,
        CostReconciliationStatus,
        MeteringUnit,
        QuotaEnforcement,
        ReservationStatus,
        UsageActorRef,
        UsageAuditEventV1,
        UsageEvaluation,
        UsageEvaluationRequest,
        UsageEventPayload,
        UsageLedgerEntry,
        UsageMeasurements,
        UsageReservation,
        UsageContractLink,
        UsageResourceRef,
        UsageSecuritySignal,
        UsageSecuritySignalCode,
        UsageSourceSurface,
        UsageSubjectRef,
        UpdateReviewControlWorkItemRequest,
        WorkItemRequestContext,
        WorkItemSource,
        WorkItemSourceType,
        WorkingGroup,
        WorkingGroupMembership,
        WorkingGroupRole
    )),
    tags(
        (name = "control-plane", description = "TaskOtter MVP control plane API")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    fn add_membership(
        state: &AppState,
        working_group_id: &str,
        actor_type: &str,
        actor_id: &str,
        role: WorkingGroupRole,
    ) -> Result<(), Box<dyn std::error::Error>> {
        state
            .store
            .lock()
            .map_err(|_| "store lock failed")?
            .memberships
            .push(WorkingGroupMembership {
                working_group_id: working_group_id.to_owned(),
                actor: PolicyActorRef {
                    r#type: actor_type.to_owned(),
                    id: actor_id.to_owned(),
                },
                role,
            });
        Ok(())
    }

    fn add_delegated_authority(
        state: &AppState,
        working_group_id: &str,
        delegated_by: PolicyActorRef,
        delegated_to: PolicyActorRef,
        run_id: &str,
        actions: Vec<&str>,
        resource_ids: Vec<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        state
            .store
            .lock()
            .map_err(|_| "store lock failed")?
            .delegated_authorities
            .push(DelegatedAuthority {
                working_group_id: working_group_id.to_owned(),
                delegated_by,
                delegated_to,
                run_id: run_id.to_owned(),
                actions: actions.into_iter().map(str::to_owned).collect(),
                resource_ids: resource_ids.into_iter().map(str::to_owned).collect(),
            });
        Ok(())
    }

    fn policy_request(
        actor_id: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        working_group_id: &str,
    ) -> Value {
        json!({
            "working_group_id": working_group_id,
            "actor": {
                "type": "user",
                "id": actor_id
            },
            "action": action,
            "resource": {
                "type": resource_type,
                "id": resource_id,
                "working_group_id": working_group_id
            },
            "correlation_id": "corr_1",
            "request_id": "req_1",
            "policy_version": "0.1.0",
            "policy_snapshot_id": "polsnap_1"
        })
    }

    fn policy_http_request(request: Value) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        Ok(Request::builder()
            .method("POST")
            .uri("/v1/policy/decisions")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(request.to_string()))?)
    }

    fn review_control_create_request() -> Value {
        json!({
            "schema_version": "review_control.work_item.v1",
            "request": {
                "source": {
                    "source_type": "multica_issue",
                    "source_ref": "issue_BOG_571"
                },
                "summary": "Implement review control contract."
            },
            "acceptance_criteria": [
                {
                    "id": "ac_create",
                    "text": "Create and transitions are contract tested.",
                    "required": true
                }
            ],
            "risk_tier": "high",
            "autonomy_level": "human_approval_required",
            "audit": {
                "correlation_id": "corr_BOG_571",
                "request_id": "req_BOG_571"
            }
        })
    }

    fn json_http_request(
        method: &str,
        uri: &str,
        body: Value,
    ) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        Ok(Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))?)
    }

    #[tokio::test]
    async fn exposes_generated_openapi_contract() -> Result<(), Box<dyn std::error::Error>> {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let document: Value = serde_json::from_slice(&body)?;

        assert!(document["paths"]["/v1/policy/decisions"].is_object());
        assert!(document["paths"]["/v1/authorization/memberships"].is_null());
        assert!(document["paths"]["/v1/authorization/role-bindings"].is_null());
        assert!(document["paths"]["/v1/registry"].is_object());
        assert!(document["paths"]["/v1/usage/evaluate"].is_object());
        assert!(document["paths"]["/v1/usage/events"].is_object());
        assert!(document["paths"]["/v1/remote/usage-reports"].is_object());
        assert!(document["paths"]["/v1/review-time/evaluate"].is_null());
        assert!(document["paths"]["/v1/review-control/work-items"].is_object());
        assert!(document["paths"]["/v1/review-control/work-items/{work_item_id}"].is_object());
        assert!(
            document["paths"]["/v1/review-control/work-items/{work_item_id}/approve-plan"]
                .is_object()
        );
        assert!(document["paths"]["/v1/review-control/work-items/{work_item_id}/done"].is_object());
        assert!(
            document["paths"]["/v1/review-control/work-items/{work_item_id}/rework"].is_object()
        );
        assert!(document["paths"]["/v1/audit/events"].is_object());
        assert!(document["paths"]["/v1/operations/timeline/events"].is_object());
        assert!(document["paths"]["/v1/operations/audit/events"].is_object());
        assert!(document["paths"]["/v1/operations/health/events"].is_object());
        assert!(
            document["components"]["schemas"]["PolicyDecision"]["properties"]["allowed"]
                .is_object()
        );
        assert!(
            document["components"]["schemas"]["UsageAuditEventV1"]["properties"]["status"]
                .is_object()
        );
        assert!(
            document["components"]["schemas"]["RunTimelineEventV1"]["properties"]["stage"]
                .is_object()
        );
        assert!(
            document["components"]["schemas"]["ReviewControlWorkItem"]["properties"]["state"]
                .is_object()
        );
        let error = &document["components"]["schemas"]["ErrorEnvelope"];
        assert!(error["properties"]["code"].is_object());
        assert!(error["properties"]["message_key"].is_object());
        assert!(error["properties"]["field_errors"].is_object());
        assert!(error["properties"]["support"].is_object());
        Ok(())
    }

    #[tokio::test]
    async fn validation_errors_return_stable_localization_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/working-groups")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "slug": "   ",
                            "name": "Platform"
                        })
                        .to_string(),
                    ))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;

        assert_eq!(envelope["error"]["code"], "validation_failed");
        assert_eq!(envelope["error"]["message_key"], "errors.validation_failed");
        assert_eq!(envelope["error"]["severity"], "error");
        assert_eq!(envelope["error"]["retryable"], false);
        assert_eq!(envelope["error"]["field_errors"][0]["field"], "slug");
        assert_eq!(envelope["error"]["field_errors"][0]["code"], "required");
        assert_eq!(
            envelope["error"]["field_errors"][0]["message_key"],
            "errors.field.required"
        );
        assert_eq!(envelope["error"]["support"]["redacted"], true);
        assert!(envelope["error"].get("message").is_none());
        assert!(
            envelope["error"]["field_errors"][0]
                .get("message")
                .is_none()
        );
        Ok(())
    }

    #[tokio::test]
    async fn ingestion_errors_redact_sensitive_payload_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/operations/audit/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "id": "evt_12345678901234567890123456",
                            "type": "operations.audit.recorded",
                            "envelope": {
                                "version": "0.1.0",
                                "occurred_at": "2026-06-01T00:00:00Z",
                                "working_group_id": "wg_12345678901234567890123456",
                                "tenant_id": "tenant_12345678901234567890123456",
                                "correlation_id": "corr_12345678901234567890123456",
                                "request_id": "req_12345678901234567890123456",
                                "source": "control_plane",
                                "actor": { "type": "agent", "id": "agent_12345678901234567890123456" },
                                "resource": { "type": "provider", "id": "secret_token_value" },
                                "redaction": "internal_reference_only"
                            },
                            "action": "credential_access",
                            "outcome": "failed",
                            "evidence": {
                                "evidence_id": "evidence_12345678901234567890123456",
                                "policy_decision_id": "poldec_12345678901234567890123456",
                                "secret_ref": "secret_12345678901234567890123456"
                            }
                        })
                        .to_string(),
                    ))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        let serialized = serde_json::to_string(&envelope)?;

        assert_eq!(envelope["error"]["code"], "validation_failed");
        assert_eq!(envelope["error"]["field_errors"][0]["field"], "body");
        assert_eq!(envelope["error"]["field_errors"][0]["code"], "invalid");
        assert_eq!(envelope["error"]["support"]["redacted"], true);
        assert!(!serialized.contains("secret_token_value"));
        assert!(!serialized.contains("credential_access"));
        assert!(!serialized.contains("poldec_12345678901234567890123456"));
        Ok(())
    }

    async fn post_json_body(uri: &str, body: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_owned()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        Ok(serde_json::from_slice(&body)?)
    }

    fn assert_json_rejection_is_redacted(
        envelope: &Value,
        forbidden_values: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(envelope)?;

        assert_eq!(envelope["error"]["code"], "validation_failed");
        assert_eq!(envelope["error"]["message_key"], "errors.validation_failed");
        assert_eq!(envelope["error"]["field_errors"][0]["field"], "body");
        assert_eq!(envelope["error"]["field_errors"][0]["code"], "invalid");
        assert_eq!(envelope["error"]["support"]["redacted"], true);
        assert!(envelope["error"].get("message").is_none());
        assert!(
            envelope["error"]["field_errors"][0]
                .get("message")
                .is_none()
        );

        for forbidden in forbidden_values {
            assert!(
                !serialized.contains(forbidden),
                "error response leaked forbidden text: {forbidden}"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn json_extractor_rejections_use_redacted_error_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let valid_audit_prefix = r#"{
            "id": "evt_12345678901234567890123456",
            "type": "operations.audit.recorded",
            "envelope": {
                "version": "0.1.0",
                "occurred_at": "2026-06-01T00:00:00Z",
                "working_group_id": "wg_12345678901234567890123456",
                "tenant_id": "tenant_12345678901234567890123456",
                "correlation_id": "corr_12345678901234567890123456",
                "request_id": "req_12345678901234567890123456",
                "source": "secret_token_value",
                "actor": { "type": "agent", "id": "agent_12345678901234567890123456" },
                "resource": { "type": "provider", "id": "provider_12345678901234567890123456" },
                "redaction": "internal_reference_only"
            },
            "action": "credential_access",
            "outcome": "failed",
            "evidence": {
                "evidence_id": "evidence_12345678901234567890123456",
                "policy_decision_id": "poldec_12345678901234567890123456",
                "secret_ref": "secret_12345678901234567890123456"
            }
        }"#;

        let cases = [
            (
                "unknown enum",
                "/v1/operations/audit/events",
                valid_audit_prefix,
                [
                    "secret_token_value",
                    "unknown variant",
                    "OperationsSourceSurface",
                    "serde",
                    "Json",
                    "line",
                    "column",
                    "stack",
                    "backtrace",
                ]
                .as_slice(),
            ),
            (
                "invalid type",
                "/v1/working-groups",
                r#"{"slug":{"nested":"secret_token_value"},"name":"Platform"}"#,
                [
                    "secret_token_value",
                    "invalid type",
                    "nested",
                    "serde",
                    "Json",
                    "line",
                    "column",
                    "stack",
                    "backtrace",
                ]
                .as_slice(),
            ),
            (
                "missing field",
                "/v1/working-groups",
                r#"{"slug":"secret_token_value"}"#,
                [
                    "secret_token_value",
                    "missing field",
                    "name",
                    "serde",
                    "Json",
                    "line",
                    "column",
                    "stack",
                    "backtrace",
                ]
                .as_slice(),
            ),
            (
                "malformed JSON",
                "/v1/working-groups",
                r#"{"slug":"secret_token_value","name":"#,
                [
                    "secret_token_value",
                    "EOF",
                    "expected",
                    "serde",
                    "Json",
                    "line",
                    "column",
                    "stack",
                    "backtrace",
                ]
                .as_slice(),
            ),
        ];

        for (case, uri, body, forbidden_values) in cases {
            let envelope = post_json_body(uri, body).await?;
            assert_json_rejection_is_redacted(&envelope, forbidden_values)
                .map_err(|error| format!("{case}: {error}"))?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn policy_api_returns_gateway_compatible_decision()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "agent", "agent_1", WorkingGroupRole::Admin)?;
        let request = json!({
            "subject": {
                "user_id": "user_1",
                "working_group_id": "wg_1",
                "agent_id": "agent_1"
            },
            "provider": {
                "provider_id": "provider_1",
                "kind": "open_ai_compatible",
                "model": "test-model",
                "endpoint_id": "endpoint_1"
            },
            "operation": "ai.relay"
        });

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/policy/decisions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;

        assert_eq!(decision["allowed"], true);
        assert_eq!(decision["decision_id"], "local-policy:ai.relay");
        assert_eq!(decision["max_tokens"], 8_192);
        assert_eq!(decision["max_cost_micro_usd"], 50_000);
        assert_eq!(decision["schema_version"], "policy-decision-scaffold@0.1.0");
        assert_eq!(decision["reason_code"], "role_matched");
        assert_eq!(decision["actor"]["type"], "agent");
        assert_eq!(decision["resource"]["type"], "provider");
        assert_eq!(decision["effect"], "allow");
        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.policy_decisions.len(), 1);
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_does_not_claim_canonical_policy_decision_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "agent", "agent_1", WorkingGroupRole::Admin)?;
        let request = json!({
            "subject": {
                "user_id": "user_1",
                "working_group_id": "wg_1",
                "agent_id": "agent_1"
            },
            "provider": {
                "provider_id": "provider_1",
                "kind": "open_ai_compatible",
                "model": "test-model"
            },
            "operation": "ai.relay",
            "correlation_id": "corr_1",
            "request_id": "req_1",
            "policy_snapshot_id": "polsnap_local"
        });

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["schema_version"], "policy-decision-scaffold@0.1.0");
        assert_ne!(decision["schema_version"], "policy-decision@0.1.0");
        assert_eq!(decision["decision_id"], "local-policy:ai.relay");
        assert_eq!(decision["policy_snapshot_id"], "polsnap_local");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_denies_by_default_without_membership()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = policy_request("usr_missing", "issue.read", "issue", "iss_1", "wg_1");
        let response = build_router(AppState::default())
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "missing_membership");
        assert_eq!(decision["effect"], "deny");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_denies_cross_working_group_resource()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "user", "usr_1", WorkingGroupRole::Owner)?;
        let mut request = policy_request("usr_1", "issue.read", "issue", "iss_1", "wg_1");
        request["resource"]["working_group_id"] = json!("wg_2");

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "cross_working_group_denied");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_denies_insufficient_role() -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "user", "usr_1", WorkingGroupRole::Viewer)?;
        let request = policy_request("usr_1", "comment.create", "comment", "cmt_1", "wg_1");

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "insufficient_role");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_denies_delegated_authority_narrowing()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "agent", "agt_1", WorkingGroupRole::Admin)?;
        let user = PolicyActorRef {
            r#type: "user".to_owned(),
            id: "usr_1".to_owned(),
        };
        let agent = PolicyActorRef {
            r#type: "agent".to_owned(),
            id: "agt_1".to_owned(),
        };
        add_delegated_authority(
            &state,
            "wg_1",
            user.clone(),
            agent,
            "run_1",
            vec!["issue.read"],
            vec!["iss_1"],
        )?;
        let mut request = policy_request("agt_1", "issue.update", "issue", "iss_1", "wg_1");
        request["actor"]["type"] = json!("agent");
        request["run_context"] = json!({
            "run_id": "run_1",
            "workflow_id": "wfrun_1",
            "delegated_by": user
        });

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "delegated_authority_narrowed");
        assert_eq!(decision["run_context"]["run_id"], "run_1");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_denies_forged_delegated_actor_chain()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "agent", "agt_1", WorkingGroupRole::Admin)?;
        let mut request = policy_request("agt_1", "issue.update", "issue", "iss_1", "wg_1");
        request["actor"]["type"] = json!("agent");
        request["delegated_actor_chain"] = json!([{ "type": "user", "id": "usr_1" }]);
        request["run_context"] = json!({
            "run_id": "run_forged",
            "workflow_id": "wfrun_1",
            "delegated_by": { "type": "user", "id": "usr_1" }
        });

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "delegated_authority_untrusted");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_rejects_wrong_run_delegated_authority_grant()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "agent", "agt_1", WorkingGroupRole::Admin)?;
        let user = PolicyActorRef {
            r#type: "user".to_owned(),
            id: "usr_1".to_owned(),
        };
        let agent = PolicyActorRef {
            r#type: "agent".to_owned(),
            id: "agt_1".to_owned(),
        };
        add_delegated_authority(
            &state,
            "wg_1",
            user.clone(),
            agent,
            "run_trusted",
            vec!["issue.update"],
            vec!["iss_1"],
        )?;
        let mut request = policy_request("agt_1", "issue.update", "issue", "iss_1", "wg_1");
        request["actor"]["type"] = json!("agent");
        request["run_context"] = json!({
            "run_id": "run_forged",
            "delegated_by": user
        });

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "delegated_authority_untrusted");
        Ok(())
    }

    #[tokio::test]
    async fn authz_mutation_endpoints_are_not_publicly_routed()
    -> Result<(), Box<dyn std::error::Error>> {
        let self_grant = json!({
            "working_group_id": "wg_1",
            "actor": { "type": "user", "id": "usr_attacker" },
            "role": "owner"
        });

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/authorization/memberships")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(self_grant.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let role_binding_self_grant = json!({
            "working_group_id": "wg_1",
            "actor": { "type": "user", "id": "usr_attacker" },
            "actions": ["runner.job.execute"]
        });
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/authorization/role-bindings")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(role_binding_self_grant.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_denies_object_resource_without_working_group_scope()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "user", "usr_1", WorkingGroupRole::Owner)?;
        let mut request = policy_request("usr_1", "issue.read", "issue", "iss_1", "wg_1");
        if let Some(resource) = request["resource"].as_object_mut() {
            resource.remove("working_group_id");
        }

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "missing_resource_scope");
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_classifies_high_risk_resource_action()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        add_membership(&state, "wg_1", "user", "usr_1", WorkingGroupRole::Member)?;
        let request = policy_request(
            "usr_1",
            "gateway.mcp.session.open",
            "mcp_server",
            "mcp_1",
            "wg_1",
        );

        let response = build_router(state)
            .oneshot(policy_http_request(request)?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;
        assert_eq!(decision["allowed"], false);
        assert_eq!(decision["reason_code"], "protected_resource_action");
        assert_eq!(
            decision["constraints"]["high_risk_capabilities"][0]["effect"],
            "deny"
        );
        Ok(())
    }

    #[tokio::test]
    async fn usage_api_preserves_and_semantics() -> Result<(), Box<dyn std::error::Error>> {
        let request = json!({
            "snapshot": {
                "working_group_id": "018f30d5-9471-7c4c-85c4-0e14c3f76c01",
                "monthly_cost_cents": 1200,
                "daily_tokens": 1000,
                "hourly_actions": 50
            },
            "policy_set": {
                "limits": [
                    {
                        "name": "monthly-cost",
                        "max_monthly_cost_cents": 1500,
                        "max_daily_tokens": null,
                        "max_hourly_actions": null
                    },
                    {
                        "name": "hourly-actions",
                        "max_monthly_cost_cents": null,
                        "max_daily_tokens": null,
                        "max_hourly_actions": 10
                    }
                ]
            }
        });

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/evaluate")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let evaluation: Value = serde_json::from_slice(&body)?;

        assert_eq!(evaluation["allowed"], false);
        assert_eq!(evaluation["enforcement"], "hard_deny");
        assert_eq!(evaluation["denial_reason"], "usage_limit_reached");
        assert_eq!(evaluation["failed_limits"], json!(["hourly-actions"]));
        Ok(())
    }

    #[tokio::test]
    async fn ingests_gateway_usage_event_fixture_idempotently()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let request: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let event: Value = serde_json::from_slice(&body)?;

        assert_eq!(event["version"], "0.1.0");
        assert_eq!(event["idempotency_key"], request["idempotency_key"]);
        assert_eq!(event["status"], "succeeded");

        let duplicate = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;
        assert_eq!(duplicate.status(), StatusCode::ACCEPTED);

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.usage_events.len(), 1);
        assert_eq!(store.usage_ledger_entries.len(), 1);
        assert_eq!(store.usage_ledger_entries[0].metered_quantity, 1);
        assert_eq!(
            store.usage_ledger_entries[0].cost_reconciliation_status,
            CostReconciliationStatus::PendingProviderReconciliation
        );
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn idempotency_payload_mismatch_conflicts_and_records_security_signal()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let request: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let mut mismatched = request;
        mismatched["id"] = json!("evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ");
        mismatched["policy_decision_id"] = json!("poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ");
        mismatched["payload"]["subject"]["id"] = json!("gwreq_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ");
        mismatched["payload"]["measurements"]["estimated_cost_micros"] = json!(9900);

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(mismatched.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.usage_events.len(), 1);
        assert_eq!(store.usage_ledger_entries.len(), 1);
        assert_eq!(store.usage_security_signals.len(), 1);
        assert_eq!(
            store.usage_security_signals[0].reason_code,
            UsageSecuritySignalCode::IdempotencyPayloadMismatch
        );
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_denied_usage_without_reason_and_sensitive_raw_payloads()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut denied: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.high-risk-runtime-denied.json"
        ))?;
        denied["status"] = json!("denied");
        denied["denial_reason"] = Value::Null;

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(denied.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let mut raw_payload: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;
        raw_payload["payload"]["raw_prompt"] = json!("do not store me");

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(raw_payload.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        assert_json_rejection_is_redacted(
            &envelope,
            &[
                "raw_prompt",
                "do not store me",
                "unknown field",
                "serde",
                "Json",
            ],
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn rejects_sensitive_denial_reason_and_reference_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut denied: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.high-risk-runtime-denied.json"
        ))?;
        denied["status"] = json!("denied");
        denied["denial_reason"] = json!("raw_prompt: customer entered bearer token");

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(denied.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let mut bad_ref: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;
        bad_ref["resource"]["id"] =
            json!("prv_access_token_customer_sensitive_artifact_body_value_that_should_not_store");

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(bad_ref.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let mut bad_runtime: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.high-risk-runtime-denied.json"
        ))?;
        bad_runtime["payload"]["measurements"]["runtime_capability"] =
            json!("gateway.raw_prompt_billing");

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(bad_runtime.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_usage_event_missing_attribution() -> Result<(), Box<dyn std::error::Error>> {
        let mut request: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;
        request["actor"]["id"] = json!("");

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[tokio::test]
    async fn usage_ledger_records_actual_cost_reconciliation()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let mut request: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.gateway-request.json"
        ))?;
        request["payload"]["measurements"]["actual_cost_micros"] = json!(2400);
        request["payload"]["measurements"]["metering_unit"] = json!("token");

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usage/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.usage_ledger_entries.len(), 1);
        assert_eq!(store.usage_ledger_entries[0].estimated_cost_micros, 2_300);
        assert_eq!(
            store.usage_ledger_entries[0].actual_cost_micros,
            Some(2_400)
        );
        assert_eq!(
            store.usage_ledger_entries[0].cost_reconciliation_status,
            CostReconciliationStatus::Actual
        );
        assert_eq!(store.usage_ledger_entries[0].metered_quantity, 1_520);
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn ingests_remote_usage_report() -> Result<(), Box<dyn std::error::Error>> {
        let request = json!({
            "schema_version": "remote_usage_report.v1",
            "job_id": "job_1",
            "runner_id": "runner_1",
            "request_id": "018f30d5-9471-7c4c-85c4-0e14c3f76c03",
            "correlation_id": "corr_1",
            "status": "succeeded",
            "duration_ms": 10,
            "cpu_time_ms": null,
            "peak_memory_bytes": null,
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "estimated_cost_micro_usd": 0
        });

        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/remote/usage-reports")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let report: Value = serde_json::from_slice(&body)?;

        assert_eq!(report["schema_version"], "remote_usage_report.v1");
        assert_eq!(report["status"], "succeeded");
        Ok(())
    }

    #[tokio::test]
    async fn review_time_api_evaluates_prototype_fixture() -> Result<(), Box<dyn std::error::Error>>
    {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/review-time/evaluate")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(include_str!(
                        "../../../contracts/fixtures/review-time-telemetry.prototype.json"
                    )))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let metrics: Value = serde_json::from_slice(&body)?;
        assert_eq!(metrics["completed_agent_tasks"], 1);
        assert_eq!(metrics["human_review_minutes"], 11);
        assert_eq!(metrics["human_minutes_per_completed_agent_task"], 11);
        assert_eq!(metrics["rework_loops"], 1);
        assert_eq!(metrics["missing_stop_events"], 0);
        assert_eq!(metrics["baseline"]["human_review_minutes"], 38);
        Ok(())
    }

    #[tokio::test]
    async fn imports_fixture_result_into_review_packet_and_audit_timeline()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let request: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/agent-result-import.manual.json"
        ))?;

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/agent-result-imports")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let packet: Value = serde_json::from_slice(&body)?;

        assert_eq!(packet["work_item_id"], request["work_item_id"]);
        assert!(
            packet["evidence_id"]
                .as_str()
                .unwrap_or("")
                .starts_with("evidence_")
        );
        assert_eq!(packet["changed_files"].as_array().map(Vec::len), Some(2));
        assert_eq!(packet["commands"][0]["outcome"], "passed");
        assert_eq!(packet["redaction_summary"]["redacted"], false);
        assert!(
            packet["timeline_event_id"]
                .as_str()
                .unwrap_or("")
                .starts_with("evt_")
        );
        assert!(Uuid::parse_str(packet["audit_event_id"].as_str().unwrap_or("")).is_ok());

        let work_item_id = request["work_item_id"]
            .as_str()
            .ok_or("fixture work_item_id must be a string")?;
        let review_packet_uri = format!("/v1/review-packets/{work_item_id}");
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(review_packet_uri)
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let fetched: Value = serde_json::from_slice(&body)?;
        assert_eq!(fetched["evidence_id"], packet["evidence_id"]);

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.imported_agent_result_evidence.len(), 1);
        assert_eq!(store.review_packet_refs.len(), 1);
        assert_eq!(store.run_timeline_events.len(), 1);
        assert_eq!(
            store.run_timeline_events[0].stage,
            RunTimelineStage::Completed
        );
        assert_eq!(store.audit_events.len(), 1);
        assert_eq!(store.audit_events[0].action, "agent_result.imported");
        assert_eq!(
            store.audit_events[0].id.0.to_string(),
            packet["audit_event_id"]
        );
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn import_rejects_malformed_result_payload_without_leaking_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/agent-result-imports")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "schema_version": "agent_result_import.v1",
                            "work_item_id": "018f30d5-9471-7c4c-85c4-0e14c3f76c10",
                            "source_agent_run_ref": "run_01J9Z4P4BS0M9P2QJ6T8Z6W2AR",
                            "summary": "Fixture mentions secret_token_value",
                            "raw_log": "secret_token_value"
                        })
                        .to_string(),
                    ))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        assert_json_rejection_is_redacted(
            &envelope,
            &["secret_token_value", "raw_log", "unknown field", "serde"],
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn import_redacts_sensitive_payload_before_storage_and_display()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let mut request: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/agent-result-import.manual.json"
        ))?;
        request["summary"] = json!("Fixture accidentally included bearer token details.");
        request["commands"][0]["summary"] = json!("raw_log output intentionally omitted.");

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/agent-result-imports")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let packet: Value = serde_json::from_slice(&body)?;
        let serialized = serde_json::to_string(&packet)?;

        assert_eq!(packet["summary"], "[redacted]");
        assert_eq!(packet["commands"][0]["summary"], "[redacted]");
        assert_eq!(packet["redaction_summary"]["redacted"], true);
        assert!(!serialized.contains("bearer token"));
        assert!(!serialized.contains("raw_log"));

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(
            store.imported_agent_result_evidence[0].summary,
            "[redacted]"
        );
        assert_eq!(
            store.imported_agent_result_evidence[0].commands[0].summary,
            "[redacted]"
        );
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn review_control_create_update_approve_done_transition()
    -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let app = build_router(state);
        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                review_control_create_request(),
            )?)
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let created: Value = serde_json::from_slice(&body)?;
        let work_item_id = created["id"]
            .as_str()
            .ok_or("work item id should serialize as string")?;
        assert_eq!(created["state"], "draft");
        assert_eq!(created["plan_approval"]["state"], "pending");
        assert_eq!(created["review_decision"]["state"], "pending");

        let uri = format!("/v1/review-control/work-items/{work_item_id}");
        let response = app
            .clone()
            .oneshot(json_http_request(
                "PATCH",
                &uri,
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "imported_result_refs": ["import_agent_result_1"],
                    "evidence_refs": ["review_packet_1"],
                    "audit_event_ref": "audit_update_1"
                }),
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let updated: Value = serde_json::from_slice(&body)?;
        assert_eq!(updated["state"], "ready_for_review");
        assert_eq!(updated["imported_result_refs"][0], "import_agent_result_1");

        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                &format!("/v1/review-control/work-items/{work_item_id}/approve-plan"),
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "decision_ref": "plan_decision_1",
                    "audit_event_ref": "audit_plan_1"
                }),
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let approved: Value = serde_json::from_slice(&body)?;
        assert_eq!(approved["plan_approval"]["state"], "approved");

        let response = app
            .oneshot(json_http_request(
                "POST",
                &format!("/v1/review-control/work-items/{work_item_id}/done"),
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "decision_ref": "done_decision_1",
                    "reason_code": "acceptance_criteria_met",
                    "audit_event_ref": "audit_done_1"
                }),
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let done: Value = serde_json::from_slice(&body)?;
        assert_eq!(done["state"], "done");
        assert_eq!(done["review_decision"]["state"], "done");
        Ok(())
    }

    #[tokio::test]
    async fn review_control_approve_rework_transition() -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::default();
        let app = build_router(state);
        let mut create = review_control_create_request();
        create["evidence_refs"] = json!(["review_packet_1"]);
        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                create,
            )?)
            .await?;
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let created: Value = serde_json::from_slice(&body)?;
        let work_item_id = created["id"]
            .as_str()
            .ok_or("work item id should serialize as string")?;

        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                &format!("/v1/review-control/work-items/{work_item_id}/approve-plan"),
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "decision_ref": "plan_decision_1",
                    "audit_event_ref": "audit_plan_1"
                }),
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(json_http_request(
                "POST",
                &format!("/v1/review-control/work-items/{work_item_id}/rework"),
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "decision_ref": "rework_decision_1",
                    "reason_code": "reviewer_requested_rework",
                    "audit_event_ref": "audit_rework_1"
                }),
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let rework: Value = serde_json::from_slice(&body)?;
        assert_eq!(rework["state"], "rework");
        assert_eq!(rework["review_decision"]["state"], "rework");
        Ok(())
    }

    #[tokio::test]
    async fn review_control_invalid_transition_returns_stable_conflict()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = build_router(AppState::default());
        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                review_control_create_request(),
            )?)
            .await?;
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let created: Value = serde_json::from_slice(&body)?;
        let work_item_id = created["id"]
            .as_str()
            .ok_or("work item id should serialize as string")?;

        let response = app
            .oneshot(json_http_request(
                "POST",
                &format!("/v1/review-control/work-items/{work_item_id}/done"),
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "decision_ref": "done_decision_1",
                    "reason_code": "acceptance_criteria_met",
                    "audit_event_ref": "audit_done_1"
                }),
            )?)
            .await?;

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        assert_eq!(envelope["error"]["code"], "conflict");
        assert_eq!(envelope["error"]["message_key"], "errors.conflict");
        assert_eq!(envelope["error"]["support"]["redacted"], true);
        Ok(())
    }

    #[tokio::test]
    async fn review_control_contract_redacts_and_rejects_secret_shaped_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = build_router(AppState::default());
        let mut request = review_control_create_request();
        request["request"]["summary"] = json!("Summary accidentally includes bearer token.");
        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                request,
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let created: Value = serde_json::from_slice(&body)?;
        let serialized = serde_json::to_string(&created)?;
        assert_eq!(created["request"]["summary"], "[redacted]");
        assert_eq!(created["redaction_summary"]["redacted"], true);
        assert!(!serialized.contains("bearer token"));

        let mut unsafe_request = review_control_create_request();
        unsafe_request["evidence_refs"] = json!(["raw_log_token_unsafe"]);
        let response = app
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                unsafe_request,
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        let serialized = serde_json::to_string(&envelope)?;
        assert_eq!(envelope["error"]["code"], "validation_failed");
        assert!(!serialized.contains("raw_log_token_unsafe"));
        Ok(())
    }

    #[tokio::test]
    async fn review_control_api_enforces_approval_required_invariant()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = build_router(AppState::default());
        let mut high_risk = review_control_create_request();
        high_risk["autonomy_level"] = json!("agent_can_prepare");
        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                high_risk,
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        assert_eq!(envelope["error"]["code"], "validation_failed");
        assert_eq!(envelope["error"]["support"]["redacted"], true);

        let mut allowed = review_control_create_request();
        allowed["risk_tier"] = json!("medium");
        allowed["autonomy_level"] = json!("agent_can_prepare");
        let response = app
            .clone()
            .oneshot(json_http_request(
                "POST",
                "/v1/review-control/work-items",
                allowed,
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let created: Value = serde_json::from_slice(&body)?;
        let work_item_id = created["id"]
            .as_str()
            .ok_or("work item id should serialize as string")?;
        assert_eq!(created["autonomy_level"], "agent_can_prepare");
        assert_eq!(created["request"]["protected_operation"], false);

        let response = app
            .oneshot(json_http_request(
                "PATCH",
                &format!("/v1/review-control/work-items/{work_item_id}"),
                json!({
                    "schema_version": "review_control.work_item.v1",
                    "protected_operation": true
                }),
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        assert_eq!(envelope["error"]["code"], "validation_failed");
        Ok(())
    }

    #[tokio::test]
    async fn review_packet_returns_not_found_without_imported_evidence()
    -> Result<(), Box<dyn std::error::Error>> {
        let response = build_router(AppState::default())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/review-packets/018f30d5-9471-7c4c-85c4-0e14c3f76c10")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let envelope: Value = serde_json::from_slice(&body)?;
        assert_eq!(envelope["error"]["code"], "not_found");
        assert_eq!(envelope["error"]["support"]["redacted"], true);
        Ok(())
    }
}
