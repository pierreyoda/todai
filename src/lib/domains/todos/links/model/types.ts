import type { UUID } from "node:crypto";

import type { IntegrationProblem, ProviderProblems } from "#core/integrations";

/**
 * What a link points to, recognized from its URL:
 * - `github_pull`: a pull request on github.com;
 * - `github_issue`: an issue on github.com;
 * - `notion`: a page, or a whole database, in Notion;
 * - `url`: any other web page.
 */
export type TodoLinkKind = "github_pull" | "github_issue" | "notion" | "url";

/** A page, or a database, in Notion. */
export interface NotionRef {
  /** Its ID: 32 lowercase hexadecimal digits, as in its URL. */
  id: string;
}

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
  /**
   * Short name: `repo#number` for GitHub (e.g. "todai#58"); a Notion page's unique ID or title once fetched, else as
   * its URL's slug tells it; the domain otherwise (e.g. "docs.rs").
   */
  label: string;
  /** Set exactly for the GitHub kinds. */
  github?: GitHubRef;
  /** Set exactly for the Notion kind. */
  notion?: NotionRef;
}

/** A link on a todo: a GitHub pull request or issue, or any other web page. */
export interface TodoLink extends ParsedLink {
  /** UUID v7. */
  id: UUID;
  todoId: UUID;
  createdAt: Date;
  /** What its provider last told about its item, if fetched since it was added. */
  snapshot?: LinkSnapshot;
  /** When its provider last answered about it. */
  fetchedAt?: Date;
  /** Why its provider's last answer had nothing about it, if so: `snapshot` is then the one before, if any. */
  fetchError?: LinkFetchError;
}

/**
 * Why its provider's last answer had nothing about a link's item:
 * - `no_access`: missing, deleted, or private to others (providers can't tell these apart);
 * - `sso_required`: GitHub only, its organization requires single sign-on, which the GitHub CLI's token isn't
 *   authorized for.
 */
export type LinkFetchError = "no_access" | "sso_required";

/** A linked item, as its provider last told about it: plain facts, from which its status word is picked. */
export type LinkSnapshot = GitHubSnapshot | NotionSnapshot;

/** A Notion page or database, as Notion last told about it. */
export type NotionSnapshot = NotionPageSnapshot | NotionDatabaseSnapshot;

/** One of Notion's colors, as of its options and tags. */
export type NotionColor =
  | "default"
  | "gray"
  | "brown"
  | "orange"
  | "yellow"
  | "green"
  | "blue"
  | "purple"
  | "pink"
  | "red";

/** The three groups a Notion Status property's options are filed under. */
export type StatusGroup = "to_do" | "in_progress" | "complete";

export interface NotionPageSnapshot {
  type: "notion_page";
  /** Empty for an untitled page. */
  title: string;
  /** Its emoji icon, if any. */
  icon?: string;
  /** Its database's unique ID, e.g. TOD-31, if it has one. */
  uniqueId?: { prefix?: string; number: number };
  /** Its Status, if its database has one: `group` is absent when the database's schema couldn't be read. */
  status?: { name: string; color: NotionColor; group?: StatusGroup };
  /** The first few people of its Assignee (or first people) property. */
  assignees: string[];
  /** The start of its Due (or first date) property: `YYYY-MM-DD`, or a date-time. */
  due?: string;
  /** The first few options of its Tags (or first multi-select) property. */
  tags: { name: string; color: NotionColor }[];
  lastEditedAt: Date;
  /** Whether it's in Notion's trash. */
  inTrash: boolean;
  /** The database it's in, if any: `title` is absent when its schema couldn't be read. */
  database?: { dataSourceId: string; title?: string };
}

/** What kind of Notion column can tell a page is done. */
export type NotionColumnKind =
  | "status"
  | "select"
  | "multi_select"
  | "checkbox";

/** An option of a status, select or multi-select column. */
export interface NotionOption {
  /** Notion's ID, which stays the same when renamed. */
  id: string;
  name: string;
  color: NotionColor;
}

/** A column of a database that can tell its pages are done, with every option it offers. */
export interface NotionSchemaColumn {
  /** Notion's ID of the property, which stays the same when renamed. */
  propertyId: string;
  name: string;
  kind: NotionColumnKind;
  /** None for a checkbox. */
  options: NotionOption[];
}

/** A database's columns that can tell its pages are done (`get_notion_columns`). */
export interface NotionColumns {
  title: string;
  columns: NotionSchemaColumn[];
}

/**
 * Which values of which column of a Notion database mean its pages are done, rather than their Status being in its
 * Complete group: one per database. The backend shows the pages as it tells (their status becomes the column's value).
 */
export interface NotionDoneRule {
  dataSourceId: string;
  /** As when the rule was set. */
  databaseTitle: string;
  propertyId: string;
  /** As when the rule was set. */
  propertyName: string;
  propertyKind: NotionColumnKind;
  /** By ID, with their name as when the rule was set. Empty for a checkbox, done when checked. */
  doneOptions: { id: string; name: string }[];
  updatedAt?: Date;
}

/** A whole database: a link to one shows its title, without a status. */
export interface NotionDatabaseSnapshot {
  type: "notion_database";
  title: string;
  icon?: string;
  lastEditedAt: Date;
  inTrash: boolean;
}

/** A pull request or issue, as GitHub last told about it. */
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
  labels: { name: string /** Format: #RRGGBB. */; color: string }[];
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
export type ItemState =
  | "open"
  | "draft"
  | "merged"
  | "closed"
  | "completed"
  | "not_planned"
  | "duplicate";

/** A pull request or issue mentioned by another one, e.g. an issue a pull request closes. */
export interface RelatedItem extends ParsedLink {
  title: string;
  state: ItemState;
}

/** What a link would show once added, before adding it (`preview_link`). */
export interface LinkPreview {
  /** As recognized from its URL, or where its provider says it is: e.g. a pull request pasted as an issue. */
  link: ParsedLink;
  /** What its provider tells about its item, if it could be asked. */
  snapshot?: LinkSnapshot;
  /** Why its provider had nothing to tell about it, if so. */
  fetchError?: LinkFetchError;
  /** Why its provider couldn't be asked, if so: e.g. offline. */
  problem?: IntegrationProblem;
}

/** What refreshing links' statuses did. */
export interface RefreshOutcome {
  /** Links whose status was fetched: the todos are worth listing again if any. */
  refreshed: number;
  /** Why a provider couldn't be asked, for those that couldn't: their links keep their last known status. */
  problems: ProviderProblems;
}
