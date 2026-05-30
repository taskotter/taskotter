use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{Actor, WorkingGroupId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicyDecisionRequest {
    pub working_group_id: WorkingGroupId,
    pub actor: Actor,
    pub action: String,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicyDecision {
    pub effect: PolicyEffect,
    pub reason: String,
}

pub trait PolicyEvaluator: Send + Sync {
    fn evaluate(&self, request: &PolicyDecisionRequest) -> PolicyDecision;
}

#[derive(Debug, Default)]
pub struct BaselinePolicyEvaluator;

impl PolicyEvaluator for BaselinePolicyEvaluator {
    fn evaluate(&self, request: &PolicyDecisionRequest) -> PolicyDecision {
        if request.action.trim().is_empty() || request.resource.trim().is_empty() {
            return PolicyDecision {
                effect: PolicyEffect::Deny,
                reason: "action and resource are required".to_owned(),
            };
        }

        PolicyDecision {
            effect: PolicyEffect::Allow,
            reason: "baseline policy allows non-empty scoped requests".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::domain::{UserId, WorkingGroupId};

    #[test]
    fn baseline_policy_denies_incomplete_requests() {
        let evaluator = BaselinePolicyEvaluator;
        let decision = evaluator.evaluate(&PolicyDecisionRequest {
            working_group_id: WorkingGroupId(Uuid::new_v4()),
            actor: Actor::User {
                id: UserId(Uuid::new_v4()),
            },
            action: String::new(),
            resource: "issue".to_owned(),
        });

        assert_eq!(decision.effect, PolicyEffect::Deny);
    }
}
