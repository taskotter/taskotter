# i18n QA And E2E Matrix

This matrix is the repository-local QA artifact for TaskOtter localization work.
It turns the roadmap QA requirements into checks that can be automated or run as
manual release charters as each surface becomes available.

## Current Scope

In scope for this repository now:

- Web app shell, issue list, issue detail, threaded comments, run progress, setup
  path, loading state, and unavailable state.
- Client i18n runtime: locale precedence, fallback, pseudo-localization,
  interpolation, missing-key behavior, date/time, number, and currency
  formatting helpers.
- Server-message rendering policy for notification and email templates.
- Hard-coded UI string scanner, module-boundary checks, contract fixtures, unit
  tests, and Playwright app-shell smoke tests.

Out of scope until implemented elsewhere:

- Desktop shell native menus and operating-system notification copy.
- Mobile shell layout, push permission copy, and mobile-specific accessibility
  checks.
- Saved user language settings, Working Group language administration UI, and
  persistence across sessions.
- Real first-locale translation completeness. The first additional production
  locale is still a placeholder decision; `en-XA` is only a pseudo-locale.
- Live email, push, or in-app notification delivery. Current checks cover
  template rendering and resource-key policy only.

## Locale Profiles

| Profile                    | Purpose                                                   | Current support                         | Required gate                                                                             |
| -------------------------- | --------------------------------------------------------- | --------------------------------------- | ----------------------------------------------------------------------------------------- |
| `en`                       | Product fallback and source baseline                      | Implemented                             | Unit, hard-coded string scan, Playwright smoke                                            |
| `en-XA`                    | Pseudo-localized expansion and accent smoke               | Implemented                             | Unit plus UI smoke for visible labels and accessible names                                |
| First production locale    | First real translated locale after product decision       | Not implemented                         | Translation completeness, formatter, layout, a11y, notification, and support-review gates |
| Unsupported locale         | Deterministic fallback behavior                           | Implemented through fallback resolution | Unit tests for user, Working Group, browser, and fallback precedence                      |
| Distinct formatting locale | Content language differs from number/date/currency locale | Implemented in runtime helpers          | Unit tests for formatter locale and explicit time zone                                    |

## Behavior Matrix

| Behavior                                              | Current automated coverage                                                              | Manual or future E2E coverage                                                                                                                |
| ----------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| English baseline text renders through resources       | `npm run check:i18n`, `npm run test:unit`, `npm run test:ui`                            | Spot-check app shell, issue list/detail, setup path, loading, unavailable, validation, empty, and destructive states before release          |
| Pseudo-localized long strings                         | `src/i18n/i18n.test.ts`, `src/App.test.tsx`                                             | Playwright screenshot comparison at desktop and mobile widths; fail on overlap, truncation of critical states, or hidden controls            |
| Missing keys                                          | `src/i18n/i18n.test.ts` asserts safe fallback copy                                      | Release smoke should verify users never see raw keys in primary web, desktop, mobile, notification, or email surfaces                        |
| Fallback precedence                                   | `src/i18n/i18n.test.ts`, `src/serverMessages.test.ts`                                   | Settings E2E once user language and Working Group default persistence exist                                                                  |
| Interpolation                                         | `src/i18n/i18n.test.ts`, `src/serverMessages.test.ts`                                   | Verify issue keys, agent names, provider names, amounts, and request IDs remain unlocalized identifiers                                      |
| Plurals                                               | `src/i18n/i18n.test.ts`                                                                 | Real first-locale review must cover locale plural rules before launch                                                                        |
| Date/time formatting                                  | `src/i18n/i18n.test.ts`                                                                 | Due dates, scheduled automations, audit timestamps, notification timing, and cross-time-zone handoff flows                                   |
| Number and currency formatting                        | `src/i18n/i18n.test.ts`                                                                 | Usage, cost, quota, billing, token, percentage, and file-size surfaces when those views ship                                                 |
| Timezone-sensitive flows                              | `src/i18n/i18n.test.ts` covers explicit runtime time zone                               | E2E for due dates, scheduled automations, audit display, reminders, and notification delivery when backed by API data                        |
| Accessibility names and ARIA text                     | `src/App.test.tsx`, `tests/e2e/app-shell.spec.ts`                                       | Keyboard-only and screen-reader smoke for nav, menus, tabs, dialogs, forms, live regions, and error announcements in every locale            |
| Validation, empty, loading, error, destructive states | Loading and unavailable copy are resource-backed; hard-coded scanner runs in `npm test` | Add component or E2E states as the corresponding forms/dialogs ship; destructive confirmations require locale review                         |
| Notification and email templates                      | `src/serverMessages.test.ts`                                                            | Delivery-channel smoke for subject, title, body, action label, fallback text, accessibility text, redaction, and user-authored interpolation |
| API/server-message policy                             | `src/serverMessages.test.ts`, contract fixtures                                         | API contract checks must keep client behavior on stable codes and `message_key`, not localized prose                                         |

## Playwright UI Gate

For UI-impacting localization changes, run `npm run test:ui` and include at
least these assertions in the affected tests:

- English baseline and `en-XA` pseudo-locale render the same critical workflow.
- Navigation, buttons, textboxes, forms, status badges, and panels keep localized
  accessible names.
- Long strings do not push metadata chips, buttons, form labels, dialogs, or
  status text outside their containers at desktop and mobile widths.
- Missing translations fall back to safe user copy instead of raw keys.
- Dynamic identifiers such as issue keys, request IDs, agent names, provider
  names, and amounts remain visible and are not pseudo-localized unless they are
  declared system-authored copy.

## Release Checklist

- `npm run format`
- `npm run lint`
- `npm run typecheck`
- `npm test`
- `npm run test:ui` for UI-impacting changes
- Record which locale profiles passed, which profiles are not implemented, and
  which checks remain manual or deferred.
