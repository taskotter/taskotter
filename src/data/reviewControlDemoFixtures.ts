import type {
  DemoAcceptanceCriterion,
  DemoCanonicalReviewPacket,
  DemoReviewControlSeed,
  DemoReviewPacketSignal,
  DemoReviewWorkItem,
} from "./contracts";

const generatedAt = "2026-06-01T18:00:00.000Z";

function criteria(
  values: readonly [id: string, text: string, satisfied: boolean][],
): DemoAcceptanceCriterion[] {
  return values.map(([id, text, satisfied]) => ({
    id,
    text,
    required: true,
    satisfied,
  }));
}

function packet({
  issueKey,
  summary,
  changedArtifacts,
  acceptanceCriteria,
  verificationEvidence,
  riskSignals = [],
  uncertainty = [],
  rollbackOrReworkGuidance,
  missingEvidenceWarnings = [],
  redactions = [],
}: {
  issueKey: string;
  summary: string;
  changedArtifacts: DemoCanonicalReviewPacket["changedArtifacts"];
  acceptanceCriteria: DemoAcceptanceCriterion[];
  verificationEvidence: DemoCanonicalReviewPacket["verificationEvidence"];
  riskSignals?: readonly DemoReviewPacketSignal[];
  uncertainty?: string[];
  rollbackOrReworkGuidance: string;
  missingEvidenceWarnings?: readonly string[];
  redactions?: readonly string[];
}): DemoCanonicalReviewPacket {
  return {
    schemaVersion: "review_packet.v0",
    issueKey,
    summary,
    changedArtifacts,
    acceptanceChecklist: acceptanceCriteria.map((criterion) => ({
      id: criterion.id,
      text: criterion.text,
      status: criterion.satisfied ? "covered" : "missing",
      evidenceRefs: criterion.satisfied
        ? verificationEvidence.map((evidence) => evidence.id)
        : [],
    })),
    riskSignals,
    uncertainty,
    rollbackOrReworkGuidance,
    verificationEvidence,
    missingEvidenceWarnings,
    audit: {
      correlationIds: verificationEvidence
        .map((evidence) => evidence.correlationId)
        .filter((value): value is string => value !== undefined),
      redactions,
    },
  };
}

function workItem(
  item: Omit<DemoReviewWorkItem, "redactionSafety"> & {
    redactionSafety?: DemoReviewWorkItem["redactionSafety"];
  },
): DemoReviewWorkItem {
  return {
    ...item,
    redactionSafety: item.redactionSafety ?? {
      dataClassification: "generated_fake",
      fakeOnly: true,
    },
  };
}

const happyPathCriteria = criteria([
  [
    "ac_demo_101_1",
    "Issue summary renders status, assignee, and acceptance checklist.",
    true,
  ],
  [
    "ac_demo_101_2",
    "Reviewer can mark the packet done without live integrations.",
    true,
  ],
]);

const missingEvidenceCriteria = criteria([
  ["ac_demo_102_1", "Review packet lists the missing visual proof.", false],
]);

const failedTestCriteria = criteria([
  [
    "ac_demo_103_1",
    "Fixture validation passes before review can approve done.",
    false,
  ],
]);

const reworkCriteria = criteria([
  [
    "ac_demo_104_1",
    "Review packet explains concrete rework before done.",
    false,
  ],
]);

const highRiskCriteria = criteria([
  ["ac_demo_105_1", "Protected side effect waits for manual approval.", true],
  [
    "ac_demo_105_2",
    "Secret-shaped evidence is redacted and marked generated fake.",
    true,
  ],
]);

export const demoReviewControlSeed: DemoReviewControlSeed = {
  fixtureId: "fixture_review_control_demo_seed_v1",
  generatedAt,
  description:
    "Generated fake review-control sample work items for local prototype and QA flows.",
  workItems: [
    workItem({
      scenario: "happy_path",
      id: "demo_work_item_001",
      key: "DEMO-101",
      title: "Review generated issue summary panel",
      status: "in_review",
      riskTier: "low",
      assignee: "Frontend Implementation Engineer",
      requestSource: "github_issue",
      acceptanceCriteria: happyPathCriteria,
      planApproval: {
        state: "approved",
        requiredBefore: "agent_start",
        approvalRef: "approval_demo_101_plan",
        approvedBy: "human_reviewer_demo",
        approvedAt: "2026-06-01T17:22:00.000Z",
        rationale: "Low-risk UI fixture change with bounded local data.",
      },
      importedEvidenceSummary: {
        status: "complete",
        sourceRunId: "run_demo_101_agent_result",
        importedAt: "2026-06-01T17:44:00.000Z",
        changedFilesCount: 3,
        artifactCount: 2,
        checks: [
          {
            name: "unit.fixture-render",
            status: "passed",
            summary: "Fixture-backed review panel rendered expected sections.",
            artifactRef: "artifact_demo_101_unit_log",
          },
          {
            name: "static.redaction-scan",
            status: "passed",
            summary: "No raw credentials or private logs detected.",
            artifactRef: "artifact_demo_101_redaction_scan",
          },
        ],
      },
      reviewPacket: packet({
        issueKey: "DEMO-101",
        summary: "DEMO-101: Review generated issue summary panel",
        changedArtifacts: [
          {
            path: "src/data/reviewControlDemoFixtures.ts",
            kind: "fixture",
            summary: "Generated fake review control seed fixture.",
          },
          {
            path: "src/data/contracts.ts",
            kind: "source",
            summary: "Typed demo fixture surface.",
          },
        ],
        acceptanceCriteria: happyPathCriteria,
        verificationEvidence: [
          {
            id: "ev_demo_101_unit",
            kind: "test",
            status: "passed",
            summary: "Fixture-backed review panel rendered expected sections.",
            command: "npm run test:unit",
            artifactRefs: ["src/data/reviewControlDemoFixtures.test.ts"],
            correlationId: "corr_demo_101_review",
          },
        ],
        uncertainty: ["Design polish pending UX review"],
        rollbackOrReworkGuidance:
          "Remove the demo seed export and adapter property.",
      }),
      demoAuditChainSummary: {
        correlationId: "corr_demo_101_review",
        requestId: "req_demo_101_plan",
        policyDecisionId: "poldec_demo_101_allow",
        approvalId: "appr_demo_101_plan",
        evidenceImportId: "evimp_demo_101",
        reviewPacketId: "rvpkt_demo_101",
        doneDecisionId: "rvdec_demo_101_done",
        workflowPath: "done_approved",
        eventIds: [
          "evt_demo_101_plan_approved",
          "evt_demo_101_evidence_imported",
          "evt_demo_101_packet_generated",
          "evt_demo_101_human_decision_done",
        ],
      },
      demoReviewTimeSummaryMetric: {
        source: "demo_summary_metric_not_event_telemetry",
        derivedFromEventTelemetry: false,
        baselineHumanReviewMinutes: 10,
        humanReviewMinutes: 4,
        humanMinutesPerCompletedAgentTask: 4,
        completedAgentTasks: 1,
        reworkLoops: 0,
        missingStopEvents: 0,
        reviewerRole: "human_reviewer",
      },
    }),
    workItem({
      scenario: "missing_evidence",
      id: "demo_work_item_002",
      key: "DEMO-102",
      title: "Review run without screenshot evidence",
      status: "blocked",
      riskTier: "medium",
      assignee: "QA Test Engineer",
      requestSource: "multica_issue",
      acceptanceCriteria: missingEvidenceCriteria,
      planApproval: {
        state: "approved",
        requiredBefore: "agent_start",
        approvalRef: "approval_demo_102_plan",
        approvedBy: "delivery_lead_demo",
        approvedAt: "2026-06-01T17:10:00.000Z",
        rationale:
          "Plan is safe, but review cannot finish until evidence is attached.",
      },
      importedEvidenceSummary: {
        status: "partial",
        sourceRunId: "run_demo_102_agent_result",
        importedAt: "2026-06-01T17:36:00.000Z",
        changedFilesCount: 2,
        artifactCount: 1,
        checks: [
          {
            name: "unit.review-packet",
            status: "passed",
            summary: "Packet generated from available fake evidence.",
            artifactRef: "artifact_demo_102_unit_log",
          },
          {
            name: "browser.screenshot",
            status: "missing",
            summary: "Expected prototype screenshot artifact was not imported.",
          },
        ],
        missingEvidence: [
          "browser.screenshot.desktop",
          "browser.screenshot.mobile",
        ],
      },
      reviewPacket: packet({
        issueKey: "DEMO-102",
        summary: "DEMO-102: Review run without screenshot evidence",
        changedArtifacts: [
          { path: "src/App.tsx", kind: "source" },
          { path: "src/styles.css", kind: "source" },
        ],
        acceptanceCriteria: missingEvidenceCriteria,
        verificationEvidence: [
          {
            id: "ev_demo_102_unit",
            kind: "test",
            status: "passed",
            summary: "Packet generated from available fake evidence.",
            artifactRefs: ["artifact_demo_102_unit_log"],
            correlationId: "corr_demo_102_review",
          },
        ],
        riskSignals: [
          {
            code: "missing_acceptance_evidence",
            severity: "warning",
            message:
              "Acceptance criteria are missing linked verification evidence.",
            evidenceRefs: ["ac_demo_102_1"],
          },
        ],
        uncertainty: ["Responsive layout has not been visually verified."],
        rollbackOrReworkGuidance:
          "Run the browser smoke flow and attach desktop and mobile screenshots.",
        missingEvidenceWarnings: [
          "Missing evidence for ac_demo_102_1",
          "Missing browser.screenshot.desktop",
          "Missing browser.screenshot.mobile",
        ],
      }),
      demoAuditChainSummary: {
        correlationId: "corr_demo_102_review",
        requestId: "req_demo_102_packet",
        policyDecisionId: "poldec_demo_102_allow",
        approvalId: "appr_demo_102_plan",
        evidenceImportId: "evimp_demo_102",
        reviewPacketId: "rvpkt_demo_102",
        reworkDecisionId: "rvdec_demo_102_rework",
        workflowPath: "missing_evidence",
        eventIds: [
          "evt_demo_102_evidence_missing",
          "evt_demo_102_packet_generated",
        ],
      },
      demoReviewTimeSummaryMetric: {
        source: "demo_summary_metric_not_event_telemetry",
        derivedFromEventTelemetry: false,
        baselineHumanReviewMinutes: 12,
        humanReviewMinutes: 8,
        completedAgentTasks: 0,
        reworkLoops: 1,
        missingStopEvents: 0,
        reviewerRole: "qa_agent",
      },
    }),
    workItem({
      scenario: "failed_test",
      id: "demo_work_item_003",
      key: "DEMO-103",
      title: "Review adapter change with failing fixture test",
      status: "blocked",
      riskTier: "medium",
      assignee: "Frontend Implementation Engineer",
      requestSource: "github_issue",
      acceptanceCriteria: failedTestCriteria,
      planApproval: {
        state: "approved",
        requiredBefore: "agent_start",
        approvalRef: "approval_demo_103_plan",
        approvedBy: "delivery_lead_demo",
        approvedAt: "2026-06-01T16:56:00.000Z",
        rationale:
          "Adapter work is allowed, but failed checks must route to rework.",
      },
      importedEvidenceSummary: {
        status: "failed",
        sourceRunId: "run_demo_103_agent_result",
        importedAt: "2026-06-01T17:18:00.000Z",
        changedFilesCount: 4,
        artifactCount: 2,
        checks: [
          {
            name: "npm run test:fixtures",
            status: "failed",
            summary: "Fixture validation rejected an incomplete review packet.",
            artifactRef: "artifact_demo_103_fixture_log",
          },
          {
            name: "npm run typecheck",
            status: "passed",
            summary: "TypeScript accepted the adapter contract.",
            artifactRef: "artifact_demo_103_typecheck_log",
          },
        ],
      },
      reviewPacket: packet({
        issueKey: "DEMO-103",
        summary: "DEMO-103: Review adapter change with failing fixture test",
        changedArtifacts: [
          { path: "src/data/taskotterAdapter.ts", kind: "source" },
          { path: "tests/contracts/fixtures.test.mjs", kind: "test" },
        ],
        acceptanceCriteria: failedTestCriteria,
        verificationEvidence: [
          {
            id: "ev_demo_103_fixture",
            kind: "test",
            status: "failed",
            summary: "Fixture validation rejected an incomplete review packet.",
            command: "npm run test:fixtures",
            artifactRefs: ["artifact_demo_103_fixture_log"],
            correlationId: "corr_demo_103_review",
          },
        ],
        riskSignals: [
          {
            code: "verification_failed",
            severity: "danger",
            message: "test evidence failed.",
            evidenceRefs: ["ev_demo_103_fixture"],
          },
        ],
        uncertainty: [
          "Review packet completeness cannot be trusted until fixed.",
        ],
        rollbackOrReworkGuidance:
          "Add the missing review packet fields and rerun fixture validation.",
        missingEvidenceWarnings: ["Missing evidence for ac_demo_103_1"],
      }),
      demoAuditChainSummary: {
        correlationId: "corr_demo_103_review",
        requestId: "req_demo_103_packet",
        policyDecisionId: "poldec_demo_103_allow",
        approvalId: "appr_demo_103_plan",
        evidenceImportId: "evimp_demo_103",
        reviewPacketId: "rvpkt_demo_103",
        reworkDecisionId: "rvdec_demo_103_rework",
        workflowPath: "rework_requested",
        eventIds: ["evt_demo_103_check_failed", "evt_demo_103_rework"],
      },
      demoReviewTimeSummaryMetric: {
        source: "demo_summary_metric_not_event_telemetry",
        derivedFromEventTelemetry: false,
        baselineHumanReviewMinutes: 15,
        humanReviewMinutes: 5,
        completedAgentTasks: 0,
        reworkLoops: 1,
        missingStopEvents: 0,
        reviewerRole: "qa_agent",
      },
    }),
    workItem({
      scenario: "rework_requested",
      id: "demo_work_item_004",
      key: "DEMO-104",
      title: "Review copy update that needs clearer rollback guidance",
      status: "in_review",
      riskTier: "low",
      assignee: "Frontend Delivery Lead",
      requestSource: "manual_request",
      acceptanceCriteria: reworkCriteria,
      planApproval: {
        state: "not_required",
        requiredBefore: "not_required",
        rationale:
          "Copy-only fake fixture scenario has no protected side effect.",
      },
      importedEvidenceSummary: {
        status: "complete",
        sourceRunId: "run_demo_104_agent_result",
        importedAt: "2026-06-01T17:05:00.000Z",
        changedFilesCount: 1,
        artifactCount: 1,
        checks: [
          {
            name: "copy.static-review",
            status: "passed",
            summary: "No private roadmap text or client records found.",
            artifactRef: "artifact_demo_104_copy_scan",
          },
        ],
      },
      reviewPacket: packet({
        issueKey: "DEMO-104",
        summary: "DEMO-104: Review copy update needing rework guidance",
        changedArtifacts: [
          { path: "src/i18n/resources/en/issues.ts", kind: "source" },
        ],
        acceptanceCriteria: reworkCriteria,
        verificationEvidence: [
          {
            id: "ev_demo_104_copy",
            kind: "review",
            status: "passed",
            summary: "No private roadmap text or client records found.",
            artifactRefs: ["artifact_demo_104_copy_scan"],
            correlationId: "corr_demo_104_review",
          },
        ],
        riskSignals: [
          {
            code: "rework_requested",
            severity: "danger",
            message:
              "Current evidence indicates rework is needed before approval.",
            evidenceRefs: ["ac_demo_104_1"],
          },
        ],
        uncertainty: [
          "Rollback instruction is too broad for a future reviewer.",
        ],
        rollbackOrReworkGuidance:
          "Add a one-step rollback instruction and rerun copy scan.",
        missingEvidenceWarnings: ["Missing evidence for ac_demo_104_1"],
      }),
      demoAuditChainSummary: {
        correlationId: "corr_demo_104_review",
        requestId: "req_demo_104_rework",
        evidenceImportId: "evimp_demo_104",
        reviewPacketId: "rvpkt_demo_104",
        reworkDecisionId: "rvdec_demo_104_rework",
        workflowPath: "rework_requested",
        eventIds: ["evt_demo_104_packet_generated", "evt_demo_104_rework"],
      },
      demoReviewTimeSummaryMetric: {
        source: "demo_summary_metric_not_event_telemetry",
        derivedFromEventTelemetry: false,
        baselineHumanReviewMinutes: 7,
        humanReviewMinutes: 4,
        completedAgentTasks: 0,
        reworkLoops: 1,
        missingStopEvents: 0,
        reviewerRole: "delivery_lead",
      },
    }),
    workItem({
      scenario: "high_risk_approval_required",
      id: "demo_work_item_005",
      key: "DEMO-105",
      title: "Review protected provider action with redacted placeholder",
      status: "waiting_approval",
      riskTier: "high",
      assignee: "Security Reviewer",
      requestSource: "multica_issue",
      acceptanceCriteria: highRiskCriteria,
      planApproval: {
        state: "pending",
        requiredBefore: "protected_side_effect",
        approvalRef: "approval_demo_105_protected_side_effect",
        rationale:
          "High-risk provider action must pause until manual approval.",
      },
      importedEvidenceSummary: {
        status: "partial",
        sourceRunId: "run_demo_105_agent_result",
        importedAt: "2026-06-01T16:42:00.000Z",
        changedFilesCount: 2,
        artifactCount: 2,
        checks: [
          {
            name: "policy.high-risk-gate",
            status: "passed",
            summary: "Protected action stayed paused behind manual approval.",
            artifactRef: "artifact_demo_105_policy_gate",
          },
          {
            name: "static.redaction-scan",
            status: "passed",
            summary: "Placeholder evidence is redacted and fake-only.",
            artifactRef: "artifact_demo_105_redaction_scan",
          },
        ],
        missingEvidence: ["security.manual-approval-decision"],
      },
      reviewPacket: packet({
        issueKey: "DEMO-105",
        summary:
          "DEMO-105: Manual approval is required before the protected action can proceed.",
        changedArtifacts: [
          {
            path: "contracts/fixtures/policy-decision.deny.high-risk-runtime.json",
            kind: "contract",
            riskTags: ["security", "provider"],
          },
        ],
        acceptanceCriteria: highRiskCriteria,
        verificationEvidence: [
          {
            id: "ev_demo_105_policy_gate",
            kind: "review",
            status: "blocked",
            summary: "Protected action stayed paused behind manual approval.",
            artifactRefs: ["artifact_demo_105_policy_gate"],
            correlationId: "corr_demo_105_review",
          },
        ],
        riskSignals: [
          {
            code: "high_risk_change",
            severity: "warning",
            message:
              "High-risk protected side effect requires manual approval.",
            evidenceRefs: ["ev_demo_105_policy_gate"],
          },
          {
            code: "high_risk_change",
            severity: "warning",
            message:
              "High-risk artifacts or risk tags require reviewer attention.",
            evidenceRefs: [
              "contracts/fixtures/policy-decision.deny.high-risk-runtime.json",
            ],
          },
        ],
        uncertainty: [
          "Human approval decision is intentionally not fixture-backed.",
        ],
        rollbackOrReworkGuidance:
          "Attach the manual approval decision or cancel the protected action.",
        missingEvidenceWarnings: ["Missing security.manual-approval-decision"],
        redactions: ["fake_secret_shaped_placeholder"],
      }),
      demoAuditChainSummary: {
        correlationId: "corr_demo_105_review",
        requestId: "req_demo_105_protected_action",
        policyDecisionId: "poldec_demo_105_requires_approval",
        approvalId: "appr_demo_105_protected_side_effect",
        evidenceImportId: "evimp_demo_105",
        reviewPacketId: "rvpkt_demo_105",
        workflowPath: "denied",
        eventIds: [
          "evt_demo_105_approval_requested",
          "evt_demo_105_packet_generated",
          "evt_demo_105_redaction_verified",
        ],
      },
      demoReviewTimeSummaryMetric: {
        source: "demo_summary_metric_not_event_telemetry",
        derivedFromEventTelemetry: false,
        baselineHumanReviewMinutes: 20,
        humanReviewMinutes: 9,
        completedAgentTasks: 0,
        reworkLoops: 0,
        missingStopEvents: 1,
        reviewerRole: "human_reviewer",
      },
      redactionSafety: {
        dataClassification: "generated_fake",
        fakeOnly: true,
        secretShapedCase: {
          inputLabel: "generated fake credential-like evidence",
          displayValue: "[REDACTED_FAKE_SECRET_SHAPED_VALUE]",
          rawValueStored: false,
          validationNote:
            "The fixture stores only a redacted placeholder and never stores a raw token, key, password, private credential, prompt, or log body.",
        },
      },
    }),
  ],
};
