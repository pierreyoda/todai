import { hasLink } from "./links";
import type { LinkStatus } from "./statuses";
import type {
  Check,
  Checks,
  GitHubSnapshot,
  ItemState,
  ParsedLink,
  PullRequestSnapshot,
  RelatedItem,
} from "./types";

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
export const stateStatus = (state: ItemState): LinkStatus =>
  STATE_STATUSES[state];

/** Picked by priority: what matters most first, e.g. failing checks before a review. */
const pullRequestStatus = (pull: PullRequestSnapshot): LinkStatus => {
  const state = itemState(pull);
  if (state !== "open") return stateStatus(state);
  if (pull.checks?.state === "failure")
    return { word: "Checks failing", tone: "bad" };
  if (pull.reviewDecision === "changes_requested")
    return { word: "Changes requested", tone: "bad" };
  if (pull.reviewDecision === "approved")
    return { word: "Approved", tone: "good" };
  if (pull.reviewDecision === "review_required")
    return { word: "Review required", tone: "pending" };
  if (pull.checks?.state === "pending")
    return { word: "Checks running", tone: "pending" };
  return stateStatus(state);
};

/**
 * The one status word of a pull request or issue, picked by priority:
 * - a pull request: Merged › Closed › Draft › Checks failing › Changes requested › Approved › Review required › Checks
 *   running › Open;
 * - an issue: Completed › Not planned › Duplicate › Open.
 */
export const githubStatus = (snapshot: GitHubSnapshot): LinkStatus =>
  snapshot.type === "pull_request"
    ? pullRequestStatus(snapshot)
    : stateStatus(itemState(snapshot));

/** Where a pull request or issue stands, then its `status` if it tells more: e.g. "Open", then "Checks running". */
export const githubStatusWords = (
  snapshot: GitHubSnapshot,
  status: LinkStatus | null,
): LinkStatus[] => {
  const state = stateStatus(itemState(snapshot));
  return status && status.word !== state.word ? [state, status] : [state];
};

/** Why a todo linked to the pull request or issue `snapshot` (shown as `label`) could be marked as done, if so. */
export const githubDone = (
  snapshot: GitHubSnapshot,
  label: string,
): { word: string; reason: string } | undefined => {
  const state = itemState(snapshot);
  if (state !== "merged" && state !== "completed") return undefined;
  return { word: stateStatus(state).word, reason: `${label} was ${state}` };
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
    ? snapshot.closedBy.find(
        (item) => item.state !== "closed" && !hasLink(links, item.url),
      )
    : undefined;

/** E.g. "2 of 3 passed, 1 running", or "4 skipped" when none ran. */
export const describeChecks = ({
  passed,
  failed,
  running,
  skipped,
}: Checks): string => {
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

const CHECK_ORDER: Record<Check["outcome"], number> = {
  failed: 0,
  running: 1,
  passed: 2,
  skipped: 3,
};

/** The checks worth listing, failing then running then passed ones, without the skipped ones: at most `max`, if given. */
export const notableChecks = (checks: Checks, max?: number): Check[] =>
  checks.runs
    .filter((check) => check.outcome !== "skipped")
    .sort((a, b) => CHECK_ORDER[a.outcome] - CHECK_ORDER[b.outcome])
    .slice(0, max);

/** E.g. "check · svelte-check": a GitHub Actions job is named after its workflow first. */
export const checkName = ({
  name,
  workflow,
}: Pick<Check, "name" | "workflow">): string =>
  workflow ? `${workflow} · ${name}` : name;

const REVIEW_DECISIONS: Record<
  NonNullable<PullRequestSnapshot["reviewDecision"]>,
  string
> = {
  approved: "Approved",
  changes_requested: "Changes requested",
  review_required: "Required, not given yet",
};

/** E.g. "Approved", or "Not required" when its repository requires none. */
export const describeReview = ({
  reviewDecision,
}: Pick<PullRequestSnapshot, "reviewDecision">): string =>
  reviewDecision ? REVIEW_DECISIONS[reviewDecision] : "Not required";
