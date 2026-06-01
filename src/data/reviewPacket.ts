export type ReviewPacketSeverity = "info" | "warning" | "danger";

export type VerificationStatus = "passed" | "failed" | "not_run" | "blocked";

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

export interface ReviewPacketInput {
  issueKey: string;
  title: string;
  changedArtifacts: readonly ReviewPacketArtifactInput[];
  acceptanceCriteria: readonly AcceptanceCriterionInput[];
  verificationEvidence: readonly VerificationEvidenceInput[];
  reworkRequested?: boolean;
  uncertaintyNotes?: readonly string[];
  rollbackHint?: string;
}

export interface ReviewPacketChecklistItem {
  id: string;
  text: string;
  status: "covered" | "missing";
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
  summary: string;
  changedArtifacts: readonly ReviewPacketArtifactInput[];
  acceptanceChecklist: readonly ReviewPacketChecklistItem[];
  riskSignals: readonly ReviewPacketSignal[];
  uncertainty: readonly string[];
  rollbackOrReworkGuidance: string;
  verificationEvidence: readonly ReviewPacketVerificationEvidence[];
  missingEvidenceWarnings: readonly string[];
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
  evidenceIds: Set<string>,
  redactions: Set<string>,
): ReviewPacketChecklistItem[] {
  return criteria.map((criterion) => {
    const refs = sanitizeStringList(criterion.evidenceRefs, redactions).filter(
      (ref) => evidenceIds.has(ref),
    );

    return {
      id: sanitizeText(criterion.id, redactions),
      text: sanitizeText(criterion.text, redactions),
      status: refs.length > 0 ? "covered" : "missing",
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
    .filter((item) => item.status === "missing")
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
    .filter((item) => item.status === "missing")
    .map((item) => `Missing evidence for ${item.id}`);

  if (!evidence.some((item) => item.kind === "test")) {
    warnings.push("Missing test evidence");
  }

  return warnings;
}

export async function generateReviewPacket(
  input: ReviewPacketInput,
  provider: ReviewPacketTextProvider = new FakeReviewPacketTextProvider(),
): Promise<ReviewPacket> {
  const redactions = new Set<string>();
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
  const evidenceIds = new Set(evidence.map((item) => item.id));
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
    evidenceIds,
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

  return {
    schemaVersion: "review_packet.v0",
    issueKey: sanitizeText(input.issueKey, redactions),
    summary,
    changedArtifacts,
    acceptanceChecklist,
    riskSignals,
    uncertainty,
    rollbackOrReworkGuidance,
    verificationEvidence: evidence,
    missingEvidenceWarnings,
    audit: {
      correlationIds: evidence
        .map((item) => item.correlationId)
        .filter((value): value is string => value !== undefined),
      redactions: Array.from(redactions).sort(),
    },
  };
}
