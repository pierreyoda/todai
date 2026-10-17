import { type IntegrationProblem, needsUser } from "#core/integrations";
import type { NotionAccount } from "#domains/workspaces";
import type { GhAccount, GhStatus, NotionApiStatus } from "./types";

/**
 * Where a workspace stands with GitHub:
 * - `not_connected`: it uses no GitHub CLI account;
 * - `checking`: connected, while the GitHub CLI's status is fetched for the first time;
 * - `connected`: through an account the GitHub CLI is signed in to;
 * - `signed_out`: through an account the GitHub CLI isn't signed in to anymore (or doesn't know anymore);
 * - `gh_missing`: connected, but the GitHub CLI isn't installed anymore (or not where expected).
 */
export type GitHubConnectionState =
  | "not_connected"
  | "checking"
  | "connected"
  | "signed_out"
  | "gh_missing";

/** Where a workspace connected to the GitHub CLI account `login` (if any) stands, given the GitHub CLI's `status`. */
export const githubConnectionState = (
  login: string | null | undefined,
  status: GhStatus | undefined,
): GitHubConnectionState => {
  if (!login) return "not_connected";
  if (!status) return "checking";
  if (status.path === null) return "gh_missing";
  return status.accounts.some(
    (account) => account.login === login && account.signedIn,
  )
    ? "connected"
    : "signed_out";
};

/**
 * Where a workspace stands with Notion:
 * - `not_connected`: it has no personal access token;
 * - `checking`: connected, while Notion is asked about its token for the first time;
 * - `connected`: Notion accepts its token (or couldn't be asked, e.g. offline, which passes on its own);
 * - `attention`: Notion refused its token, or it isn't in the Keychain anymore.
 */
export type NotionConnectionState =
  | "not_connected"
  | "checking"
  | "connected"
  | "attention";

/**
 * Where a workspace connected to Notion as `account` (if any) stands, given why its connection needed the user the last
 * time Notion was asked (`attention`), and what Notion says of it now (`status`, once asked).
 */
export const notionConnectionState = (
  account: NotionAccount | undefined,
  attention: IntegrationProblem | undefined,
  status: NotionApiStatus | undefined,
): NotionConnectionState => {
  if (!account) return "not_connected";
  if (attention || (status?.problem && needsUser(status.problem)))
    return "attention";
  return status ? "connected" : "checking";
};

/**
 * The account to preselect when connecting a workspace: its `current` one if still signed in, else the one the GitHub
 * CLI uses by default, else the first one signed in. None if the GitHub CLI isn't signed in to any.
 */
export const preselectedGhAccount = (
  accounts: readonly GhAccount[],
  current?: string | null,
): GhAccount | undefined => {
  const signedIn = accounts.filter((account) => account.signedIn);
  return (
    signedIn.find((account) => account.login === current) ??
    signedIn.find((account) => account.active) ??
    signedIn[0]
  );
};
