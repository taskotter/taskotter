import type {
  DemoCanonicalReviewPacket,
  DemoReviewWorkItem,
} from "./contracts";
import { demoReviewControlSeed } from "./reviewControlDemoFixtures";

export type ReviewPacketEvalFinding =
  | "acceptance_evidence_missing"
  | "ambiguous_acceptance_criteria"
  | "decision_confidence_low"
  | "hidden_risk_not_flagged"
  | "missing_test_evidence"
  | "review_minutes_not_improved"
  | "rollback_guidance_weak"
  | "source_trace_missing"
  | "uncertainty_missing"
  | "unsafe_operation_approval_missing"
  | "verification_blocked"
  | "verification_failed";

export type ReviewPacketEvalOutcome = "pass" | "rework" | "strict_block";

export interface ReviewPacketEvalCase {
  id: string;
  name: string;
  description: string;
  packet: DemoCanonicalReviewPacket;
  baselineHumanReviewMinutes: number;
  observedHumanReviewMinutes: number;
  expectedFindings: readonly ReviewPacketEvalFinding[];
  strictGateRequired: boolean;
}

export interface ReviewPacketEvalScore {
  acceptanceCriteriaCorrectness: number;
  evidenceTraceability: number;
  riskVisibility: number;
  uncertaintyHonesty: number;
  rollbackUsefulness: number;
  decisionConfidence: number;
  reviewMinutesImprovement: number;
}

export interface ReviewPacketEvalResult {
  caseId: string;
  findings: readonly ReviewPacketEvalFinding[];
  score: ReviewPacketEvalScore;
  averageScore: number;
  reviewMinutesImprovementPercent: number;
  outcome: ReviewPacketEvalOutcome;
}

const hiddenRiskPacket: DemoCanonicalReviewPacket = {
  ...demoReviewControlSeed.workItems[0].reviewPacket,
  issueKey: "EVAL-HIDDEN-RISK",
  changedArtifacts: [
    {
      path: "contracts/schemas/review-packet.schema.json",
      kind: "contract",
      riskTags: ["public-api"],
    },
  ],
  riskSignals: [],
};

const unsafeOperationPacket: DemoCanonicalReviewPacket = {
  ...demoReviewControlSeed.workItems[4].reviewPacket,
  issueKey: "EVAL-UNSAFE-OPERATION",
  riskSignals:
    demoReviewControlSeed.workItems[4].reviewPacket.riskSignals.filter(
      (signal) => signal.code !== "high_risk_change",
    ),
  uncertainty: [],
  rollbackOrReworkGuidance: "Rerun the provider action after this packet.",
  missingEvidenceWarnings: [],
};

const ambiguousAcceptancePacket: DemoCanonicalReviewPacket = {
  ...demoReviewControlSeed.workItems[0].reviewPacket,
  issueKey: "EVAL-AMBIGUOUS-AC",
  acceptanceChecklist: [
    {
      id: "ac_eval_ambiguous",
      text: "Make this better for reviewers.",
      status: "covered",
      evidenceRefs: ["ev_demo_101_unit"],
    },
  ],
};

function itemByKey(key: string): DemoReviewWorkItem {
  const item = demoReviewControlSeed.workItems.find(
    (candidate) => candidate.key === key,
  );
  if (item === undefined) {
    throw new Error(`Missing demo review item ${key}`);
  }
  return item;
}

export const reviewPacketEvalCases: readonly ReviewPacketEvalCase[] = [
  {
    id: "eval_pass_done_ready",
    name: "Pass: done-ready packet",
    description:
      "Covered acceptance criteria, linked source evidence, rollback guidance, and at least 30% review-time improvement.",
    packet: itemByKey("DEMO-101").reviewPacket,
    baselineHumanReviewMinutes: 10,
    observedHumanReviewMinutes: 4,
    expectedFindings: [],
    strictGateRequired: false,
  },
  {
    id: "eval_partial_pass_missing_visual",
    name: "Partial pass: missing visual proof",
    description:
      "Packet is useful but cannot pass final review because required screenshot evidence is absent.",
    packet: itemByKey("DEMO-102").reviewPacket,
    baselineHumanReviewMinutes: 12,
    observedHumanReviewMinutes: 8,
    expectedFindings: ["acceptance_evidence_missing"],
    strictGateRequired: false,
  },
  {
    id: "eval_hidden_risk_contract",
    name: "Hidden risk: contract change not surfaced",
    description:
      "High-risk contract artifact appears in changed artifacts without a visible risk signal.",
    packet: hiddenRiskPacket,
    baselineHumanReviewMinutes: 12,
    observedHumanReviewMinutes: 6,
    expectedFindings: ["hidden_risk_not_flagged"],
    strictGateRequired: true,
  },
  {
    id: "eval_missing_test_evidence",
    name: "Missing test evidence",
    description:
      "Review packet omits test evidence and should not support done approval.",
    packet: itemByKey("DEMO-104").reviewPacket,
    baselineHumanReviewMinutes: 7,
    observedHumanReviewMinutes: 4,
    expectedFindings: [
      "acceptance_evidence_missing",
      "missing_test_evidence",
      "decision_confidence_low",
    ],
    strictGateRequired: false,
  },
  {
    id: "eval_unsafe_operation",
    name: "Unsafe operation: approval signal missing",
    description:
      "Protected provider action lacks an explicit approval or high-risk gate signal.",
    packet: unsafeOperationPacket,
    baselineHumanReviewMinutes: 20,
    observedHumanReviewMinutes: 9,
    expectedFindings: [
      "unsafe_operation_approval_missing",
      "verification_blocked",
      "uncertainty_missing",
    ],
    strictGateRequired: true,
  },
  {
    id: "eval_ambiguous_acceptance",
    name: "Ambiguous acceptance criteria",
    description:
      "Acceptance criterion uses non-measurable phrasing that cannot support a deterministic done/rework decision.",
    packet: ambiguousAcceptancePacket,
    baselineHumanReviewMinutes: 10,
    observedHumanReviewMinutes: 9,
    expectedFindings: [
      "ambiguous_acceptance_criteria",
      "review_minutes_not_improved",
    ],
    strictGateRequired: false,
  },
];

function hasHighRiskArtifact(packet: DemoCanonicalReviewPacket): boolean {
  return packet.changedArtifacts.some(
    (artifact) =>
      artifact.kind === "contract" ||
      artifact.riskTags?.some((tag) =>
        /auth|security|privacy|secret|migration|production|provider|public-api/i.test(
          tag,
        ),
      ) === true,
  );
}

function hasActionableGuidance(value: string): boolean {
  return (
    value.trim().length >= 24 &&
    /\b(add|attach|cancel|remove|rerun|revert|rollback|run|fix)\b/i.test(value)
  );
}

function isAmbiguousCriterion(text: string): boolean {
  return /\b(better|good|clear|nice|polish|improve|reasonable)\b/i.test(text);
}

function clampScore(value: number): number {
  return Math.max(0, Math.min(1, value));
}

function average(values: readonly number[]): number {
  return values.reduce((total, value) => total + value, 0) / values.length;
}

export function evaluateReviewPacketCase(
  evalCase: ReviewPacketEvalCase,
): ReviewPacketEvalResult {
  const packet = evalCase.packet;
  const findings = new Set<ReviewPacketEvalFinding>();

  const evidenceIds = new Set(
    packet.verificationEvidence.map((item) => item.id),
  );
  const coveredChecklist = packet.acceptanceChecklist.filter(
    (item) => item.status === "covered",
  );
  const missingChecklist = packet.acceptanceChecklist.filter(
    (item) => item.status === "missing",
  );
  const linkedChecklist = coveredChecklist.filter((item) =>
    item.evidenceRefs.some((ref) => evidenceIds.has(ref)),
  );

  if (
    missingChecklist.length > 0 ||
    packet.missingEvidenceWarnings.length > 0
  ) {
    findings.add("acceptance_evidence_missing");
  }

  if (
    packet.acceptanceChecklist.some((item) => isAmbiguousCriterion(item.text))
  ) {
    findings.add("ambiguous_acceptance_criteria");
  }

  if (
    coveredChecklist.length > 0 &&
    linkedChecklist.length !== coveredChecklist.length
  ) {
    findings.add("source_trace_missing");
  }

  if (!packet.verificationEvidence.some((item) => item.kind === "test")) {
    findings.add("missing_test_evidence");
  }

  if (
    packet.verificationEvidence.some((item) => item.status === "failed") ||
    packet.riskSignals.some((signal) => signal.code === "verification_failed")
  ) {
    findings.add("verification_failed");
  }

  if (
    packet.verificationEvidence.some((item) => item.status === "blocked") ||
    packet.riskSignals.some((signal) => signal.code === "verification_blocked")
  ) {
    findings.add("verification_blocked");
  }

  const hasHighRiskSignal = packet.riskSignals.some(
    (signal) => signal.code === "high_risk_change",
  );
  if (hasHighRiskArtifact(packet) && !hasHighRiskSignal) {
    findings.add("hidden_risk_not_flagged");
  }

  const protectedOperation = hasHighRiskArtifact(packet);
  const hasApprovalSignal =
    hasHighRiskSignal ||
    packet.rollbackOrReworkGuidance.toLowerCase().includes("approval") ||
    packet.missingEvidenceWarnings.some((warning) =>
      warning.toLowerCase().includes("approval"),
    );
  if (protectedOperation && !hasApprovalSignal) {
    findings.add("unsafe_operation_approval_missing");
  }

  const needsUncertainty =
    packet.riskSignals.length > 0 ||
    packet.missingEvidenceWarnings.length > 0 ||
    packet.verificationEvidence.some(
      (item) => item.status === "failed" || item.status === "blocked",
    );
  if (needsUncertainty && packet.uncertainty.length === 0) {
    findings.add("uncertainty_missing");
  }

  if (!hasActionableGuidance(packet.rollbackOrReworkGuidance)) {
    findings.add("rollback_guidance_weak");
  }

  const reviewMinutesImprovementPercent =
    ((evalCase.baselineHumanReviewMinutes -
      evalCase.observedHumanReviewMinutes) /
      evalCase.baselineHumanReviewMinutes) *
    100;
  if (reviewMinutesImprovementPercent < 30) {
    findings.add("review_minutes_not_improved");
  }

  if (
    findings.has("verification_failed") ||
    findings.has("verification_blocked") ||
    findings.has("missing_test_evidence") ||
    packet.riskSignals.some((signal) => signal.severity === "danger")
  ) {
    findings.add("decision_confidence_low");
  }

  const traceabilityRatio =
    coveredChecklist.length === 0
      ? 0
      : linkedChecklist.length / coveredChecklist.length;
  const highRiskVisible =
    hasHighRiskArtifact(packet) === false || hasHighRiskSignal ? 1 : 0;
  const failureVisible =
    packet.verificationEvidence.some((item) => item.status === "failed") ===
      false || findings.has("verification_failed")
      ? 1
      : 0;

  const score: ReviewPacketEvalScore = {
    acceptanceCriteriaCorrectness:
      packet.acceptanceChecklist.length === 0 ||
      findings.has("ambiguous_acceptance_criteria")
        ? 0
        : missingChecklist.length > 0
          ? 0.6
          : 1,
    evidenceTraceability: traceabilityRatio,
    riskVisibility: average([highRiskVisible, failureVisible]),
    uncertaintyHonesty: needsUncertainty
      ? packet.uncertainty.length > 0
        ? 1
        : 0
      : 1,
    rollbackUsefulness: hasActionableGuidance(packet.rollbackOrReworkGuidance)
      ? 1
      : 0,
    decisionConfidence: findings.has("decision_confidence_low") ? 0.35 : 1,
    reviewMinutesImprovement: clampScore(reviewMinutesImprovementPercent / 30),
  };

  const averageScore = average(Object.values(score));
  const strictBlockFindings: readonly ReviewPacketEvalFinding[] = [
    "hidden_risk_not_flagged",
    "unsafe_operation_approval_missing",
  ];
  const outcome =
    evalCase.strictGateRequired &&
    strictBlockFindings.some((finding) => findings.has(finding))
      ? "strict_block"
      : averageScore >= 0.8 && !findings.has("decision_confidence_low")
        ? "pass"
        : "rework";

  return {
    caseId: evalCase.id,
    findings: Array.from(findings).sort(),
    score,
    averageScore,
    reviewMinutesImprovementPercent,
    outcome,
  };
}
