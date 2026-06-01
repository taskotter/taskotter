export type IssueStatus =
  | "triage"
  | "in_progress"
  | "waiting_approval"
  | "blocked"
  | "in_review"
  | "done";

export type RunStatus =
  | "queued"
  | "running"
  | "waiting_approval"
  | "retrying"
  | "failed"
  | "cancelled"
  | "completed";

export type Severity = "neutral" | "success" | "warning" | "danger" | "info";

export type FirstRunDenialReasonCode =
  | "policy_denied"
  | "cost_limited"
  | "paid_activation_required"
  | "runner_offline";

export interface FirstRunTimelineItem {
  id: string;
  messageKey: string;
  status: RunStatus;
  severity: Severity;
  safeRefs: string[];
  redacted: true;
}

export interface FirstRunPermissionState {
  role: "admin" | "member";
  canConfigure: boolean;
  canRunDiagnostic: boolean;
  readOnlyReasonCode?: "member_read_only" | "owner_required";
  workingGroupId: string;
}

export interface FirstRunOnboarding {
  workingGroupId: string;
  permissions: FirstRunPermissionState;
  permissionFixtures: {
    member: FirstRunPermissionState;
  };
  providerRoute: {
    providerName: string;
    modelName: string;
    credentialRef: string;
    credentialStatus: "reference_present" | "missing" | "expired";
  };
  costUsageDefaults: {
    monthlyLimitMicros: number;
    perRunLimitMicros: number;
    usageDeltaMicros: 0;
    billable: false;
  };
  runnerAvailability: {
    runnerState: WorkingGroup["runnerState"];
    mcpState: "available" | "unavailable" | "policy_disabled";
    blockedReasonCode?: FirstRunDenialReasonCode;
  };
  binding: {
    agentName: string;
    skillName: string;
  };
  diagnosticContract: {
    mode: "fixture" | "policy_check_only";
    allowPaidCall: false;
    idempotencyKey: string;
    billable: false;
    usageDeltaMicros: 0;
    policyDecisionId: string;
    denialReasonCode?: FirstRunDenialReasonCode;
    blockedReasonCode?: FirstRunDenialReasonCode;
  };
  timeline: FirstRunTimelineItem[];
}

export interface WorkingGroup {
  id: string;
  name: string;
  plan: string;
  role: "owner" | "admin" | "member";
  memberCount: number;
  runnerState: "online" | "offline" | "limited";
  defaultLanguage?: string;
  timeZone?: string;
}

export interface IssueSummary {
  id: string;
  key: string;
  title: string;
  status: IssueStatus;
  priority: "low" | "medium" | "high" | "urgent";
  assignee: string;
  labels: string[];
  updatedAt: string;
  commentCount: number;
  runStatus: RunStatus;
  policyState: "allowed" | "policy_denied" | "cost_limited" | "runner_offline";
  group: "Assigned" | "Needs review" | "Blocked";
}

export interface Comment {
  id: string;
  author: string;
  role: "human" | "agent";
  body: string;
  createdAt: string;
  replies?: Comment[];
}

export interface IssueDetail extends IssueSummary {
  description: string;
  acceptance: string[];
  parent?: string;
  children: string[];
  comments: Comment[];
}

export interface RunStep {
  id: string;
  label: string;
  status: RunStatus;
  timestamp: string;
  detail: string;
  severity: Severity;
}

export interface SetupStep {
  id: string;
  title: string;
  state: "complete" | "active" | "locked" | "error";
  detail: string;
}

export interface ConsoleData {
  workingGroup: WorkingGroup;
  localePreferences: {
    userLanguage?: string;
    workingGroupDefaultLanguage?: string;
    browserLanguage?: string;
    formattingLocale?: string;
    timeZone?: string;
  };
  issues: IssueSummary[];
  selectedIssue: IssueDetail;
  runSteps: RunStep[];
  setupSteps: SetupStep[];
  firstRunOnboarding: FirstRunOnboarding;
}

export interface TaskOtterDataAdapter {
  getConsoleData(): Promise<ConsoleData>;
}
