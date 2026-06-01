import type { TranslationResource } from "../../types";

export const notifications: TranslationResource = {
  "assignment.created.subject": "Issue assigned",
  "assignment.created.title": "{issueKey} needs your attention",
  "assignment.created.body":
    "{assigneeName} was assigned in {workingGroupName}.",
  "assignment.created.action": "Open issue",
  "assignment.created.accessibility":
    "Notification for assigned issue {issueKey}.",
  "run.failed_summary.subject": "Run failed",
  "run.failed_summary.title": "{runName} failed",
  "run.failed_summary.body":
    "TaskOtter stopped the run after a protected failure. Diagnostic summary: {diagnosticSummary}.",
  "run.failed_summary.action": "Review run",
  "run.failed_summary.accessibility": "Notification for failed run {runId}.",
};
