import type { Tag, Todo } from "../client/types";
import {
  createTodoSearch,
  filterTodos,
  type TodoSearchMatches,
} from "./todoSearch";

/**
 * Fuzzy search over `getTodos()`, by their title, the names of their tags (from `getTags()`), and their description:
 * bind a search field to `query`, then `filter` the todos to show. Pages keep their own order and grouping.
 *
 * ```ts
 * const search = new TodoSearch(() => todos.data ?? [], () => tags);
 * const shownTodos = $derived(search.filter(todos.data ?? []));
 * ```
 */
export class TodoSearch {
  query = $state("");

  readonly #getTodos: () => readonly Todo[];
  readonly #getTags: () => readonly Tag[];
  // Indexed again when the todos or the tags change, e.g. after an edit or a tag renaming
  readonly #search = $derived.by(() =>
    createTodoSearch(this.#getTodos(), this.#getTags()),
  );
  /** The matching todos by ID, `null` while `query` is blank. */
  readonly matches: TodoSearchMatches = $derived(this.#search(this.query));

  constructor(getTodos: () => readonly Todo[], getTags: () => readonly Tag[]) {
    this.#getTodos = getTodos;
    this.#getTags = getTags;
  }

  /** Whether `query` filters the todos, i.e. isn't blank. */
  get active(): boolean {
    return this.matches !== null;
  }

  /** The matching ones of `todos`, in their own order; all of them while inactive. */
  filter(todos: readonly Todo[]): readonly Todo[] {
    return filterTodos(todos, this.matches);
  }

  clear() {
    this.query = "";
  }
}
