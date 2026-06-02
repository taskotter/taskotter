export type ReviewPacketSeverity = "info" | "warning" | "danger";

export type VerificationStatus = "passed" | "failed" | "not_run" | "blocked";

export type AcceptanceChecklistStatus = "accepted" | "unverified" | "failed";

export type ReviewDecisionPromptAction =
  | "approve_done"
  | "request_evidence"
  | "request_rework";

export type ReviewPacketEvidenceKind =
  | "test"
  | "lint"
  | "typecheck"
  | "build"
  | "review"
  | "runtime";

export interface ReviewPacketArtifactInput {
  path: string;
  kind: "source" | "test" | "fixture" | "contract" | "doc" | "config";
  summary?: string;
  riskTags?: readonly string[];
}

export interface AcceptanceCriterionInput {
  id: string;
  text: string;
  evidenceRefs?: readonly string[];
}

export interface VerificationEvidenceInput {
  id: string;
  kind: ReviewPacketEvidenceKind;
  status: VerificationStatus;
  summary: string;
  command?: string;
  artifactRefs?: readonly string[];
  correlationId?: string;
}

export interface ReviewPacketIssueRequestInput {
  source: "multica_issue" | "github_issue" | "manual_fixture";
  summary: string;
  requesterRef?: string;
}

export interface ReviewPacketPlanInput {
  summary: string;
  approvalRef?: string;
  approved: boolean;
}

export interface ReviewPacketRiskNoteInput {
  id: string;
  severity: ReviewPacketSeverity;
  summary: string;
  evidenceRefs?: readonly string[];
}

export interface ReviewPacketRollbackNoteInput {
  summary: string;
  artifactRefs?: readonly string[];
}

export interface ReviewPacketInput {
  schemaVersion?: "review_packet_fixture_input.v0";
  issueKey: string;
  title: string;
  issueRequest?: ReviewPacketIssueRequestInput;
  plan?: ReviewPacketPlanInput;
  changedArtifacts: readonly ReviewPacketArtifactInput[];
  acceptanceCriteria: readonly AcceptanceCriterionInput[];
  verificationEvidence: readonly VerificationEvidenceInput[];
  riskNotes?: readonly ReviewPacketRiskNoteInput[];
  rollbackNote?: ReviewPacketRollbackNoteInput;
  reworkRequested?: boolean;
  uncertaintyNotes?: readonly string[];
  rollbackHint?: string;
}

export interface ReviewPacketChecklistItem {
  id: string;
  text: string;
  status: AcceptanceChecklistStatus;
  evidenceRefs: readonly string[];
}

export interface ReviewPacketSignal {
  code:
    | "missing_acceptance_evidence"
    | "missing_tests"
    | "verification_failed"
    | "verification_blocked"
    | "high_risk_change"
    | "rework_requested";
  severity: ReviewPacketSeverity;
  message: string;
  evidenceRefs: readonly string[];
}

export interface ReviewPacketVerificationEvidence {
  id: string;
  kind: ReviewPacketEvidenceKind;
  status: VerificationStatus;
  summary: string;
  command?: string;
  artifactRefs: readonly string[];
  correlationId?: string;
}

export interface ReviewPacket {
  schemaVersion: "review_packet.v0";
  issueKey: string;
  sourceSchemaVersion: "review_packet_fixture_input.v0";
  summary: string;
  issueRequest?: ReviewPacketIssueRequestInput;
  plan?: ReviewPacketPlanInput;
  changedArtifacts: readonly ReviewPacketArtifactInput[];
  acceptanceChecklist: readonly ReviewPacketChecklistItem[];
  riskSignals: readonly ReviewPacketSignal[];
  uncertainty: readonly string[];
  rollbackOrReworkGuidance: string;
  rollbackNote?: ReviewPacketRollbackNoteInput;
  verificationEvidence: readonly ReviewPacketVerificationEvidence[];
  missingEvidenceWarnings: readonly string[];
  decisionPrompt: {
    recommendedAction: ReviewDecisionPromptAction;
    reasons: readonly string[];
    requiredFollowUps: readonly string[];
  };
  audit: {
    correlationIds: readonly string[];
    redactions: readonly string[];
  };
}

export interface ReviewPacketTextProvider {
  summarize(input: ReviewPacketInput): Promise<string>;
}

export class FakeReviewPacketTextProvider implements ReviewPacketTextProvider {
  async summarize(input: ReviewPacketInput): Promise<string> {
    return `${input.issueKey}: ${input.title}`;
  }
}

const sensitivePatterns: readonly { label: string; pattern: RegExp }[] = [
  { label: "bearer_token", pattern: /bearer\s+[a-z0-9._-]+/gi },
  {
    label: "api_key",
    pattern: /api[_-]?key\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "access_token",
    pattern: /access[_-]?token\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "refresh_token",
    pattern: /refresh[_-]?token\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "client_secret",
    pattern: /client[_-]?secret\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "password",
    pattern: /password\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "private_key",
    pattern:
      /-----BEGIN [^-]+ PRIVATE KEY-----[\s\S]*?-----END [^-]+ PRIVATE KEY-----/gi,
  },
  {
    label: "raw_prompt",
    pattern: /raw[_ -]?prompt\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "raw_log",
    pattern: /raw[_ -]?log\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "artifact_body",
    pattern: /artifact[_ -]?body\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "transcript_copy",
    pattern: /transcript[_ -]?copy\s*[:=]\s*["']?[^"',\s]+/gi,
  },
  {
    label: "raw_diff",
    pattern: /raw[_ -]?diff\s*[:=]\s*["']?[^"',\s]+/gi,
  },
];

function sanitizeText(value: string, redactions: Set<string>): string {
  let sanitized = value;

  for (const { label, pattern } of sensitivePatterns) {
    sanitized = sanitized.replace(pattern, () => {
      redactions.add(label);
      return "[redacted]";
    });
  }

  return sanitized;
}

function sanitizeStringList(
  values: readonly string[] | undefined,
  redactions: Set<string>,
): readonly string[] {
  return values?.map((value) => sanitizeText(value, redactions)) ?? [];
}

function sanitizeIssueRequest(
  issueRequest: ReviewPacketIssueRequestInput | undefined,
  redactions: Set<string>,
): ReviewPacketIssueRequestInput | undefined {
  if (issueRequest === undefined) {
    return undefined;
  }

  return {
    source: issueRequest.source,
    summary: sanitizeText(issueRequest.summary, redactions),
    requesterRef:
      issueRequest.requesterRef === undefined
        ? undefined
        : sanitizeText(issueRequest.requesterRef, redactions),
  };
}

function sanitizePlan(
  plan: ReviewPacketPlanInput | undefined,
  redactions: Set<string>,
): ReviewPacketPlanInput | undefined {
  if (plan === undefined) {
    return undefined;
  }

  return {
    summary: sanitizeText(plan.summary, redactions),
    approvalRef:
      plan.approvalRef === undefined
        ? undefined
        : sanitizeText(plan.approvalRef, redactions),
    approved: plan.approved,
  };
}

function sanitizeRollbackNote(
  rollbackNote: ReviewPacketRollbackNoteInput | undefined,
  redactions: Set<string>,
): ReviewPacketRollbackNoteInput | undefined {
  if (rollbackNote === undefined) {
    return undefined;
  }

  return {
    summary: sanitizeText(rollbackNote.summary, redactions),
    artifactRefs: sanitizeStringList(rollbackNote.artifactRefs, redactions),
  };
}

function isHighRiskArtifact(artifact: ReviewPacketArtifactInput): boolean {
  return (
    artifact.kind === "contract" ||
    artifact.riskTags?.some((tag) =>
      /auth|security|privacy|secret|migration|production|provider|public-api/i.test(
        tag,
      ),
    ) === true
  );
}

function evaluateAcceptance(
  criteria: readonly AcceptanceCriterionInput[],
  evidenceById: ReadonlyMap<string, ReviewPacketVerificationEvidence>,
  redactions: Set<string>,
): ReviewPacketChecklistItem[] {
  return criteria.map((criterion) => {
    const refs = sanitizeStringList(criterion.evidenceRefs, redactions).filter(
      (ref) => evidenceById.has(ref),
    );
    const linkedEvidence = refs.map((ref) => evidenceById.get(ref));
    const hasFailedEvidence = linkedEvidence.some(
      (item) => item?.status === "failed" || item?.status === "blocked",
    );
    const status =
      refs.length === 0
        ? "unverified"
        : hasFailedEvidence
          ? "failed"
          : "accepted";

    return {
      id: sanitizeText(criterion.id, redactions),
      text: sanitizeText(criterion.text, redactions),
      status,
      evidenceRefs: refs,
    };
  });
}

function buildRiskSignals(
  input: ReviewPacketInput,
  checklist: readonly ReviewPacketChecklistItem[],
  evidence: readonly ReviewPacketVerificationEvidence[],
  changedArtifacts: readonly ReviewPacketArtifactInput[],
  redactions: Set<string>,
): ReviewPacketSignal[] {
  const signals: ReviewPacketSignal[] = [];
  const testEvidence = evidence.filter((item) => item.kind === "test");

  const missingAcceptanceRefs = checklist
    .filter((item) => item.status === "unverified")
    .map((item) => item.id);
  if (missingAcceptanceRefs.length > 0) {
    signals.push({
      code: "missing_acceptance_evidence",
      severity: "warning",
      message: "Acceptance criteria are missing linked verification evidence.",
      evidenceRefs: missingAcceptanceRefs,
    });
  }

  if (testEvidence.length === 0) {
    signals.push({
      code: "missing_tests",
      severity: "warning",
      message: "No test evidence was provided for this packet.",
      evidenceRefs: [],
    });
  }

  for (const item of evidence) {
    if (item.status === "failed") {
      signals.push({
        code: "verification_failed",
        severity: "danger",
        message: `${item.kind} evidence failed.`,
        evidenceRefs: [item.id],
      });
    }

    if (item.status === "blocked") {
      signals.push({
        code: "verification_blocked",
        severity: "warning",
        message: `${item.kind} evidence is blocked.`,
        evidenceRefs: [item.id],
      });
    }
  }

  for (const note of input.riskNotes ?? []) {
    signals.push({
      code:
        note.severity === "danger" ? "rework_requested" : "high_risk_change",
      severity: note.severity,
      message: sanitizeText(note.summary, redactions),
      evidenceRefs: sanitizeStringList(note.evidenceRefs, redactions),
    });
  }

  const highRiskRefs = changedArtifacts
    .filter(isHighRiskArtifact)
    .map((artifact) => artifact.path);
  if (highRiskRefs.length > 0) {
    signals.push({
      code: "high_risk_change",
      severity: "warning",
      message: "High-risk artifacts or risk tags require reviewer attention.",
      evidenceRefs: highRiskRefs,
    });
  }

  if (input.reworkRequested === true) {
    signals.push({
      code: "rework_requested",
      severity: "danger",
      message: "Current evidence indicates rework is needed before approval.",
      evidenceRefs: evidence
        .filter((item) => item.status === "failed" || item.status === "blocked")
        .map((item) => item.id),
    });
  }

  return signals.map((signal) => ({
    ...signal,
    message: sanitizeText(signal.message, redactions),
    evidenceRefs: signal.evidenceRefs.map((ref) =>
      sanitizeText(ref, redactions),
    ),
  }));
}

function buildMissingEvidenceWarnings(
  checklist: readonly ReviewPacketChecklistItem[],
  evidence: readonly ReviewPacketVerificationEvidence[],
): readonly string[] {
  const warnings = checklist
    .filter((item) => item.status === "unverified")
    .map((item) => `Missing evidence for ${item.id}`);

  if (!evidence.some((item) => item.kind === "test")) {
    warnings.push("Missing test evidence");
  }

  return warnings;
}

function buildDecisionPrompt(
  checklist: readonly ReviewPacketChecklistItem[],
  riskSignals: readonly ReviewPacketSignal[],
  missingEvidenceWarnings: readonly string[],
): ReviewPacket["decisionPrompt"] {
  const hasReworkRisk = riskSignals.some(
    (signal) =>
      signal.severity === "danger" ||
      signal.code === "verification_failed" ||
      signal.code === "rework_requested",
  );
  const hasUnverifiedAcceptance = checklist.some(
    (item) => item.status === "unverified",
  );
  const hasFailedAcceptance = checklist.some(
    (item) => item.status === "failed",
  );

  if (hasReworkRisk || hasFailedAcceptance) {
    return {
      recommendedAction: "request_rework",
      reasons: riskSignals
        .filter((signal) => signal.severity === "danger")
        .map((signal) => signal.message),
      requiredFollowUps: [
        "Resolve failed or blocked evidence before marking the work done.",
      ],
    };
  }

  if (hasUnverifiedAcceptance || missingEvidenceWarnings.length > 0) {
    return {
      recommendedAction: "request_evidence",
      reasons: missingEvidenceWarnings,
      requiredFollowUps: [
        "Attach verification evidence for every acceptance criterion.",
      ],
    };
  }

  return {
    recommendedAction: "approve_done",
    reasons: ["All acceptance criteria have linked passing evidence."],
    requiredFollowUps: [],
  };
}

export async function generateReviewPacket(
  input: ReviewPacketInput,
  provider: ReviewPacketTextProvider = new FakeReviewPacketTextProvider(),
): Promise<ReviewPacket> {
  const redactions = new Set<string>();
  const issueRequest = sanitizeIssueRequest(input.issueRequest, redactions);
  const plan = sanitizePlan(input.plan, redactions);
  const rollbackNote = sanitizeRollbackNote(input.rollbackNote, redactions);
  const evidence = input.verificationEvidence.map((item) => ({
    id: sanitizeText(item.id, redactions),
    kind: item.kind,
    status: item.status,
    summary: sanitizeText(item.summary, redactions),
    command:
      item.command === undefined
        ? undefined
        : sanitizeText(item.command, redactions),
    artifactRefs: sanitizeStringList(item.artifactRefs, redactions),
    correlationId:
      item.correlationId === undefined
        ? undefined
        : sanitizeText(item.correlationId, redactions),
  }));
  const changedArtifacts = input.changedArtifacts.map((artifact) => ({
    path: sanitizeText(artifact.path, redactions),
    kind: artifact.kind,
    summary:
      artifact.summary === undefined
        ? undefined
        : sanitizeText(artifact.summary, redactions),
    riskTags: sanitizeStringList(artifact.riskTags, redactions),
  }));
  const acceptanceChecklist = evaluateAcceptance(
    input.acceptanceCriteria,
    new Map(evidence.map((item) => [item.id, item])),
    redactions,
  );
  const riskSignals = buildRiskSignals(
    input,
    acceptanceChecklist,
    evidence,
    changedArtifacts,
    redactions,
  );
  const uncertainty = sanitizeStringList(input.uncertaintyNotes, redactions);
  const summary = sanitizeText(await provider.summarize(input), redactions);
  const rollbackOrReworkGuidance = sanitizeText(
    input.rollbackHint ??
      (input.reworkRequested === true
        ? "Rework failed or blocked evidence before requesting approval."
        : "Use changed artifact paths and linked evidence IDs to revert or narrow follow-up work."),
    redactions,
  );
  const missingEvidenceWarnings = buildMissingEvidenceWarnings(
    acceptanceChecklist,
    evidence,
  );
  const decisionPrompt = buildDecisionPrompt(
    acceptanceChecklist,
    riskSignals,
    missingEvidenceWarnings,
  );

  return {
    schemaVersion: "review_packet.v0",
    issueKey: sanitizeText(input.issueKey, redactions),
    sourceSchemaVersion:
      input.schemaVersion ?? "review_packet_fixture_input.v0",
    summary,
    issueRequest,
    plan,
    changedArtifacts,
    acceptanceChecklist,
    riskSignals,
    uncertainty,
    rollbackOrReworkGuidance,
    rollbackNote,
    verificationEvidence: evidence,
    missingEvidenceWarnings,
    decisionPrompt,
    audit: {
      correlationIds: evidence
        .map((item) => item.correlationId)
        .filter((value): value is string => value !== undefined),
      redactions: Array.from(redactions).sort(),
    },
  };
}
