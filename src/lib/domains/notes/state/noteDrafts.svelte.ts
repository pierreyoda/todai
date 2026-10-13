import { SvelteMap } from "svelte/reactivity";

import type { Day } from "#core/time";

/**
 * The notes being written, by day, so that what shows whether a day has a note follows its editor rather than its
 * autosave, e.g. the side panel's dot for today's note.
 *
 * ```ts
 * onMount(() => noteDrafts.open(day, () => autosave.draft));
 * const hasNote = $derived(noteDrafts.hasNote(day, !!note.data));
 * ```
 */
export class NoteDrafts {
  /** Reads each open draft, by day: reactive when the draft is. */
  readonly #drafts = new SvelteMap<Day, () => string>();

  /** Follows the draft of `day`, read by `getContent`, until the returned function is called. */
  open(day: Day, getContent: () => string): () => void {
    this.#drafts.set(day, getContent);
    return () => {
      // Unless another editor of that day opened since
      if (this.#drafts.get(day) === getContent) this.#drafts.delete(day);
    };
  }

  /** Whether `day` has a note: as written while it's open, `saved` otherwise. */
  hasNote(day: Day, saved: boolean): boolean {
    const getContent = this.#drafts.get(day);
    // Blank, it's deleted once saved
    return getContent ? getContent().trim() !== "" : saved;
  }
}

/** The app's open drafts, opened by `NotePanel`. */
export const noteDrafts = new NoteDrafts();
