use crate::domain::{Action, PrincipalRef, ResourceRef};
use crate::policy::PolicyEffect;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    pub request_id: String,
    pub principal: PrincipalRef,
    pub action: Action,
    pub resource: ResourceRef,
    pub outcome: AuditOutcome,
    pub policy_effect: Option<PolicyEffect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditOutcome {
    Accepted,
    Rejected,
    Failed,
}
