# Localization Release Readiness

This checklist defines the release-readiness evidence required before a
localization-impacting TaskOtter change is considered ready to ship. It
complements `docs/i18n-architecture.md`; it does not define the automated QA
matrix or add localization test implementation.

Use this checklist for pull requests, release candidates, and release notes
that add a new locale, expand translated surface area, change locale
formatting, alter server-message localization, or touch notification and email
copy.

## Release Scope

Record the localization scope in the pull request or release note:

- Supported content language, formatting locale, and time-zone behavior.
- Product surfaces changed, including web, desktop, mobile, backend-rendered
  messages, notifications, emails, settings, and support-facing messages.
- Translation namespaces changed, added, or intentionally deferred.
- Whether the change is a new locale, a copy update, an i18n runtime change, a
  formatter change, or a fallback behavior change.
- Known unsupported surfaces and user-visible limitations.

## Required Evidence

Before release, attach or link evidence for each applicable gate.

| Gate                             | Required evidence                                                                                                                                          |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Translation resources            | Required namespaces have complete locale resources for the changed surface, no duplicate keys, and no unused release-blocking keys.                        |
| Missing keys                     | Automated or manual missing-key checks show that production UI does not expose raw keys or unresolved placeholders.                                        |
| Pseudo-localization              | Pseudo-localized smoke coverage shows expanded strings in dense layouts without clipped critical text, overlapping controls, or unusable navigation.       |
| Locale formatting                | Dates, times, relative time, numbers, percentages, currency, token counts, file sizes, sorting, and list formatting use locale-aware formatter helpers.    |
| Time zones                       | Due dates, scheduled automations, audit timestamps, notification timing, and release notes distinguish content language, formatting locale, and time zone. |
| Accessibility                    | Visible text, accessible names, ARIA labels, alt text, validation messages, dialogs, menus, tabs, and live-region text remain aligned after localization.  |
| Notification and email templates | Subjects, titles, body copy, action labels, fallback text, and accessibility text resolve through localization resources with safe interpolation.          |
| API and server messages          | Client behavior depends on stable codes and structured fields, not localized strings; `message_key` values map to localized presentation.                  |
| Fallbacks                        | Missing or unsupported translations fall back deterministically without exposing raw keys in production.                                                   |
| Support triage                   | Release notes or support handoff identify how to report incorrect translations, ambiguous copy, locale-specific layout defects, and fallback defects.      |

## Pull Request Readiness

Localization-impacting pull requests should include:

- The exact checks run, such as `npm run check:i18n`, unit tests, Playwright
  smoke tests, or manual exploratory checks.
- Screenshots or screen recordings for changed high-density surfaces,
  responsive layouts, dialogs, menus, forms, and error states.
- Accessibility evidence for localized labels, validation errors, focus order,
  keyboard navigation, and live-region announcements where relevant.
- Confirmation that user-authored issue titles, comments, prompts, artifacts,
  labels, logs, and attachments are not translated or used as translation keys.
- Confirmation that secrets, raw provider errors, private logs, credentials,
  customer-sensitive payloads, and unredacted support data are not interpolated
  into localized strings.
- Review notes for critical approval, cost, permission, destructive-action, and
  recovery copy in each supported locale.
- Any known limitation that should be repeated in release notes.

If a pull request changes only documentation or test scaffolding, mark runtime
localization evidence as not applicable and explain why.

## Release Notes

Localization release notes should state:

- Newly supported locales or expanded localized surfaces.
- Whether changes affect content language, formatting locale, time-zone
  handling, fallback behavior, notification copy, email copy, or API error
  presentation.
- Known localization limitations, untranslated surfaces, or fallback behavior
  users may see.
- Any compatibility note for remote runners, gateways, mobile or desktop
  clients, generated API clients, or release branches.
- How users and support should report translation quality issues or
  locale-specific defects.

Do not imply a production release, support commitment, security fix disclosure,
or paid translation/vendor commitment unless that wording has been approved.

## Rollback Or Disable Strategy

Every localization release candidate needs a rollback or mitigation note:

- How to disable a partially localized surface, locale selector, or locale flag
  if the release exposes severe translation or layout defects.
- Whether fallback to `en` is sufficient, or whether feature flags, traffic
  rollback, client rollback, or release-branch mitigation is required.
- Whether server-rendered notification or email templates can be rolled back
  independently from the client.
- Whether API or contract changes are backward-compatible with existing clients.
- Which user data, if any, is affected by reverting language, formatting locale,
  or time-zone settings.

If rollback is not practical, document the mitigation and escalation path
instead of claiming that the release is reversible.

## Relationship To QA Automation

This checklist defines the release decision artifact. The QA and end-to-end
matrix owns the automated coverage for missing keys, pseudo-localization,
long-string layouts, locale formatting, time zones, accessibility, and fallback
behavior.

Release readiness should link to the latest QA evidence rather than duplicating
the test implementation plan in this document.
