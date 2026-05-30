use crate::domain::{Action, PrincipalRef, ResourceRef};

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
    pub effect: PolicyEffect,
    pub constraints: Vec<PolicyConstraintDecision>,
}

impl PolicyDecision {
    pub fn from_constraints(constraints: Vec<PolicyConstraintDecision>) -> Self {
        let effect = if constraints
            .iter()
            .all(|decision| decision.effect == PolicyEffect::Allow)
        {
            PolicyEffect::Allow
        } else {
            PolicyEffect::Deny
        };

        Self {
            effect,
            constraints,
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
        let decision = PolicyDecision::from_constraints(vec![
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
        ]);

        assert_eq!(decision.effect, PolicyEffect::Deny);
        assert!(!decision.is_allowed());
    }
}
