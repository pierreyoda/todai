/**
 * A component's own reactions to its mutation, e.g. closing its modal. Passed to the mutation's `createXxxMutation`, which
 * calls them first, before refreshing the cache: unlike `mutate`'s, which wait for it, and are skipped once the component
 * is gone.
 */
export type MutationHooks<V, D = void> = {
  onSuccess?: (data: D, variables: V) => void;
  onError?: (error: Error, variables: V) => void;
};
