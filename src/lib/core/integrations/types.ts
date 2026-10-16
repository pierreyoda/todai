/** A service the pull requests, issues... linked to todos are fetched from, to show where they stand. */
export type Provider = "github";

/**
 * Why a provider couldn't be asked: links then keep showing their last known status.
 * - `offline`: no connection, or the provider failed to answer;
 * - `gh_missing`: the GitHub CLI wasn't found;
 * - `signed_out`: the GitHub CLI has no working token for the account anymore;
 * - `rate_limited`: the provider asks to wait until `resetAt`;
 * - `failed`: anything else.
 */
export type IntegrationProblem =
  | { kind: "offline" }
  | { kind: "gh_missing" }
  | { kind: "signed_out"; login: string }
  | { kind: "rate_limited"; resetAt: Date }
  | { kind: "failed"; message: string };

/** Why each provider couldn't be asked, for those that couldn't. */
export type ProviderProblems = Partial<Record<Provider, IntegrationProblem>>;
