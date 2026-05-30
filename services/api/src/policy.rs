use crate::domain::{Action, PrincipalRef, ResourceId, ResourceRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecisionRequest {
    pub principal: PrincipalRef,
    pub action: Action,
    pub resource: ResourceRef,
    pub context: PolicyContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyContext {
    pub request_id: String,
    pub source: PolicySource,
    pub requires_human_approval: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicySource {
    Api,
    Automation,
    Runner,
    Gateway,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyConstraintDecision {
    pub name: String,
    pub effect: PolicyEffect,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecision {
    pub decision_id: String,
    pub request_id: String,
    pub scope: PolicyDecisionScope,
    pub effect: PolicyEffect,
    pub constraints: Vec<PolicyConstraintDecision>,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecisionScope {
    pub working_group_id: ResourceId,
    pub resource_id: Option<ResourceId>,
    pub actions: Vec<Action>,
}

impl PolicyDecision {
    pub fn new(
        decision_id: impl Into<String>,
        request_id: impl Into<String>,
        scope: PolicyDecisionScope,
        constraints: Vec<PolicyConstraintDecision>,
        expires_at_unix_seconds: u64,
    ) -> Self {
        let effect = if constraints
            .iter()
            .all(|decision| decision.effect == PolicyEffect::Allow)
        {
            PolicyEffect::Allow
        } else {
            PolicyEffect::Deny
        };

        Self {
            decision_id: decision_id.into(),
            request_id: request_id.into(),
            scope,
            effect,
            constraints,
            expires_at_unix_seconds,
        }
    }

    pub fn is_allowed(&self) -> bool {
        self.effect == PolicyEffect::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_when_any_constraint_denies() {
        let decision = PolicyDecision::new(
            "decision_123",
            "request_123",
            PolicyDecisionScope {
                working_group_id: ResourceId::new("wg_123").expect("valid working group id"),
                resource_id: None,
                actions: vec![Action::Read],
            },
            vec![
                PolicyConstraintDecision {
                    name: "role".to_string(),
                    effect: PolicyEffect::Allow,
                    reason: "role permits action".to_string(),
                },
                PolicyConstraintDecision {
                    name: "quota".to_string(),
                    effect: PolicyEffect::Deny,
                    reason: "quota exhausted".to_string(),
                },
            ],
            1_800_000_000,
        );

        assert_eq!(decision.effect, PolicyEffect::Deny);
        assert!(!decision.is_allowed());
    }
}
