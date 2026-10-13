/**
 * Why GitHub couldn't be asked: links then keep showing their last known status.
 * - `offline`: no connection, or GitHub failed to answer;
 * - `gh_missing`: the GitHub CLI wasn't found;
 * - `signed_out`: the GitHub CLI has no working token for the account anymore;
 * - `rate_limited`: GitHub asks to wait until `resetAt`;
 * - `failed`: anything else.
 */
export type GitHubProblem =
  | { kind: "offline" }
  | { kind: "gh_missing" }
  | { kind: "signed_out"; login: string }
  | { kind: "rate_limited"; resetAt: Date }
  | { kind: "failed"; message: string };
