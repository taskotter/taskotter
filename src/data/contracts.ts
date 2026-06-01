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

export type LocalizedText = {
  key: import("../i18n/types").TranslationKey;
  values?: import("../i18n/types").TranslationValues;
};

export type DisplayText = LocalizedText | { text: string };

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
  group: "assigned" | "needs_review" | "blocked";
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
  label: DisplayText;
  status: RunStatus;
  timestamp: DisplayText;
  detail: DisplayText;
  severity: Severity;
}

export interface SetupStep {
  id: string;
  title: LocalizedText;
  state: "complete" | "active" | "locked" | "error";
  detail: DisplayText;
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
}

export interface TaskOtterDataAdapter {
  getConsoleData(): Promise<ConsoleData>;
}
