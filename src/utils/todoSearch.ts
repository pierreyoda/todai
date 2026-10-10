import Fuse, { type IFuseOptions, type RangeTuple } from "fuse.js";

import type { Tag, Todo } from "../client/types";
import { isDefined } from ".";

/** What a todo is searched by: its own fields, and the names of its tags. */
type TodoSearchDocument = {
  todo: Todo;
  tagNames: readonly string[];
};

export type TodoSearchMatch = {
  /** From 0 (perfect match) to 1. */
  score: number;
  /** Characters of the title matching the query, as inclusive ranges: empty if only other fields match. */
  titleRanges: readonly RangeTuple[];
};

/** The todos matching a query, by ID; `null` for a blank query, which doesn't filter anything. */
export type TodoSearchMatches = ReadonlyMap<Todo["id"], TodoSearchMatch> | null;

const TITLE_KEY = "todo.title";

const FUSE_OPTIONS: IFuseOptions<TodoSearchDocument> = {
  keys: [
    { name: TITLE_KEY, weight: 0.7 },
    { name: "tagNames", weight: 0.2 },
    { name: "todo.description", weight: 0.1 },
  ],
  // Matches anywhere in the text, not only near its start
  ignoreLocation: true,
  // "evenement" finds "événement", and the other way around
  ignoreDiacritics: true,
  // Tolerates a typo in a word, but not two in a short one: swapped letters count as two, so "reprot" doesn't find
  // "report", but "report" doesn't find "réparer" either
  threshold: 0.3,
  includeScore: true,
  includeMatches: true,
};

export const createTodoSearch = (
  todos: readonly Todo[],
  tags: readonly Tag[],
): ((query: string) => TodoSearchMatches) => {
  // Resolved from the tags list, so a renamed tag is found by its new name
  const tagNames = new Map(tags.map((tag) => [tag.id, tag.name]));
  const fuse = new Fuse(
    todos.map((todo) => ({
      todo,
      tagNames: todo.tagIds.map((id) => tagNames.get(id)).filter(isDefined),
    })),
    FUSE_OPTIONS,
  );

  return (query) => {
    const pattern = query.trim();
    if (!pattern) return null;
    return new Map(
      fuse.search(pattern).map(({ item, score, matches }) => [
        item.todo.id,
        {
          score: score ?? 0,
          titleRanges:
            matches?.find((match) => match.key === TITLE_KEY)?.indices ?? [],
        },
      ]),
    );
  };
};

/** The ones of `todos` in `matches`, in their own order; all of them without a search. */
export const filterTodos = (
  todos: readonly Todo[],
  matches: TodoSearchMatches,
): readonly Todo[] =>
  matches ? todos.filter((todo) => matches.has(todo.id)) : todos;
