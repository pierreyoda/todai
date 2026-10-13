import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Workspace } from "#domains/workspaces";
import type { GhStatus, GitHubApiStatus } from "../model/types";

/** Commands about the app itself rather than the active workspace's content. */
type IntegrationCommands = {
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

export const invokeIntegrations = createInvoker<IntegrationCommands>("app command");
