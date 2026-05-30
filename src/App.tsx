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
  IssueStatus,
  IssueSummary,
  RunStatus,
  Severity,
  SetupStep,
} from "./data/contracts";
import { taskOtterDataAdapter } from "./data/taskotterAdapter";

const navItems = [
  { label: "Inbox", icon: Inbox, count: 8 },
  { label: "Issues", icon: LayoutDashboard, count: 24 },
  { label: "Chat", icon: MessageSquare, count: 3 },
  { label: "Agents", icon: Bot, count: 12 },
  { label: "Skills", icon: Workflow, count: 37 },
  { label: "Providers", icon: GitBranch, count: 2 },
  { label: "Usage", icon: CircleDollarSign, count: 1 },
  { label: "Settings", icon: Settings, count: 0 },
];

const statusLabel: Record<IssueStatus | RunStatus, string> = {
  blocked: "Blocked",
  cancelled: "Cancelled",
  completed: "Completed",
  done: "Done",
  failed: "Failed",
  in_progress: "In progress",
  in_review: "In review",
  queued: "Queued",
  retrying: "Retrying",
  running: "Running",
  triage: "Triage",
  waiting_approval: "Waiting approval",
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
  status,
  label,
}: {
  status: IssueStatus | RunStatus | Severity;
  label?: string;
}) {
  const severity =
    status in statusSeverity
      ? statusSeverity[status as IssueStatus | RunStatus]
      : (status as Severity);
  const text =
    label ??
    (status in statusLabel
      ? statusLabel[status as IssueStatus | RunStatus]
      : status);

  return (
    <span className={`status-badge status-${severity}`}>
      <span className="status-dot" aria-hidden="true" />
      {text}
    </span>
  );
}

function policyLabel(policyState: IssueSummary["policyState"]) {
  switch (policyState) {
    case "allowed":
      return "Policy OK";
    case "policy_denied":
      return "Policy denied";
    case "cost_limited":
      return "Cost limited";
    case "runner_offline":
      return "Runner offline";
  }
}

function AppShell({ data }: { data: ConsoleData }) {
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

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true">
            TO
          </div>
          <div>
            <strong>TaskOtter</strong>
            <span>Admin console</span>
          </div>
        </div>
        <button
          aria-label={`Switch working group: ${data.workingGroup.name}, ${data.workingGroup.role}, ${data.workingGroup.memberCount} members`}
          className="workspace-switcher"
          type="button"
        >
          <span>
            <strong>{data.workingGroup.name}</strong>
            <small>
              {data.workingGroup.role} · {data.workingGroup.memberCount} members
            </small>
          </span>
          <ChevronDown size={16} aria-hidden="true" />
        </button>
        <nav className="nav-list" aria-label="TaskOtter navigation">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <a
                aria-label={item.label}
                aria-current={item.label === "Issues" ? "page" : undefined}
                href={`#${item.label.toLowerCase()}`}
                key={item.label}
              >
                <Icon size={17} aria-hidden="true" />
                <span>{item.label}</span>
                {item.count > 0 ? <small>{item.count}</small> : null}
              </a>
            );
          })}
        </nav>
        <div className="sidebar-health">
          <StatusBadge
            status={
              data.workingGroup.runnerState === "online" ? "success" : "warning"
            }
            label={`Runner ${data.workingGroup.runnerState}`}
          />
          <p>{data.workingGroup.plan}</p>
        </div>
      </aside>

      <main className="workspace" aria-labelledby="issues-title">
        <header className="page-header">
          <div>
            <p className="eyebrow">Working Group / Issues</p>
            <h1 id="issues-title">Issue operations</h1>
          </div>
          <div className="header-actions" aria-label="Issue actions">
            <button type="button">
              <Command size={16} aria-hidden="true" />
              Command
            </button>
            <button type="button" className="primary-action">
              <PlayCircle size={16} aria-hidden="true" />
              New run
            </button>
          </div>
        </header>

        <section className="setup-band" aria-labelledby="setup-title">
          <div>
            <p className="eyebrow">First-run setup</p>
            <h2 id="setup-title">Working Group setup path</h2>
          </div>
          <ol className="setup-steps">
            {data.setupSteps.map((step) => (
              <SetupStepItem key={step.id} step={step} />
            ))}
          </ol>
        </section>

        <section className="toolbar" aria-label="Issue filtering controls">
          <label className="search-field">
            <Search size={16} aria-hidden="true" />
            <span className="sr-only">Search issues</span>
            <input placeholder="Search issues, agents, comments" />
          </label>
          <button type="button">
            <ListFilter size={16} aria-hidden="true" />
            Grouped
          </button>
          <button type="button">
            <SlidersHorizontal size={16} aria-hidden="true" />
            Filters
          </button>
        </section>

        <section className="issue-surface" aria-label="Grouped issue list">
          {Object.entries(groupedIssues).map(([group, issues]) => (
            <div className="issue-group" key={group}>
              <h2>
                {group}
                <span>{issues.length}</span>
              </h2>
              <div className="issue-rows">
                {issues.map((issue) => (
                  <IssueRow
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
        <IssueDetail data={data} />
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

function IssueRow({
  issue,
  selected,
}: {
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
          {issue.assignee} · updated {issue.updatedAt}
        </p>
      </div>
      <div className="issue-row-meta">
        <StatusBadge status={issue.status} />
        <StatusBadge
          status={issue.policyState === "allowed" ? "success" : "warning"}
          label={policyLabel(issue.policyState)}
        />
        <span className="comment-count">{issue.commentCount} comments</span>
      </div>
    </article>
  );
}

function IssueDetail({ data }: { data: ConsoleData }) {
  const issue = data.selectedIssue;

  return (
    <div className="detail-layout">
      <header className="focus-header">
        <div>
          <p className="eyebrow">Focus panel</p>
          <h2 id="focus-title">{issue.key}</h2>
        </div>
        <StatusBadge status={issue.status} />
      </header>

      <section className="detail-section" aria-labelledby="detail-title">
        <h3 id="detail-title">{issue.title}</h3>
        <p>{issue.description}</p>
        <dl className="detail-grid">
          <div>
            <dt>Parent</dt>
            <dd>{issue.parent}</dd>
          </div>
          <div>
            <dt>Priority</dt>
            <dd>{issue.priority}</dd>
          </div>
          <div>
            <dt>Assignee</dt>
            <dd>{issue.assignee}</dd>
          </div>
          <div>
            <dt>Children</dt>
            <dd>{issue.children.join(", ")}</dd>
          </div>
        </dl>
      </section>

      <section className="detail-section" aria-labelledby="acceptance-title">
        <h3 id="acceptance-title">Acceptance criteria</h3>
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
        <h3 id="run-title">Agent run progress</h3>
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
                  {statusLabel[step.status]} · {step.timestamp} · {step.detail}
                </p>
              </div>
            </li>
          ))}
        </ol>
      </section>

      <section className="detail-section" aria-labelledby="comments-title">
        <h3 id="comments-title">Threaded comments</h3>
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
        <form className="composer" aria-label="Reply composer">
          <label htmlFor="reply">Reply</label>
          <textarea
            id="reply"
            placeholder="Write a concise handoff or blocker."
          />
          <div>
            <button type="button">Cancel</button>
            <button type="submit" className="primary-action">
              Send reply
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

export function App() {
  const [data, setData] = useState<ConsoleData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;

    taskOtterDataAdapter
      .getConsoleData()
      .then((nextData) => {
        if (mounted) setData(nextData);
      })
      .catch(() => {
        if (mounted) setError("Unable to load TaskOtter console data.");
      });

    return () => {
      mounted = false;
    };
  }, []);

  if (error) {
    return (
      <main className="load-state" aria-live="polite">
        <AlertTriangle size={24} aria-hidden="true" />
        <h1>Console unavailable</h1>
        <p>{error}</p>
        <button type="button">Retry</button>
      </main>
    );
  }

  if (!data) {
    return (
      <main className="load-state" aria-busy="true" aria-live="polite">
        <div className="skeleton-block" />
        <h1>Loading TaskOtter console</h1>
        <p>Preparing issue workspace, focus panel, and setup path.</p>
      </main>
    );
  }

  return <AppShell data={data} />;
}
