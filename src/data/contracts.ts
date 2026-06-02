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

export type ReviewSignalState =
  | "ready"
  | "loading"
  | "empty"
  | "error"
  | "missing"
  | "high_risk";

export type ReviewDecisionKind =
  | "approve"
  | "request_changes"
  | "done"
  | "rework";

export interface ReviewControlSignal {
  id: string;
  label: string;
  detail: string;
  state: ReviewSignalState;
}

export interface ReviewControlPlanStep {
  id: string;
  title: string;
  detail: string;
  status: "ready" | "needs_attention" | "blocked";
}

export interface ReviewControlEvidence {
  id: string;
  label: string;
  detail: string;
  state: ReviewSignalState;
}

export interface ReviewControlData {
  request: {
    key: string;
    title: string;
    source: string;
    summary: string;
  };
  riskTier: "low" | "medium" | "high";
  autonomyLevel: string;
  planSteps: ReviewControlPlanStep[];
  evidence: ReviewControlEvidence[];
  signals: ReviewControlSignal[];
  reviewChecklist: string[];
  rollbackGuidance: string;
  auditEvents: string[];
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

export type DemoReviewPacketSeverity = "info" | "warning" | "danger";

export type DemoVerificationStatus =
  | "passed"
  | "failed"
  | "not_run"
  | "blocked";

export type DemoReviewPacketEvidenceKind =
  | "test"
  | "lint"
  | "typecheck"
  | "build"
  | "review"
  | "runtime";

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

export interface DemoReviewPacketArtifact {
  path: string;
  kind: "source" | "test" | "fixture" | "contract" | "doc" | "config";
  summary?: string;
  riskTags?: readonly string[];
}

export interface DemoReviewPacketChecklistItem {
  id: string;
  text: string;
  status: "covered" | "missing";
  evidenceRefs: readonly string[];
}

export interface DemoReviewPacketSignal {
  code:
    | "missing_acceptance_evidence"
    | "missing_tests"
    | "verification_failed"
    | "verification_blocked"
    | "high_risk_change"
    | "rework_requested";
  severity: DemoReviewPacketSeverity;
  message: string;
  evidenceRefs: readonly string[];
}

export interface DemoReviewPacketVerificationEvidence {
  id: string;
  kind: DemoReviewPacketEvidenceKind;
  status: DemoVerificationStatus;
  summary: string;
  command?: string;
  artifactRefs: readonly string[];
  correlationId?: string;
}

export interface DemoCanonicalReviewPacket {
  schemaVersion: "review_packet.v0";
  issueKey: string;
  summary: string;
  changedArtifacts: readonly DemoReviewPacketArtifact[];
  acceptanceChecklist: readonly DemoReviewPacketChecklistItem[];
  riskSignals: readonly DemoReviewPacketSignal[];
  uncertainty: string[];
  rollbackOrReworkGuidance: string;
  verificationEvidence: readonly DemoReviewPacketVerificationEvidence[];
  missingEvidenceWarnings: readonly string[];
  audit: {
    correlationIds: readonly string[];
    redactions: readonly string[];
  };
}

export interface DemoAuditChainSummary {
  correlationId: string;
  requestId: string;
  policyDecisionId?: string;
  approvalId?: string;
  evidenceImportId: string;
  reviewPacketId: string;
  doneDecisionId?: string;
  reworkDecisionId?: string;
  workflowPath:
    | "done_approved"
    | "missing_evidence"
    | "rework_requested"
    | "denied";
  eventIds: string[];
}

export interface DemoReviewTimeSummaryMetric {
  source: "demo_summary_metric_not_event_telemetry";
  derivedFromEventTelemetry: false;
  baselineHumanReviewMinutes: number;
  humanReviewMinutes: number;
  humanMinutesPerCompletedAgentTask?: number;
  completedAgentTasks: number;
  reworkLoops: number;
  missingStopEvents: number;
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
  reviewPacket: DemoCanonicalReviewPacket;
  demoAuditChainSummary: DemoAuditChainSummary;
  demoReviewTimeSummaryMetric: DemoReviewTimeSummaryMetric;
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
  firstRunOnboarding: FirstRunOnboarding;
  reviewControl: ReviewControlData;
  demoReviewControlSeed?: DemoReviewControlSeed;
}

export interface TaskOtterDataAdapter {
  getConsoleData(): Promise<ConsoleData>;
}
