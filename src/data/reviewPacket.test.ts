import { describe, expect, it } from "vitest";
import {
  FakeReviewPacketTextProvider,
  generateReviewPacket,
  type ReviewPacketInput,
  type ReviewPacketTextProvider,
} from "./reviewPacket";

const baseInput = {
  issueKey: "BOG-573",
  title: "Review packet generation scaffold",
  changedArtifacts: [
    {
      path: "src/data/reviewPacket.ts",
      kind: "source",
      summary: "Deterministic packet generator",
    },
    {
      path: "src/data/reviewPacket.test.ts",
      kind: "test",
      summary: "Fixture coverage",
    },
  ],
  acceptanceCriteria: [
    {
      id: "ac-stable-packet",
      text: "Fixture evidence produces a stable review packet.",
      evidenceRefs: ["ev-unit"],
    },
    {
      id: "ac-redaction",
      text: "Packet output avoids raw secret-shaped values.",
      evidenceRefs: ["ev-redaction"],
    },
  ],
  verificationEvidence: [
    {
      id: "ev-unit",
      kind: "test",
      status: "passed",
      summary: "reviewPacket fixture tests passed",
      command: "npm run test:unit -- src/data/reviewPacket.test.ts",
      artifactRefs: ["src/data/reviewPacket.test.ts"],
      correlationId: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
    },
    {
      id: "ev-redaction",
      kind: "test",
      status: "passed",
      summary: "redaction-sensitive fixture passed",
      artifactRefs: ["src/data/reviewPacket.test.ts"],
      correlationId: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ",
    },
  ],
  uncertaintyNotes: ["Provider text generation is stubbed only."],
} satisfies ReviewPacketInput;

describe("review packet generation", () => {
  it("builds a stable deterministic packet from fixture evidence", async () => {
    const packet = await generateReviewPacket(
      baseInput,
      new FakeReviewPacketTextProvider(),
    );

    expect(packet).toMatchObject({
      schemaVersion: "review_packet.v0",
      issueKey: "BOG-573",
      summary: "BOG-573: Review packet generation scaffold",
      missingEvidenceWarnings: [],
      acceptanceChecklist: [
        {
          id: "ac-stable-packet",
          status: "covered",
          evidenceRefs: ["ev-unit"],
        },
        {
          id: "ac-redaction",
          status: "covered",
          evidenceRefs: ["ev-redaction"],
        },
      ],
      audit: {
        correlationIds: [
          "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
          "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ",
        ],
        redactions: [],
      },
    });
    expect(packet.riskSignals).toEqual([]);
    expect(packet.verificationEvidence[0]).toMatchObject({
      id: "ev-unit",
      command: "npm run test:unit -- src/data/reviewPacket.test.ts",
    });
  });

  it("surfaces missing acceptance coverage and missing tests explicitly", async () => {
    const packet = await generateReviewPacket({
      ...baseInput,
      acceptanceCriteria: [
        {
          id: "ac-uncovered",
          text: "Reviewer can see missing evidence.",
        },
      ],
      verificationEvidence: [],
    });

    expect(packet.acceptanceChecklist).toEqual([
      {
        id: "ac-uncovered",
        text: "Reviewer can see missing evidence.",
        status: "missing",
        evidenceRefs: [],
      },
    ]);
    expect(packet.missingEvidenceWarnings).toEqual([
      "Missing evidence for ac-uncovered",
      "Missing test evidence",
    ]);
    expect(packet.riskSignals.map((signal) => signal.code)).toEqual([
      "missing_acceptance_evidence",
      "missing_tests",
    ]);
  });

  it("flags failed verification evidence with linked evidence refs", async () => {
    const packet = await generateReviewPacket({
      ...baseInput,
      verificationEvidence: [
        {
          id: "ev-fixture-fail",
          kind: "test",
          status: "failed",
          summary: "Fixture snapshot mismatch",
          command: "npm run test:unit -- src/data/reviewPacket.test.ts",
          correlationId: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2ER",
        },
      ],
      acceptanceCriteria: [
        {
          id: "ac-test-fail-visible",
          text: "Failed tests are visible to reviewers.",
          evidenceRefs: ["ev-fixture-fail"],
        },
      ],
    });

    expect(packet.riskSignals).toContainEqual({
      code: "verification_failed",
      severity: "danger",
      message: "test evidence failed.",
      evidenceRefs: ["ev-fixture-fail"],
    });
    expect(packet.audit.correlationIds).toEqual([
      "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2ER",
    ]);
  });

  it("keeps rework-needed and high-risk signals out of prose-only summaries", async () => {
    const packet = await generateReviewPacket({
      ...baseInput,
      reworkRequested: true,
      changedArtifacts: [
        {
          path: "contracts/schemas/review-packet.schema.json",
          kind: "contract",
          riskTags: ["public-api", "security"],
        },
      ],
      verificationEvidence: [
        {
          id: "ev-typecheck-blocked",
          kind: "typecheck",
          status: "blocked",
          summary: "Generated schema package unavailable",
        },
      ],
      acceptanceCriteria: [],
    });

    expect(packet.riskSignals.map((signal) => signal.code)).toEqual([
      "missing_tests",
      "verification_blocked",
      "high_risk_change",
      "rework_requested",
    ]);
    expect(packet.rollbackOrReworkGuidance).toBe(
      "Rework failed or blocked evidence before requesting approval.",
    );
  });

  it("redacts secret-shaped input across provider text, evidence, and artifacts", async () => {
    const provider = {
      summarize: async () =>
        "Provider summary included bearer sk-live-token and must be safe.",
    } satisfies ReviewPacketTextProvider;
    const packet = await generateReviewPacket(
      {
        ...baseInput,
        changedArtifacts: [
          {
            path: "src/data/reviewPacket.ts",
            kind: "source",
            summary: "Removed api_key=raw-provider-key from display.",
          },
        ],
        acceptanceCriteria: [
          {
            id: "ac-secret",
            text: "Do not leak client_secret=raw-client-secret in packets.",
            evidenceRefs: ["ev-secret"],
          },
        ],
        verificationEvidence: [
          {
            id: "ev-secret",
            kind: "test",
            status: "passed",
            summary: "Saw access_token=raw-access-token in raw fixture.",
            command: "echo password=raw-password",
            correlationId: "corr_safe_review_packet_redaction",
          },
        ],
        rollbackHint:
          "Rollback note contained refresh_token=raw-refresh-token.",
      },
      provider,
    );

    const serialized = JSON.stringify(packet);
    expect(serialized).not.toContain("sk-live-token");
    expect(serialized).not.toContain("raw-provider-key");
    expect(serialized).not.toContain("raw-client-secret");
    expect(serialized).not.toContain("raw-access-token");
    expect(serialized).not.toContain("raw-password");
    expect(serialized).not.toContain("raw-refresh-token");
    expect(packet.summary).toBe(
      "Provider summary included [redacted] and must be safe.",
    );
    expect(packet.audit.redactions).toHaveLength(6);
  });
});
