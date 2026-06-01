import type { DemoReviewControlSeed, DemoReviewWorkItem } from "./contracts";

const generatedAt = "2026-06-01T18:00:00.000Z";

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
      acceptanceCriteria: [
        {
          id: "ac_demo_101_1",
          text: "Issue summary renders status, assignee, and acceptance checklist.",
          required: true,
          satisfied: true,
        },
        {
          id: "ac_demo_101_2",
          text: "Reviewer can mark the packet done without live integrations.",
          required: true,
          satisfied: true,
        },
      ],
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
      reviewPacket: {
        packetId: "packet_demo_101",
        generatedAt: "2026-06-01T17:45:00.000Z",
        summary: "Small UI fixture update is ready for done approval.",
        changedFiles: [
          "src/data/reviewControlDemoFixtures.ts",
          "src/data/contracts.ts",
          "src/data/reviewControlDemoFixtures.test.ts",
        ],
        artifacts: [
          "artifact_demo_101_unit_log",
          "artifact_demo_101_screenshot",
        ],
        acceptanceChecklist: [
          {
            id: "ac_demo_101_1",
            text: "Issue summary renders status, assignee, and acceptance checklist.",
            required: true,
            satisfied: true,
          },
          {
            id: "ac_demo_101_2",
            text: "Reviewer can mark the packet done without live integrations.",
            required: true,
            satisfied: true,
          },
        ],
        riskSignals: ["fixture_only", "no_live_provider_call"],
        uncertainty: ["Design polish pending UX review"],
        rollbackGuidance: "Remove the demo seed export and adapter property.",
        reworkGuidance:
          "Adjust copy or fixture fields only; no API migration is required.",
        decisionRecommendation: "approve_done",
      },
      auditCorrelation: {
        correlationId: "corr_demo_101_review",
        requestId: "req_demo_101_plan",
        policyDecisionId: "poldec_demo_101_allow",
        auditEventIds: [
          "audit_demo_101_plan_approved",
          "audit_demo_101_packet_generated",
        ],
        runTimelineEventIds: [
          "timeline_demo_101_imported",
          "timeline_demo_101_reviewed",
        ],
      },
      reviewTimeMetric: {
        baselineSeconds: 600,
        observedSeconds: 210,
        startedAt: "2026-06-01T17:45:00.000Z",
        completedAt: "2026-06-01T17:48:30.000Z",
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
      acceptanceCriteria: [
        {
          id: "ac_demo_102_1",
          text: "Review packet lists the missing visual proof.",
          required: true,
          satisfied: false,
        },
      ],
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
      reviewPacket: {
        packetId: "packet_demo_102",
        generatedAt: "2026-06-01T17:37:00.000Z",
        summary: "Review is blocked by missing screenshot evidence.",
        changedFiles: ["src/App.tsx", "src/styles.css"],
        artifacts: ["artifact_demo_102_unit_log"],
        acceptanceChecklist: [
          {
            id: "ac_demo_102_1",
            text: "Review packet lists the missing visual proof.",
            required: true,
            satisfied: false,
          },
        ],
        riskSignals: ["missing_visual_evidence"],
        uncertainty: ["Responsive layout has not been visually verified."],
        rollbackGuidance:
          "Keep current branch unmerged until evidence is re-imported.",
        reworkGuidance:
          "Run the browser smoke flow and attach desktop and mobile screenshots.",
        decisionRecommendation: "request_rework",
      },
      auditCorrelation: {
        correlationId: "corr_demo_102_review",
        requestId: "req_demo_102_packet",
        policyDecisionId: "poldec_demo_102_allow",
        auditEventIds: ["audit_demo_102_packet_generated"],
        runTimelineEventIds: ["timeline_demo_102_imported_partial"],
      },
      reviewTimeMetric: {
        baselineSeconds: 720,
        observedSeconds: 480,
        startedAt: "2026-06-01T17:37:00.000Z",
        completedAt: "2026-06-01T17:45:00.000Z",
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
      acceptanceCriteria: [
        {
          id: "ac_demo_103_1",
          text: "Fixture validation passes before review can approve done.",
          required: true,
          satisfied: false,
        },
      ],
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
      reviewPacket: {
        packetId: "packet_demo_103",
        generatedAt: "2026-06-01T17:19:00.000Z",
        summary:
          "Review should request rework because fixture validation failed.",
        changedFiles: [
          "src/data/taskotterAdapter.ts",
          "src/data/contracts.ts",
          "tests/contracts/fixtures.test.mjs",
        ],
        artifacts: [
          "artifact_demo_103_fixture_log",
          "artifact_demo_103_typecheck_log",
        ],
        acceptanceChecklist: [
          {
            id: "ac_demo_103_1",
            text: "Fixture validation passes before review can approve done.",
            required: true,
            satisfied: false,
          },
        ],
        riskSignals: ["failed_required_check"],
        uncertainty: [
          "Review packet completeness cannot be trusted until fixed.",
        ],
        rollbackGuidance: "Revert the adapter fixture mapping from the branch.",
        reworkGuidance:
          "Add the missing review packet fields and rerun fixture validation.",
        decisionRecommendation: "request_rework",
      },
      auditCorrelation: {
        correlationId: "corr_demo_103_review",
        requestId: "req_demo_103_packet",
        policyDecisionId: "poldec_demo_103_allow",
        auditEventIds: ["audit_demo_103_check_failed"],
        runTimelineEventIds: ["timeline_demo_103_fixture_failed"],
      },
      reviewTimeMetric: {
        baselineSeconds: 900,
        observedSeconds: 300,
        startedAt: "2026-06-01T17:19:00.000Z",
        completedAt: "2026-06-01T17:24:00.000Z",
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
      acceptanceCriteria: [
        {
          id: "ac_demo_104_1",
          text: "Review packet explains concrete rework before done.",
          required: true,
          satisfied: false,
        },
      ],
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
      reviewPacket: {
        packetId: "packet_demo_104",
        generatedAt: "2026-06-01T17:06:00.000Z",
        summary: "Reviewer requested clearer rollback guidance before done.",
        changedFiles: ["src/i18n/resources/en/issues.ts"],
        artifacts: ["artifact_demo_104_copy_scan"],
        acceptanceChecklist: [
          {
            id: "ac_demo_104_1",
            text: "Review packet explains concrete rework before done.",
            required: true,
            satisfied: false,
          },
        ],
        riskSignals: ["reviewer_requested_rework"],
        uncertainty: [
          "Rollback instruction is too broad for a future reviewer.",
        ],
        rollbackGuidance:
          "Replace the fixture text with the previous generated fake string.",
        reworkGuidance:
          "Add a one-step rollback instruction and rerun copy scan.",
        decisionRecommendation: "request_rework",
      },
      auditCorrelation: {
        correlationId: "corr_demo_104_review",
        requestId: "req_demo_104_rework",
        auditEventIds: ["audit_demo_104_rework_requested"],
        runTimelineEventIds: ["timeline_demo_104_reviewed"],
      },
      reviewTimeMetric: {
        baselineSeconds: 420,
        observedSeconds: 240,
        startedAt: "2026-06-01T17:06:00.000Z",
        completedAt: "2026-06-01T17:10:00.000Z",
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
      acceptanceCriteria: [
        {
          id: "ac_demo_105_1",
          text: "Protected side effect waits for manual approval.",
          required: true,
          satisfied: true,
        },
        {
          id: "ac_demo_105_2",
          text: "Secret-shaped evidence is redacted and marked generated fake.",
          required: true,
          satisfied: true,
        },
      ],
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
      reviewPacket: {
        packetId: "packet_demo_105",
        generatedAt: "2026-06-01T16:44:00.000Z",
        summary:
          "Manual approval is required before the protected action can proceed.",
        changedFiles: [
          "contracts/fixtures/policy-decision.deny.high-risk-runtime.json",
        ],
        artifacts: [
          "artifact_demo_105_policy_gate",
          "artifact_demo_105_redaction_scan",
        ],
        acceptanceChecklist: [
          {
            id: "ac_demo_105_1",
            text: "Protected side effect waits for manual approval.",
            required: true,
            satisfied: true,
          },
          {
            id: "ac_demo_105_2",
            text: "Secret-shaped evidence is redacted and marked generated fake.",
            required: true,
            satisfied: true,
          },
        ],
        riskSignals: [
          "approval_required",
          "protected_side_effect",
          "redacted_fake_secret_shape",
        ],
        uncertainty: [
          "Human approval decision is intentionally not fixture-backed.",
        ],
        rollbackGuidance:
          "Keep the protected action paused; do not execute provider calls.",
        reworkGuidance:
          "Attach the manual approval decision or cancel the protected action.",
        decisionRecommendation: "manual_approval",
      },
      auditCorrelation: {
        correlationId: "corr_demo_105_review",
        requestId: "req_demo_105_protected_action",
        policyDecisionId: "poldec_demo_105_requires_approval",
        auditEventIds: [
          "audit_demo_105_approval_requested",
          "audit_demo_105_redaction_verified",
        ],
        runTimelineEventIds: ["timeline_demo_105_paused_for_approval"],
      },
      reviewTimeMetric: {
        baselineSeconds: 1200,
        observedSeconds: 540,
        startedAt: "2026-06-01T16:44:00.000Z",
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
