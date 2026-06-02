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
      expect(item.reviewPacket.schemaVersion).toBe("review_packet.v0");
      expect(item.reviewPacket.sourceSchemaVersion).toBe(
        "review_packet_fixture_input.v0",
      );
      expect(item.reviewPacket.issueKey).toBe(item.key);
      expect(item.reviewPacket.changedArtifacts.length).toBeGreaterThan(0);
      expect(item.reviewPacket.acceptanceChecklist).toEqual(
        item.acceptanceCriteria.map((criterion) => ({
          id: criterion.id,
          text: criterion.text,
          status: criterion.satisfied ? "accepted" : "unverified",
          evidenceRefs: criterion.satisfied
            ? item.reviewPacket.verificationEvidence.map(
                (evidence) => evidence.id,
              )
            : [],
        })),
      );
      expect(item.reviewPacket.decisionPrompt.recommendedAction).toMatch(
        /approve_done|request_evidence|request_rework/,
      );
      expect(item.importedEvidenceSummary.checks.length).toBeGreaterThan(0);
      expect(item.reviewPacket.audit.correlationIds).toContain(
        item.demoAuditChainSummary.correlationId,
      );
      expect(item.demoAuditChainSummary.correlationId).toMatch(/^corr_demo_/);
      expect(item.demoAuditChainSummary.requestId).toMatch(/^req_demo_/);
      expect(item.demoAuditChainSummary.evidenceImportId).toMatch(
        /^evimp_demo_/,
      );
      expect(item.demoAuditChainSummary.reviewPacketId).toMatch(/^rvpkt_demo_/);
      expect(item.demoAuditChainSummary.eventIds.length).toBeGreaterThan(0);
      expect(item.demoReviewTimeSummaryMetric.source).toBe(
        "demo_summary_metric_not_event_telemetry",
      );
      expect(item.demoReviewTimeSummaryMetric.derivedFromEventTelemetry).toBe(
        false,
      );
      expect(
        item.demoReviewTimeSummaryMetric.baselineHumanReviewMinutes,
      ).toBeGreaterThan(0);
      expect(
        item.demoReviewTimeSummaryMetric.humanReviewMinutes,
      ).toBeGreaterThan(0);
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
      byScenario
        .get("rework_requested")
        ?.reviewPacket.riskSignals.some(
          (signal) => signal.code === "rework_requested",
        ),
    ).toBe(true);

    const highRisk = byScenario.get("high_risk_approval_required");
    expect(highRisk?.planApproval).toMatchObject({
      state: "pending",
      requiredBefore: "protected_side_effect",
    });
    expect(
      highRisk?.reviewPacket.riskSignals.some(
        (signal) =>
          signal.code === "high_risk_change" &&
          signal.message.includes("manual approval") &&
          signal.evidenceRefs.includes("ev_demo_105_policy_gate"),
      ),
    ).toBe(true);
    expect(highRisk?.demoAuditChainSummary).toMatchObject({
      workflowPath: "denied",
      approvalId: "appr_demo_105_protected_side_effect",
      policyDecisionId: "poldec_demo_105_requires_approval",
    });
    expect(highRisk?.demoReviewTimeSummaryMetric).toMatchObject({
      source: "demo_summary_metric_not_event_telemetry",
      derivedFromEventTelemetry: false,
      missingStopEvents: 1,
    });
    expect(highRisk?.reviewPacket.missingEvidenceWarnings).toContain(
      "Missing security.manual-approval-decision",
    );
    expect(highRisk?.reviewPacket.audit.redactions).toContain(
      "fake_secret_shaped_placeholder",
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
    expect(serialized).toContain("review_packet.v0");
    expect(serialized).toContain("demo_summary_metric_not_event_telemetry");
    expect(serialized).not.toContain("taskotter-roadmap");
    expect(serialized).not.toContain("BEGIN ");
    expect(serialized).not.toContain("Authorization:");
    expect(serialized).not.toContain("Cookie:");
    expect(serialized).not.toContain("human_review_minutes");

    for (const item of demoReviewControlSeed.workItems) {
      expect(item.reviewPacket).not.toHaveProperty("changedFiles");
      expect(item.reviewPacket).not.toHaveProperty("rollbackGuidance");
      expect(item.reviewPacket).not.toHaveProperty("reworkGuidance");
      expect(item.reviewPacket).not.toHaveProperty("decisionRecommendation");
      expect(item).not.toHaveProperty("reviewTimeMetric");
    }
  });
});
