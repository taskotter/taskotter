use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
pub struct PolicyDecisionRequest {
    pub subject: PolicySubject,
    pub provider: ProviderRef,
    pub operation: String,
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
}

pub trait PolicyEvaluator: Send + Sync {
    fn evaluate(&self, request: &PolicyDecisionRequest) -> PolicyDecision;
}

#[derive(Debug, Default)]
pub struct BaselinePolicyEvaluator;

impl PolicyEvaluator for BaselinePolicyEvaluator {
    fn evaluate(&self, request: &PolicyDecisionRequest) -> PolicyDecision {
        let denied = request.operation.trim().is_empty()
            || request.subject.user_id == "denied"
            || request.provider.provider_id.starts_with("disabled_");

        if denied {
            return PolicyDecision {
                allowed: false,
                decision_id: format!("local-policy:{}", request.operation),
                reason: Some("request denied by control-plane policy".to_owned()),
                max_tokens: Some(8_192),
                max_cost_micro_usd: Some(50_000),
            };
        }

        PolicyDecision {
            allowed: true,
            decision_id: format!("local-policy:{}", request.operation),
            reason: None,
            max_tokens: Some(8_192),
            max_cost_micro_usd: Some(50_000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_policy_denies_incomplete_requests() {
        let evaluator = BaselinePolicyEvaluator;
        let decision = evaluator.evaluate(&PolicyDecisionRequest {
            subject: PolicySubject {
                user_id: "user_1".to_owned(),
                working_group_id: "wg_1".to_owned(),
                agent_id: None,
                workflow_id: None,
            },
            provider: ProviderRef {
                provider_id: "provider_1".to_owned(),
                kind: ProviderKind::OpenAiCompatible,
                model: "test-model".to_owned(),
                endpoint_id: None,
            },
            operation: String::new(),
        });

        assert!(!decision.allowed);
    }
}
