import type { TranslationResource } from "../../types";

export const emails: TranslationResource = {
  "approval.requested.subject": "Approval requested for {issueKey}",
  "approval.requested.title": "{requesterName} requested approval",
  "approval.requested.body":
    "Review {issueKey} in {workingGroupName}. User-provided context: {userContext}.",
  "approval.requested.action": "Review approval",
  "approval.requested.accessibility":
    "Email for approval request {approvalId}.",
  "failure.summary.subject": "Failure summary for {issueKey}",
  "failure.summary.title": "{issueKey} needs review",
  "failure.summary.body":
    "Latest safe status: {safeStatusSummary}. Diagnostic summary: {diagnosticSummary}.",
  "failure.summary.action": "Open summary",
  "failure.summary.accessibility":
    "Email containing a redacted failure summary for {issueKey}.",
};
