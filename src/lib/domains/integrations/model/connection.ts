import type { GhAccount, GhStatus } from "./types";

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
