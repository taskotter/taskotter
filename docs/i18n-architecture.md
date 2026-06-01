# Internationalization Architecture

TaskOtter treats internationalization as product architecture. UI copy,
notifications, email templates, validation errors, accessibility text, and
user-facing status messages must be localizable without changing control flow or
mutating user-authored content.

This document defines the repository-local convention for the `taskotter`
application. It is intentionally an architecture and resource-convention
artifact, not a complete locale implementation.

## Goals

- Keep system-authored text behind stable translation keys.
- Keep API, realtime, audit, usage, runner, and gateway behavior dependent on
  stable codes and structured fields, not localized strings.
- Separate content language from formatting locale and time zone.
- Make accessibility labels, empty states, loading states, error states,
  validation messages, destructive confirmations, notification copy, and email
  copy part of the same localization path as visible UI text.
- Define QA gates that catch missing keys, long-string layout failures,
  pseudo-localization regressions, locale formatting bugs, and time-zone
  mistakes before release.

## Non-Goals

- Translating the product into an additional locale in this change.
- Choosing or purchasing a paid translation platform.
- Automatically translating user-authored issue titles, comments, prompts,
  artifacts, attachments, labels, or logs.
- Defining public marketing or SEO localization policy.
- Changing production, release, or protected-branch behavior.

## Resource Layout

When implementation starts, translation resources should live under an app-local
resource root:

```text
src/i18n/
  config.ts
  formatters.ts
  keys.ts
  locales/
    en/
      common.json
      navigation.json
      issues.json
      agents.json
      settings.json
      notifications.json
      emails.json
      errors.json
      accessibility.json
    pseudo/
      ...
```

Namespace ownership:

| Namespace       | Owner                                       | Examples                                           |
| --------------- | ------------------------------------------- | -------------------------------------------------- |
| `common`        | shared UI primitives                        | Save, Cancel, Retry, loading, empty copy           |
| `navigation`    | app shell                                   | sidebar labels, workspace switcher, command labels |
| `issues`        | issue surfaces                              | statuses, filters, composer, review states         |
| `agents`        | agent and run surfaces                      | run progress, tool states, approval prompts        |
| `settings`      | account and Working Group settings          | language, formatting locale, time zone             |
| `notifications` | in-app and push notifications               | assignment, approval, run completion               |
| `emails`        | transactional email templates               | invites, approvals, summaries                      |
| `errors`        | user-facing API and validation presentation | stable error-code mappings                         |
| `accessibility` | non-visible assistive text                  | ARIA labels, live-region updates, shortcut labels  |

Backend-owned or cross-repository message catalogs may use the same namespace
names, but clients must consume them through generated contract fields or a
versioned resource package rather than importing backend internals.

## Key Naming

Keys describe product meaning, not an English sentence fragment.

Use:

```text
issues.detail.status.inProgress
issues.composer.placeholder
agents.runProgress.step.running
errors.validation.requiredField
notifications.assignment.created.title
accessibility.issueRow.selected
```

Avoid:

```text
click_here
new_run_button_text
request_validation_failed_period
```

Rules:

- Use lower camel case segments after the namespace.
- Keep keys stable across copy edits.
- Use one key for one product meaning. Do not reuse a key only because English
  text happens to match.
- Keep interpolation names semantic, for example `{issueKey}`, `{agentName}`,
  `{count}`, `{amount}`, `{dueAt}`.
- Never build translated sentences by concatenating partial translated strings.
- Rich text boundaries must be explicit. Translators should not edit raw HTML.

## Interpolation, Plurals, And Formatting

Translation strings may contain variables, but locale-sensitive formatting is
owned by formatter helpers.

Examples:

- Counts use plural-aware message selection.
- Dates and times use `Intl.DateTimeFormat` with an explicit formatting locale
  and time zone.
- Relative time uses `Intl.RelativeTimeFormat`.
- Numbers, percentages, currency, token counts, file sizes, and costs use
  locale-aware formatters.
- Durations and schedules must not infer the user's time zone from text.

Do not pass preformatted English strings into translations unless they are
official names, identifiers, code values, file names, URLs, or immutable product
terms.

## Locale Preference And Precedence

TaskOtter has three related settings:

- Content language: language for system-authored product copy.
- Formatting locale: locale used for dates, numbers, currencies, sorting, and
  list formatting.
- Time zone: time zone used for due dates, scheduled automations, audit display,
  and notification timing.

Precedence:

1. Explicit user setting.
2. Explicit Working Group default.
3. Browser or OS locale as an initial suggestion for the first session.
4. Product fallback locale, initially `en`.

Rules:

- Browser or OS locale must not override a saved user setting.
- Content language and formatting locale may be different.
- Time zone must be stored and resolved separately from language.
- Missing translations fall back deterministically to `en`, then to a safe
  developer-visible marker in non-production checks.
- Production UI must not expose raw translation keys to end users.
- Locale persistence must be covered by tests once settings are implemented.

## User-Authored Versus System-Authored Content

User-authored content remains as entered unless a future translation feature is
explicitly approved.

User-authored examples:

- Issue titles and descriptions.
- Comments and chat messages.
- Prompts and agent instructions.
- Attachments and generated artifacts.
- User-created labels, skill descriptions, workflow names, and integration
  names.

System-authored examples:

- Navigation labels, buttons, menus, tabs, tooltips, and headings.
- Empty, loading, disabled, success, warning, and error states.
- Validation errors and recovery guidance.
- Approval, destructive-action, cost, policy, and permission confirmations.
- Notification and email templates.
- Accessibility labels, status announcements, and live-region text.

Persist stable facts and codes. Derive localized presentation at render time
where possible.

## API And Message Policy

Product clients must not rely on localized strings for control flow.

API error responses should expose stable machine-readable fields:

```json
{
  "error": {
    "code": "validation_failed",
    "message_key": "errors.validation_failed",
    "severity": "error",
    "retryable": false,
    "field_errors": [
      {
        "field": "title",
        "code": "required",
        "message_key": "errors.field.required"
      }
    ],
    "support": {
      "redacted": true
    }
  }
}
```

Client behavior:

- Branch on `error.code`, structured `field_errors[].code`, HTTP status, event
  type, status enum, and retry metadata.
- Map stable codes to localized presentation in the client or a versioned
  message catalog.
- Preserve `request_id`, `correlation_id`, issue keys, agent names, provider
  names, and other identifiers without translation.
- Treat `message_key` as a resource lookup key, not as display copy from the
  server.

Server, notification, and email behavior:

- Server-generated user-facing copy must use the same locale precedence as the
  product UI.
- Server-generated notification and email rendering uses the
  `serverMessageTemplates` catalog plus `notifications.*` and `emails.*`
  resources. The renderer resolves locale in the order user preference, Working
  Group default, browser suggestion, then the product fallback locale.
- Audit and compliance records should store stable event codes and raw facts,
  then localize display labels at presentation time.
- Notification and email templates must define subject, title, body, action
  labels, fallback text, and accessibility text through translation resources.
- User-authored issue/comment/prompt/artifact text may be passed only through
  declared interpolation variables and is not translated or used as a lookup
  key.
- Secrets, raw provider errors, private logs, credentials, and customer-sensitive
  payloads must never be inserted into localized strings. Template variables
  with sensitive names must be listed as redacted variables before rendering.

## Hard-Coded String Guardrails

Code review should reject new hard-coded user-visible strings in components,
service handlers, validation branches, fixtures intended for UI display, and
notification or email templates unless the string is one of:

- Product or company names.
- API identifiers, enum values, schema names, file names, issue keys, URLs, or
  command names.
- Test-only text scoped to a test fixture.
- Developer-only logs that are not surfaced to users.

Candidate automation:

- ESLint rule or custom scanner for JSX text and user-visible string props.
- Missing-key and unused-key checks in CI.
- Type-safe key helper generated from locale resources.
- Unit tests for locale precedence, fallback behavior, pluralization, and
  formatters.
- Playwright smoke tests using pseudo-localized and long-string resources.

## QA Gates

Minimum checks for localization-impacting changes:

| Gate                | Required evidence                                                                                                      |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Missing keys        | CI fails when required locale keys are absent or unresolved                                                            |
| Pseudo-localization | UI renders expanded pseudo strings without overlap or clipped critical text                                            |
| Long strings        | Dense app shell, issue rows, focus panel, composer, menus, and buttons remain usable                                   |
| Locale formatting   | Dates, times, relative time, numbers, percentages, currency, cost, and file sizes use formatter helpers                |
| Time zones          | Due dates, scheduled automations, audit timestamps, and notification timing resolve with explicit time-zone precedence |
| Accessibility       | Localized visible text and accessible names stay aligned; live regions and validation errors remain announced          |
| Error mapping       | API errors use stable codes and localized presentation without control flow depending on strings                       |
| Fallbacks           | Missing translations fall back deterministically without exposing raw keys in production                               |

Manual smoke coverage should include small, medium, and large viewports; keyboard
navigation; screen-reader spot checks for issue detail and run progress; and at
least one slow or failed request path.

## Follow-Up Work Split

1. Frontend implementation: choose the i18n runtime, add `src/i18n`, convert app
   shell and issue surface strings, and add typed key and formatter helpers.
2. Backend/API policy: standardize error codes, details reasons, event status
   codes, and localized presentation responsibilities across OpenAPI and
   generated clients.
3. Notifications and emails: define template resource ownership, locale
   selection, fallback behavior, and safe interpolation rules.
4. QA matrix: add pseudo-localization, missing-key, long-string, locale-format,
   time-zone, accessibility, and Playwright smoke coverage.
5. Release checklist: document locale completeness, known limitations, support
   triage, and rollback expectations for localized releases. See
   `docs/localization-release-readiness.md`.

## Initial Implementation Note

This first artifact does not add runtime i18n scaffolding because the active app
still uses a single fixture-driven React shell and the library/resource-loading
choice should be made with the frontend conversion issue. Adding a partial
runtime now would create a second convention before any product surface consumes
it.
