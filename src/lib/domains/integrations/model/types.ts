import type { IntegrationProblem } from "#core/integrations";
import type { NotionAccount, Workspace } from "#domains/workspaces";

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

/** A workspace's Notion connection, as Notion sees it. */
export interface NotionApiStatus {
  /** Whose token it is, if Notion could be asked. */
  account?: NotionAccount;
  /** Why Notion couldn't be asked, if so: e.g. it refused the token. */
  problem?: IntegrationProblem;
}

/**
 * What connecting a workspace to Notion gave: connected, with the token kept, or not, when Notion couldn't vouch for
 * the token (nothing was kept, and the connection is as it was).
 */
export type NotionConnectOutcome =
  | { kind: "connected"; workspace: Workspace }
  | { kind: "not_connected"; problem: IntegrationProblem };

/** A workspace's GitHub connection, as GitHub sees it. */
export interface GitHubApiStatus {
  /** The connected account's rate limit, if GitHub could be asked. */
  rateLimit?: GitHubRateLimit;
  /** Why GitHub couldn't be asked, if so. */
  problem?: IntegrationProblem;
}
