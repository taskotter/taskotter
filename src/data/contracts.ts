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

export type DemoReviewScenario =
  | "happy_path"
  | "missing_evidence"
  | "failed_test"
  | "rework_requested"
  | "high_risk_approval_required";

export type ReviewRiskTier = "low" | "medium" | "high";

export type PlanApprovalState =
  | "not_required"
  | "pending"
  | "approved"
  | "rejected";

export type EvidenceCheckStatus =
  | "passed"
  | "failed"
  | "missing"
  | "not_applicable";

export interface DemoAcceptanceCriterion {
  id: string;
  text: string;
  required: boolean;
  satisfied: boolean;
}

export interface DemoPlanApproval {
  state: PlanApprovalState;
  requiredBefore: "agent_start" | "protected_side_effect" | "not_required";
  approvalRef?: string;
  approvedBy?: string;
  approvedAt?: string;
  rationale: string;
}

export interface DemoEvidenceCheck {
  name: string;
  status: EvidenceCheckStatus;
  summary: string;
  artifactRef?: string;
}

export interface DemoImportedEvidenceSummary {
  status: "complete" | "partial" | "failed";
  sourceRunId: string;
  importedAt: string;
  changedFilesCount: number;
  artifactCount: number;
  checks: DemoEvidenceCheck[];
  missingEvidence?: string[];
}

export interface DemoReviewPacket {
  packetId: string;
  generatedAt: string;
  summary: string;
  changedFiles: string[];
  artifacts: string[];
  acceptanceChecklist: DemoAcceptanceCriterion[];
  riskSignals: string[];
  uncertainty: string[];
  rollbackGuidance: string;
  reworkGuidance: string;
  decisionRecommendation: "approve_done" | "request_rework" | "manual_approval";
}

export interface DemoAuditCorrelation {
  correlationId: string;
  requestId: string;
  policyDecisionId?: string;
  auditEventIds: string[];
  runTimelineEventIds: string[];
}

export interface DemoReviewTimeMetric {
  baselineSeconds: number;
  observedSeconds: number;
  startedAt: string;
  completedAt?: string;
  reviewerRole: "human_reviewer" | "qa_agent" | "delivery_lead";
}

export interface DemoRedactionSafety {
  dataClassification: "generated_fake";
  fakeOnly: true;
  secretShapedCase?: {
    inputLabel: string;
    displayValue: "[REDACTED_FAKE_SECRET_SHAPED_VALUE]";
    rawValueStored: false;
    validationNote: string;
  };
}

export interface DemoReviewWorkItem {
  scenario: DemoReviewScenario;
  id: string;
  key: string;
  title: string;
  status: IssueStatus;
  riskTier: ReviewRiskTier;
  assignee: string;
  requestSource: "github_issue" | "multica_issue" | "manual_request";
  acceptanceCriteria: DemoAcceptanceCriterion[];
  planApproval: DemoPlanApproval;
  importedEvidenceSummary: DemoImportedEvidenceSummary;
  reviewPacket: DemoReviewPacket;
  auditCorrelation: DemoAuditCorrelation;
  reviewTimeMetric: DemoReviewTimeMetric;
  redactionSafety: DemoRedactionSafety;
}

export interface DemoReviewControlSeed {
  fixtureId: string;
  generatedAt: string;
  description: string;
  workItems: DemoReviewWorkItem[];
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
  demoReviewControlSeed?: DemoReviewControlSeed;
}

export interface TaskOtterDataAdapter {
  getConsoleData(): Promise<ConsoleData>;
}
