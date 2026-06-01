use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{FromRequest, Request, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    audit::{AuditEvent, AuditEventId, CreateAuditEventRequest},
    authorization::{
        AuthorizationContext, DelegatedAuthority, RoleBinding, WorkingGroupMembership,
        WorkingGroupRole,
    },
    domain::{
        Comment, CommentId, CreateCommentRequest, CreateIssueRequest, CreateRegistryEntryRequest,
        CreateWorkingGroupRequest, Issue, IssueId, IssueStatus, RegistryEntry, WorkingGroup,
        WorkingGroupId,
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
    runner::{
        RunnerCapability, RunnerControlPlaneFlags, RunnerDispatchDecision,
        RunnerDispatchDiagnostic, RunnerDispatchRequest, RunnerDispatchStatus,
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
    runner_control_plane_flags: Arc<RunnerControlPlaneFlags>,
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
    runner_dispatch_decisions: Vec<RunnerDispatchDecision>,
    run_timeline_events: Vec<RunTimelineEventV1>,
    operations_audit_events: Vec<OperationsAuditEventV1>,
    operations_health_events: Vec<OperationsHealthEventV1>,
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
        .route("/v1/runner/dispatch", post(dispatch_runner_job))
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
    path = "/v1/runner/dispatch",
    request_body = RunnerDispatchRequest,
    responses((status = 200, body = RunnerDispatchDecision), (status = 400, body = ErrorResponse))
)]
async fn dispatch_runner_job(
    State(state): State<AppState>,
    SafeJson(request): SafeJson<RunnerDispatchRequest>,
) -> Result<Json<RunnerDispatchDecision>, ApiError> {
    require_non_empty("working_group_id", &request.working_group_id)?;
    require_non_empty("run_id", &request.run_id)?;
    require_non_empty("runner_id", &request.runner_id)?;

    let decision = request.evaluate_for_dispatch(&state.runner_control_plane_flags);
    state
        .store
        .lock()
        .map_err(|_| ApiError::internal())?
        .runner_dispatch_decisions
        .push(decision.clone());

    Ok(Json(decision))
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
        dispatch_runner_job,
        create_audit_event,
        create_run_timeline_event,
        create_operations_audit_event,
        create_operations_health_event
    ),
    components(schemas(
        AuditEvent,
        CreateAuditEventRequest,
        Comment,
        CreateCommentRequest,
        CreateIssueRequest,
        CreateRegistryEntryRequest,
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
        PolicyDecision,
        PolicyDecisionRequest,
        PolicyActorRef,
        RedactionClassification,
        RegistryEntry,
        RunnerCapability,
        RunnerControlPlaneFlags,
        RunnerDispatchDecision,
        RunnerDispatchDiagnostic,
        RunnerDispatchRequest,
        RunnerDispatchStatus,
        RoleBinding,
        RemoteUsageReportV1,
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
        assert!(document["paths"]["/v1/runner/dispatch"].is_object());
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
            document["components"]["schemas"]["RunnerControlPlaneFlags"]["properties"]
                ["kill_switch_engaged"]
                .is_object()
        );
        assert!(
            document["components"]["schemas"]["RunnerDispatchRequest"]["properties"]
                .get("control_plane_flags")
                .is_none()
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
    async fn runner_dispatch_defaults_to_refusal_with_audit_diagnostic_reason()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = json!({
            "working_group_id": "wg_1",
            "run_id": "run_1",
            "runner_id": "runner_1",
            "required_capabilities": ["local_tools"],
            "correlation_id": "corr_1",
            "request_id": "req_1"
        });

        let state = AppState::default();
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runner/dispatch")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;

        assert_eq!(decision["accepted"], false);
        assert_eq!(decision["status"], "refused");
        assert_eq!(
            decision["diagnostic"]["reason_code"],
            "control_plane_kill_switch_engaged"
        );
        assert_eq!(
            decision["diagnostic"]["audit_event_type"],
            "runner.dispatch.refused"
        );
        assert_eq!(
            decision["diagnostic"]["feature_flag"],
            "runner.kill_switch.engaged"
        );

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.runner_dispatch_decisions.len(), 1);
        assert_eq!(
            store.runner_dispatch_decisions[0].diagnostic.reason_code,
            "control_plane_kill_switch_engaged"
        );
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn runner_dispatch_refuses_disabled_capability_before_dispatch()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = json!({
            "working_group_id": "wg_1",
            "run_id": "run_1",
            "runner_id": "runner_1",
            "required_capabilities": ["computer_use"],
            "correlation_id": "corr_1",
            "request_id": "req_1"
        });

        let state = app_state_with_runner_flags(RunnerControlPlaneFlags {
            kill_switch_engaged: false,
            runner_dispatch_enabled: true,
            ..RunnerControlPlaneFlags::default()
        });
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runner/dispatch")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;

        assert_eq!(decision["accepted"], false);
        assert_eq!(decision["status"], "refused");
        assert_eq!(
            decision["diagnostic"]["reason_code"],
            "runner_capability_disabled"
        );
        assert_eq!(decision["diagnostic"]["capability"], "computer_use");
        assert_eq!(
            decision["diagnostic"]["feature_flag"],
            "runner.computer_use.enabled"
        );
        assert_eq!(
            decision["diagnostic"]["message_key"],
            "runner.dispatch.refused.capability_disabled"
        );

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.runner_dispatch_decisions.len(), 1);
        assert!(!store.runner_dispatch_decisions[0].accepted);
        drop(store);
        Ok(())
    }

    #[tokio::test]
    async fn runner_dispatch_ignores_body_flag_override_and_uses_server_kill_switch()
    -> Result<(), Box<dyn std::error::Error>> {
        let request = json!({
            "working_group_id": "wg_1",
            "run_id": "run_1",
            "runner_id": "runner_1",
            "required_capabilities": ["computer_use"],
            "control_plane_flags": {
                "kill_switch_engaged": false,
                "runner_dispatch_enabled": true,
                "computer_use_enabled": true
            },
            "correlation_id": "corr_1",
            "request_id": "req_1"
        });

        let state = AppState::default();
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runner/dispatch")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request.to_string()))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let decision: Value = serde_json::from_slice(&body)?;

        assert_eq!(decision["accepted"], false);
        assert_eq!(decision["status"], "refused");
        assert_eq!(
            decision["diagnostic"]["reason_code"],
            "control_plane_kill_switch_engaged"
        );
        assert_eq!(
            decision["diagnostic"]["feature_flag"],
            "runner.kill_switch.engaged"
        );

        let store = state.store.lock().map_err(|_| "store lock failed")?;
        assert_eq!(store.runner_dispatch_decisions.len(), 1);
        assert_eq!(
            store.runner_dispatch_decisions[0].diagnostic.reason_code,
            "control_plane_kill_switch_engaged"
        );
        drop(store);
        Ok(())
    }

    fn app_state_with_runner_flags(
        runner_control_plane_flags: RunnerControlPlaneFlags,
    ) -> AppState {
        AppState {
            runner_control_plane_flags: Arc::new(runner_control_plane_flags),
            ..AppState::default()
        }
    }
}
