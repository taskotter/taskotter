import { describe, expect, it } from "vitest";
import type { DemoReviewScenario } from "./contracts";
import { demoReviewControlSeed } from "./reviewControlDemoFixtures";

const requiredScenarios = new Set<DemoReviewScenario>([
  "happy_path",
  "missing_evidence",
  "failed_test",
  "rework_requested",
  "high_risk_approval_required",
]);

const rawSecretPatterns = [
  /bearer\s+[a-z0-9._-]{12,}/i,
  /-----BEGIN [A-Z ]*PRIVATE KEY-----/,
  /sk-[a-z0-9_-]{12,}/i,
  /gh[pousr]_[a-z0-9_]{12,}/i,
  /xox[baprs]-[a-z0-9-]{12,}/i,
  /AKIA[0-9A-Z]{12,}/,
  /raw[_ -]?(prompt|log|artifact body)/i,
  /customer[_ -]?(email|name|data)/i,
];

describe("review control demo seed fixtures", () => {
  it("covers the prototype review scenarios with review packet fields", () => {
    expect(demoReviewControlSeed.fixtureId).toBe(
      "fixture_review_control_demo_seed_v1",
    );

    const scenarios = new Set(
      demoReviewControlSeed.workItems.map((item) => item.scenario),
    );
    expect(scenarios).toEqual(requiredScenarios);

    for (const item of demoReviewControlSeed.workItems) {
      expect(item.acceptanceCriteria.length).toBeGreaterThan(0);
      expect(item.reviewPacket.acceptanceChecklist).toEqual(
        item.acceptanceCriteria,
      );
      expect(item.importedEvidenceSummary.checks.length).toBeGreaterThan(0);
      expect(item.reviewPacket.changedFiles.length).toBeGreaterThan(0);
      expect(item.auditCorrelation.correlationId).toMatch(/^corr_demo_/);
      expect(item.auditCorrelation.requestId).toMatch(/^req_demo_/);
      expect(item.auditCorrelation.auditEventIds.length).toBeGreaterThan(0);
      expect(item.auditCorrelation.runTimelineEventIds.length).toBeGreaterThan(
        0,
      );
      expect(item.reviewTimeMetric.baselineSeconds).toBeGreaterThan(0);
      expect(item.reviewTimeMetric.observedSeconds).toBeGreaterThan(0);
      expect(item.redactionSafety).toMatchObject({
        dataClassification: "generated_fake",
        fakeOnly: true,
      });
    }
  });

  it("models incomplete and high-risk states without live integrations", () => {
    const byScenario = new Map(
      demoReviewControlSeed.workItems.map((item) => [item.scenario, item]),
    );

    expect(
      byScenario.get("missing_evidence")?.importedEvidenceSummary
        .missingEvidence,
    ).toEqual(["browser.screenshot.desktop", "browser.screenshot.mobile"]);
    expect(
      byScenario
        .get("failed_test")
        ?.importedEvidenceSummary.checks.some(
          (check) => check.status === "failed",
        ),
    ).toBe(true);
    expect(
      byScenario.get("rework_requested")?.reviewPacket.decisionRecommendation,
    ).toBe("request_rework");

    const highRisk = byScenario.get("high_risk_approval_required");
    expect(highRisk?.planApproval).toMatchObject({
      state: "pending",
      requiredBefore: "protected_side_effect",
    });
    expect(highRisk?.reviewPacket.decisionRecommendation).toBe(
      "manual_approval",
    );
    expect(highRisk?.redactionSafety.secretShapedCase).toMatchObject({
      displayValue: "[REDACTED_FAKE_SECRET_SHAPED_VALUE]",
      rawValueStored: false,
    });
  });

  it("stays generated-fake and redaction-safe", () => {
    const serialized = JSON.stringify(demoReviewControlSeed);

    for (const pattern of rawSecretPatterns) {
      expect(serialized).not.toMatch(pattern);
    }

    expect(serialized).toContain("[REDACTED_FAKE_SECRET_SHAPED_VALUE]");
    expect(serialized).not.toContain("taskotter-roadmap");
    expect(serialized).not.toContain("BEGIN ");
    expect(serialized).not.toContain("Authorization:");
    expect(serialized).not.toContain("Cookie:");
  });
});
