use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{Actor, WorkingGroupId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct AuditEventId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub working_group_id: WorkingGroupId,
    pub actor: Actor,
    pub action: String,
    pub resource: String,
    pub outcome: AuditOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Succeeded,
    Denied,
    Failed,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateAuditEventRequest {
    pub working_group_id: WorkingGroupId,
    pub actor: Actor,
    pub action: String,
    pub resource: String,
    pub outcome: AuditOutcome,
}

impl AuditEventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AuditEventId {
    fn default() -> Self {
        Self::new()
    }
}
