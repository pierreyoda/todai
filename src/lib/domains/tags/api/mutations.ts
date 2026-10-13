import { createMutation, useQueryClient, type QueryClient } from "@tanstack/svelte-query";

import type { MutationHooks } from "#shared/ipc/mutations.ts";
import type { Tag } from "../model/types";
import { invokeTags } from "./commands";
import { tagKeys } from "./queries";

type NewTag = Pick<Tag, "name" | "color">;
type TagChanges = Pick<Tag, "id"> & Partial<NewTag>;

/**
 * Refreshes the tags list, the only one to refresh as todos reference tags by ID. Returned by the mutations, so that
 * they stay pending until it's refetched.
 */
const refreshTags = (queryClient: QueryClient) => queryClient.invalidateQueries({ queryKey: tagKeys.all });

/** Creates a tag. Must be called during component initialization. */
export const createAddTagMutation = (hooks: MutationHooks<NewTag, Tag> = {}) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (tag: NewTag) => invokeTags({ name: "create_tag", args: tag }),
    onSuccess: (created, tag) => {
      hooks.onSuccess?.(created, tag);
      return refreshTags(queryClient);
    },
    onError: (error, tag) => hooks.onError?.(error, tag),
  }));
};

/** Renames a tag and/or or changes its color. Must be called during component initialization. */
export const createUpdateTagMutation = (hooks: MutationHooks<TagChanges, Tag> = {}) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (changes: TagChanges) => invokeTags({ name: "update_tag", args: changes }),
    onSuccess: (updated, changes) => {
      hooks.onSuccess?.(updated, changes);
      return refreshTags(queryClient);
    },
    onError: (error, changes) => hooks.onError?.(error, changes),
  }));
};

/** Deletes a tag: its todos lose it. Must be called during component initialization. */
export const createDeleteTagMutation = (hooks: MutationHooks<Tag["id"]> = {}) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: Tag["id"]) => invokeTags({ name: "delete_tag", args: { id } }),
    onSuccess: (_, id) => {
      hooks.onSuccess?.(undefined, id);
      return refreshTags(queryClient);
    },
    onError: (error, id) => hooks.onError?.(error, id),
  }));
};
