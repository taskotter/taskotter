use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct WorkingGroupId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct IssueId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct CommentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct AgentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct SkillId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct ProviderId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct IntegrationId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct WorkingGroup {
    pub id: WorkingGroupId,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Issue {
    pub id: IssueId,
    pub working_group_id: WorkingGroupId,
    pub title: String,
    pub status: IssueStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Comment {
    pub id: CommentId,
    pub issue_id: IssueId,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Actor {
    User { id: UserId },
    Agent { id: AgentId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistryKind {
    Agent,
    Skill,
    Provider,
    Integration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RegistryEntry {
    pub kind: RegistryKind,
    pub id: Uuid,
    pub working_group_id: WorkingGroupId,
    pub display_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateWorkingGroupRequest {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateIssueRequest {
    pub working_group_id: WorkingGroupId,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    pub issue_id: IssueId,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateRegistryEntryRequest {
    pub kind: RegistryKind,
    pub working_group_id: WorkingGroupId,
    pub display_name: String,
    pub enabled: bool,
}

impl WorkingGroupId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkingGroupId {
    fn default() -> Self {
        Self::new()
    }
}

impl IssueId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for IssueId {
    fn default() -> Self {
        Self::new()
    }
}

impl CommentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CommentId {
    fn default() -> Self {
        Self::new()
    }
}
