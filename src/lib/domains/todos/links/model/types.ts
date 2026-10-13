import type { UUID } from "node:crypto";

import type { GitHubProblem } from "#core/github";

/**
 * What a link points to, recognized from its URL:
 * - `github_pull`: a pull request on github.com;
 * - `github_issue`: an issue on github.com;
 * - `url`: any other web page.
 */
export type TodoLinkKind = "github_pull" | "github_issue" | "url";

/** A pull request or issue on github.com. */
export interface GitHubRef {
  owner: string;
  repo: string;
  /** Positive integer. */
  number: number;
}

/** A URL recognized as a link (`parse_link`), before it's added to a todo. */
export interface ParsedLink {
  kind: TodoLinkKind;
  /** Normalized: a pull request's or issue's URL without its tab, query or fragment. Always http(s). */
  url: string;
  /** Short name: `repo#number` for GitHub (e.g. "todai#58"), the domain otherwise (e.g. "docs.rs"). */
  label: string;
  /** Set exactly for the GitHub kinds. */
  github?: GitHubRef;
}

/** A link on a todo: a GitHub pull request or issue, or any other web page. */
export interface TodoLink extends ParsedLink {
  /** UUID v7. */
  id: UUID;
  todoId: UUID;
  createdAt: Date;
  /** What GitHub last told about its pull request or issue, if fetched since it was added. */
  snapshot?: GitHubSnapshot;
  /** When GitHub last answered about it. */
  fetchedAt?: Date;
  /** Why GitHub's last answer had nothing about it, if so: `snapshot` is then the one before, if any. */
  fetchError?: LinkFetchError;
}

/**
 * Why GitHub's last answer had nothing about a link's pull request or issue:
 * - `no_access`: missing, deleted, or private to others (GitHub can't tell these apart);
 * - `sso_required`: its organization requires single sign-on, which the GitHub CLI's token isn't authorized for.
 */
export type LinkFetchError = "no_access" | "sso_required";

/** A pull request or issue, as GitHub last told about it: plain facts, from which its status word is picked. */
export type GitHubSnapshot = PullRequestSnapshot | IssueSnapshot;

export interface PullRequestSnapshot {
  type: "pull_request";
  title: string;
  state: "open" | "closed" | "merged";
  isDraft: boolean;
  /** Absent when its repository requires no review, as most personal ones. */
  reviewDecision?: "approved" | "changes_requested" | "review_required";
  /** Absent without any check. */
  checks?: Checks;
  additions: number;
  deletions: number;
  changedFiles: number;
  /** Its branch. */
  headRef: string;
  /** The branch it's merged into. */
  baseRef: string;
  /** Absent for a deleted account. */
  author?: string;
  updatedAt: Date;
  /** The first few issues it closes once merged. */
  closes: RelatedItem[];
}

/** A pull request's checks: GitHub Actions' jobs and other apps' check runs, and commit statuses. */
export interface Checks {
  /** All of them, as GitHub sums them up. */
  state: "success" | "failure" | "pending";
  passed: number;
  failed: number;
  running: number;
  /** Skipped, neutral or stale. */
  skipped: number;
  /** The first ones, in GitHub's order. */
  runs: Check[];
}

export interface Check {
  name: string;
  /** The GitHub Actions workflow it's a job of, if any. */
  workflow?: string;
  outcome: "passed" | "failed" | "running" | "skipped";
  startedAt?: Date;
}

export interface IssueSnapshot {
  type: "issue";
  title: string;
  state: "open" | "closed";
  /** Why it was closed, if it is. */
  stateReason?: "completed" | "not_planned" | "duplicate";
  /** Absent for a deleted account. */
  author?: string;
  updatedAt: Date;
  /** The first few. */
  assignees: string[];
  /** The first few. */
  labels: { name: string; /** Format: #RRGGBB. */ color: string }[];
  milestone?: string;
  comments: number;
  /** The first few pull requests that close it once merged, or did. */
  closedBy: RelatedItem[];
}

/**
 * Where a pull request or issue stands:
 * - `open`: an open pull request or issue;
 * - `draft`, `merged`: a pull request;
 * - `closed`: a pull request closed without being merged (or an issue closed for another reason than below);
 * - `completed`, `not_planned`, `duplicate`: a closed issue.
 */
export type ItemState = "open" | "draft" | "merged" | "closed" | "completed" | "not_planned" | "duplicate";

/** A pull request or issue mentioned by another one, e.g. an issue a pull request closes. */
export interface RelatedItem extends ParsedLink {
  title: string;
  state: ItemState;
}

/** What a link would show once added, before adding it (`preview_link`). */
export interface LinkPreview {
  /** As recognized from its URL, or where GitHub says it is: e.g. a pull request pasted as an issue. */
  link: ParsedLink;
  /** What GitHub tells about its pull request or issue, if it could be asked. */
  snapshot?: GitHubSnapshot;
  /** Why GitHub had nothing to tell about it, if so. */
  fetchError?: LinkFetchError;
  /** Why GitHub couldn't be asked, if so: e.g. offline. */
  problem?: GitHubProblem;
}

/** What refreshing links' statuses did. */
export interface RefreshOutcome {
  /** Links whose status was fetched: the todos are worth listing again if any. */
  refreshed: number;
  /** Why GitHub couldn't be asked, if so. */
  problem?: GitHubProblem;
}
