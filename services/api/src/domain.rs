use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::EmptyIdentifier);
        }

        if value.len() > 128 {
            return Err(DomainError::IdentifierTooLong);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    EmptyIdentifier,
    IdentifierTooLong,
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentifier => formatter.write_str("identifier cannot be empty"),
            Self::IdentifierTooLong => formatter.write_str("identifier cannot exceed 128 bytes"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrincipalKind {
    User,
    Agent,
    Service,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrincipalRef {
    pub kind: PrincipalKind,
    pub id: ResourceId,
    pub working_group_id: ResourceId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    WorkingGroup,
    Issue,
    Comment,
    Chat,
    Agent,
    Skill,
    Integration,
    Provider,
    Workflow,
    Runner,
    Gateway,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceRef {
    pub kind: ResourceKind,
    pub id: ResourceId,
    pub working_group_id: ResourceId,
    pub owner_id: Option<ResourceId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Read,
    Write,
    Manage,
    Admin,
    Execute,
    Dispatch,
    Approve,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssueStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
    Backlog,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssuePriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssueRecord {
    pub id: ResourceId,
    pub working_group_id: ResourceId,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub assignee: Option<PrincipalRef>,
    pub parent_issue_id: Option<ResourceId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_resource_ids() {
        assert_eq!(ResourceId::new("   "), Err(DomainError::EmptyIdentifier));
    }

    #[test]
    fn accepts_normal_resource_ids() {
        let id = ResourceId::new("issue_123").expect("valid id");

        assert_eq!(id.as_str(), "issue_123");
    }
}
