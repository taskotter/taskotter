import {
  TaskOtterClient,
  type TaskOtterFetch,
} from "../../packages/api-client/src/index";
import type {
  Comment,
  ConsoleData,
  DisplayText,
  IssueDetail,
  IssueStatus,
  IssueSummary,
  LocalizedText,
  RunStatus,
  RunStep,
  SetupStep,
  TaskOtterDataAdapter,
  WorkingGroup,
} from "./contracts";
import { createI18n, resolveLocalePreferences } from "../i18n";
import { taskotterConsoleFixture } from "./taskotterFixtures";

type I18n = ReturnType<typeof createI18n>;

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

function localized(
  key: LocalizedText["key"],
  values?: LocalizedText["values"],
) {
  return { key, values } satisfies LocalizedText;
}

function displayText(text: string): DisplayText {
  return { text };
}

function resourceStatusLabel(
  i18n: I18n,
  status: ScopedResource["status"],
): string {
  return i18n.t(
    `commonErrors.resourceStatus.${status}` as LocalizedText["key"],
  );
}

function formatTimestamp(value: string, i18n: I18n): DisplayText {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return displayText(value);

  return displayText(i18n.formatDateTime(date));
}

function displayKey(id: string): string {
  const [, suffix = id] = id.split("_");
  return `ISS-${suffix.slice(-4).toUpperCase()}`;
}

function actorLabel(actor?: ActorRef, agents: ScopedResource[] = []): string {
  if (!actor) return "";
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
  if (status === "blocked" || status === "cancelled") return "blocked";
  if (status === "in_review" || status === "done") return "needs_review";
  return "assigned";
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
  i18n: I18n,
): Comment[] {
  return comments
    .filter((comment) => comment.issue_id === issueId)
    .map((comment) => ({
      id: comment.id,
      author: actorLabel(comment.author, agents),
      role: comment.author.type === "user" ? "human" : "agent",
      body: comment.body,
      createdAt: i18n.formatDateTime(comment.created_at),
    }));
}

function mapSetupSteps(
  snapshot: GeneratedConsoleSnapshot,
  i18n: I18n,
): SetupStep[] {
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
      title: localized("issues.setup.step.wg.title"),
      state: workingGroup ? "complete" : "locked",
      detail: workingGroup
        ? localized("issues.setup.step.wg.available", {
            name: workingGroup.name,
          })
        : localized("issues.setup.step.wg.waiting"),
    },
    {
      id: "provider",
      title: localized("issues.setup.step.provider.title"),
      state:
        activeProviders.length > 0
          ? "complete"
          : disabledProviders.length > 0
            ? "error"
            : "locked",
      detail: activeProviders[0]?.name
        ? displayText(activeProviders[0].name)
        : disabledProviders[0]?.name
          ? displayText(disabledProviders[0].name)
          : localized("issues.setup.step.provider.none"),
    },
    {
      id: "limits",
      title: localized("issues.setup.step.limits.title"),
      state: snapshot.auditEvents.data.some(
        (event) => event.payload.outcome === "denied",
      )
        ? "error"
        : gatewayUsage
          ? "active"
          : "locked",
      detail: gatewayUsage
        ? localized("issues.setup.step.limits.usage", {
            durationMs: i18n.formatNumber(
              gatewayUsage.payload.measurements.duration_ms,
            ),
          })
        : localized("issues.setup.step.limits.waiting"),
    },
    {
      id: "runner",
      title: localized("issues.setup.step.runner.title"),
      state: activeIntegrations.length > 0 ? "active" : "locked",
      detail: activeIntegrations[0]?.name
        ? displayText(activeIntegrations[0].name)
        : localized("issues.setup.step.runner.unavailable"),
    },
  ];
}

function mapRunSteps(
  snapshot: GeneratedConsoleSnapshot,
  i18n: I18n,
): RunStep[] {
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
      label: selectedIssue
        ? displayText(displayKey(selectedIssue.id))
        : localized("issues.run.issueQueue"),
      status: selectedIssue ? mapRunStatus(selectedIssue.status) : "queued",
      timestamp: selectedIssue
        ? formatTimestamp(selectedIssue.updated_at, i18n)
        : localized("issues.empty.pending"),
      detail: selectedIssue?.title
        ? displayText(selectedIssue.title)
        : localized("issues.run.waitingIssuePayloads"),
      severity: selectedIssue?.status === "blocked" ? "danger" : "info",
    },
    {
      id: "workflow-state",
      label: workflow?.name
        ? displayText(workflow.name)
        : localized("issues.run.workflowScope"),
      status: workflow?.status === "active" ? "running" : "queued",
      timestamp: workflow
        ? formatTimestamp(workflow.created_at, i18n)
        : localized("issues.empty.pending"),
      detail: workflow
        ? localized("issues.run.workflowState", {
            status: resourceStatusLabel(i18n, workflow.status),
          })
        : localized("issues.run.workflowMissing"),
      severity: workflow?.status === "active" ? "info" : "neutral",
    },
    {
      id: "policy-state",
      label: localized("issues.run.policyDecision"),
      status: deniedAudit ? "waiting_approval" : "completed",
      timestamp: deniedAudit
        ? formatTimestamp(deniedAudit.occurred_at, i18n)
        : localized("commonErrors.status.completed"),
      detail: deniedAudit
        ? displayText(deniedAudit.payload.action)
        : localized("issues.run.noDeniedAudit"),
      severity: deniedAudit ? "warning" : "success",
    },
    {
      id: "usage-state",
      label: usage?.payload.subject.type
        ? displayText(usage.payload.subject.type)
        : localized("issues.run.usageTelemetry"),
      status: usage ? "completed" : "queued",
      timestamp: usage
        ? formatTimestamp(usage.occurred_at, i18n)
        : localized("issues.empty.pending"),
      detail: usage
        ? localized("issues.run.usageDetail", {
            durationMs: i18n.formatNumber(
              usage.payload.measurements.duration_ms,
            ),
            toolCount: i18n.formatNumber(
              usage.payload.measurements.tool_invocations ?? 0,
            ),
          })
        : localized("issues.run.waitingUsage"),
      severity: usage ? "success" : "neutral",
    },
    {
      id: "runner-state",
      label: runnerIntegration?.name
        ? displayText(runnerIntegration.name)
        : localized("issues.run.runnerIntegration"),
      status: runnerIntegration?.status === "active" ? "running" : "failed",
      timestamp: runnerIntegration
        ? formatTimestamp(runnerIntegration.created_at, i18n)
        : localized("issues.empty.unavailable"),
      detail: runnerIntegration
        ? localized("issues.run.integrationState", {
            status: resourceStatusLabel(i18n, runnerIntegration.status),
          })
        : localized("issues.run.noRunnerIntegration"),
      severity: runnerIntegration?.status === "active" ? "info" : "danger",
    },
  ];
}

function mapIssueSummary(
  issue: ControlPlaneIssue,
  snapshot: GeneratedConsoleSnapshot,
  i18n: I18n,
): IssueSummary {
  return {
    id: issue.id,
    key: displayKey(issue.id),
    title: issue.title,
    status: mapIssueStatus(issue.status),
    priority: issue.priority,
    assignee:
      actorLabel(issue.assignee, snapshot.agents.data) ||
      i18n.t("issues.empty.unassigned"),
    labels: [],
    updatedAt: i18n.formatDateTime(issue.updated_at),
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
  const i18n = createI18n(
    resolveLocalePreferences({
      workingGroupDefaultLanguage: firstWorkingGroup ? "en" : undefined,
      formattingLocale: "en-US",
    }),
  );
  const firstIssue = snapshot.issues.data[0];
  const issues = snapshot.issues.data.map((issue) =>
    mapIssueSummary(issue, snapshot, i18n),
  );
  const selectedSummary =
    issues[0] ??
    ({
      id: "issue_empty",
      key: "ISS-EMPTY",
      title: i18n.t("issues.empty.issueTitle"),
      status: "triage",
      priority: "medium",
      assignee: i18n.t("issues.empty.unassigned"),
      labels: [],
      updatedAt: i18n.t("issues.empty.pending"),
      commentCount: 0,
      runStatus: "queued",
      policyState: "allowed",
      group: "assigned",
    } satisfies IssueSummary);
  const childIds = firstIssue
    ? snapshot.issues.data
        .filter((issue) => issue.parent_issue_id === firstIssue.id)
        .map((issue) => displayKey(issue.id))
    : [];
  const selectedIssue: IssueDetail = {
    ...selectedSummary,
    description:
      firstIssue?.description ?? i18n.t("issues.empty.issueDescription"),
    acceptance: [
      i18n.t("issues.empty.acceptance.generated"),
      i18n.t("issues.empty.acceptance.adapter"),
    ],
    parent: firstIssue?.parent_issue_id
      ? displayKey(firstIssue.parent_issue_id)
      : undefined,
    children: childIds,
    comments: firstIssue
      ? mapComments(
          snapshot.comments.data,
          firstIssue.id,
          snapshot.agents.data,
          i18n,
        )
      : [],
  };

  return {
    workingGroup: {
      id: firstWorkingGroup?.id ?? "wg_unavailable",
      name:
        firstWorkingGroup?.name ?? i18n.t("webShell.workspace.unavailableName"),
      plan:
        snapshot.workflows.data[0]?.name ??
        i18n.t("webShell.workspace.defaultPlan"),
      role: "admin",
      memberCount: snapshot.users.data.filter(
        (user) => user.status === "active",
      ).length,
      runnerState: runnerState(
        firstWorkingGroup?.id ?? "",
        snapshot.integrations.data,
      ),
      defaultLanguage: "en",
    },
    localePreferences: {
      workingGroupDefaultLanguage: firstWorkingGroup ? "en" : undefined,
      formattingLocale: "en-US",
    },
    issues,
    selectedIssue,
    runSteps: mapRunSteps(snapshot, i18n),
    setupSteps: mapSetupSteps(snapshot, i18n),
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
