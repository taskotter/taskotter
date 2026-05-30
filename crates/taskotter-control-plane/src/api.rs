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
    usage::{UsageEvaluation, UsageEvaluationRequest},
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
        UsageEvaluation,
        UsageEvaluationRequest,
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
        assert!(document["paths"]["/v1/audit/events"].is_object());
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
        assert_eq!(evaluation["failed_limits"], json!(["hourly-actions"]));
        Ok(())
    }
}
