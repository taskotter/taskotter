import type {
  AcceptanceCriterionInput,
  ReviewPacketArtifactInput,
  ReviewPacketEvidenceKind,
  ReviewPacketInput,
  VerificationStatus,
} from "./reviewPacket";

export type AgentResultImportSourceType =
  | "manual_paste"
  | "uploaded_fixture"
  | "github_pr_link"
  | "local_cli_adapter_fixture";

export type AgentResultImportCommandOutcome = "passed" | "failed" | "skipped";

export interface ImportedAcceptanceCriterionContract {
  id: string;
  text: string;
  evidence_refs: readonly string[];
}

export interface ImportedChangedFileContract {
  path: string;
  change_type: "added" | "modified" | "deleted";
  summary: string;
}

export interface ImportedArtifactContract {
  artifact_ref: string;
  label: string;
  media_type: string;
}

export interface ImportedCommandContract {
  command: string;
  outcome: AgentResultImportCommandOutcome;
  summary: string;
}

export interface ImportedAgentResultContract {
  schema_version: "agent_result_import.v1";
  work_item_id: string;
  request_ref: string;
  source_type: AgentResultImportSourceType;
  source_agent_run_ref: string;
  plan_summary: string;
  summary: string;
  acceptance_criteria: readonly ImportedAcceptanceCriterionContract[];
  changed_files: readonly ImportedChangedFileContract[];
  artifacts: readonly ImportedArtifactContract[];
  commands: readonly ImportedCommandContract[];
  uncertainty?: string | null;
  error_notes?: string | null;
  retry_notes?: string | null;
  risk_notes?: string | null;
  rollback_notes?: string | null;
}

function commandOutcomeToStatus(
  outcome: AgentResultImportCommandOutcome,
): VerificationStatus {
  if (outcome === "failed") {
    return "failed";
  }
  if (outcome === "skipped") {
    return "not_run";
  }
  return "passed";
}

function commandKind(command: string): ReviewPacketEvidenceKind {
  if (/\b(format|lint|eslint|prettier)\b/i.test(command)) {
    return "lint";
  }
  if (/\b(typecheck|tsc)\b/i.test(command)) {
    return "typecheck";
  }
  if (/\b(build|vite build|cargo build)\b/i.test(command)) {
    return "build";
  }
  if (/\b(test|vitest|cargo test|playwright)\b/i.test(command)) {
    return "test";
  }
  return "review";
}

function artifactKind(path: string): ReviewPacketArtifactInput["kind"] {
  if (/\.(test|spec)\./.test(path) || path.includes("/tests/")) {
    return "test";
  }
  if (path.startsWith("contracts/")) {
    return path.includes("/fixtures/") ? "fixture" : "contract";
  }
  if (path.startsWith("docs/")) {
    return "doc";
  }
  if (/\.(json|ya?ml|toml|config\.)/.test(path)) {
    return "config";
  }
  return "source";
}

function riskTagsForImport(
  importResult: ImportedAgentResultContract,
): readonly string[] {
  const tags = ["agent-result-import", importResult.source_type];
  if (
    importResult.risk_notes !== undefined &&
    importResult.risk_notes !== null
  ) {
    tags.push("review-control-contract");
  }
  return tags;
}

export function agentResultImportToReviewPacketInput(
  importResult: ImportedAgentResultContract,
  issueTitle: string,
): ReviewPacketInput {
  const evidenceIds = new Set<string>();
  const changedArtifacts = importResult.changed_files.map((file) => ({
    path: file.path,
    kind: artifactKind(file.path),
    summary: file.summary,
    riskTags: riskTagsForImport(importResult),
  }));
  const artifactEvidence = importResult.artifacts.map((artifact) => {
    const id = `artifact:${artifact.artifact_ref}`;
    evidenceIds.add(id);
    return {
      id,
      kind: "review" as const,
      status: "passed" as const,
      summary: artifact.label,
      artifactRefs: [artifact.artifact_ref],
      correlationId: importResult.request_ref,
    };
  });
  const commandEvidence = importResult.commands.map((command, index) => {
    const id = `command:${index + 1}`;
    evidenceIds.add(id);
    return {
      id,
      kind: commandKind(command.command),
      status: commandOutcomeToStatus(command.outcome),
      summary: command.summary,
      command: command.command,
      artifactRefs: importResult.artifacts.map(
        (artifact) => artifact.artifact_ref,
      ),
      correlationId: importResult.source_agent_run_ref,
    };
  });
  const acceptanceCriteria: AcceptanceCriterionInput[] =
    importResult.acceptance_criteria.map((criterion) => ({
      id: criterion.id,
      text: criterion.text,
      evidenceRefs: criterion.evidence_refs.filter((ref) =>
        evidenceIds.has(ref),
      ),
    }));
  const uncertaintyNotes = [
    importResult.plan_summary,
    importResult.summary,
    importResult.uncertainty,
    importResult.error_notes,
    importResult.retry_notes,
    importResult.risk_notes,
  ].filter(
    (value): value is string => typeof value === "string" && value.length > 0,
  );

  return {
    issueKey: importResult.request_ref,
    title: issueTitle,
    changedArtifacts,
    acceptanceCriteria,
    verificationEvidence: [...artifactEvidence, ...commandEvidence],
    uncertaintyNotes,
    rollbackHint: importResult.rollback_notes ?? undefined,
  };
}
