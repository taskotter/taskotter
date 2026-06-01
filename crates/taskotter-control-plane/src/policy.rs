use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::authorization::{AuthorizationContext, AuthorizationRequest, RbacAuthorizer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicySubject {
    pub user_id: String,
    pub working_group_id: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Hosted,
    OpenAiCompatible,
    LocalRunner,
    FutureAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProviderRef {
    pub provider_id: String,
    pub kind: ProviderKind,
    pub model: String,
    #[serde(default)]
    pub endpoint_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyActorRef {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyResourceRef {
    pub r#type: String,
    pub id: String,
    #[serde(default)]
    pub working_group_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyRunContext {
    pub run_id: String,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub delegated_by: Option<PolicyActorRef>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyUsageContractRef {
    #[serde(default)]
    pub usage_decision_id: Option<String>,
    #[serde(default)]
    pub reservation_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyAuditContractRef {
    #[serde(default)]
    pub operations_audit_event_id: Option<String>,
    #[serde(default)]
    pub run_timeline_event_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyDecisionRequest {
    #[serde(default)]
    pub working_group_id: Option<String>,
    #[serde(default)]
    pub actor: Option<PolicyActorRef>,
    #[serde(default)]
    pub delegated_actor_chain: Vec<PolicyActorRef>,
    #[serde(default)]
    pub run_context: Option<PolicyRunContext>,
    #[serde(default)]
    pub resource: Option<PolicyResourceRef>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub policy_version: Option<String>,
    #[serde(default)]
    pub policy_snapshot_id: Option<String>,
    #[serde(default)]
    pub usage_contract: Option<PolicyUsageContractRef>,
    #[serde(default)]
    pub audit_contract: Option<PolicyAuditContractRef>,

    #[serde(default)]
    pub subject: Option<PolicySubject>,
    #[serde(default)]
    pub provider: Option<ProviderRef>,
    #[serde(default)]
    pub operation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicyDecisionProvenance {
    pub source: String,
    pub evaluated_by: String,
    pub policy_bundle_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicyDecisionConstraints {
    pub network_zones: Vec<String>,
    pub tool_scopes: Vec<String>,
    pub provider_model_ids: Vec<String>,
    #[serde(default)]
    pub high_risk_capabilities: Vec<HighRiskCapabilityGate>,
    pub max_runtime_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HighRiskCapabilityGate {
    pub capability: String,
    pub feature_flag: String,
    pub enabled: bool,
    pub effect: PolicyEffect,
    #[serde(default)]
    pub approval_ref: Option<String>,
    pub metering_unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub decision_id: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub max_cost_micro_usd: Option<u64>,
    pub schema_version: String,
    pub working_group_id: String,
    pub actor: PolicyActorRef,
    #[serde(default)]
    pub delegated_actor_chain: Vec<PolicyActorRef>,
    #[serde(default)]
    pub run_context: Option<PolicyRunContext>,
    pub action: String,
    pub resource: PolicyResourceRef,
    pub effect: PolicyEffect,
    pub policy_version: String,
    pub policy_snapshot_id: String,
    pub reason_code: String,
    pub correlation_id: String,
    pub request_id: String,
    pub provenance: PolicyDecisionProvenance,
    pub evaluated_at: String,
    pub ttl_seconds: u64,
    pub constraints: PolicyDecisionConstraints,
    #[serde(default)]
    pub usage_contract: Option<PolicyUsageContractRef>,
    #[serde(default)]
    pub audit_contract: Option<PolicyAuditContractRef>,
}

pub trait PolicyEvaluator: Send + Sync {
    fn evaluate(
        &self,
        request: &PolicyDecisionRequest,
        context: &AuthorizationContext,
    ) -> PolicyDecision;
}

#[derive(Debug, Default)]
pub struct BaselinePolicyEvaluator {
    authorizer: RbacAuthorizer,
}

impl PolicyEvaluator for BaselinePolicyEvaluator {
    fn evaluate(
        &self,
        request: &PolicyDecisionRequest,
        context: &AuthorizationContext,
    ) -> PolicyDecision {
        let normalized = NormalizedPolicyRequest::from(request);
        let requires_delegated_authority = normalized
            .run_context
            .as_ref()
            .is_some_and(|run_context| run_context.delegated_by.is_some());
        let delegated_authority = normalized.run_context.as_ref().and_then(|run_context| {
            let delegated_by = run_context.delegated_by.as_ref()?;
            context.delegated_authorities.iter().find(|grant| {
                grant.working_group_id == normalized.working_group_id
                    && grant.delegated_by == *delegated_by
                    && grant.delegated_to == normalized.actor
                    && grant.run_id == run_context.run_id
            })
        });
        let authz = self.authorizer.authorize(
            &AuthorizationRequest {
                working_group_id: &normalized.working_group_id,
                actor: &normalized.actor,
                action: &normalized.action,
                resource: &normalized.resource,
                requires_delegated_authority,
                delegated_authority,
            },
            context,
        );

        let legacy_denied = request.operation.as_deref().is_some_and(str::is_empty)
            || request
                .subject
                .as_ref()
                .is_some_and(|subject| subject.user_id == "denied")
            || request
                .provider
                .as_ref()
                .is_some_and(|provider| provider.provider_id.starts_with("disabled_"));

        let allowed = authz.allowed && !legacy_denied;
        let reason_code = if legacy_denied {
            "legacy_policy_denied"
        } else {
            authz.reason_code
        };

        PolicyDecision {
            allowed,
            decision_id: format!("local-policy:{}", normalized.action),
            reason: (!allowed).then(|| "request denied by control-plane policy".to_owned()),
            max_tokens: Some(8_192),
            max_cost_micro_usd: Some(50_000),
            schema_version: "policy-decision@0.1.0".to_owned(),
            working_group_id: normalized.working_group_id,
            actor: normalized.actor,
            delegated_actor_chain: request.delegated_actor_chain.clone(),
            run_context: normalized.run_context,
            action: normalized.action,
            resource: normalized.resource,
            effect: if allowed {
                PolicyEffect::Allow
            } else {
                PolicyEffect::Deny
            },
            policy_version: request
                .policy_version
                .clone()
                .unwrap_or_else(|| "0.1.0".to_owned()),
            policy_snapshot_id: request
                .policy_snapshot_id
                .clone()
                .unwrap_or_else(|| "polsnap_local".to_owned()),
            reason_code: reason_code.to_owned(),
            correlation_id: request
                .correlation_id
                .clone()
                .unwrap_or_else(|| "corr_local".to_owned()),
            request_id: request
                .request_id
                .clone()
                .unwrap_or_else(|| "req_local".to_owned()),
            provenance: PolicyDecisionProvenance {
                source: "control_plane".to_owned(),
                evaluated_by: "policy_engine".to_owned(),
                policy_bundle_ref: "policy-bundle/taskotter-mvp/0.1.0".to_owned(),
            },
            evaluated_at: "2026-01-01T00:00:00.000Z".to_owned(),
            ttl_seconds: 300,
            constraints: constraints_for(&request.provider, authz.protected_action, allowed),
            usage_contract: request.usage_contract.clone(),
            audit_contract: request.audit_contract.clone(),
        }
    }
}

struct NormalizedPolicyRequest {
    working_group_id: String,
    actor: PolicyActorRef,
    run_context: Option<PolicyRunContext>,
    action: String,
    resource: PolicyResourceRef,
}

impl From<&PolicyDecisionRequest> for NormalizedPolicyRequest {
    fn from(request: &PolicyDecisionRequest) -> Self {
        let subject = request.subject.as_ref();
        let provider = request.provider.as_ref();
        let actor = request.actor.clone().unwrap_or_else(|| {
            if let Some(agent_id) = subject.and_then(|subject| subject.agent_id.clone()) {
                PolicyActorRef {
                    r#type: "agent".to_owned(),
                    id: agent_id,
                }
            } else {
                PolicyActorRef {
                    r#type: "user".to_owned(),
                    id: subject
                        .map(|subject| subject.user_id.clone())
                        .unwrap_or_else(|| "usr_local".to_owned()),
                }
            }
        });

        let working_group_id = request
            .working_group_id
            .clone()
            .or_else(|| subject.map(|subject| subject.working_group_id.clone()))
            .unwrap_or_else(|| "wg_local".to_owned());
        let action = request
            .action
            .clone()
            .or_else(|| request.operation.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let resource = request
            .resource
            .clone()
            .unwrap_or_else(|| PolicyResourceRef {
                r#type: "provider".to_owned(),
                id: provider
                    .map(|provider| provider.provider_id.clone())
                    .unwrap_or_else(|| "prv_local".to_owned()),
                working_group_id: Some(working_group_id.clone()),
            });

        Self {
            working_group_id,
            actor,
            run_context: request.run_context.clone(),
            action,
            resource,
        }
    }
}

fn constraints_for(
    provider: &Option<ProviderRef>,
    protected_action: bool,
    allowed: bool,
) -> PolicyDecisionConstraints {
    PolicyDecisionConstraints {
        network_zones: vec!["default".to_owned()],
        tool_scopes: if allowed {
            vec!["repo:read".to_owned(), "repo:write".to_owned()]
        } else {
            Vec::new()
        },
        provider_model_ids: provider
            .as_ref()
            .map(|provider| vec![provider.model.clone()])
            .unwrap_or_default(),
        high_risk_capabilities: protected_action
            .then(|| HighRiskCapabilityGate {
                capability: "gateway.hosted_mcp_billing".to_owned(),
                feature_flag: "gateway.hosted_mcp_billing.enabled".to_owned(),
                enabled: allowed,
                effect: if allowed {
                    PolicyEffect::Allow
                } else {
                    PolicyEffect::Deny
                },
                approval_ref: (!allowed)
                    .then(|| "approval_required_before_paid_runtime".to_owned()),
                metering_unit: "hosted_mcp_runtime_ms".to_owned(),
            })
            .into_iter()
            .collect(),
        max_runtime_seconds: 1_800,
    }
}

#[cfg(test)]
mod tests {
    use crate::authorization::{WorkingGroupMembership, WorkingGroupRole};

    use super::*;

    #[test]
    fn baseline_policy_denies_incomplete_requests() {
        let evaluator = BaselinePolicyEvaluator::default();
        let decision = evaluator.evaluate(
            &PolicyDecisionRequest {
                subject: Some(PolicySubject {
                    user_id: "user_1".to_owned(),
                    working_group_id: "wg_1".to_owned(),
                    agent_id: None,
                    workflow_id: None,
                }),
                provider: Some(ProviderRef {
                    provider_id: "provider_1".to_owned(),
                    kind: ProviderKind::OpenAiCompatible,
                    model: "test-model".to_owned(),
                    endpoint_id: None,
                }),
                operation: Some(String::new()),
                ..policy_request()
            },
            &auth_context(),
        );

        assert!(!decision.allowed);
        assert_eq!(decision.reason_code, "legacy_policy_denied");
    }

    fn policy_request() -> PolicyDecisionRequest {
        PolicyDecisionRequest {
            working_group_id: Some("wg_1".to_owned()),
            actor: Some(PolicyActorRef {
                r#type: "user".to_owned(),
                id: "usr_1".to_owned(),
            }),
            delegated_actor_chain: Vec::new(),
            run_context: None,
            resource: Some(PolicyResourceRef {
                r#type: "issue".to_owned(),
                id: "iss_1".to_owned(),
                working_group_id: Some("wg_1".to_owned()),
            }),
            action: Some("issue.read".to_owned()),
            correlation_id: Some("corr_1".to_owned()),
            request_id: Some("req_1".to_owned()),
            policy_version: None,
            policy_snapshot_id: None,
            usage_contract: None,
            audit_contract: None,
            subject: None,
            provider: None,
            operation: None,
        }
    }

    fn auth_context() -> AuthorizationContext {
        AuthorizationContext {
            memberships: vec![WorkingGroupMembership {
                working_group_id: "wg_1".to_owned(),
                actor: PolicyActorRef {
                    r#type: "user".to_owned(),
                    id: "usr_1".to_owned(),
                },
                role: WorkingGroupRole::Owner,
            }],
            ..AuthorizationContext::default()
        }
    }
}
