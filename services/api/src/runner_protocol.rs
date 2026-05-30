use crate::domain::{PrincipalRef, ResourceId};
use crate::policy::PolicyDecision;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerRegistration {
    pub runner_id: ResourceId,
    pub working_group_id: ResourceId,
    pub labels: Vec<String>,
    pub max_concurrency: u16,
    pub capabilities: RunnerCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerCapabilities {
    pub operating_system: String,
    pub architecture: String,
    pub tool_names: Vec<String>,
    pub supports_mcp_hosting: bool,
    pub supports_local_models: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerJobDispatch {
    pub job_id: ResourceId,
    pub runner_id: ResourceId,
    pub requested_by: PrincipalRef,
    pub policy_decision: PolicyDecision,
    pub timeout_seconds: u32,
}

impl RunnerRegistration {
    pub fn accepts_jobs(&self) -> bool {
        self.max_concurrency > 0
    }
}
