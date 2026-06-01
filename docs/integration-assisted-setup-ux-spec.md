# Integration-Assisted Setup UX Spec

Status: product/spec draft for BOG-601.

Mode: standard product review. This is a user-facing setup flow with permission,
privacy, accessibility, and cross-system state-boundary impact, but it does not
authorize release, production credentials, or live integration enablement.

## Source Context

This spec translates the private roadmap candidate for integration-assisted
agent, skill, routing, and workspace setup into implementation-ready product
requirements for TaskOtter.

Roadmap inputs:

- `taskotter-roadmap/docs/capabilities.md`: Integration-assisted setup should
  inspect allowed external metadata, summarize project structure, and propose
  agents, skills, routing rules, workspace defaults, and sync boundaries.
- `taskotter-roadmap/docs/planning.md`: External metadata may seed TaskOtter
  setup, while TaskOtter keeps Work Graph, Event Log, policy, audit, and
  execution state as the owned workflow boundary.

## Users And Jobs

Primary user: workspace admin or operations lead configuring TaskOtter for a
team that already works in tools such as Confluence, Linear, Jira, GitHub
Issues, or Git repositories.

Secondary users:

- Security or compliance reviewer validating requested metadata access before
  approval.
- Future implementation and QA owners splitting backend, frontend, and security
  work from this spec.

Jobs to be done:

- Understand what external context TaskOtter wants to inspect before granting
  access.
- Preview generated TaskOtter configuration before it affects a workspace.
- Accept, edit, or reject proposed agents, skills, routing rules, and defaults.
- Preserve a clear boundary between imported external context and TaskOtter-owned
  workflow, policy, event, and audit state.

## Goals

- Provide an assisted setup flow that turns external metadata into proposed
  TaskOtter configuration.
- Make permission scope, data use, and generated output understandable before
  any approval or save action.
- Require explicit user action for every generated configuration change.
- Keep credentials, webhooks, and third-party authorization setup outside this
  spec except as gated placeholders and follow-up requirements.
- Produce backend, frontend, and security acceptance criteria that can become
  follow-up issues.

## Non-Goals

- Real OAuth, SAML, webhook, app-install, token exchange, credential storage, or
  production third-party access implementation.
- Silent production creation of agents, skills, routes, workspace defaults, or
  external sync jobs.
- Bidirectional workflow-state sync with external platforms.
- Public documentation publication.

## Product Principle

External systems can provide context and trigger candidates. TaskOtter owns the
resulting workflow state. No imported field becomes Work Graph, Event Log,
policy decision, audit record, execution state, or production routing state until
TaskOtter creates its own first-party record through an explicit acceptance path.

## Information Architecture

Entry point:

- Workspace setup or admin settings surface: "Add integration-assisted setup".
- Available only to users with workspace configuration permissions.

Flow sections:

1. Select source system.
2. Review metadata permission scope.
3. Connect or simulate access.
4. Inspect metadata summary.
5. Preview proposed TaskOtter setup.
6. Accept, edit, or reject proposal groups.
7. Review final changes.
8. Save draft or apply approved TaskOtter configuration.
9. Show completion, audit summary, and next steps.

The first version may support a no-credential demo or fixture-backed preview so
teams can validate the proposal UX without live third-party access.

## Key Screens And States

### 1. Source Selection

Purpose: choose the external source type and explain what TaskOtter can use it
for.

Required content:

- Source cards for Confluence, Linear, Jira, GitHub Issues, and Git repositories.
- Each source states supported metadata categories, setup status, and whether the
  flow is live, fixture-backed, or unavailable.
- Disabled sources explain the missing capability without implying a credential
  failure.

States:

- Empty: no sources enabled for this workspace.
- Loading: source templates are being fetched.
- Error: source template list failed, with retry.
- Disabled/unavailable: source is visible but cannot be selected.

### 2. Permission Scope Review

Purpose: show exactly which metadata TaskOtter asks to inspect and why.

Required content:

- Scope grouped by category: projects/spaces, labels/components, issue/document
  types, repository names, branch names, file paths, README or docs index
  metadata, and team or ownership labels where available.
- For each category: purpose, example values, whether stored, retention policy,
  and whether the value can influence generated configuration.
- "Not requested" list for sensitive or out-of-scope fields such as issue body
  text, document body text, comments, attachments, secrets, raw commit contents,
  private keys, tokens, customer PII, payment data, and credentials.

States:

- Scope unavailable: provider has no template or the template version is unknown.
- Scope conflict: user permission is lower than the selected setup goal.
- Security review required: requested scope exceeds workspace policy threshold.

### 3. Connect Or Simulate Access

Purpose: collect permission intent without implementing credential handling in
this spec.

Required content:

- Placeholder action for "Connect account" or "Use sample metadata".
- Clear copy that credential exchange, webhook creation, and token storage are
  handled by separate implementation work.
- Policy gate indicator before any live third-party authorization can be started.

States:

- Not connected.
- Fixture preview selected.
- Connection pending.
- Connection denied or cancelled.
- Connection expired or revoked.

### 4. Metadata Summary

Purpose: let the admin verify what TaskOtter inferred before proposals appear.

Required content:

- Summary of discovered projects, spaces, repositories, labels, issue types,
  components, document spaces, and high-level ownership patterns.
- Data freshness and source timestamp.
- Counts and representative examples, not bulk raw external data.
- Warning when the metadata is partial, stale, or filtered by permission.

States:

- No usable metadata found.
- Partial metadata due to limited permission.
- Provider rate-limited or temporarily unavailable.
- Conflicting labels or project states detected.

### 5. Proposal Preview

Purpose: present generated TaskOtter configuration as proposed changes, not
applied state.

Proposal groups:

- Suggested agents or agent teams.
- Suggested skills.
- Suggested routing rules.
- Suggested workspace defaults.
- Suggested evidence expectations and review gates.
- Suggested integration access boundaries.

Each proposal item must show:

- Source metadata that influenced the proposal.
- Confidence or rationale.
- Impacted TaskOtter object type.
- Default action: accept, edit, or reject.
- Validation status and blocking issues.
- Whether applying it creates new TaskOtter-owned records.

The preview must support bulk decisions at group level and item-level overrides.
The default for high-impact or low-confidence items is review required, not
preselected acceptance.

### 6. Accept, Edit, Reject

Purpose: give the admin control over generated configuration.

Behavior:

- Accept marks a proposed item for creation or update in TaskOtter.
- Edit opens a constrained editor for the target object type.
- Reject removes the item from the pending apply set and records a local reason
  when the user provides one.
- Undo is available until final apply.
- Rejecting all proposal items leaves the integration metadata un-applied and
  offers "save summary only" or "start over".

Edit constraints:

- Generated agents cannot be granted broader external access than the reviewed
  permission scope.
- Routing rules must show trigger conditions and affected work item types.
- Workspace defaults must identify which future objects they affect.
- Evidence and review gates must remain explicit approval gates.

### 7. Final Review And Apply

Purpose: confirm the exact TaskOtter records that will be created or changed.

Required content:

- Diff-style summary grouped by object type.
- Permission and policy gate summary.
- TaskOtter-owned state that will be created.
- External state that will remain read-only or external-only.
- Required approvals or blockers.

Apply behavior:

- Nothing is applied until the user confirms final review.
- If an item fails validation, the user can remove it, edit it, or return to
  preview.
- Applying partial successful items must identify skipped and failed items.
- A completion state links to created TaskOtter records and the audit summary.

## Metadata Permission Scope

Allowed first-version metadata categories:

| Platform family                | Allowed metadata                                                                                                                                                  | Setup use                                                                                                    |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Confluence-like docs           | Space names, page tree structure, labels, owner/team metadata when exposed, freshness timestamps                                                                  | Suggest knowledge-focused agents, documentation skills, evidence expectations, and routing by document space |
| Linear/Jira-like issue systems | Project names, issue types, labels, components, workflow state names, priority names, team names, custom-field names without raw values unless explicitly allowed | Suggest delivery agents, routing rules, issue templates, review gates, and workspace defaults                |
| GitHub Issues-like trackers    | Repository names, labels, milestones, issue templates, project names, assignee/team metadata where permissioned                                                   | Suggest triage agents, routing rules, issue templates, and evidence expectations                             |
| Git repositories               | Repository names, default branch names, top-level directory names, package manifests, CODEOWNERS-style ownership metadata, README/doc index metadata              | Suggest repo-aware skills, module-specific routing, test expectations, and review gates                      |

Excluded by default:

- Credentials, tokens, private keys, webhook secrets, session cookies.
- Raw issue bodies, comments, attachments, document bodies, private discussion
  content, and full commit diffs.
- Customer PII, billing data, payment data, legal content, and regulated data
  categories unless a separate security review approves a narrower scope.
- External workflow state as an authoritative TaskOtter workflow state.

Storage and retention requirements:

- Store only normalized metadata needed to explain a proposal and support audit.
- Prefer references, hashes, counts, and short labels over raw copied external
  content.
- Show retention behavior before access is granted.
- Provide an admin-visible way to discard imported metadata before applying
  proposals.

## TaskOtter-Owned State Boundary

TaskOtter-owned after explicit apply:

- Work Graph nodes and edges created from accepted setup items.
- Event Log entries documenting proposal generation, edits, approvals, rejections,
  and apply outcomes.
- Policy decisions authorizing setup actions.
- Audit records for permission review, proposal changes, and final apply.
- Agent, skill, route, and workspace-default records created in TaskOtter.
- Execution state for any TaskOtter-run workflow.

External-only context unless a future bidirectional contract exists:

- External issue status, project status, document state, repository branch state,
  label definitions, comments, approvals, and history.
- External user or team identity beyond mapped references needed for display and
  policy checks.
- Provider-specific webhook delivery state.

Boundary rules:

- Imported metadata can explain a recommendation but cannot silently mutate
  TaskOtter config.
- External state changes can invalidate or refresh proposals, but cannot rewrite
  accepted TaskOtter records without a TaskOtter approval path.
- TaskOtter audit must record the source metadata version or timestamp used for
  generated proposals.
- Conflicts between external labels/states and TaskOtter issue states must be
  shown as mapping choices, not merged automatically.

## Accessibility And UX Requirements

- All setup steps, source cards, proposal items, menus, dialogs, and editors are
  keyboard reachable with visible focus.
- Proposal groups use semantic headings and lists so screen reader users can
  navigate by source, object type, and decision status.
- Accept/edit/reject controls have visible labels and programmatic names.
- Color is not the only indicator for accepted, edited, rejected, blocked, or
  low-confidence proposals.
- Dynamic proposal generation and apply results announce status through live
  regions or equivalent accessible feedback.
- Error messages identify the failed step, explain the consequence, and offer a
  recovery action.
- Long labels, repository names, and translated strings wrap without overlapping
  controls.
- Destructive or broad actions, such as rejecting all proposals or applying many
  routes, require clear confirmation and undo before final apply where practical.

## Empty, Loading, Error, And Edge States

- No integrations configured: explain that fixture preview is available if
  supported.
- No metadata returned: let the user change scope, select another source, or stop
  without creating TaskOtter config.
- Partial permission: show which proposal groups are unavailable and why.
- Provider error: preserve prior user decisions and allow retry.
- Stale metadata: require refresh or explicit continue-with-stale-data choice.
- Duplicate proposals: merge only after user review; otherwise show duplicates
  with source rationale.
- Conflicting mappings: require user choice for label, project, workflow state, or
  route conflicts.
- Policy denial: block apply and show the required approval path.
- Session expiry: preserve preview decisions locally where safe and ask the user
  to reconnect or restart.

## Success Metrics

- Admin can complete fixture-backed preview without granting live credentials.
- Admin can identify requested metadata scope before connect or simulate action.
- Admin can accept, edit, and reject proposal items without losing context.
- No generated configuration reaches production state without final review and
  explicit apply.
- Security reviewer can trace which metadata categories informed accepted
  configuration.
- QA can verify permission, accessibility, edge-state, and proposal-decision
  behavior before live integration work starts.

## Follow-Up Issue Acceptance Criteria

### Backend/API Follow-Up

- Defines a proposal-generation API that accepts a source type, permission scope,
  metadata snapshot reference, and fixture/live mode.
- Returns proposal groups for agents, skills, routing rules, workspace defaults,
  evidence expectations, and review gates with source rationale and validation
  status.
- Separates external metadata snapshot records from TaskOtter-owned Work Graph,
  Event Log, policy, audit, and execution records.
- Records proposal generation, edit, accept, reject, and apply outcomes in
  TaskOtter audit/event structures.
- Blocks apply when policy denies scope, when metadata is stale beyond configured
  limits, or when required approvals are missing.
- Provides fixture-backed data paths for Confluence-like, Jira/Linear-like,
  GitHub Issues-like, and Git repository sources without requiring real
  credentials.
- Includes contract tests for accepted, edited, rejected, partial, denied, stale,
  duplicate, and conflict proposal states.

### Frontend Follow-Up

- Implements the setup IA: source selection, permission scope review,
  connect/simulate, metadata summary, proposal preview, accept/edit/reject, final
  review, and completion.
- Uses existing TaskOtter UI density, terminology, and generated contract
  boundaries; frontend surfaces consume generated contracts rather than backend
  internals.
- Makes all proposal decisions reversible until final apply.
- Shows scope, rationale, confidence, validation status, source metadata
  timestamp, and final apply impact for each proposal.
- Covers empty, loading, partial-permission, provider error, stale metadata,
  duplicate, conflict, policy-denied, and session-expired states.
- Meets keyboard, focus, label, contrast, live-region, and long-text wrapping
  accessibility requirements.
- Includes Playwright or equivalent UI checks for the fixture-backed happy path,
  edit/reject path, policy-denied path, and at least one mobile-width responsive
  smoke check with screenshots or traces as review evidence.

### Security/Privacy Follow-Up

- Defines per-platform metadata allowlists and explicitly excluded fields.
- Defines policy gates for live credentials, webhook creation, imported metadata
  retention, proposal apply, and generated agent access to connected resources.
- Requires explicit admin consent before live third-party authorization,
  credential storage, webhook registration, or broad metadata collection.
- Verifies that raw credentials, secrets, tokens, private keys, cookies, customer
  PII, and full external content bodies are not stored in proposal records.
- Defines metadata retention, discard, and audit visibility requirements.
- Reviews public/private wording so setup copy does not imply TaskOtter owns or
  synchronizes external workflow state before a contract exists.
- Provides threat-model coverage for overbroad scope, confused-deputy routing,
  stale metadata, privilege escalation through generated agents, and accidental
  public exposure of private labels or repository structure.

## Open Questions

- Which source should be the first live provider after fixture-backed validation?
- Should admins be able to save proposal drafts for later review, or must they
  complete setup in one session?
- What retention window should apply to imported metadata snapshots that were
  rejected before apply?
- Which policy role can approve broader-than-default metadata scopes?
- Should confidence scores be numeric, categorical, or hidden behind rationale
  until generation quality is validated?
