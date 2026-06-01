import {
  createI18n,
  resolveLocalePreferences,
  type LocalePreferences,
  type ResolvedLocale,
} from "./i18n";
import type { TranslationKey } from "./i18n/types";

export type ServerMessageChannel = "in_app_notification" | "email";

export type ServerMessagePart =
  | "subject"
  | "title"
  | "body"
  | "action"
  | "accessibility";

type ServerMessageTemplate = {
  channel: ServerMessageChannel;
  eventCode: string;
  resourceKeys: Record<ServerMessagePart, TranslationKey>;
  variables: readonly string[];
  userAuthoredVariables: readonly string[];
  redactedVariables: readonly string[];
};

export const serverMessageTemplates = {
  "notification.assignment.created": {
    channel: "in_app_notification",
    eventCode: "notification.assignment.created",
    resourceKeys: {
      subject: "notifications.assignment.created.subject",
      title: "notifications.assignment.created.title",
      body: "notifications.assignment.created.body",
      action: "notifications.assignment.created.action",
      accessibility: "notifications.assignment.created.accessibility",
    },
    variables: ["issueKey", "assigneeName", "workingGroupName"],
    userAuthoredVariables: [],
    redactedVariables: [],
  },
  "notification.run.failed_summary": {
    channel: "in_app_notification",
    eventCode: "run.failed_summary",
    resourceKeys: {
      subject: "notifications.run.failed_summary.subject",
      title: "notifications.run.failed_summary.title",
      body: "notifications.run.failed_summary.body",
      action: "notifications.run.failed_summary.action",
      accessibility: "notifications.run.failed_summary.accessibility",
    },
    variables: ["runId", "runName", "diagnosticSummary"],
    userAuthoredVariables: [],
    redactedVariables: ["diagnosticSummary"],
  },
  "email.approval.requested": {
    channel: "email",
    eventCode: "workflow.approval.requested",
    resourceKeys: {
      subject: "emails.approval.requested.subject",
      title: "emails.approval.requested.title",
      body: "emails.approval.requested.body",
      action: "emails.approval.requested.action",
      accessibility: "emails.approval.requested.accessibility",
    },
    variables: [
      "approvalId",
      "issueKey",
      "requesterName",
      "workingGroupName",
      "userContext",
    ],
    userAuthoredVariables: ["userContext"],
    redactedVariables: [],
  },
  "email.failure.summary": {
    channel: "email",
    eventCode: "run.failure.summary",
    resourceKeys: {
      subject: "emails.failure.summary.subject",
      title: "emails.failure.summary.title",
      body: "emails.failure.summary.body",
      action: "emails.failure.summary.action",
      accessibility: "emails.failure.summary.accessibility",
    },
    variables: ["issueKey", "safeStatusSummary", "diagnosticSummary"],
    userAuthoredVariables: [],
    redactedVariables: ["diagnosticSummary"],
  },
} as const satisfies Record<string, ServerMessageTemplate>;

export type ServerMessageTemplateKey = keyof typeof serverMessageTemplates;

export type ServerMessageVariables = Record<
  string,
  string | number | undefined
>;

export interface RenderedServerMessage {
  channel: ServerMessageChannel;
  eventCode: string;
  locale: ResolvedLocale;
  parts: Record<ServerMessagePart, string>;
  resourceKeys: Record<ServerMessagePart, TranslationKey>;
  userAuthoredVariables: readonly string[];
  redactedVariables: readonly string[];
}

const sensitiveVariablePattern =
  /(secret|token|credential|password|private|raw|prompt|log|stack|trace)/i;

function sanitizeVariables(
  template: ServerMessageTemplate,
  variables: ServerMessageVariables,
  redactedLabel: string,
) {
  const allowed = new Set(template.variables);
  const redacted = new Set(template.redactedVariables);
  const sanitized: Record<string, string | number> = {};

  for (const [name, value] of Object.entries(variables)) {
    if (!allowed.has(name)) {
      throw new Error(`Unknown template variable: ${name}`);
    }

    if (sensitiveVariablePattern.test(name) && !redacted.has(name)) {
      throw new Error(`Sensitive template variable must be redacted: ${name}`);
    }

    if (value === undefined) continue;
    sanitized[name] = redacted.has(name) ? redactedLabel : value;
  }

  for (const name of template.variables) {
    if (!(name in sanitized)) {
      throw new Error(`Missing template variable: ${name}`);
    }
  }

  return sanitized;
}

export function renderServerMessageTemplate(
  templateKey: ServerMessageTemplateKey,
  preferences: LocalePreferences,
  variables: ServerMessageVariables,
): RenderedServerMessage {
  const template = serverMessageTemplates[templateKey];
  const locale = resolveLocalePreferences(preferences);
  const i18n = createI18n(locale);
  const sanitizedVariables = sanitizeVariables(
    template,
    variables,
    i18n.t("commonErrors.redacted"),
  );

  return {
    channel: template.channel,
    eventCode: template.eventCode,
    locale,
    resourceKeys: template.resourceKeys,
    userAuthoredVariables: template.userAuthoredVariables,
    redactedVariables: template.redactedVariables,
    parts: {
      subject: i18n.t(template.resourceKeys.subject, sanitizedVariables),
      title: i18n.t(template.resourceKeys.title, sanitizedVariables),
      body: i18n.t(template.resourceKeys.body, sanitizedVariables),
      action: i18n.t(template.resourceKeys.action, sanitizedVariables),
      accessibility: i18n.t(
        template.resourceKeys.accessibility,
        sanitizedVariables,
      ),
    },
  };
}
