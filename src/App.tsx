import {
  AlertTriangle,
  Bot,
  CheckCircle2,
  ChevronDown,
  CircleDollarSign,
  Clock3,
  Command,
  GitBranch,
  Inbox,
  LayoutDashboard,
  ListFilter,
  MessageSquare,
  PlayCircle,
  Search,
  Settings,
  ShieldCheck,
  SlidersHorizontal,
  Workflow,
  XCircle,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type {
  ConsoleData,
  FirstRunDenialReasonCode,
  FirstRunOnboarding,
  FirstRunTimelineItem,
  IssueStatus,
  IssueSummary,
  RunStatus,
  Severity,
  SetupStep,
} from "./data/contracts";
import { taskOtterDataAdapter } from "./data/taskotterAdapter";
import { createI18n, resolveLocalePreferences } from "./i18n";
import type { TranslationKey } from "./i18n/types";

type I18n = ReturnType<typeof createI18n>;
type PermissionMode = FirstRunOnboarding["permissions"]["role"];

const navItems = [
  { key: "inbox", labelKey: "webShell.nav.inbox", icon: Inbox, count: 8 },
  {
    key: "issues",
    labelKey: "webShell.nav.issues",
    icon: LayoutDashboard,
    count: 24,
  },
  { key: "chat", labelKey: "webShell.nav.chat", icon: MessageSquare, count: 3 },
  { key: "agents", labelKey: "webShell.nav.agents", icon: Bot, count: 12 },
  { key: "skills", labelKey: "webShell.nav.skills", icon: Workflow, count: 37 },
  {
    key: "providers",
    labelKey: "webShell.nav.providers",
    icon: GitBranch,
    count: 2,
  },
  {
    key: "usage",
    labelKey: "webShell.nav.usage",
    icon: CircleDollarSign,
    count: 1,
  },
  {
    key: "settings",
    labelKey: "webShell.nav.settings",
    icon: Settings,
    count: 0,
  },
];

const statusLabelKey: Record<IssueStatus | RunStatus, TranslationKey> = {
  blocked: "commonErrors.status.blocked",
  cancelled: "commonErrors.status.cancelled",
  completed: "commonErrors.status.completed",
  done: "commonErrors.status.done",
  failed: "commonErrors.status.failed",
  in_progress: "commonErrors.status.in_progress",
  in_review: "commonErrors.status.in_review",
  queued: "commonErrors.status.queued",
  retrying: "commonErrors.status.retrying",
  running: "commonErrors.status.running",
  triage: "commonErrors.status.triage",
  waiting_approval: "commonErrors.status.waiting_approval",
};

const statusSeverity: Record<IssueStatus | RunStatus, Severity> = {
  blocked: "danger",
  cancelled: "neutral",
  completed: "success",
  done: "success",
  failed: "danger",
  in_progress: "info",
  in_review: "warning",
  queued: "neutral",
  retrying: "warning",
  running: "info",
  triage: "neutral",
  waiting_approval: "warning",
};

function StatusBadge({
  i18n,
  status,
  label,
}: {
  i18n: I18n;
  status: IssueStatus | RunStatus | Severity;
  label?: string;
}) {
  const severity =
    status in statusSeverity
      ? statusSeverity[status as IssueStatus | RunStatus]
      : (status as Severity);
  const text =
    label ??
    (status in statusLabelKey
      ? i18n.t(statusLabelKey[status as IssueStatus | RunStatus])
      : status);

  return (
    <span className={`status-badge status-${severity}`}>
      <span className="status-dot" aria-hidden="true" />
      {text}
    </span>
  );
}

function policyLabel(i18n: I18n, policyState: IssueSummary["policyState"]) {
  return i18n.t(`commonErrors.policy.${policyState}` as TranslationKey);
}

function AppShell({ data, i18n }: { data: ConsoleData; i18n: I18n }) {
  const groupedIssues = useMemo(
    () =>
      data.issues.reduce<Record<IssueSummary["group"], IssueSummary[]>>(
        (groups, issue) => {
          groups[issue.group].push(issue);
          return groups;
        },
        { Assigned: [], "Needs review": [], Blocked: [] },
      ),
    [data.issues],
  );
  const memberLabel = i18n.plural(
    "webShell.workspace.member",
    data.workingGroup.memberCount,
  );

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true">
            {i18n.t("webShell.brand.initials")}
          </div>
          <div>
            <strong>{i18n.t("webShell.brand.name")}</strong>
            <span>{i18n.t("webShell.brand.subtitle")}</span>
          </div>
        </div>
        <button
          aria-label={i18n.t("webShell.workspace.switcherLabel", {
            name: data.workingGroup.name,
            role: data.workingGroup.role,
            memberCount: data.workingGroup.memberCount,
            memberLabel,
          })}
          className="workspace-switcher"
          type="button"
        >
          <span>
            <strong>{data.workingGroup.name}</strong>
            <small>
              {data.workingGroup.role} · {data.workingGroup.memberCount}{" "}
              {memberLabel}
            </small>
          </span>
          <ChevronDown size={16} aria-hidden="true" />
        </button>
        <nav className="nav-list" aria-label={i18n.t("webShell.nav.label")}>
          {navItems.map((item) => {
            const Icon = item.icon;
            const label = i18n.t(item.labelKey as TranslationKey);
            return (
              <a
                aria-label={label}
                aria-current={item.key === "issues" ? "page" : undefined}
                href={`#${item.key}`}
                key={item.key}
              >
                <Icon size={17} aria-hidden="true" />
                <span>{label}</span>
                {item.count > 0 ? <small>{item.count}</small> : null}
              </a>
            );
          })}
        </nav>
        <div className="sidebar-health">
          <StatusBadge
            i18n={i18n}
            status={
              data.workingGroup.runnerState === "online" ? "success" : "warning"
            }
            label={i18n.t("webShell.runner.state", {
              state: data.workingGroup.runnerState,
            })}
          />
          <p>{data.workingGroup.plan}</p>
        </div>
      </aside>

      <main className="workspace" aria-labelledby="issues-title">
        <header className="page-header">
          <div>
            <p className="eyebrow">{i18n.t("webShell.page.eyebrow")}</p>
            <h1 id="issues-title">{i18n.t("webShell.page.title")}</h1>
          </div>
          <div
            className="header-actions"
            aria-label={i18n.t("webShell.actions.label")}
          >
            <button type="button">
              <Command size={16} aria-hidden="true" />
              {i18n.t("webShell.actions.command")}
            </button>
            <button type="button" className="primary-action">
              <PlayCircle size={16} aria-hidden="true" />
              {i18n.t("webShell.actions.newRun")}
            </button>
          </div>
        </header>

        <section className="setup-band" aria-labelledby="setup-title">
          <div>
            <p className="eyebrow">{i18n.t("issues.setup.eyebrow")}</p>
            <h2 id="setup-title">{i18n.t("issues.setup.title")}</h2>
          </div>
          <ol className="setup-steps">
            {data.setupSteps.map((step) => (
              <SetupStepItem key={step.id} step={step} />
            ))}
          </ol>
        </section>

        <FirstRunOnboardingPanel
          onboarding={data.firstRunOnboarding}
          i18n={i18n}
        />

        <section
          className="toolbar"
          aria-label={i18n.t("issues.toolbar.label")}
        >
          <label className="search-field">
            <Search size={16} aria-hidden="true" />
            <span className="sr-only">
              {i18n.t("issues.toolbar.searchLabel")}
            </span>
            <input placeholder={i18n.t("issues.toolbar.searchPlaceholder")} />
          </label>
          <button type="button">
            <ListFilter size={16} aria-hidden="true" />
            {i18n.t("issues.toolbar.grouped")}
          </button>
          <button type="button">
            <SlidersHorizontal size={16} aria-hidden="true" />
            {i18n.t("issues.toolbar.filters")}
          </button>
        </section>

        <section
          className="issue-surface"
          aria-label={i18n.t("issues.list.label")}
        >
          {Object.entries(groupedIssues).map(([group, issues]) => (
            <div className="issue-group" key={group}>
              <h2>
                {i18n.t(`issues.group.${group}` as TranslationKey)}
                <span>{issues.length}</span>
              </h2>
              <div className="issue-rows">
                {issues.map((issue) => (
                  <IssueRow
                    i18n={i18n}
                    key={issue.id}
                    issue={issue}
                    selected={issue.id === data.selectedIssue.id}
                  />
                ))}
              </div>
            </div>
          ))}
        </section>
      </main>

      <aside className="focus-panel" aria-labelledby="focus-title">
        <IssueDetail data={data} i18n={i18n} />
      </aside>
    </div>
  );
}

function SetupStepItem({ step }: { step: SetupStep }) {
  const Icon =
    step.state === "complete"
      ? CheckCircle2
      : step.state === "error"
        ? AlertTriangle
        : step.state === "locked"
          ? ShieldCheck
          : Clock3;

  return (
    <li className={`setup-step setup-${step.state}`}>
      <Icon size={17} aria-hidden="true" />
      <span>
        <strong>{step.title}</strong>
        <small>{step.detail}</small>
      </span>
    </li>
  );
}

function denialLabel(i18n: I18n, code?: FirstRunDenialReasonCode) {
  return code
    ? i18n.t(`commonErrors.policy.${code}` as TranslationKey)
    : i18n.t("commonErrors.policy.allowed");
}

function FirstRunOnboardingPanel({
  onboarding,
  i18n,
}: {
  onboarding: FirstRunOnboarding;
  i18n: I18n;
}) {
  const [providerRoute, setProviderRoute] = useState(
    onboarding.providerRoute.providerName,
  );
  const [statusMessage, setStatusMessage] = useState(
    i18n.t("issues.firstRun.status.ready"),
  );
  const [fieldError, setFieldError] = useState<string | null>(null);
  const [didRunDiagnostic, setDidRunDiagnostic] = useState(false);
  const [permissionMode, setPermissionMode] = useState(
    onboarding.permissions.role as PermissionMode,
  );
  const [shouldFocusErrorSummary, setShouldFocusErrorSummary] = useState(false);
  const errorSummaryRef = useRef(null as HTMLDivElement | null);
  const providerFieldId = "first-run-provider-route";
  const providerErrorId = "first-run-provider-error";
  const errorSummaryId = "first-run-error-summary";
  const activePermissions =
    permissionMode === "member"
      ? onboarding.permissionFixtures.member
      : onboarding.permissions;
  const steps = [
    "wg",
    "provider",
    "limits",
    "runner",
    "binding",
    "diagnostic",
  ] as const;

  const validateProviderRoute = () => {
    if (providerRoute.trim()) {
      setFieldError(null);
      return true;
    }

    setFieldError(i18n.t("issues.firstRun.errors.providerRequired"));
    setShouldFocusErrorSummary(true);
    return false;
  };

  useEffect(() => {
    if (!shouldFocusErrorSummary || !fieldError) return;

    errorSummaryRef.current?.focus();
    setShouldFocusErrorSummary(false);
  }, [fieldError, shouldFocusErrorSummary]);

  const saveReadiness = () => {
    if (!validateProviderRoute()) return;

    setStatusMessage(i18n.t("issues.firstRun.status.saving"));
    window.setTimeout(() => {
      setStatusMessage(i18n.t("issues.firstRun.status.saved"));
    }, 0);
  };

  const runDiagnostic = () => {
    if (!validateProviderRoute()) return;

    setStatusMessage(i18n.t("issues.firstRun.status.running"));
    setDidRunDiagnostic(true);
    window.setTimeout(() => {
      setStatusMessage(i18n.t("issues.firstRun.status.complete"));
    }, 0);
  };

  return (
    <section className="first-run-panel" aria-labelledby="first-run-title">
      <div className="first-run-heading">
        <div>
          <p className="eyebrow">{i18n.t("issues.firstRun.eyebrow")}</p>
          <h2 id="first-run-title">{i18n.t("issues.firstRun.title")}</h2>
          <p>{i18n.t("issues.firstRun.summary")}</p>
        </div>
        <StatusBadge
          i18n={i18n}
          status={
            onboarding.diagnosticContract.blockedReasonCode
              ? "warning"
              : "success"
          }
          label={denialLabel(
            i18n,
            onboarding.diagnosticContract.blockedReasonCode,
          )}
        />
      </div>

      <ol
        className="first-run-stepper"
        aria-label={i18n.t("issues.firstRun.steps.label")}
      >
        {steps.map((step, index) => (
          <li
            key={step}
            aria-current={step === "diagnostic" ? "step" : undefined}
          >
            <span aria-hidden="true">{index + 1}</span>
            <strong>
              {i18n.t(`issues.firstRun.step.${step}` as TranslationKey)}
            </strong>
            {step === "diagnostic" ? (
              <small>{i18n.t("issues.firstRun.currentStep")}</small>
            ) : null}
          </li>
        ))}
      </ol>

      {fieldError ? (
        <div
          className="error-summary"
          id={errorSummaryId}
          ref={errorSummaryRef}
          tabIndex={-1}
          role="alert"
        >
          <strong>{i18n.t("issues.firstRun.errors.title")}</strong>
          <a href={`#${providerFieldId}`}>{fieldError}</a>
        </div>
      ) : null}

      <div className="first-run-grid">
        <section aria-labelledby="first-run-provider-title">
          <h3 id="first-run-provider-title">
            {i18n.t("issues.firstRun.provider.title")}
          </h3>
          <label className="field-stack" htmlFor={providerFieldId}>
            <span>{i18n.t("issues.firstRun.provider.name")}</span>
            <input
              aria-describedby={fieldError ? providerErrorId : undefined}
              aria-invalid={fieldError ? "true" : undefined}
              id={providerFieldId}
              value={providerRoute}
              onChange={(event) => setProviderRoute(event.target.value)}
            />
          </label>
          {fieldError ? (
            <p className="field-error" id={providerErrorId}>
              {fieldError}
            </p>
          ) : null}
          <dl className="compact-facts">
            <div>
              <dt>{i18n.t("issues.firstRun.provider.model")}</dt>
              <dd>{onboarding.providerRoute.modelName}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.provider.credential")}</dt>
              <dd>
                {onboarding.providerRoute.credentialRef} ·{" "}
                {i18n.t(
                  `issues.firstRun.credential.${onboarding.providerRoute.credentialStatus}` as TranslationKey,
                )}
              </dd>
            </div>
          </dl>
        </section>

        <section aria-labelledby="first-run-limits-title">
          <h3 id="first-run-limits-title">
            {i18n.t("issues.firstRun.limits.title")}
          </h3>
          <dl className="compact-facts">
            <div>
              <dt>{i18n.t("issues.firstRun.limits.monthly")}</dt>
              <dd>{onboarding.costUsageDefaults.monthlyLimitMicros}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.limits.perRun")}</dt>
              <dd>{onboarding.costUsageDefaults.perRunLimitMicros}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.limits.delta")}</dt>
              <dd>{onboarding.costUsageDefaults.usageDeltaMicros}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.limits.billable")}</dt>
              <dd>{String(onboarding.costUsageDefaults.billable)}</dd>
            </div>
          </dl>
        </section>

        <section aria-labelledby="first-run-runner-title">
          <h3 id="first-run-runner-title">
            {i18n.t("issues.firstRun.runner.title")}
          </h3>
          <dl className="compact-facts">
            <div>
              <dt>{i18n.t("issues.firstRun.runner.runnerState")}</dt>
              <dd>{onboarding.runnerAvailability.runnerState}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.runner.mcpState")}</dt>
              <dd>{onboarding.runnerAvailability.mcpState}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.contract.blockedReason")}</dt>
              <dd>
                {denialLabel(
                  i18n,
                  onboarding.runnerAvailability.blockedReasonCode,
                )}
              </dd>
            </div>
          </dl>
        </section>

        <section aria-labelledby="first-run-binding-title">
          <h3 id="first-run-binding-title">
            {i18n.t("issues.firstRun.binding.title")}
          </h3>
          <dl className="compact-facts">
            <div>
              <dt>{i18n.t("issues.firstRun.binding.agent")}</dt>
              <dd>{onboarding.binding.agentName}</dd>
            </div>
            <div>
              <dt>{i18n.t("issues.firstRun.binding.skill")}</dt>
              <dd>{onboarding.binding.skillName}</dd>
            </div>
          </dl>
        </section>
      </div>

      <section
        className="first-run-contract"
        aria-labelledby="first-run-contract-title"
      >
        <h3 id="first-run-contract-title">
          {i18n.t("issues.firstRun.contract.title")}
        </h3>
        <dl className="contract-grid">
          <div>
            <dt>{i18n.t("issues.firstRun.contract.mode")}</dt>
            <dd>{onboarding.diagnosticContract.mode}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.firstRun.contract.allowPaidCall")}</dt>
            <dd>{String(onboarding.diagnosticContract.allowPaidCall)}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.firstRun.contract.idempotencyKey")}</dt>
            <dd>{onboarding.diagnosticContract.idempotencyKey}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.firstRun.contract.billable")}</dt>
            <dd>{String(onboarding.diagnosticContract.billable)}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.firstRun.limits.delta")}</dt>
            <dd>{onboarding.diagnosticContract.usageDeltaMicros}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.firstRun.contract.policyDecisionId")}</dt>
            <dd>{onboarding.diagnosticContract.policyDecisionId}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.firstRun.contract.denialReason")}</dt>
            <dd>
              {denialLabel(
                i18n,
                onboarding.diagnosticContract.denialReasonCode,
              )}
            </dd>
          </div>
        </dl>
      </section>

      <section
        className="first-run-permissions"
        aria-labelledby="first-run-permissions-title"
      >
        <h3 id="first-run-permissions-title">
          {i18n.t("issues.firstRun.permissions.title")}
        </h3>
        <div
          className="permission-toggle"
          role="group"
          aria-label={i18n.t("issues.firstRun.permissions.fixtureLabel")}
        >
          <button
            aria-pressed={permissionMode === "admin"}
            type="button"
            onClick={() => setPermissionMode("admin")}
          >
            {i18n.t("issues.firstRun.permissions.adminFixture")}
          </button>
          <button
            aria-pressed={permissionMode === "member"}
            type="button"
            onClick={() => setPermissionMode("member")}
          >
            {i18n.t("issues.firstRun.permissions.memberFixture")}
          </button>
        </div>
        <StatusBadge
          i18n={i18n}
          status={activePermissions.canConfigure ? "success" : "warning"}
          label={i18n.t("issues.firstRun.permissions.configure")}
        />
        <StatusBadge
          i18n={i18n}
          status={activePermissions.canRunDiagnostic ? "success" : "warning"}
          label={i18n.t("issues.firstRun.permissions.diagnostic")}
        />
        {activePermissions.readOnlyReasonCode ? (
          <div className="read-only-preview">
            <strong>{i18n.t("issues.firstRun.permissions.readOnly")}</strong>
            <p>
              {i18n.t(
                `issues.firstRun.readOnly.${activePermissions.readOnlyReasonCode}` as TranslationKey,
              )}
            </p>
            <button type="button" disabled>
              {i18n.t("issues.firstRun.action.disabled")}
            </button>
          </div>
        ) : null}
      </section>

      <div className="first-run-actions">
        <button
          type="button"
          onClick={saveReadiness}
          disabled={!activePermissions.canConfigure}
        >
          <CheckCircle2 size={16} aria-hidden="true" />
          {i18n.t("issues.firstRun.action.save")}
        </button>
        <button
          type="button"
          className="primary-action"
          onClick={runDiagnostic}
          disabled={!activePermissions.canRunDiagnostic}
        >
          <PlayCircle size={16} aria-hidden="true" />
          {i18n.t("issues.firstRun.action.run")}
        </button>
      </div>

      <p className="status-region" role="status" aria-live="polite">
        {statusMessage}
      </p>

      <section aria-labelledby="first-run-timeline-title">
        <h3 id="first-run-timeline-title">
          {i18n.t("issues.firstRun.timeline.title")}
        </h3>
        <ol className="first-run-timeline">
          {(didRunDiagnostic
            ? onboarding.timeline
            : onboarding.timeline.slice(0, 3)
          ).map((item) => (
            <FirstRunTimelineEntry key={item.id} item={item} i18n={i18n} />
          ))}
        </ol>
      </section>
    </section>
  );
}

function FirstRunTimelineEntry({
  item,
  i18n,
}: {
  item: FirstRunTimelineItem;
  i18n: I18n;
}) {
  return (
    <li className={`first-run-timeline-item run-${item.severity}`}>
      {item.status === "failed" ? (
        <XCircle size={16} aria-hidden="true" />
      ) : (
        <span className="run-marker" aria-hidden="true" />
      )}
      <div>
        <strong>{i18n.t(item.messageKey as TranslationKey)}</strong>
        <p>
          {i18n.t(statusLabelKey[item.status])} ·{" "}
          {i18n.t("issues.firstRun.timeline.redacted")}={String(item.redacted)}
        </p>
        <small>
          {i18n.t("issues.firstRun.timeline.refs")}: {item.safeRefs.join(", ")}
        </small>
      </div>
    </li>
  );
}

function IssueRow({
  i18n,
  issue,
  selected,
}: {
  i18n: I18n;
  issue: IssueSummary;
  selected: boolean;
}) {
  return (
    <article
      aria-label={`${issue.key} ${issue.title}`}
      className="issue-row"
      aria-current={selected ? "true" : undefined}
      role="button"
      tabIndex={0}
    >
      <div className="issue-row-main">
        <strong>
          <span>{issue.key}</span>
          {issue.title}
        </strong>
        <p>
          {issue.assignee} ·{" "}
          {i18n.t("issues.row.updated", { value: issue.updatedAt })}
        </p>
      </div>
      <div className="issue-row-meta">
        <StatusBadge i18n={i18n} status={issue.status} />
        <StatusBadge
          i18n={i18n}
          status={issue.policyState === "allowed" ? "success" : "warning"}
          label={policyLabel(i18n, issue.policyState)}
        />
        <span className="comment-count">
          {i18n.plural("issues.row.comments", issue.commentCount)}
        </span>
      </div>
    </article>
  );
}

function IssueDetail({ data, i18n }: { data: ConsoleData; i18n: I18n }) {
  const issue = data.selectedIssue;

  return (
    <div className="detail-layout">
      <header className="focus-header">
        <div>
          <p className="eyebrow">{i18n.t("issues.detail.eyebrow")}</p>
          <h2 id="focus-title">{issue.key}</h2>
        </div>
        <StatusBadge i18n={i18n} status={issue.status} />
      </header>

      <section className="detail-section" aria-labelledby="detail-title">
        <h3 id="detail-title">{issue.title}</h3>
        <p>{issue.description}</p>
        <dl className="detail-grid">
          <div>
            <dt>{i18n.t("issues.detail.parent")}</dt>
            <dd>{issue.parent}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.detail.priority")}</dt>
            <dd>{issue.priority}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.detail.assignee")}</dt>
            <dd>{issue.assignee}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.detail.children")}</dt>
            <dd>{issue.children.join(", ")}</dd>
          </div>
        </dl>
      </section>

      <section className="detail-section" aria-labelledby="acceptance-title">
        <h3 id="acceptance-title">{i18n.t("issues.acceptance.title")}</h3>
        <ul className="check-list">
          {issue.acceptance.map((item) => (
            <li key={item}>
              <CheckCircle2 size={16} aria-hidden="true" />
              <span>{item}</span>
            </li>
          ))}
        </ul>
      </section>

      <section className="detail-section" aria-labelledby="run-title">
        <h3 id="run-title">{i18n.t("issues.run.title")}</h3>
        <ol className="run-timeline">
          {data.runSteps.map((step) => (
            <li key={step.id} className={`run-step run-${step.severity}`}>
              {step.status === "failed" ? (
                <XCircle size={16} aria-hidden="true" />
              ) : (
                <span className="run-marker" aria-hidden="true" />
              )}
              <div>
                <strong>{step.label}</strong>
                <p>
                  {i18n.t(statusLabelKey[step.status])} · {step.timestamp} ·{" "}
                  {step.detail}
                </p>
              </div>
            </li>
          ))}
        </ol>
      </section>

      <section className="detail-section" aria-labelledby="comments-title">
        <h3 id="comments-title">{i18n.t("issues.comments.title")}</h3>
        <div className="comment-thread">
          {issue.comments.map((comment) => (
            <article className="comment" key={comment.id}>
              <header>
                <strong>{comment.author}</strong>
                <span>{comment.role}</span>
                <time>{comment.createdAt}</time>
              </header>
              <p>{comment.body}</p>
              {comment.replies?.map((reply) => (
                <article className="comment reply" key={reply.id}>
                  <header>
                    <strong>{reply.author}</strong>
                    <span>{reply.role}</span>
                    <time>{reply.createdAt}</time>
                  </header>
                  <p>{reply.body}</p>
                </article>
              ))}
            </article>
          ))}
        </div>
        <form className="composer" aria-label={i18n.t("issues.composer.label")}>
          <label htmlFor="reply">{i18n.t("issues.composer.replyLabel")}</label>
          <textarea
            id="reply"
            placeholder={i18n.t("issues.composer.placeholder")}
          />
          <div>
            <button type="button">{i18n.t("issues.composer.cancel")}</button>
            <button type="submit" className="primary-action">
              {i18n.t("issues.composer.send")}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

export function App() {
  const [data, setData] = useState<ConsoleData | null>(null);
  const [error, setError] = useState(false);
  const resolvedLocale = useMemo(
    () => resolveLocalePreferences(data?.localePreferences),
    [data?.localePreferences],
  );
  const i18n = useMemo(() => createI18n(resolvedLocale), [resolvedLocale]);

  useEffect(() => {
    let mounted = true;

    taskOtterDataAdapter
      .getConsoleData()
      .then((nextData) => {
        if (mounted) setData(nextData);
      })
      .catch(() => {
        if (mounted) setError(true);
      });

    return () => {
      mounted = false;
    };
  }, []);

  if (error) {
    return (
      <main className="load-state" aria-live="polite">
        <AlertTriangle size={24} aria-hidden="true" />
        <h1>{i18n.t("commonErrors.console.unavailable.title")}</h1>
        <p>{i18n.t("commonErrors.console.unavailable.detail")}</p>
        <button type="button">{i18n.t("commonErrors.actions.retry")}</button>
      </main>
    );
  }

  if (!data) {
    return (
      <main className="load-state" aria-busy="true" aria-live="polite">
        <div className="skeleton-block" />
        <h1>{i18n.t("webShell.loading.title")}</h1>
        <p>{i18n.t("webShell.loading.detail")}</p>
      </main>
    );
  }

  return <AppShell data={data} i18n={i18n} />;
}
