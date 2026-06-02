import type { ReviewPacketInput } from "./reviewPacket";

export const reviewPacketFixtureInputs = [
  {
    schemaVersion: "review_packet_fixture_input.v0",
    issueKey: "FIX-101",
    title: "Low-risk fixture-backed review packet",
    issueRequest: {
      source: "multica_issue",
      summary:
        "Reviewer needs a compact done/rework packet for a local fixture change.",
      requesterRef: "member_fixture_reviewer",
    },
    plan: {
      summary: "Use existing fixture adapter and add focused unit coverage.",
      approvalRef: "approval_fixture_101_plan",
      approved: true,
    },
    changedArtifacts: [
      {
        path: "src/data/reviewPacket.ts",
        kind: "source",
        summary: "Deterministic review packet assembler.",
      },
      {
        path: "src/data/reviewPacket.test.ts",
        kind: "test",
        summary: "Assembler unit coverage.",
      },
    ],
    acceptanceCriteria: [
      {
        id: "ac_fixture_101_deterministic",
        text: "Assembler output is deterministic for fixed fixture input.",
        evidenceRefs: ["ev_fixture_101_unit"],
      },
      {
        id: "ac_fixture_101_redaction",
        text: "Packet output contains no raw secret-shaped placeholders.",
        evidenceRefs: ["ev_fixture_101_redaction"],
      },
    ],
    verificationEvidence: [
      {
        id: "ev_fixture_101_unit",
        kind: "test",
        status: "passed",
        summary: "Unit test produced stable packet output.",
        command: "npm run test:unit -- src/data/reviewPacket.test.ts",
        artifactRefs: ["src/data/reviewPacket.test.ts"],
        correlationId: "corr_fixture_101_unit",
      },
      {
        id: "ev_fixture_101_redaction",
        kind: "test",
        status: "passed",
        summary: "Synthetic redaction scan found no raw credential material.",
        artifactRefs: ["src/data/reviewPacketFixtures.ts"],
        correlationId: "corr_fixture_101_redaction",
      },
    ],
    rollbackNote: {
      summary: "Revert the fixture assembler files if UX contract changes.",
      artifactRefs: ["src/data/reviewPacket.ts"],
    },
    uncertaintyNotes: ["Schema child output not finalized."],
  },
  {
    schemaVersion: "review_packet_fixture_input.v0",
    issueKey: "FIX-102",
    title: "Partial review packet with unverified acceptance",
    issueRequest: {
      source: "github_issue",
      summary:
        "Reviewer can inspect available evidence without opening full logs or diffs.",
    },
    plan: {
      summary:
        "Assemble current evidence and keep missing browser proof visible.",
      approved: true,
    },
    changedArtifacts: [
      {
        path: "src/App.tsx",
        kind: "source",
        summary: "Review panel copy and layout update.",
      },
    ],
    acceptanceCriteria: [
      {
        id: "ac_fixture_102_unit",
        text: "Unit evidence confirms visible packet sections.",
        evidenceRefs: ["ev_fixture_102_unit"],
      },
      {
        id: "ac_fixture_102_visual",
        text: "Desktop and mobile screenshots are attached for QA.",
        evidenceRefs: ["ev_fixture_102_missing_visual"],
      },
    ],
    verificationEvidence: [
      {
        id: "ev_fixture_102_unit",
        kind: "test",
        status: "passed",
        summary: "Unit smoke passed for the generated fake packet.",
        command: "npm run test:unit",
        artifactRefs: ["src/App.test.tsx"],
        correlationId: "corr_fixture_102_unit",
      },
    ],
    riskNotes: [
      {
        id: "risk_fixture_102_missing_visual",
        severity: "warning",
        summary: "Visual evidence is still required before done.",
        evidenceRefs: ["ac_fixture_102_visual"],
      },
    ],
    rollbackNote: {
      summary:
        "Keep rework limited to the display adapter until screenshots pass.",
    },
    uncertaintyNotes: ["Screenshot artifacts are intentionally absent."],
  },
  {
    schemaVersion: "review_packet_fixture_input.v0",
    issueKey: "FIX-103",
    title: "Rework-needed packet with failed verification",
    issueRequest: {
      source: "manual_fixture",
      summary:
        "Reviewer must request rework when command evidence fails or risk remains.",
    },
    plan: {
      summary: "Hold done decision until failing fixture validation is fixed.",
      approvalRef: "approval_fixture_103_plan",
      approved: true,
    },
    changedArtifacts: [
      {
        path: "contracts/schemas/review-packet.schema.json",
        kind: "contract",
        summary: "Temporary review packet schema candidate.",
        riskTags: ["public-api", "schema"],
      },
    ],
    acceptanceCriteria: [
      {
        id: "ac_fixture_103_contract",
        text: "Contract fixture validation passes.",
        evidenceRefs: ["ev_fixture_103_contract_fail"],
      },
    ],
    verificationEvidence: [
      {
        id: "ev_fixture_103_contract_fail",
        kind: "test",
        status: "failed",
        summary:
          "Contract fixture rejected a missing decisionPrompt.requiredFollowUps field.",
        command: "npm run test:fixtures",
        artifactRefs: ["contracts/fixtures/review-packet.rework-needed.json"],
        correlationId: "corr_fixture_103_contract",
      },
    ],
    riskNotes: [
      {
        id: "risk_fixture_103_schema_gap",
        severity: "danger",
        summary:
          "Schema child output is not final, so this packet must stay in rework.",
        evidenceRefs: ["ev_fixture_103_contract_fail"],
      },
    ],
    rollbackNote: {
      summary:
        "Do not expose this temporary contract as a generated API until schema/eval children align.",
      artifactRefs: ["contracts/schemas/review-packet.schema.json"],
    },
    reworkRequested: true,
    uncertaintyNotes: [
      "Eval harness rubric may rename decision prompt fields after schema child completion.",
    ],
  },
] satisfies readonly ReviewPacketInput[];
