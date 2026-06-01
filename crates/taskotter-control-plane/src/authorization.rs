use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::policy::{PolicyActorRef, PolicyResourceRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkingGroupRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct WorkingGroupMembership {
    pub working_group_id: String,
    pub actor: PolicyActorRef,
    pub role: WorkingGroupRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RoleBinding {
    pub working_group_id: String,
    pub actor: PolicyActorRef,
    #[serde(default)]
    pub resource_type: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DelegatedAuthority {
    pub delegated_by: PolicyActorRef,
    pub run_id: String,
    pub actions: Vec<String>,
    #[serde(default)]
    pub resource_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthorizationContext {
    pub memberships: Vec<WorkingGroupMembership>,
    #[serde(default)]
    pub role_bindings: Vec<RoleBinding>,
    #[serde(default)]
    pub delegated_authorities: Vec<DelegatedAuthority>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationRequest<'a> {
    pub working_group_id: &'a str,
    pub actor: &'a PolicyActorRef,
    pub action: &'a str,
    pub resource: &'a PolicyResourceRef,
    pub delegated_authority: Option<&'a DelegatedAuthority>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason_code: &'static str,
    pub role: Option<WorkingGroupRole>,
    pub protected_action: bool,
}

#[derive(Debug, Default)]
pub struct RbacAuthorizer;

impl RbacAuthorizer {
    pub fn authorize(
        &self,
        request: &AuthorizationRequest<'_>,
        context: &AuthorizationContext,
    ) -> AuthorizationDecision {
        if request
            .resource
            .working_group_id
            .as_deref()
            .is_some_and(|resource_wg| resource_wg != request.working_group_id)
        {
            return denied("cross_working_group_denied", None, request.action);
        }

        if let Some(delegated) = request.delegated_authority {
            if !delegated
                .actions
                .iter()
                .any(|action| action == request.action)
            {
                return denied("delegated_authority_narrowed", None, request.action);
            }
            if !delegated.resource_ids.is_empty()
                && !delegated
                    .resource_ids
                    .iter()
                    .any(|resource_id| resource_id == &request.resource.id)
            {
                return denied("delegated_authority_narrowed", None, request.action);
            }
        }

        let membership = context.memberships.iter().find(|membership| {
            membership.working_group_id == request.working_group_id
                && membership.actor == *request.actor
        });
        let Some(membership) = membership else {
            return denied("missing_membership", None, request.action);
        };

        let protected_action = is_protected_action(request.action, &request.resource.r#type);
        if protected_action
            && !has_matching_role_binding(
                request.working_group_id,
                request.actor,
                request.action,
                request.resource,
                context,
            )
            && !matches!(
                membership.role,
                WorkingGroupRole::Owner | WorkingGroupRole::Admin
            )
        {
            return denied(
                "protected_resource_action",
                Some(membership.role.clone()),
                request.action,
            );
        }

        if role_allows(&membership.role, request.action) {
            return AuthorizationDecision {
                allowed: true,
                reason_code: if protected_action {
                    "protected_action_role_matched"
                } else {
                    "role_matched"
                },
                role: Some(membership.role.clone()),
                protected_action,
            };
        }

        if has_matching_role_binding(
            request.working_group_id,
            request.actor,
            request.action,
            request.resource,
            context,
        ) {
            return AuthorizationDecision {
                allowed: true,
                reason_code: "role_binding_matched",
                role: Some(membership.role.clone()),
                protected_action,
            };
        }

        denied(
            "insufficient_role",
            Some(membership.role.clone()),
            request.action,
        )
    }
}

fn denied(
    reason_code: &'static str,
    role: Option<WorkingGroupRole>,
    action: &str,
) -> AuthorizationDecision {
    AuthorizationDecision {
        allowed: false,
        reason_code,
        role,
        protected_action: is_protected_action(action, ""),
    }
}

fn role_allows(role: &WorkingGroupRole, action: &str) -> bool {
    match role {
        WorkingGroupRole::Owner | WorkingGroupRole::Admin => true,
        WorkingGroupRole::Member => {
            matches!(
                action,
                "issue.read"
                    | "issue.create"
                    | "issue.update"
                    | "comment.read"
                    | "comment.create"
                    | "agent.run.delegate"
            )
        }
        WorkingGroupRole::Viewer => matches!(action, "issue.read" | "comment.read"),
    }
}

fn has_matching_role_binding(
    working_group_id: &str,
    actor: &PolicyActorRef,
    action: &str,
    resource: &PolicyResourceRef,
    context: &AuthorizationContext,
) -> bool {
    context.role_bindings.iter().any(|binding| {
        binding.working_group_id == working_group_id
            && binding.actor == *actor
            && binding.actions.iter().any(|granted| granted == action)
            && binding
                .resource_type
                .as_ref()
                .is_none_or(|resource_type| resource_type == &resource.r#type)
            && binding
                .resource_id
                .as_ref()
                .is_none_or(|resource_id| resource_id == &resource.id)
    })
}

fn is_protected_action(action: &str, _resource_type: &str) -> bool {
    matches!(
        action,
        "runner.job.execute"
            | "runner.register"
            | "integration.invoke"
            | "provider.invoke"
            | "gateway.mcp.session.open"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str) -> PolicyActorRef {
        PolicyActorRef {
            r#type: "user".to_owned(),
            id: id.to_owned(),
        }
    }

    fn issue(wg: &str) -> PolicyResourceRef {
        PolicyResourceRef {
            r#type: "issue".to_owned(),
            id: "iss_1".to_owned(),
            working_group_id: Some(wg.to_owned()),
        }
    }

    #[test]
    fn denies_cross_working_group_resources() {
        let authorizer = RbacAuthorizer;
        let context = AuthorizationContext {
            memberships: vec![WorkingGroupMembership {
                working_group_id: "wg_1".to_owned(),
                actor: user("usr_1"),
                role: WorkingGroupRole::Owner,
            }],
            ..AuthorizationContext::default()
        };

        let resource = issue("wg_2");
        let decision = authorizer.authorize(
            &AuthorizationRequest {
                working_group_id: "wg_1",
                actor: &user("usr_1"),
                action: "issue.read",
                resource: &resource,
                delegated_authority: None,
            },
            &context,
        );

        assert!(!decision.allowed);
        assert_eq!(decision.reason_code, "cross_working_group_denied");
    }
}
