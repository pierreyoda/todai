import type { IntegrationProblem, Provider } from "#core/integrations";
import type { Workspace } from "./types";

/** What a workspace tells of its connections. */
type Connections = Pick<Workspace, "githubLogin" | "githubAttention">;

/** Whether `workspace` is connected to `provider`: its links' items then show where they stand. */
export const isConnected = (
  workspace: Connections | null | undefined,
  provider: Provider,
): boolean => {
  switch (provider) {
    case "github":
      return !!workspace?.githubLogin;
  }
};

/**
 * Why `workspace`'s connection to `provider` needs the user, if it did the last time `provider` was asked (since todai
 * was opened): e.g. the GitHub CLI is missing.
 */
export const attentionOf = (
  workspace: Connections | null | undefined,
  provider: Provider,
): IntegrationProblem | undefined => {
  switch (provider) {
    case "github":
      return workspace?.githubAttention;
  }
};
