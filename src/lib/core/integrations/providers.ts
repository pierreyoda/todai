import type { IntegrationProblem, Provider, ProviderProblems } from "./types";

/** How a provider is shown. */
export interface ProviderInfo {
  /** E.g. "GitHub". */
  name: string;
  /** Where its items are, as in "Open on GitHub". */
  where: string;
}

const PROVIDER_INFOS: Readonly<Record<Provider, ProviderInfo>> = {
  github: { name: "GitHub", where: "on GitHub" },
};

/** Every provider, in the order they're shown. */
export const PROVIDERS: readonly Provider[] = ["github"];

/** How `provider` is shown, e.g. its name. */
export const providerInfo = (provider: Provider): ProviderInfo =>
  PROVIDER_INFOS[provider];

/**
 * Whether only the user can solve `problem`, in todai's settings or elsewhere (e.g. by signing in again), rather than it
 * passing on its own (e.g. once online again).
 */
export const needsUser = (problem: IntegrationProblem): boolean =>
  problem.kind === "signed_out" || problem.kind === "gh_missing";

/** The providers of `problems` with their problem, in the order they're shown, keeping only those `keep` says. */
export const problemEntries = (
  problems: ProviderProblems | undefined,
  keep: (provider: Provider) => boolean = () => true,
): [Provider, IntegrationProblem][] =>
  PROVIDERS.flatMap((provider) => {
    const problem = problems?.[provider];
    return problem && keep(provider)
      ? [[provider, problem] as [Provider, IntegrationProblem]]
      : [];
  });

/** `problems`, keeping only the providers `keep` says. */
export const keepProblems = (
  problems: ProviderProblems | undefined,
  keep: (provider: Provider) => boolean,
): ProviderProblems => Object.fromEntries(problemEntries(problems, keep));
