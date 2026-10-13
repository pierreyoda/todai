import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";
import type { GitHubProblem } from "#core/github";
import type { Workspace } from "#domains/workspaces";

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

/** Commands about the app itself rather than the active workspace's content. */
export type ClientInvocationAppCommand = {
  /** `gh` is looked for once, then remembered: `refind` looks for it again, e.g. after installing it. */
  get_gh_status: {
    args?: {
      refind?: boolean;
    };
    returns: GhStatus;
  };
  /** Instead of any other account. Fails unless `gh` is signed in to `login` on github.com. */
  connect_github: {
    args: {
      workspaceId: UUID;
      login: string;
    };
    returns: Workspace;
  };
  /** Its links then only show what their URL tells. */
  disconnect_github: {
    args: {
      workspaceId: UUID;
    };
    returns: Workspace;
  };
  /**
   * Asks GitHub for the rate limit of the account a workspace is connected to, which also checks that GitHub accepts
   * its token. Fails if it isn't connected.
   */
  get_github_api_status: {
    args: {
      workspaceId: UUID;
    };
    returns: GitHubApiStatus;
  };
  set_workspace_links_behaviors: {
    args: {
      workspaceId: UUID;
      linksAutoRefresh: boolean;
      linksSuggestDone: boolean;
      linksPasteToCreate: boolean;
    };
    returns: Workspace;
  };
};

export const invokeApiClient = createInvoker<ClientInvocationAppCommand>("app command");
