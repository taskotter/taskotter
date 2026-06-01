use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::State,
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
    domain::{
        Comment, CommentId, CreateCommentRequest, CreateIssueRequest, CreateRegistryEntryRequest,
        CreateWorkingGroupRequest, Issue, IssueId, IssueStatus, RegistryEntry, WorkingGroup,
        WorkingGroupId,
    },
    policy::{BaselinePolicyEvaluator, PolicyDecision, PolicyDecisionRequest, PolicyEvaluator},
    usage::{
        CostReconciliationStatus, MeteringUnit, QuotaEnforcement, RemoteUsageReportV1,
        ReservationStatus, UsageActorRef, UsageAuditEventV1, UsageEvaluation,
        UsageEvaluationRequest, UsageEventPayload, UsageLedgerEntry, UsageMeasurements,
        UsageReservation, UsageResourceRef, UsageSourceSurface, UsageSubjectRef,
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
    audit_events: Vec<AuditEvent>,
    usage_events: Vec<UsageAuditEventV1>,
    usage_ledger_entries: Vec<UsageLedgerEntry>,
    usage_reservations: Vec<UsageReservation>,
    remote_usage_reports: Vec<RemoteUsageReportV1>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
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
        .route("/v1/audit/events", post(create_audit_event))
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
    Json(request): Json<CreateWorkingGroupRequest>,
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
    Json(request): Json<CreateIssueRequest>,
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
    Json(request): Json<CreateCommentRequest>,
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
    Json(request): Json<CreateRegistryEntryRequest>,
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
    Json(request): Json<PolicyDecisionRequest>,
) -> Json<PolicyDecision> {
    Json(state.policy_evaluator.evaluate(&request))
}

#[utoipa::path(
    post,
    path = "/v1/usage/evaluate",
    request_body = UsageEvaluationRequest,
    responses((status = 200, body = UsageEvaluation))
)]
async fn evaluate_usage(Json(request): Json<UsageEvaluationRequest>) -> Json<UsageEvaluation> {
    Json(request.policy_set.evaluate(&request.snapshot))
}

#[utoipa::path(
    post,
    path = "/v1/usage/events",
    request_body = UsageAuditEventV1,
    responses((status = 202, body = UsageAuditEventV1), (status = 400, body = ErrorResponse))
)]
async fn create_usage_event(
    State(state): State<AppState>,
    Json(event): Json<UsageAuditEventV1>,
) -> Result<(StatusCode, Json<UsageAuditEventV1>), ApiError> {
    let ledger_entry = event
        .to_ledger_entry()
        .map_err(|error| ApiError::bad_request(error.to_string()))?;

    let mut store = state.store.lock().map_err(|_| ApiError::internal())?;
    if let Some(existing) = store
        .usage_events
        .iter()
        .find(|stored| stored.idempotency_key == event.idempotency_key)
        .cloned()
    {
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
    Json(report): Json<RemoteUsageReportV1>,
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
    path = "/v1/audit/events",
    request_body = CreateAuditEventRequest,
    responses((status = 201, body = AuditEvent), (status = 400, body = ErrorResponse))
)]
async fn create_audit_event(
    State(state): State<AppState>,
    Json(request): Json<CreateAuditEventRequest>,
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

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::bad_request(format!("{field} is required")));
    }

    Ok(())
}

fn require_schema_version(expected: &'static str, actual: &str) -> Result<(), ApiError> {
    if actual != expected {
        return Err(ApiError::bad_request(format!(
            "schema_version must be {expected}"
        )));
    }

    Ok(())
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal error".to_owned(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
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
        create_audit_event
    ),
    components(schemas(
        AuditEvent,
        CreateAuditEventRequest,
        Comment,
        CreateCommentRequest,
        CreateIssueRequest,
        CreateRegistryEntryRequest,
        CreateWorkingGroupRequest,
        ErrorResponse,
        HealthResponse,
        Issue,
        PolicyDecision,
        PolicyDecisionRequest,
        RegistryEntry,
        RemoteUsageReportV1,
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
        UsageResourceRef,
        UsageSourceSurface,
        UsageSubjectRef,
        WorkingGroup
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
        assert!(document["paths"]["/v1/registry"].is_object());
        assert!(document["paths"]["/v1/usage/evaluate"].is_object());
        assert!(document["paths"]["/v1/usage/events"].is_object());
        assert!(document["paths"]["/v1/remote/usage-reports"].is_object());
        assert!(document["paths"]["/v1/audit/events"].is_object());
        assert!(
            document["components"]["schemas"]["PolicyDecision"]["properties"]["allowed"]
                .is_object()
        );
        assert!(
            document["components"]["schemas"]["UsageAuditEventV1"]["properties"]["status"]
                .is_object()
        );
        Ok(())
    }

    #[tokio::test]
    async fn policy_api_returns_gateway_compatible_decision()
    -> Result<(), Box<dyn std::error::Error>> {
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

        let response = build_router(AppState::default())
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
        assert_eq!(
            evaluation["denial_reason"],
            "quota hard deny before dispatch: hourly-actions"
        );
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
    async fn rejects_denied_usage_without_reason_and_sensitive_raw_payloads()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut denied: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/usage-event.high-risk-runtime-denied.json"
        ))?;
        denied["status"] = json!("denied");

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
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
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
}
