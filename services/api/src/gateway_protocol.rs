use crate::domain::{PrincipalRef, ResourceId};
use crate::policy::PolicyDecision;
use crate::usage::UsageUnit;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayRequest {
    pub request_id: ResourceId,
    pub working_group_id: ResourceId,
    pub requested_by: PrincipalRef,
    pub provider_id: ResourceId,
    pub model: String,
    pub policy_decision: PolicyDecision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayUsageReport {
    pub request_id: ResourceId,
    pub units: Vec<UsageUnit>,
}
