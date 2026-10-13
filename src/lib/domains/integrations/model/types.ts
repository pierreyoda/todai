import type { GitHubProblem } from "#core/github";

/** An account the GitHub CLI (`gh`) knows on github.com. */
export interface GhAccount {
  login: string;
  /** The one `gh` uses by default (`gh auth switch` changes it). */
  active: boolean;
  /** Whether its token works: not once revoked or expired, until signing in again with `gh auth login`. */
  signedIn: boolean;
}

/** What the GitHub CLI offers to connect workspaces with. */
export interface GhStatus {
  /** Where `gh` was found: `null` if it's not installed (or not where expected). */
  path: string | null;
  /** The accounts it knows on github.com, signed in or not anymore: none without `gh`. */
  accounts: GhAccount[];
}

/** GitHub's rate limit for an account, in points: requests cost 1 or 2, for up to 50 pull requests and issues. */
export interface GitHubRateLimit {
  limit: number;
  remaining: number;
  resetAt: Date;
}

/** A workspace's GitHub connection, as GitHub sees it. */
export interface GitHubApiStatus {
  /** The connected account's rate limit, if GitHub could be asked. */
  rateLimit?: GitHubRateLimit;
  /** Why GitHub couldn't be asked, if so. */
  problem?: GitHubProblem;
}
