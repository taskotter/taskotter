use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::IssueId;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportAgentResultRequest {
    pub schema_version: String,
    pub work_item_id: IssueId,
    pub source_agent_run_ref: String,
    pub summary: String,
    #[serde(default)]
    pub changed_files: Vec<ChangedFileEvidence>,
    #[serde(default)]
    pub artifacts: Vec<ArtifactEvidence>,
    #[serde(default)]
    pub commands: Vec<CommandEvidence>,
    #[serde(default)]
    pub uncertainty: Option<String>,
    #[serde(default)]
    pub error_notes: Option<String>,
    #[serde(default)]
    pub retry_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangedFileEvidence {
    pub path: String,
    pub change_type: ChangedFileKind,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangedFileKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEvidence {
    pub artifact_ref: String,
    pub label: String,
    pub media_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CommandEvidence {
    pub command: String,
    pub outcome: CommandOutcome,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommandOutcome {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ImportedAgentResultEvidence {
    pub evidence_id: String,
    pub work_item_id: IssueId,
    pub source_agent_run_ref: String,
    pub summary: String,
    pub changed_files: Vec<ChangedFileEvidence>,
    pub artifacts: Vec<ArtifactEvidence>,
    pub commands: Vec<CommandEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_notes: Option<String>,
    pub redaction_summary: RedactionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct RedactionSummary {
    pub redacted: bool,
    pub redacted_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ReviewPacketEvidence {
    pub work_item_id: IssueId,
    pub evidence_id: String,
    pub summary: String,
    pub changed_files: Vec<ChangedFileEvidence>,
    pub artifacts: Vec<ArtifactEvidence>,
    pub commands: Vec<CommandEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_notes: Option<String>,
    pub timeline_event_id: String,
    pub audit_event_id: String,
    pub redaction_summary: RedactionSummary,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImportAgentResultError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("schema_version must be agent_result_import.v1")]
    UnsupportedSchemaVersion,
    #[error("at least one changed file, artifact, or command summary is required")]
    MissingEvidence,
    #[error("{field} contains an unsafe reference")]
    UnsafeReference { field: &'static str },
}

impl ImportAgentResultRequest {
    pub fn into_evidence(
        self,
    ) -> Result<(ImportedAgentResultEvidence, String, String), ImportAgentResultError> {
        if self.schema_version != "agent_result_import.v1" {
            return Err(ImportAgentResultError::UnsupportedSchemaVersion);
        }
        require_text("source_agent_run_ref", &self.source_agent_run_ref)?;
        reject_unsafe_reference("source_agent_run_ref", &self.source_agent_run_ref)?;
        require_text("summary", &self.summary)?;
        if self.changed_files.is_empty() && self.artifacts.is_empty() && self.commands.is_empty() {
            return Err(ImportAgentResultError::MissingEvidence);
        }

        let mut redacted_fields = Vec::new();
        let summary = sanitize_text("summary", self.summary, &mut redacted_fields);
        let changed_files = self
            .changed_files
            .into_iter()
            .enumerate()
            .map(|(index, file)| sanitize_changed_file(index, file, &mut redacted_fields))
            .collect::<Result<Vec<_>, _>>()?;
        let artifacts = self
            .artifacts
            .into_iter()
            .enumerate()
            .map(|(index, artifact)| sanitize_artifact(index, artifact, &mut redacted_fields))
            .collect::<Result<Vec<_>, _>>()?;
        let commands = self
            .commands
            .into_iter()
            .enumerate()
            .map(|(index, command)| sanitize_command(index, command, &mut redacted_fields))
            .collect::<Result<Vec<_>, _>>()?;
        let uncertainty = sanitize_optional("uncertainty", self.uncertainty, &mut redacted_fields);
        let error_notes = sanitize_optional("error_notes", self.error_notes, &mut redacted_fields);
        let retry_notes = sanitize_optional("retry_notes", self.retry_notes, &mut redacted_fields);
        let evidence_id = format!("evidence_{}", Uuid::new_v4());
        let timeline_event_id = format!("evt_{}", Uuid::new_v4());
        let audit_event_id = Uuid::new_v4().to_string();

        let evidence = ImportedAgentResultEvidence {
            evidence_id,
            work_item_id: self.work_item_id,
            source_agent_run_ref: self.source_agent_run_ref,
            summary,
            changed_files,
            artifacts,
            commands,
            uncertainty,
            error_notes,
            retry_notes,
            redaction_summary: RedactionSummary {
                redacted: !redacted_fields.is_empty(),
                redacted_fields,
            },
        };

        Ok((evidence, timeline_event_id, audit_event_id))
    }
}

impl ImportedAgentResultEvidence {
    pub fn to_review_packet(
        &self,
        timeline_event_id: String,
        audit_event_id: String,
    ) -> ReviewPacketEvidence {
        ReviewPacketEvidence {
            work_item_id: self.work_item_id,
            evidence_id: self.evidence_id.clone(),
            summary: self.summary.clone(),
            changed_files: self.changed_files.clone(),
            artifacts: self.artifacts.clone(),
            commands: self.commands.clone(),
            uncertainty: self.uncertainty.clone(),
            error_notes: self.error_notes.clone(),
            retry_notes: self.retry_notes.clone(),
            timeline_event_id,
            audit_event_id,
            redaction_summary: self.redaction_summary.clone(),
        }
    }
}

fn sanitize_changed_file(
    index: usize,
    file: ChangedFileEvidence,
    redacted_fields: &mut Vec<String>,
) -> Result<ChangedFileEvidence, ImportAgentResultError> {
    require_text("changed_files.path", &file.path)?;
    reject_unsafe_reference("changed_files.path", &file.path)?;
    Ok(ChangedFileEvidence {
        path: file.path,
        change_type: file.change_type,
        summary: sanitize_text(
            &format!("changed_files[{index}].summary"),
            file.summary,
            redacted_fields,
        ),
    })
}

fn sanitize_artifact(
    index: usize,
    artifact: ArtifactEvidence,
    redacted_fields: &mut Vec<String>,
) -> Result<ArtifactEvidence, ImportAgentResultError> {
    require_text("artifacts.artifact_ref", &artifact.artifact_ref)?;
    reject_unsafe_reference("artifacts.artifact_ref", &artifact.artifact_ref)?;
    require_text("artifacts.label", &artifact.label)?;
    require_text("artifacts.media_type", &artifact.media_type)?;
    require_allowed_media_type(&artifact.media_type)?;
    Ok(ArtifactEvidence {
        artifact_ref: artifact.artifact_ref,
        label: sanitize_text(
            &format!("artifacts[{index}].label"),
            artifact.label,
            redacted_fields,
        ),
        media_type: artifact.media_type,
    })
}

fn sanitize_command(
    index: usize,
    command: CommandEvidence,
    redacted_fields: &mut Vec<String>,
) -> Result<CommandEvidence, ImportAgentResultError> {
    require_text("commands.command", &command.command)?;
    require_text("commands.summary", &command.summary)?;
    Ok(CommandEvidence {
        command: sanitize_text(
            &format!("commands[{index}].command"),
            command.command,
            redacted_fields,
        ),
        outcome: command.outcome,
        summary: sanitize_text(
            &format!("commands[{index}].summary"),
            command.summary,
            redacted_fields,
        ),
    })
}

fn sanitize_optional(
    field: &str,
    value: Option<String>,
    redacted_fields: &mut Vec<String>,
) -> Option<String> {
    value.map(|value| sanitize_text(field, value, redacted_fields))
}

fn sanitize_text(field: &str, value: String, redacted_fields: &mut Vec<String>) -> String {
    if contains_sensitive_pattern(&value) {
        redacted_fields.push(field.to_owned());
        return "[redacted]".to_owned();
    }
    value
}

fn require_text(field: &'static str, value: &str) -> Result<(), ImportAgentResultError> {
    if value.trim().is_empty() {
        return Err(ImportAgentResultError::Required(field));
    }
    Ok(())
}

fn require_allowed_media_type(value: &str) -> Result<(), ImportAgentResultError> {
    reject_unsafe_reference("artifacts.media_type", value)?;
    let allowed = [
        "application/json",
        "text/plain",
        "text/markdown",
        "image/png",
        "image/jpeg",
    ];
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(ImportAgentResultError::UnsafeReference {
        field: "artifacts.media_type",
    })
}

fn reject_unsafe_reference(field: &'static str, value: &str) -> Result<(), ImportAgentResultError> {
    if contains_sensitive_pattern(value) {
        return Err(ImportAgentResultError::UnsafeReference { field });
    }
    Ok(())
}

fn contains_sensitive_pattern(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    let markers = [
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "private_key",
        "client_secret",
        "bearer ",
        "password",
        "raw_prompt",
        "raw_log",
        "artifact_body",
        "-----begin",
        "authorization:",
        "cookie:",
        "secret=",
        "token",
    ];
    markers.iter().any(|marker| normalized.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ImportAgentResultRequest {
        ImportAgentResultRequest {
            schema_version: "agent_result_import.v1".to_owned(),
            work_item_id: IssueId(Uuid::nil()),
            source_agent_run_ref: "run_01J9Z4P4BS0M9P2QJ6T8Z6W2EA".to_owned(),
            summary: "Implemented fixture import scaffold.".to_owned(),
            changed_files: vec![ChangedFileEvidence {
                path: "crates/taskotter-control-plane/src/agent_result.rs".to_owned(),
                change_type: ChangedFileKind::Added,
                summary: "Adds safe evidence mapping.".to_owned(),
            }],
            artifacts: Vec::new(),
            commands: vec![CommandEvidence {
                command: "cargo test -p taskotter-control-plane".to_owned(),
                outcome: CommandOutcome::Passed,
                summary: "Rust tests passed.".to_owned(),
            }],
            uncertainty: None,
            error_notes: None,
            retry_notes: None,
        }
    }

    #[test]
    fn maps_fixture_import_to_safe_evidence() -> Result<(), Box<dyn std::error::Error>> {
        let (evidence, timeline_event_id, audit_event_id) = request().into_evidence()?;

        assert!(evidence.evidence_id.starts_with("evidence_"));
        assert!(timeline_event_id.starts_with("evt_"));
        assert!(Uuid::parse_str(&audit_event_id).is_ok());
        assert_eq!(evidence.changed_files.len(), 1);
        assert_eq!(evidence.commands[0].outcome, CommandOutcome::Passed);
        assert!(!evidence.redaction_summary.redacted);
        Ok(())
    }

    #[test]
    fn redacts_sensitive_text_before_storage() -> Result<(), Box<dyn std::error::Error>> {
        let mut request = request();
        request.summary = "Found bearer token in local fixture.".to_owned();
        request.commands[0].summary = "raw_log path was omitted.".to_owned();

        let (evidence, _, _) = request.into_evidence()?;

        assert_eq!(evidence.summary, "[redacted]");
        assert_eq!(evidence.commands[0].summary, "[redacted]");
        assert!(evidence.redaction_summary.redacted);
        assert_eq!(
            evidence.redaction_summary.redacted_fields,
            vec!["summary", "commands[0].summary"]
        );
        Ok(())
    }

    #[test]
    fn rejects_import_without_review_evidence() {
        let mut request = request();
        request.changed_files.clear();
        request.commands.clear();

        assert_eq!(
            request.into_evidence(),
            Err(ImportAgentResultError::MissingEvidence)
        );
    }

    #[test]
    fn rejects_secret_shaped_artifact_references() {
        let mut request = request();
        request.artifacts.push(ArtifactEvidence {
            artifact_ref: "artifact_access_token_unsafe".to_owned(),
            label: "Unsafe artifact".to_owned(),
            media_type: "text/plain".to_owned(),
        });

        assert_eq!(
            request.into_evidence(),
            Err(ImportAgentResultError::UnsafeReference {
                field: "artifacts.artifact_ref"
            })
        );
    }

    #[test]
    fn rejects_secret_shaped_source_agent_run_refs_before_storage() {
        let mut request = request();
        request.source_agent_run_ref = "run_access_token_unsafe".to_owned();

        assert_eq!(
            request.into_evidence(),
            Err(ImportAgentResultError::UnsafeReference {
                field: "source_agent_run_ref"
            })
        );
    }

    #[test]
    fn rejects_unsafe_artifact_media_types_before_storage() {
        let mut request = request();
        request.artifacts.push(ArtifactEvidence {
            artifact_ref: "artifact_review_packet_fixture_002".to_owned(),
            label: "Unsafe media type".to_owned(),
            media_type: "text/html".to_owned(),
        });

        assert_eq!(
            request.into_evidence(),
            Err(ImportAgentResultError::UnsafeReference {
                field: "artifacts.media_type"
            })
        );
    }
}
