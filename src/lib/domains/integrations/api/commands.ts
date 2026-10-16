import type { UUID } from "node:crypto";
import type { Workspace } from "#domains/workspaces";
import { createInvoker } from "#shared/ipc/invoke.ts";
import type {
  GhStatus,
  GitHubApiStatus,
  NotionApiStatus,
  NotionConnectOutcome,
} from "../model/types";

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
  /**
   * Connects a workspace to Notion with a personal access token, instead of any other one: once Notion tells whose it
   * is, it's kept in the macOS Keychain. Fails only if it can't be kept.
   */
  connect_notion: {
    args: {
      workspaceId: UUID;
      token: string;
    };
    returns: NotionConnectOutcome;
  };
  /** Deletes its token from the Keychain: its links then only show what their URL tells. */
  disconnect_notion: {
    args: {
      workspaceId: UUID;
    };
    returns: Workspace;
  };
  /** Asks Notion whose token a workspace uses, which also checks that Notion still accepts it. Fails if not connected. */
  get_notion_api_status: {
    args: {
      workspaceId: UUID;
    };
    returns: NotionApiStatus;
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

export const invokeIntegrations =
  createInvoker<IntegrationCommands>("app command");
