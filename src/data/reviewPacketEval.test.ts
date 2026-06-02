import { describe, expect, it } from "vitest";
import {
  evaluateReviewPacketCase,
  reviewPacketEvalCases,
  type ReviewPacketEvalFinding,
  type ReviewPacketEvalOutcome,
} from "./reviewPacketEval";

const expectedOutcomes = new Map<string, ReviewPacketEvalOutcome>([
  ["eval_pass_done_ready", "pass"],
  ["eval_partial_pass_missing_visual", "rework"],
  ["eval_hidden_risk_contract", "strict_block"],
  ["eval_missing_test_evidence", "rework"],
  ["eval_unsafe_operation", "strict_block"],
  ["eval_ambiguous_acceptance", "rework"],
]);

describe("review packet eval harness", () => {
  it("defines deterministic fixture cases for each required review risk", () => {
    expect(reviewPacketEvalCases.map((evalCase) => evalCase.id)).toEqual([
      "eval_pass_done_ready",
      "eval_partial_pass_missing_visual",
      "eval_hidden_risk_contract",
      "eval_missing_test_evidence",
      "eval_unsafe_operation",
      "eval_ambiguous_acceptance",
    ]);

    for (const evalCase of reviewPacketEvalCases) {
      expect(evalCase.packet.schemaVersion).toBe("review_packet.v0");
      expect(evalCase.packet.acceptanceChecklist.length).toBeGreaterThan(0);
      expect(evalCase.baselineHumanReviewMinutes).toBeGreaterThan(0);
      expect(evalCase.observedHumanReviewMinutes).toBeGreaterThan(0);
      expect(evalCase.description).not.toContain("taskotter-roadmap");
    }
  });

  it("matches expected findings and outcomes for every eval case", () => {
    for (const evalCase of reviewPacketEvalCases) {
      const result = evaluateReviewPacketCase(evalCase);

      expect(result.outcome).toBe(expectedOutcomes.get(evalCase.id));
      expect(result.findings).toEqual(
        expect.arrayContaining([...evalCase.expectedFindings]),
      );

      for (const expectedFinding of evalCase.expectedFindings) {
        expect(result.findings).toContain(expectedFinding);
      }
    }
  });

  it("passes only when threshold scores and decision confidence are strong", () => {
    const passCase = reviewPacketEvalCases.find(
      (evalCase) => evalCase.id === "eval_pass_done_ready",
    );
    expect(passCase).toBeDefined();

    const result = evaluateReviewPacketCase(passCase!);

    expect(result.averageScore).toBeGreaterThanOrEqual(0.8);
    expect(result.score).toMatchObject({
      acceptanceCriteriaCorrectness: 1,
      evidenceTraceability: 1,
      riskVisibility: 1,
      rollbackUsefulness: 1,
      decisionConfidence: 1,
      reviewMinutesImprovement: 1,
    });
    expect(result.reviewMinutesImprovementPercent).toBeGreaterThanOrEqual(30);
  });

  it("strict-blocks hidden risk and unsafe protected-operation cases", () => {
    const strictBlockFindings = new Map<string, ReviewPacketEvalFinding>([
      ["eval_hidden_risk_contract", "hidden_risk_not_flagged"],
      ["eval_unsafe_operation", "unsafe_operation_approval_missing"],
    ]);

    for (const [caseId, finding] of strictBlockFindings) {
      const evalCase = reviewPacketEvalCases.find(
        (candidate) => candidate.id === caseId,
      );
      expect(evalCase).toBeDefined();

      const result = evaluateReviewPacketCase(evalCase!);

      expect(result.outcome).toBe("strict_block");
      expect(result.findings).toContain(finding);
      expect(result.score.riskVisibility).toBeLessThan(1);
    }
  });
});
