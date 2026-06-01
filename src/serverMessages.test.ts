import {
  renderServerMessageTemplate,
  serverMessageTemplates,
  type ServerMessageTemplateKey,
} from "./serverMessages";
import redactionCorpus from "../contracts/fixtures/redaction-secret-corpus.json";

describe("server message template rendering", () => {
  it("uses user preference before Working Group default for notification resources", () => {
    const message = renderServerMessageTemplate(
      "notification.assignment.created",
      {
        userLanguage: "en-XA",
        workingGroupDefaultLanguage: "en",
        browserLanguage: "fr-FR",
      },
      {
        issueKey: "BOG-498",
        assigneeName: "Backend Service Engineer",
        workingGroupName: "TaskOtter",
      },
    );

    expect(message.locale).toMatchObject({
      contentLocale: "en-XA",
      fallbackLocale: "en",
      source: "user",
    });
    expect(message.parts.title).toContain("[!!");
    expect(message.parts.title).toContain("BOG-498");
    expect(message.resourceKeys.title).toBe(
      "notifications.assignment.created.title",
    );
  });

  it("falls back deterministically to English when preferences are unsupported", () => {
    const message = renderServerMessageTemplate(
      "email.failure.summary",
      {
        userLanguage: "ko-KR",
        workingGroupDefaultLanguage: "fr-FR",
        browserLanguage: "de-DE",
      },
      {
        issueKey: "BOG-498",
        safeStatusSummary: "Retry stopped",
        diagnosticSummary: "provider error contained secret_token_value",
      },
    );

    expect(message.locale).toMatchObject({
      contentLocale: "en",
      fallbackLocale: "en",
      source: "fallback",
    });
    expect(message.parts.body).toContain("Diagnostic summary: redacted.");
    expect(message.parts.body).not.toContain("secret_token_value");
  });

  it("keeps user-authored content as an interpolation value without using it for template lookup", () => {
    const message = renderServerMessageTemplate(
      "email.approval.requested",
      {
        userLanguage: "en",
        workingGroupDefaultLanguage: "en-XA",
      },
      {
        approvalId: "approval_01J9Z4P4BS0M9P2QJ6T8Z6W2EE",
        issueKey: "BOG-498",
        requesterName: "Repository Contribution Steward",
        workingGroupName: "TaskOtter",
        userContext: "Please review this user-authored note.",
      },
    );

    expect(message.userAuthoredVariables).toEqual(["userContext"]);
    expect(message.parts.body).toContain(
      "Please review this user-authored note.",
    );
    expect(message.parts.subject).toBe("Approval requested for BOG-498");
  });

  it("rejects unknown or unredacted sensitive variables", () => {
    expect(() =>
      renderServerMessageTemplate(
        "notification.assignment.created",
        { userLanguage: "en" },
        {
          issueKey: "BOG-498",
          assigneeName: "Backend Service Engineer",
          workingGroupName: "TaskOtter",
          secretToken: "secret_token_value",
        },
      ),
    ).toThrow(/Unknown template variable/);

    const template =
      serverMessageTemplates[
        "notification.run.failed_summary" satisfies ServerMessageTemplateKey
      ];
    expect(template.redactedVariables).toEqual(["diagnosticSummary"]);
  });

  it("redacts diagnostic summaries for secret-shaped fixture corpus values", () => {
    for (const secretCase of redactionCorpus.secret_shaped_cases) {
      if (!secretCase.surfaces.includes("diagnostic")) continue;

      const message = renderServerMessageTemplate(
        "notification.run.failed_summary",
        { userLanguage: "en" },
        {
          runId: "run_01J9Z4P4BS0M9P2QJ6T8Z6W2EA",
          runName: "Fixture redaction run",
          diagnosticSummary: secretCase.value,
        },
      );
      const rendered = Object.values(message.parts).join("\n");

      expect(rendered).toContain("redacted");
      expect(rendered).not.toContain(secretCase.value);
    }
  });
});
