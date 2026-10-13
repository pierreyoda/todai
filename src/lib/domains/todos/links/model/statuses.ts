import type {
  Check,
  Checks,
  GitHubSnapshot,
  ItemState,
  ParsedLink,
  PullRequestSnapshot,
  RelatedItem,
  TodoLink,
} from "./types";
import type { Todo } from "../../model/types";
import type { GitHubProblem } from "#core/github";
import { formatTime } from "#shared/utils/timestamps.ts";
import { hasLink } from "./links";

/**
 * How a link's status reads, which colors it and picks its glyph:
 * - `open`: an open pull request or issue, with nothing more to tell;
 * - `good`, `bad`, `pending`: an open pull request's checks or review;
 * - `merged`: a merged pull request, or a completed issue;
 * - `neutral`: closed otherwise, or a draft;
 * - `warn`: GitHub can't tell about it.
 */
export type LinkTone = "open" | "good" | "bad" | "pending" | "merged" | "neutral" | "warn";

/** The one word shown after a link's label, e.g. "Checks failing". */
export interface LinkStatus {
  word: string;
  tone: LinkTone;
}

/** Where a pull request or issue stands, e.g. a pull request's `draft`, or an issue's `not_planned`. */
export const itemState = (snapshot: GitHubSnapshot): ItemState => {
  if (snapshot.type === "pull_request") {
    if (snapshot.state !== "open") return snapshot.state;
    return snapshot.isDraft ? "draft" : "open";
  }
  if (snapshot.state === "open") return "open";
  return snapshot.stateReason ?? "completed";
};

const STATE_STATUSES: Record<ItemState, LinkStatus> = {
  open: { word: "Open", tone: "open" },
  draft: { word: "Draft", tone: "neutral" },
  merged: { word: "Merged", tone: "merged" },
  closed: { word: "Closed", tone: "neutral" },
  completed: { word: "Completed", tone: "merged" },
  not_planned: { word: "Not planned", tone: "neutral" },
  duplicate: { word: "Duplicate", tone: "neutral" },
};

/** Where a pull request or issue stands, in a word: e.g. "Merged", or "Not planned". */
export const stateStatus = (state: ItemState): LinkStatus => STATE_STATUSES[state];

/** Picked by priority: what matters most first, e.g. failing checks before a review. */
const pullRequestStatus = (pull: PullRequestSnapshot): LinkStatus => {
  const state = itemState(pull);
  if (state !== "open") return stateStatus(state);
  if (pull.checks?.state === "failure") return { word: "Checks failing", tone: "bad" };
  if (pull.reviewDecision === "changes_requested") return { word: "Changes requested", tone: "bad" };
  if (pull.reviewDecision === "approved") return { word: "Approved", tone: "good" };
  if (pull.reviewDecision === "review_required") return { word: "Review required", tone: "pending" };
  if (pull.checks?.state === "pending") return { word: "Checks running", tone: "pending" };
  return stateStatus(state);
};

/**
 * The one status word of a link, picked by priority:
 * - a pull request: Merged › Closed › Draft › Checks failing › Changes requested › Approved › Review required › Checks
 *   running › Open;
 * - an issue: Completed › Not planned › Duplicate › Open.
 *
 * `null` for a link never fetched, e.g. a plain web link.
 */
export const linkStatus = ({ snapshot, fetchError }: Pick<TodoLink, "snapshot" | "fetchError">): LinkStatus | null => {
  if (fetchError === "no_access") return { word: "No access", tone: "warn" };
  if (fetchError === "sso_required") return { word: "SSO required", tone: "warn" };
  if (!snapshot) return null;
  return snapshot.type === "pull_request" ? pullRequestStatus(snapshot) : stateStatus(itemState(snapshot));
};

/**
 * Why `todo` could be marked as done, if it isn't yet: its first pull request merged, or issue completed. Only a
 * suggestion: todos are never completed on their own.
 */
export const doneSuggestion = ({
  completed,
  links,
}: Pick<Todo, "completed" | "links">): { link: TodoLink; state: "merged" | "completed" } | undefined => {
  if (completed) return undefined;
  for (const link of links) {
    const state = link.snapshot && itemState(link.snapshot);
    if (state === "merged" || state === "completed") return { link, state };
  }
  return undefined;
};

/**
 * The pull request worth adding along with the issue `snapshot`: the first one closing it (or set to) that wasn't
 * closed unmerged, unless `links` already has it.
 */
export const closingPullRequest = (
  snapshot: GitHubSnapshot | undefined,
  links: readonly Pick<ParsedLink, "url">[],
): RelatedItem | undefined =>
  snapshot?.type === "issue"
    ? snapshot.closedBy.find((item) => item.state !== "closed" && !hasLink(links, item.url))
    : undefined;

/** When the most recently fetched of `links` was: what their statuses are from, if any. */
export const statusesFetchedAt = (links: readonly Pick<TodoLink, "fetchedAt">[]): Date | undefined =>
  links.reduce<Date | undefined>(
    (latest, { fetchedAt }) => (fetchedAt && (!latest || fetchedAt > latest) ? fetchedAt : latest),
    undefined,
  );

/** E.g. "2 of 3 passed, 1 running", or "4 skipped" when none ran. */
export const describeChecks = ({ passed, failed, running, skipped }: Checks): string => {
  const ran = passed + failed + running;
  if (ran === 0) return `${skipped} skipped`;
  return [
    `${passed} of ${ran} passed`,
    failed > 0 && `${failed} failing`,
    running > 0 && `${running} running`,
  ]
    .filter(Boolean)
    .join(", ");
};

const CHECK_ORDER: Record<Check["outcome"], number> = { failed: 0, running: 1, passed: 2, skipped: 3 };

/** The checks worth listing, failing then running then passed ones, without the skipped ones: at most `max`, if given. */
export const notableChecks = (checks: Checks, max?: number): Check[] =>
  checks.runs
    .filter((check) => check.outcome !== "skipped")
    .sort((a, b) => CHECK_ORDER[a.outcome] - CHECK_ORDER[b.outcome])
    .slice(0, max);

/** E.g. "check · svelte-check": a GitHub Actions job is named after its workflow first. */
export const checkName = ({ name, workflow }: Pick<Check, "name" | "workflow">): string =>
  workflow ? `${workflow} · ${name}` : name;

const REVIEW_DECISIONS: Record<NonNullable<PullRequestSnapshot["reviewDecision"]>, string> = {
  approved: "Approved",
  changes_requested: "Changes requested",
  review_required: "Required, not given yet",
};

/** E.g. "Approved", or "Not required" when its repository requires none. */
export const describeReview = ({ reviewDecision }: Pick<PullRequestSnapshot, "reviewDecision">): string =>
  reviewDecision ? REVIEW_DECISIONS[reviewDecision] : "Not required";

/**
 * Why GitHub couldn't be asked, in a few words, then in a sentence: e.g. "Offline · link statuses from 14:02".
 *
 * @param statusesFrom When the statuses shown were fetched, if any.
 */
export const describeProblem = (
  problem: GitHubProblem,
  statusesFrom?: Date,
): { label: string; detail: string } => {
  const kept = statusesFrom
    ? `the links show their statuses from ${formatTime(statusesFrom)}`
    : "the links show what their URL tells";
  switch (problem.kind) {
    case "offline":
      return {
        label: statusesFrom ? `Offline · link statuses from ${formatTime(statusesFrom)}` : "Offline",
        detail: `GitHub could not be reached: ${kept}.`,
      };
    case "signed_out":
      return {
        label: "GitHub: sign in again",
        detail: `The GitHub CLI isn't signed in to ${problem.login} anymore: run gh auth login, then check again in Settings › Integrations. Until then, ${kept}.`,
      };
    case "gh_missing":
      return {
        label: "GitHub CLI not found",
        detail: `The GitHub CLI (gh) wasn't found: install it, then check again in Settings › Integrations. Until then, ${kept}.`,
      };
    case "rate_limited":
      return {
        label: `GitHub rate limit · until ${formatTime(problem.resetAt)}`,
        detail: `GitHub asked to wait until ${formatTime(problem.resetAt)}: until then, ${kept}.`,
      };
    case "failed":
      return {
        label: "GitHub error",
        detail: `${problem.message.replace(/\.$/, "")}. Until then, ${kept}.`,
      };
  }
};
