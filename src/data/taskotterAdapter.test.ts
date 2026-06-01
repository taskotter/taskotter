import { describe, expect, it } from "vitest";
import {
  GeneratedClientTaskOtterDataAdapter,
  mapGeneratedConsoleData,
  type GeneratedConsoleSnapshot,
} from "./taskotterAdapter";

const page = <T>(data: T[]) => ({ data, page: { has_more: false } });

const generatedSnapshot = {
  workingGroups: page([
    {
      id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      name: "Platform Delivery",
      slug: "platform-delivery",
      status: "active",
      created_at: "2026-01-01T00:00:00.000Z",
    },
  ]),
  users: page([
    {
      id: "usr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      display_name: "Workspace Owner",
      status: "active",
    },
    {
      id: "usr_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ",
      display_name: "Invited Member",
      status: "invited",
    },
  ]),
  agents: page([
    {
      id: "agt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      name: "Frontend Implementation Engineer",
      status: "active",
      created_at: "2026-01-01T00:00:00.000Z",
    },
  ]),
  issues: page([
    {
      id: "iss_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      title: "Build MVP app shell",
      description: "Render the issue workspace and focus panel.",
      status: "in_progress",
      priority: "high",
      assignee: {
        type: "agent",
        id: "agt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      },
      created_at: "2026-01-01T00:00:00.000Z",
      updated_at: "2026-01-01T00:05:00.000Z",
    },
    {
      id: "iss_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      parent_issue_id: "iss_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      title: "Review generated adapter",
      status: "in_review",
      priority: "medium",
      created_at: "2026-01-01T00:00:00.000Z",
      updated_at: "2026-01-01T00:10:00.000Z",
    },
  ]),
  comments: page([
    {
      id: "com_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      issue_id: "iss_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      author: {
        type: "agent",
        id: "agt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      },
      body: "Generated client payload is ready for UI mapping.",
      created_at: "2026-01-01T00:06:00.000Z",
    },
  ]),
  integrations: page([
    {
      id: "int_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      name: "Runner bridge",
      status: "active",
      created_at: "2026-01-01T00:00:00.000Z",
    },
  ]),
  providers: page([
    {
      id: "prv_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      name: "Gateway provider",
      status: "disabled",
      created_at: "2026-01-01T00:00:00.000Z",
    },
  ]),
  workflows: page([
    {
      id: "wfl_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      name: "MVP control plane",
      status: "active",
      created_at: "2026-01-01T00:00:00.000Z",
    },
  ]),
  usageEvents: page([
    {
      id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      type: "usage.gateway_request.recorded",
      version: "0.1.0",
      occurred_at: "2026-01-01T00:07:00.000Z",
      source: "gateway",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      actor: {
        type: "agent",
        id: "agt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      },
      resource: {
        type: "provider",
        id: "prv_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      },
      correlation_id: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      request_id: "req_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      policy_decision_id: "poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      idempotency_key: "usage_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      payload: {
        subject: {
          type: "gateway_request",
          id: "gwreq_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
        },
        measurements: {
          duration_ms: 840,
          input_tokens: 1200,
          output_tokens: 320,
          tool_invocations: 1,
          estimated_cost_micros: 2300,
        },
      },
    },
  ]),
  auditEvents: page([
    {
      id: "evt_01J9Z4P4BS0M9P2QJ6T8Z6W2EQ",
      type: "audit.policy_decision.denied",
      version: "0.1.0",
      occurred_at: "2026-01-01T00:08:00.000Z",
      source: "control_plane",
      working_group_id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      actor: {
        type: "agent",
        id: "agt_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      },
      resource: {
        type: "provider",
        id: "prv_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      },
      correlation_id: "corr_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      request_id: "req_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      policy_decision_id: "poldec_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      payload: {
        action: "gateway.provider.invoke",
        outcome: "denied",
      },
    },
  ]),
} satisfies GeneratedConsoleSnapshot;

describe("generated TaskOtter adapter mapping", () => {
  it("maps generated client/schema payloads into ConsoleData assumptions", () => {
    const data = mapGeneratedConsoleData(generatedSnapshot);

    expect(data.workingGroup).toMatchObject({
      id: "wg_01J9Z4P4BS0M9P2QJ6T8Z6W2EP",
      name: "Platform Delivery",
      plan: "MVP control plane",
      memberCount: 1,
      runnerState: "online",
    });
    expect(data.issues).toHaveLength(2);
    expect(data.issues[0]).toMatchObject({
      title: "Build MVP app shell",
      status: "in_progress",
      assignee: "Frontend Implementation Engineer",
      commentCount: 1,
      runStatus: "running",
      policyState: "policy_denied",
      group: "Assigned",
    });
    expect(data.issues[1]).toMatchObject({
      status: "in_review",
      runStatus: "waiting_approval",
      group: "Needs review",
    });
    expect(data.selectedIssue).toMatchObject({
      description: "Render the issue workspace and focus panel.",
      children: ["ISS-W2EQ"],
    });
    expect(data.selectedIssue.comments[0]).toMatchObject({
      author: "Frontend Implementation Engineer",
      role: "agent",
      body: "Generated client payload is ready for UI mapping.",
    });
    expect(data.setupSteps.map((step) => step.id)).toEqual([
      "wg",
      "provider",
      "limits",
      "runner",
    ]);
    expect(
      data.setupSteps.find((step) => step.id === "provider"),
    ).toMatchObject({
      state: "error",
      detail: "Gateway provider",
    });
    expect(data.runSteps.map((step) => step.id)).toEqual([
      "issue-state",
      "workflow-state",
      "policy-state",
      "usage-state",
      "runner-state",
    ]);
    expect(
      data.runSteps.find((step) => step.id === "usage-state"),
    ).toMatchObject({
      status: "completed",
      detail: "840ms, 1 tools",
    });
  });

  it("fetches generated client resources before mapping ConsoleData", async () => {
    const client = {
      listWorkingGroups: () => Promise.resolve(generatedSnapshot.workingGroups),
      listUsers: () => Promise.resolve(generatedSnapshot.users),
      listIssues: () => Promise.resolve(generatedSnapshot.issues),
      listIssueComments: () => Promise.resolve(generatedSnapshot.comments),
      listAgents: () => Promise.resolve(generatedSnapshot.agents),
      listIntegrations: () => Promise.resolve(generatedSnapshot.integrations),
      listProviders: () => Promise.resolve(generatedSnapshot.providers),
      listWorkflows: () => Promise.resolve(generatedSnapshot.workflows),
      listUsageEvents: () => Promise.resolve(generatedSnapshot.usageEvents),
      listAuditEvents: () => Promise.resolve(generatedSnapshot.auditEvents),
    };

    const adapter = new GeneratedClientTaskOtterDataAdapter(client);
    await expect(adapter.getConsoleData()).resolves.toMatchObject({
      workingGroup: { name: "Platform Delivery" },
      selectedIssue: { title: "Build MVP app shell" },
    });
  });
});
