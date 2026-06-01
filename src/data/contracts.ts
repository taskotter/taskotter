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

export interface WorkingGroup {
  id: string;
  name: string;
  plan: string;
  role: "owner" | "admin" | "member";
  memberCount: number;
  runnerState: "online" | "offline" | "limited";
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
  issues: IssueSummary[];
  selectedIssue: IssueDetail;
  runSteps: RunStep[];
  setupSteps: SetupStep[];
}

export interface TaskOtterDataAdapter {
  getConsoleData(): Promise<ConsoleData>;
}
