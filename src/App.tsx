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
import { useEffect, useMemo, useState } from "react";
import type {
  ConsoleData,
  DisplayText,
  IssueStatus,
  IssueSummary,
  LocalizedText,
  RunStatus,
  Severity,
  SetupStep,
} from "./data/contracts";
import { taskOtterDataAdapter } from "./data/taskotterAdapter";
import { createI18n, resolveLocalePreferences } from "./i18n";
import type { TranslationKey, TranslationValues } from "./i18n/types";

type I18n = ReturnType<typeof createI18n>;

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

function renderText(i18n: I18n, value: DisplayText | LocalizedText) {
  if ("text" in value) return value.text;
  if ("dateTime" in value) return i18n.formatDateTime(value.dateTime);
  if ("number" in value) return i18n.formatNumber(value.number);

  const renderedValues = Object.fromEntries(
    Object.entries(value.values ?? {}).map(([key, nestedValue]) => [
      key,
      typeof nestedValue === "object"
        ? renderText(i18n, nestedValue)
        : nestedValue,
    ]),
  ) as TranslationValues;

  return i18n.t(value.key, renderedValues);
}

function AppShell({ data, i18n }: { data: ConsoleData; i18n: I18n }) {
  const groupedIssues = useMemo(
    () =>
      data.issues.reduce<Record<IssueSummary["group"], IssueSummary[]>>(
        (groups, issue) => {
          groups[issue.group].push(issue);
          return groups;
        },
        { assigned: [], needs_review: [], blocked: [] },
      ),
    [data.issues],
  );
  const memberLabel = i18n.plural(
    "webShell.workspace.member",
    data.workingGroup.memberCount,
  );
  const roleLabel = i18n.t(
    `webShell.workspace.role.${data.workingGroup.role}` as TranslationKey,
  );
  const runnerStateLabel = i18n.t(
    `webShell.runner.${data.workingGroup.runnerState}` as TranslationKey,
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
            role: roleLabel,
            memberCount: data.workingGroup.memberCount,
            memberLabel,
          })}
          className="workspace-switcher"
          type="button"
        >
          <span>
            <strong>{data.workingGroup.name}</strong>
            <small>
              {roleLabel} · {data.workingGroup.memberCount} {memberLabel}
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
              state: runnerStateLabel,
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
              <SetupStepItem i18n={i18n} key={step.id} step={step} />
            ))}
          </ol>
        </section>

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

function SetupStepItem({ i18n, step }: { i18n: I18n; step: SetupStep }) {
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
        <strong>{renderText(i18n, step.title)}</strong>
        <small>{renderText(i18n, step.detail)}</small>
      </span>
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
          {i18n.t("issues.row.updated", {
            value: renderText(i18n, issue.updatedAt),
          })}
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
            <dd>{issue.parent ?? i18n.t("issues.empty.parent")}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.detail.priority")}</dt>
            <dd>
              {i18n.t(`issues.priority.${issue.priority}` as TranslationKey)}
            </dd>
          </div>
          <div>
            <dt>{i18n.t("issues.detail.assignee")}</dt>
            <dd>{issue.assignee}</dd>
          </div>
          <div>
            <dt>{i18n.t("issues.detail.children")}</dt>
            <dd>
              {issue.children.length > 0
                ? issue.children.join(", ")
                : i18n.t("issues.empty.children")}
            </dd>
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
                <strong>{renderText(i18n, step.label)}</strong>
                <p>
                  {i18n.t(statusLabelKey[step.status])} ·{" "}
                  {renderText(i18n, step.timestamp)} ·{" "}
                  {renderText(i18n, step.detail)}
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
                <span>
                  {i18n.t(
                    `issues.comment.role.${comment.role}` as TranslationKey,
                  )}
                </span>
                <time>{renderText(i18n, comment.createdAt)}</time>
              </header>
              <p>{comment.body}</p>
              {comment.replies?.map((reply) => (
                <article className="comment reply" key={reply.id}>
                  <header>
                    <strong>{reply.author}</strong>
                    <span>
                      {i18n.t(
                        `issues.comment.role.${reply.role}` as TranslationKey,
                      )}
                    </span>
                    <time>{renderText(i18n, reply.createdAt)}</time>
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
