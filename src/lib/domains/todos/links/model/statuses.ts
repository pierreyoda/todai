import {
  type IntegrationProblem,
  type Provider,
  providerInfo,
} from "#core/integrations";
import { formatTime } from "#shared/utils/timestamps.ts";
import type { Todo } from "../../model/types";
import { githubDone, githubStatus, githubStatusWords } from "./github";
import type { TodoLink } from "./types";

/**
 * How a link's status reads, which colors it and picks its glyph:
 * - `open`: an open pull request or issue, with nothing more to tell;
 * - `good`, `bad`, `pending`: an open pull request's checks or review;
 * - `merged`: a merged pull request, or a completed issue;
 * - `neutral`: closed otherwise, or a draft;
 * - `warn`: its provider can't tell about it.
 */
export type LinkTone =
  | "open"
  | "good"
  | "bad"
  | "pending"
  | "merged"
  | "neutral"
  | "warn";

/** The one word shown after a link's label, e.g. "Checks failing". */
export interface LinkStatus {
  word: string;
  tone: LinkTone;
}

/**
 * The one status word of a link, as its provider picks it by priority (e.g. "Checks failing" before "Approved" for a
 * pull request): see `githubStatus`.
 *
 * `null` for a link never fetched, e.g. a plain web link.
 */
export const linkStatus = ({
  snapshot,
  fetchError,
}: Pick<TodoLink, "snapshot" | "fetchError">): LinkStatus | null => {
  if (fetchError === "no_access") return { word: "No access", tone: "warn" };
  if (fetchError === "sso_required")
    return { word: "SSO required", tone: "warn" };
  if (!snapshot) return null;
  return githubStatus(snapshot);
};

/**
 * The words telling where a link's item stands, as in its details: e.g. "Open" then "Checks running" for a pull
 * request. None until fetched.
 */
export const linkStatusWords = (
  link: Pick<TodoLink, "snapshot" | "fetchError">,
): LinkStatus[] =>
  link.snapshot ? githubStatusWords(link.snapshot, linkStatus(link)) : [];

/** Why a todo could be marked as done. */
export interface DoneSuggestion {
  /** The link whose item is done with. */
  link: TodoLink;
  /** E.g. "Merged". */
  word: string;
  /** E.g. "todai#57 was merged". */
  reason: string;
}

/**
 * Why `todo` could be marked as done, if it isn't yet: its first link whose item is done with, e.g. a pull request
 * merged, or an issue completed. Only a suggestion: todos are never completed on their own.
 */
export const doneSuggestion = ({
  completed,
  links,
}: Pick<Todo, "completed" | "links">): DoneSuggestion | undefined => {
  if (completed) return undefined;
  for (const link of links) {
    const done = link.snapshot && githubDone(link.snapshot, link.label);
    if (done) return { link, ...done };
  }
  return undefined;
};

/** When the most recently fetched of `links` was: what their statuses are from, if any. */
export const statusesFetchedAt = (
  links: readonly Pick<TodoLink, "fetchedAt">[],
): Date | undefined =>
  links.reduce<Date | undefined>(
    (latest, { fetchedAt }) =>
      fetchedAt && (!latest || fetchedAt > latest) ? fetchedAt : latest,
    undefined,
  );

/**
 * Why `provider` couldn't be asked, in a few words, then in a sentence: e.g. "Offline · link statuses from 14:02".
 *
 * @param statusesFrom When the statuses shown were fetched, if any.
 */
export const describeProblem = (
  provider: Provider,
  problem: IntegrationProblem,
  statusesFrom?: Date,
): { label: string; detail: string } => {
  const { name } = providerInfo(provider);
  const kept = statusesFrom
    ? `the links show their statuses from ${formatTime(statusesFrom)}`
    : "the links show what their URL tells";
  switch (problem.kind) {
    case "offline":
      return {
        label: statusesFrom
          ? `Offline · link statuses from ${formatTime(statusesFrom)}`
          : "Offline",
        detail: `${name} could not be reached: ${kept}.`,
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
    case "token_refused":
      return {
        label: "Notion: paste a new token",
        detail: `Notion refused this workspace's token: it expired, or was revoked. Paste a new personal access token in Settings › Integrations. Until then, ${kept}.`,
      };
    case "token_missing":
      return {
        label: "Notion: token missing",
        detail: `This workspace's Notion token isn't in your Keychain anymore: paste it again in Settings › Integrations. Until then, ${kept}.`,
      };
    case "rate_limited":
      return {
        label: `${name} rate limit · until ${formatTime(problem.resetAt)}`,
        detail: `${name} asked to wait until ${formatTime(problem.resetAt)}: until then, ${kept}.`,
      };
    case "failed":
      return {
        label: `${name} error`,
        detail: `${problem.message.replace(/\.$/, "")}. Until then, ${kept}.`,
      };
  }
};
