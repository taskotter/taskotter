use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::WorkingGroupId;

const MAX_BODY_BYTES: usize = 16 * 1024;
const MAX_ACCEPTANCE_CRITERIA: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct WorkItemId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntakeMode {
    Preview,
    Create,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntakeSourceKind {
    Manual,
    GithubIssue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskTierHint {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct IntakeRequest {
    pub working_group_id: WorkingGroupId,
    pub mode: IntakeMode,
    pub input: IntakeSourcePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum IntakeSourcePayload {
    Manual {
        title: String,
        body: String,
        #[serde(default)]
        source_url: Option<String>,
        #[serde(default)]
        acceptance_criteria: Vec<String>,
        #[serde(default)]
        risk_tier_hint: Option<RiskTierHint>,
    },
    GithubIssue {
        title: String,
        body: String,
        #[serde(default)]
        source_url: Option<String>,
        #[serde(default)]
        acceptance_criteria: Vec<String>,
        #[serde(default)]
        risk_tier_hint: Option<RiskTierHint>,
        #[serde(default)]
        repository: Option<String>,
        #[serde(default)]
        issue_number: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PrototypeWorkItemInput {
    pub working_group_id: WorkingGroupId,
    pub title: String,
    pub body_summary: String,
    pub source: IntakeSourceKind,
    #[serde(default)]
    pub source_url: Option<String>,
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub risk_tier_hint: Option<RiskTierHint>,
    #[serde(default)]
    pub external_reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PrototypeWorkItem {
    pub id: WorkItemId,
    pub input: PrototypeWorkItemInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct IntakeResponse {
    pub mode: IntakeMode,
    pub input: PrototypeWorkItemInput,
    #[serde(default)]
    pub work_item: Option<PrototypeWorkItem>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntakeValidationError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("body exceeds maximum size")]
    BodyTooLarge,
    #[error("acceptance_criteria is required")]
    MissingAcceptanceCriteria,
    #[error("acceptance_criteria contains an empty item")]
    EmptyAcceptanceCriterion,
    #[error("acceptance_criteria has too many items")]
    TooManyAcceptanceCriteria,
    #[error("{field} contains a sensitive pattern")]
    SensitivePattern { field: &'static str },
    #[error("source_url must be http or https")]
    UnsupportedSourceUrl,
}

struct RawIntakeFields {
    title: String,
    body: String,
    source_url: Option<String>,
    acceptance_criteria: Vec<String>,
    risk_tier_hint: Option<RiskTierHint>,
    external_reference: Option<String>,
}

impl WorkItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl IntakeRequest {
    pub fn into_work_item_input(self) -> Result<PrototypeWorkItemInput, IntakeValidationError> {
        let working_group_id = self.working_group_id;
        match self.input {
            IntakeSourcePayload::Manual {
                title,
                body,
                source_url,
                acceptance_criteria,
                risk_tier_hint,
            } => build_input(
                working_group_id,
                IntakeSourceKind::Manual,
                RawIntakeFields {
                    title,
                    body,
                    source_url,
                    acceptance_criteria,
                    risk_tier_hint,
                    external_reference: None,
                },
            ),
            IntakeSourcePayload::GithubIssue {
                title,
                body,
                source_url,
                acceptance_criteria,
                risk_tier_hint,
                repository,
                issue_number,
            } => build_input(
                working_group_id,
                IntakeSourceKind::GithubIssue,
                RawIntakeFields {
                    title,
                    body,
                    source_url,
                    acceptance_criteria,
                    risk_tier_hint,
                    external_reference: github_reference(repository, issue_number),
                },
            ),
        }
    }
}

fn build_input(
    working_group_id: WorkingGroupId,
    source: IntakeSourceKind,
    fields: RawIntakeFields,
) -> Result<PrototypeWorkItemInput, IntakeValidationError> {
    let title = normalize_required("title", fields.title)?;
    let body = normalize_required("body", fields.body)?;
    let source_url = normalize_optional_url(fields.source_url)?;
    let acceptance_criteria = normalize_acceptance_criteria(fields.acceptance_criteria)?;

    reject_sensitive("title", &title)?;
    reject_sensitive("body", &body)?;
    for criterion in &acceptance_criteria {
        reject_sensitive("acceptance_criteria", criterion)?;
    }

    if body.len() > MAX_BODY_BYTES {
        return Err(IntakeValidationError::BodyTooLarge);
    }

    Ok(PrototypeWorkItemInput {
        working_group_id,
        title,
        body_summary: summarize_body(&body),
        source,
        source_url,
        acceptance_criteria,
        risk_tier_hint: fields.risk_tier_hint,
        external_reference: fields.external_reference,
    })
}

fn normalize_required(field: &'static str, value: String) -> Result<String, IntakeValidationError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(IntakeValidationError::Required(field));
    }
    Ok(normalized)
}

fn normalize_optional_url(value: Option<String>) -> Result<Option<String>, IntakeValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Ok(None);
    }
    if !(value.starts_with("https://") || value.starts_with("http://")) {
        return Err(IntakeValidationError::UnsupportedSourceUrl);
    }
    reject_sensitive("source_url", &value)?;
    Ok(Some(value))
}

fn normalize_acceptance_criteria(
    values: Vec<String>,
) -> Result<Vec<String>, IntakeValidationError> {
    if values.is_empty() {
        return Err(IntakeValidationError::MissingAcceptanceCriteria);
    }
    if values.len() > MAX_ACCEPTANCE_CRITERIA {
        return Err(IntakeValidationError::TooManyAcceptanceCriteria);
    }

    values
        .into_iter()
        .map(|value| {
            let normalized = value.trim().to_owned();
            if normalized.is_empty() {
                return Err(IntakeValidationError::EmptyAcceptanceCriterion);
            }
            Ok(normalized)
        })
        .collect()
}

fn summarize_body(body: &str) -> String {
    let summary = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if summary.chars().count() <= 500 {
        return summary;
    }

    summary.chars().take(500).collect()
}

fn github_reference(repository: Option<String>, issue_number: Option<u64>) -> Option<String> {
    let repository = repository?.trim().to_owned();
    if repository.is_empty() {
        return None;
    }

    Some(match issue_number {
        Some(issue_number) => format!("{repository}#{issue_number}"),
        None => repository,
    })
}

fn reject_sensitive(field: &'static str, value: &str) -> Result<(), IntakeValidationError> {
    let lower = value.to_ascii_lowercase();
    let sensitive_markers = [
        "authorization:",
        "bearer ",
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "private_key",
        "-----begin",
        "ghp_",
        "github_pat_",
        "sk-",
    ];

    if sensitive_markers
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(IntakeValidationError::SensitivePattern { field });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn working_group_id() -> WorkingGroupId {
        WorkingGroupId(Uuid::from_u128(1))
    }

    #[test]
    fn normalizes_manual_request_into_work_item_input() -> Result<(), IntakeValidationError> {
        let request = IntakeRequest {
            working_group_id: working_group_id(),
            mode: IntakeMode::Preview,
            input: IntakeSourcePayload::Manual {
                title: "  Build request intake ".to_owned(),
                body: "\nCreate a deterministic intake stub.\n".to_owned(),
                source_url: Some("https://example.test/request/1".to_owned()),
                acceptance_criteria: vec![" Preview returns normalized fields ".to_owned()],
                risk_tier_hint: Some(RiskTierHint::Medium),
            },
        };

        let input = request.into_work_item_input()?;

        assert_eq!(input.title, "Build request intake");
        assert_eq!(input.body_summary, "Create a deterministic intake stub.");
        assert_eq!(input.source, IntakeSourceKind::Manual);
        assert_eq!(
            input.acceptance_criteria,
            vec!["Preview returns normalized fields"]
        );
        Ok(())
    }

    #[test]
    fn records_github_issue_reference_without_external_integration()
    -> Result<(), IntakeValidationError> {
        let request = IntakeRequest {
            working_group_id: working_group_id(),
            mode: IntakeMode::Preview,
            input: IntakeSourcePayload::GithubIssue {
                title: "Issue-shaped request".to_owned(),
                body: "Use fixture payload only.".to_owned(),
                source_url: Some("https://github.com/example/repo/issues/42".to_owned()),
                acceptance_criteria: vec!["Creates a preview".to_owned()],
                risk_tier_hint: Some(RiskTierHint::Low),
                repository: Some("example/repo".to_owned()),
                issue_number: Some(42),
            },
        };

        let input = request.into_work_item_input()?;

        assert_eq!(input.source, IntakeSourceKind::GithubIssue);
        assert_eq!(input.external_reference, Some("example/repo#42".to_owned()));
        Ok(())
    }

    #[test]
    fn rejects_missing_acceptance_criteria() {
        let request = IntakeRequest {
            working_group_id: working_group_id(),
            mode: IntakeMode::Preview,
            input: IntakeSourcePayload::Manual {
                title: "Missing criteria".to_owned(),
                body: "Body".to_owned(),
                source_url: None,
                acceptance_criteria: vec![],
                risk_tier_hint: None,
            },
        };

        assert_eq!(
            request.into_work_item_input(),
            Err(IntakeValidationError::MissingAcceptanceCriteria)
        );
    }

    #[test]
    fn rejects_oversized_body() {
        let request = IntakeRequest {
            working_group_id: working_group_id(),
            mode: IntakeMode::Preview,
            input: IntakeSourcePayload::Manual {
                title: "Oversized body".to_owned(),
                body: "a".repeat(MAX_BODY_BYTES + 1),
                source_url: None,
                acceptance_criteria: vec!["Safe".to_owned()],
                risk_tier_hint: None,
            },
        };

        assert_eq!(
            request.into_work_item_input(),
            Err(IntakeValidationError::BodyTooLarge)
        );
    }

    #[test]
    fn rejects_unsupported_source_url() {
        let request = IntakeRequest {
            working_group_id: working_group_id(),
            mode: IntakeMode::Preview,
            input: IntakeSourcePayload::Manual {
                title: "Unsupported source".to_owned(),
                body: "Body".to_owned(),
                source_url: Some("file:///tmp/private.txt".to_owned()),
                acceptance_criteria: vec!["Safe".to_owned()],
                risk_tier_hint: None,
            },
        };

        assert_eq!(
            request.into_work_item_input(),
            Err(IntakeValidationError::UnsupportedSourceUrl)
        );
    }

    #[test]
    fn rejects_secret_shaped_input() {
        let request = IntakeRequest {
            working_group_id: working_group_id(),
            mode: IntakeMode::Preview,
            input: IntakeSourcePayload::GithubIssue {
                title: "Secret body".to_owned(),
                body: "Do not store Authorization: Bearer token".to_owned(),
                source_url: Some("https://github.com/example/repo/issues/7".to_owned()),
                acceptance_criteria: vec!["Safe".to_owned()],
                risk_tier_hint: Some(RiskTierHint::High),
                repository: Some("example/repo".to_owned()),
                issue_number: Some(7),
            },
        };

        assert_eq!(
            request.into_work_item_input(),
            Err(IntakeValidationError::SensitivePattern { field: "body" })
        );
    }
}
