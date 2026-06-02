import { describe, expect, it } from "vitest";
import githubPrFixture from "../../contracts/fixtures/agent-result-import.github-pr.json";
import invalidFixture from "../../contracts/fixtures/agent-result-import.invalid-sensitive-oversized.json";
import manualFixture from "../../contracts/fixtures/agent-result-import.manual.json";
import {
  agentResultImportToReviewPacketInput,
  type ImportedAgentResultContract,
} from "./agentResultImport";
import { generateReviewPacket } from "./reviewPacket";

describe("agent result import fixture consumption", () => {
  it("assembles a review packet from the manual import contract fixture", async () => {
    const input = agentResultImportToReviewPacketInput(
      manualFixture as ImportedAgentResultContract,
      "Manual agent result import contract",
    );
    const packet = await generateReviewPacket(input);

    expect(packet.issueKey).toBe("issue_BOG_619");
    expect(packet.changedArtifacts).toHaveLength(2);
    expect(packet.verificationEvidence.map((item) => item.status)).toContain(
      "passed",
    );
    expect(packet.acceptanceChecklist.map((item) => item.status)).toContain(
      "covered",
    );
    expect(JSON.stringify(packet)).not.toContain("raw_log");
    expect(packet.rollbackOrReworkGuidance).toBe(
      "Remove the fixture import route and review packet adapter wiring.",
    );
  });

  it("assembles a review packet from PR/link metadata without live GitHub data", async () => {
    const input = agentResultImportToReviewPacketInput(
      githubPrFixture as ImportedAgentResultContract,
      "Review packet metadata import",
    );
    const packet = await generateReviewPacket(input);

    expect(packet.issueKey).toBe("github_pr_taskotter_573");
    expect(packet.changedArtifacts.map((artifact) => artifact.path)).toEqual([
      "src/data/reviewPacket.ts",
      "src/data/reviewPacket.test.ts",
    ]);
    expect(packet.audit.correlationIds).toContain("pr_metadata_stub_573");
    expect(JSON.stringify(packet)).not.toContain("api.github.com");
  });

  it("keeps invalid sensitive fixture values out of assembled packets", async () => {
    const input = agentResultImportToReviewPacketInput(
      invalidFixture as ImportedAgentResultContract,
      "Invalid import validation fixture",
    );
    const packet = await generateReviewPacket(input);
    const serialized = JSON.stringify(packet);

    expect(packet.summary).toContain("Invalid import validation fixture");
    expect(serialized).not.toContain("fixture-secret");
    expect(packet.audit.redactions).toContain("password");
  });
});
