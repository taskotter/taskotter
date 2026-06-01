import {
  TaskOtterClient,
  type TaskOtterFetch,
} from "../../packages/api-client/src/index";
import type {
  Comment,
  ConsoleData,
  IssueDetail,
  IssueStatus,
  IssueSummary,
  RunStatus,
  RunStep,
  SetupStep,
  TaskOtterDataAdapter,
  WorkingGroup,
} from "./contracts";
import { taskotterConsoleFixture } from "./taskotterFixtures";

export class FixtureTaskOtterDataAdapter implements TaskOtterDataAdapter {
  async getConsoleData(): Promise<ConsoleData> {
    return structuredClone(taskotterConsoleFixture);
  }
}

type ActorRef = {
  type: "user" | "agent" | "service";
  id: string;
};

type ScopedResource = {
  id: string;
  working_group_id: string;
  name: string;
  status: "active" | "disabled" | "archived";
  created_at: string;
};

type ControlPlaneWorkingGroup = {
  id: string;
  name: string;
  slug: string;
  status: "active" | "archived";
  created_at: string;
};

type ControlPlaneUser = {
  id: string;
  display_name: string;
  status: "active" | "invited" | "suspended";
};

type ControlPlaneIssue = {
  id: string;
  working_group_id: string;
  parent_issue_id?: string;
  title: string;
  description?: string;
  status:
    | "todo"
    | "in_progress"
    | "in_review"
    | "done"
    | "blocked"
    | "backlog"
    | "cancelled";
  priority: IssueSummary["priority"];
  assignee?: ActorRef;
  created_at?: string;
  updated_at: string;
};

type ControlPlaneComment = {
  id: string;
  working_group_id: string;
  issue_id: string;
  author: ActorRef;
  body: string;
  created_at: string;
};

type UsageEvent = {
  id: string;
  type?: "usage.gateway_request.recorded";
  version?: "0.1.0";
  occurred_at: string;
  source: "control_plane" | "runner" | "gateway";
  working_group_id: string;
  actor?: ActorRef;
  resource: { type: string; id: string };
  correlation_id?: string;
  request_id?: string;
  policy_decision_id?: string;
  idempotency_key?: string;
  payload: {
    subject: {
      type: "agent_run" | "gateway_request" | "workflow_run";
      id: string;
    };
    measurements: {
      duration_ms: number;
      input_tokens?: number;
      output_tokens?: number;
      tool_invocations?: number;
      estimated_cost_micros?: number;
    };
  };
};

type AuditEvent = {
  id: string;
  type?: "audit.policy_decision.denied";
  version?: "0.1.0";
  occurred_at: string;
  source: "control_plane" | "runner" | "gateway";
  working_group_id: string;
  actor?: ActorRef;
  resource: { type: string; id: string };
  correlation_id?: string;
  request_id?: string;
  policy_decision_id?: string;
  payload: {
    action: string;
    outcome: "allowed" | "denied" | "succeeded" | "failed";
  };
};

type Page<T> = {
  data: T[];
  page: { has_more: boolean; next_cursor?: string };
};

export type GeneratedConsoleSnapshot = {
  workingGroups: Page<ControlPlaneWorkingGroup>;
  users: Page<ControlPlaneUser>;
  issues: Page<ControlPlaneIssue>;
  comments: Page<ControlPlaneComment>;
  agents: Page<ScopedResource>;
  integrations: Page<ScopedResource>;
  providers: Page<ScopedResource>;
  workflows: Page<ScopedResource>;
  usageEvents: Page<UsageEvent>;
  auditEvents: Page<AuditEvent>;
};

type GeneratedTaskOtterClient = Pick<
  TaskOtterClient,
  | "listAgents"
  | "listAuditEvents"
  | "listIntegrations"
  | "listIssueComments"
  | "listIssues"
  | "listProviders"
  | "listUsageEvents"
  | "listUsers"
  | "listWorkflows"
  | "listWorkingGroups"
>;

function emptyPage<T>(): Page<T> {
  return { data: [], page: { has_more: false } };
}

function asPage<T>(value: unknown): Page<T> {
  if (
    typeof value !== "object" ||
    value === null ||
    !Array.isArray((value as Page<T>).data)
  ) {
    return { data: [], page: { has_more: false } };
  }

  return value as Page<T>;
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return value;

  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

function displayKey(id: string): string {
  const [, suffix = id] = id.split("_");
  return `ISS-${suffix.slice(-4).toUpperCase()}`;
}

function actorLabel(actor?: ActorRef, agents: ScopedResource[] = []): string {
  if (!actor) return "Unassigned";
  const agent = agents.find((candidate) => candidate.id === actor.id);
  if (agent) return agent.name;

  return `${actor.type}:${actor.id}`;
}

function mapIssueStatus(status: ControlPlaneIssue["status"]): IssueStatus {
  switch (status) {
    case "backlog":
    case "todo":
      return "triage";
    case "cancelled":
      return "blocked";
    default:
      return status;
  }
}

function mapRunStatus(status: ControlPlaneIssue["status"]): RunStatus {
  switch (status) {
    case "todo":
    case "backlog":
      return "queued";
    case "in_progress":
      return "running";
    case "in_review":
      return "waiting_approval";
    case "done":
      return "completed";
    case "blocked":
      return "failed";
    case "cancelled":
      return "cancelled";
  }
}

function groupIssue(
  status: ControlPlaneIssue["status"],
): IssueSummary["group"] {
  if (status === "blocked" || status === "cancelled") return "Blocked";
  if (status === "in_review" || status === "done") return "Needs review";
  return "Assigned";
}

function policyStateForIssue(
  issue: ControlPlaneIssue,
  snapshot: GeneratedConsoleSnapshot,
): IssueSummary["policyState"] {
  const issueDenied = snapshot.auditEvents.data.some(
    (event) =>
      event.working_group_id === issue.working_group_id &&
      event.payload.outcome === "denied" &&
      (event.resource.id === issue.id || event.resource.type === "provider"),
  );
  if (issueDenied) return "policy_denied";

  const unavailableRuntime = [
    ...snapshot.integrations.data,
    ...snapshot.providers.data,
  ].some(
    (resource) =>
      resource.working_group_id === issue.working_group_id &&
      resource.status !== "active",
  );
  if (unavailableRuntime) return "runner_offline";

  const hasMeteredUsage = snapshot.usageEvents.data.some(
    (event) =>
      event.working_group_id === issue.working_group_id &&
      (event.resource.id === issue.id ||
        event.payload.subject.type === "gateway_request") &&
      (event.payload.measurements.estimated_cost_micros ?? 0) > 0,
  );
  if (hasMeteredUsage) return "cost_limited";

  return "allowed";
}

function runnerState(
  workingGroupId: string,
  integrations: ScopedResource[],
): WorkingGroup["runnerState"] {
  const scopedIntegrations = integrations.filter(
    (integration) => integration.working_group_id === workingGroupId,
  );
  if (scopedIntegrations.length === 0) return "offline";
  if (
    scopedIntegrations.every((integration) => integration.status === "active")
  ) {
    return "online";
  }
  return "limited";
}

function mapComments(
  comments: ControlPlaneComment[],
  issueId: string,
  agents: ScopedResource[],
): Comment[] {
  return comments
    .filter((comment) => comment.issue_id === issueId)
    .map((comment) => ({
      id: comment.id,
      author: actorLabel(comment.author, agents),
      role: comment.author.type === "user" ? "human" : "agent",
      body: comment.body,
      createdAt: formatTimestamp(comment.created_at),
    }));
}

function mapSetupSteps(snapshot: GeneratedConsoleSnapshot): SetupStep[] {
  const workingGroup = snapshot.workingGroups.data[0];
  const activeProviders = snapshot.providers.data.filter(
    (provider) =>
      provider.working_group_id === workingGroup?.id &&
      provider.status === "active",
  );
  const disabledProviders = snapshot.providers.data.filter(
    (provider) =>
      provider.working_group_id === workingGroup?.id &&
      provider.status !== "active",
  );
  const activeIntegrations = snapshot.integrations.data.filter(
    (integration) =>
      integration.working_group_id === workingGroup?.id &&
      integration.status === "active",
  );
  const gatewayUsage = snapshot.usageEvents.data.find(
    (event) =>
      event.working_group_id === workingGroup?.id &&
      event.payload.subject.type === "gateway_request",
  );

  return [
    {
      id: "wg",
      title: "Working Group basics",
      state: workingGroup ? "complete" : "locked",
      detail: workingGroup
        ? `${workingGroup.name} scope is available from the control plane.`
        : "Waiting for Working Group scope.",
    },
    {
      id: "provider",
      title: "Provider route",
      state:
        activeProviders.length > 0
          ? "complete"
          : disabledProviders.length > 0
            ? "error"
            : "locked",
      detail:
        activeProviders[0]?.name ??
        disabledProviders[0]?.name ??
        "No provider route reported.",
    },
    {
      id: "limits",
      title: "Policy and usage limits",
      state: snapshot.auditEvents.data.some(
        (event) => event.payload.outcome === "denied",
      )
        ? "error"
        : gatewayUsage
          ? "active"
          : "locked",
      detail: gatewayUsage
        ? `${gatewayUsage.payload.measurements.duration_ms}ms gateway request recorded.`
        : "Waiting for usage telemetry.",
    },
    {
      id: "runner",
      title: "Runner option",
      state: activeIntegrations.length > 0 ? "active" : "locked",
      detail: activeIntegrations[0]?.name ?? "Runner integration unavailable.",
    },
  ];
}

function mapRunSteps(snapshot: GeneratedConsoleSnapshot): RunStep[] {
  const selectedIssue = snapshot.issues.data[0];
  const workflow = snapshot.workflows.data[0];
  const usage = snapshot.usageEvents.data[0];
  const deniedAudit = snapshot.auditEvents.data.find(
    (event) => event.payload.outcome === "denied",
  );
  const runnerIntegration = snapshot.integrations.data[0];

  return [
    {
      id: "issue-state",
      label: selectedIssue ? displayKey(selectedIssue.id) : "Issue queue",
      status: selectedIssue ? mapRunStatus(selectedIssue.status) : "queued",
      timestamp: selectedIssue
        ? formatTimestamp(selectedIssue.updated_at)
        : "pending",
      detail: selectedIssue?.title ?? "Waiting for generated issue payloads.",
      severity: selectedIssue?.status === "blocked" ? "danger" : "info",
    },
    {
      id: "workflow-state",
      label: workflow?.name ?? "Workflow scope",
      status: workflow?.status === "active" ? "running" : "queued",
      timestamp: workflow ? formatTimestamp(workflow.created_at) : "pending",
      detail: workflow
        ? `Workflow is ${workflow.status}.`
        : "No workflow has been returned yet.",
      severity: workflow?.status === "active" ? "info" : "neutral",
    },
    {
      id: "policy-state",
      label: "Policy decision",
      status: deniedAudit ? "waiting_approval" : "completed",
      timestamp: deniedAudit
        ? formatTimestamp(deniedAudit.occurred_at)
        : "ready",
      detail: deniedAudit
        ? deniedAudit.payload.action
        : "No denied control-plane audit event.",
      severity: deniedAudit ? "warning" : "success",
    },
    {
      id: "usage-state",
      label: usage?.payload.subject.type ?? "Usage telemetry",
      status: usage ? "completed" : "queued",
      timestamp: usage ? formatTimestamp(usage.occurred_at) : "pending",
      detail: usage
        ? `${usage.payload.measurements.duration_ms}ms, ${usage.payload.measurements.tool_invocations ?? 0} tools`
        : "Waiting for usage event payloads.",
      severity: usage ? "success" : "neutral",
    },
    {
      id: "runner-state",
      label: runnerIntegration?.name ?? "Runner integration",
      status: runnerIntegration?.status === "active" ? "running" : "failed",
      timestamp: runnerIntegration
        ? formatTimestamp(runnerIntegration.created_at)
        : "unavailable",
      detail: runnerIntegration
        ? `Integration is ${runnerIntegration.status}.`
        : "No runner integration was returned.",
      severity: runnerIntegration?.status === "active" ? "info" : "danger",
    },
  ];
}

function mapIssueSummary(
  issue: ControlPlaneIssue,
  snapshot: GeneratedConsoleSnapshot,
): IssueSummary {
  return {
    id: issue.id,
    key: displayKey(issue.id),
    title: issue.title,
    status: mapIssueStatus(issue.status),
    priority: issue.priority,
    assignee: actorLabel(issue.assignee, snapshot.agents.data),
    labels: [],
    updatedAt: formatTimestamp(issue.updated_at),
    commentCount: snapshot.comments.data.filter(
      (comment) => comment.issue_id === issue.id,
    ).length,
    runStatus: mapRunStatus(issue.status),
    policyState: policyStateForIssue(issue, snapshot),
    group: groupIssue(issue.status),
  };
}

export function mapGeneratedConsoleData(
  partialSnapshot: Partial<GeneratedConsoleSnapshot>,
): ConsoleData {
  const snapshot: GeneratedConsoleSnapshot = {
    workingGroups: partialSnapshot.workingGroups ?? emptyPage(),
    users: partialSnapshot.users ?? emptyPage(),
    issues: partialSnapshot.issues ?? emptyPage(),
    comments: partialSnapshot.comments ?? emptyPage(),
    agents: partialSnapshot.agents ?? emptyPage(),
    integrations: partialSnapshot.integrations ?? emptyPage(),
    providers: partialSnapshot.providers ?? emptyPage(),
    workflows: partialSnapshot.workflows ?? emptyPage(),
    usageEvents: partialSnapshot.usageEvents ?? emptyPage(),
    auditEvents: partialSnapshot.auditEvents ?? emptyPage(),
  };
  const firstWorkingGroup = snapshot.workingGroups.data[0];
  const firstIssue = snapshot.issues.data[0];
  const issues = snapshot.issues.data.map((issue) =>
    mapIssueSummary(issue, snapshot),
  );
  const selectedSummary =
    issues[0] ??
    ({
      id: "issue_empty",
      key: "ISS-EMPTY",
      title: "No issues returned",
      status: "triage",
      priority: "medium",
      assignee: "Unassigned",
      labels: [],
      updatedAt: "pending",
      commentCount: 0,
      runStatus: "queued",
      policyState: "allowed",
      group: "Assigned",
    } satisfies IssueSummary);
  const childIds = firstIssue
    ? snapshot.issues.data
        .filter((issue) => issue.parent_issue_id === firstIssue.id)
        .map((issue) => displayKey(issue.id))
    : [];
  const selectedIssue: IssueDetail = {
    ...selectedSummary,
    description:
      firstIssue?.description ??
      "Control-plane issue detail has not returned a description.",
    acceptance: [
      "Generated control-plane resources map into ConsoleData.",
      "UI components consume adapter view-model types only.",
    ],
    parent: firstIssue?.parent_issue_id
      ? displayKey(firstIssue.parent_issue_id)
      : undefined,
    children: childIds,
    comments: firstIssue
      ? mapComments(snapshot.comments.data, firstIssue.id, snapshot.agents.data)
      : [],
  };

  return {
    workingGroup: {
      id: firstWorkingGroup?.id ?? "wg_unavailable",
      name: firstWorkingGroup?.name ?? "Unavailable Working Group",
      plan: snapshot.workflows.data[0]?.name ?? "MVP control plane",
      role: "admin",
      memberCount: snapshot.users.data.filter(
        (user) => user.status === "active",
      ).length,
      runnerState: runnerState(
        firstWorkingGroup?.id ?? "",
        snapshot.integrations.data,
      ),
    },
    issues,
    selectedIssue,
    runSteps: mapRunSteps(snapshot),
    setupSteps: mapSetupSteps(snapshot),
  };
}

export class GeneratedClientTaskOtterDataAdapter implements TaskOtterDataAdapter {
  constructor(private readonly client: GeneratedTaskOtterClient) {}

  async getConsoleData(): Promise<ConsoleData> {
    const workingGroups = asPage<ControlPlaneWorkingGroup>(
      await this.client.listWorkingGroups(),
    );
    const workingGroupId = workingGroups.data[0]?.id;
    const scopedOptions = workingGroupId
      ? { query: { working_group_id: workingGroupId } }
      : undefined;
    const [
      users,
      issues,
      agents,
      integrations,
      providers,
      workflows,
      usageEvents,
      auditEvents,
    ] = await Promise.all([
      this.client.listUsers(scopedOptions),
      this.client.listIssues(scopedOptions),
      this.client.listAgents(scopedOptions),
      this.client.listIntegrations(scopedOptions),
      this.client.listProviders(scopedOptions),
      this.client.listWorkflows(scopedOptions),
      this.client.listUsageEvents(scopedOptions),
      this.client.listAuditEvents(scopedOptions),
    ]);
    const issuePage = asPage<ControlPlaneIssue>(issues);
    const commentPages = await Promise.all(
      issuePage.data.map((issue) =>
        this.client.listIssueComments({ path: { issue_id: issue.id } }),
      ),
    );
    const comments: Page<ControlPlaneComment> = {
      data: commentPages.flatMap(
        (commentPage) => asPage<ControlPlaneComment>(commentPage).data,
      ),
      page: {
        has_more: commentPages.some(
          (commentPage) => asPage(commentPage).page.has_more,
        ),
      },
    };

    return mapGeneratedConsoleData({
      workingGroups,
      users: asPage<ControlPlaneUser>(users),
      issues: issuePage,
      comments,
      agents: asPage<ScopedResource>(agents),
      integrations: asPage<ScopedResource>(integrations),
      providers: asPage<ScopedResource>(providers),
      workflows: asPage<ScopedResource>(workflows),
      usageEvents: asPage<UsageEvent>(usageEvents),
      auditEvents: asPage<AuditEvent>(auditEvents),
    });
  }
}

export function createGeneratedClientTaskOtterDataAdapter(options?: {
  baseUrl?: string;
  fetch?: TaskOtterFetch;
}): TaskOtterDataAdapter {
  return new GeneratedClientTaskOtterDataAdapter(new TaskOtterClient(options));
}

export const taskOtterDataAdapter = new FixtureTaskOtterDataAdapter();
