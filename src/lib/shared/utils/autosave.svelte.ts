/**
 * Where an `Autosave` is:
 * - `idle`: nothing left to save;
 * - `pending`: changes are waiting for the delay;
 * - `saving`: a save is in progress;
 * - `error`: the last save failed, its changes kept for the next one.
 */
export type AutosaveStatus = "idle" | "pending" | "saving" | "error";

/** The delay of an `Autosave`, in milliseconds. */
export const AUTOSAVE_DELAY = 2000;

/**
 * Saves a draft once it's left unchanged for `delay` ms, one save at a time: what changes during a save is saved right
 * after it. A failed save is retried with the next change, or `flush`.
 *
 * ```ts
 * const autosave = new Autosave(note?.content ?? "", (content) => save(content));
 * // <textarea bind:value={autosave.draft}></textarea>
 * ```
 */
export class Autosave {
  status = $state<AutosaveStatus>("idle");
  /** Why the last save failed, while `status` is `error`. */
  error = $state<unknown>();

  #draft = $state("");
  /** The content last saved, or loaded: the draft has unsaved changes while it differs. */
  #saved = $state("");
  readonly #save: (content: string) => Promise<unknown>;
  readonly #delay: number;
  #timer: ReturnType<typeof setTimeout> | undefined;
  /** The save in progress, if any. */
  #saving: Promise<void> | undefined;

  /**
   * @param saved The content already saved, e.g. loaded: the initial draft.
   * @param save Rejects if it fails.
   */
  constructor(saved: string, save: (content: string) => Promise<unknown>, delay = AUTOSAVE_DELAY) {
    this.#draft = saved;
    this.#saved = saved;
    this.#save = save;
    this.#delay = delay;
  }

  get draft(): string {
    return this.#draft;
  }

  /** Changes the draft, saved once left unchanged for the delay. */
  set draft(content: string) {
    this.#draft = content;
    this.#cancelTimer();
    if (this.dirty) {
      this.#timer = setTimeout(() => void this.flush(), this.#delay);
    }
    if (!this.#saving) this.#setStatus(this.dirty ? "pending" : "idle");
  }

  /** Whether the draft has changes not saved yet. */
  get dirty(): boolean {
    return this.#draft !== this.#saved;
  }

  /**
   * Saves the draft right away, after the save in progress if any, unless it has no unsaved changes.
   *
   * Resolves once done, with whether the draft is saved: `false` if saving failed, or if it changed meanwhile.
   */
  async flush(): Promise<boolean> {
    this.#cancelTimer();
    while (this.#saving) await this.#saving;
    if (!this.dirty) return true;

    const content = this.#draft;
    this.#setStatus("saving");
    this.#saving = Promise.resolve()
      .then(() => this.#save(content))
      .then(
        () => {
          this.#saved = content;
          // Changed during the save: its timer saves it
          this.#setStatus(this.dirty ? "pending" : "idle");
        },
        (error: unknown) => {
          this.#setStatus("error");
          this.error = error;
        },
      )
      .finally(() => (this.#saving = undefined));
    await this.#saving;
    return !this.dirty;
  }

  #setStatus(status: AutosaveStatus) {
    this.status = status;
    if (status !== "error") this.error = undefined;
  }

  #cancelTimer() {
    clearTimeout(this.#timer);
    this.#timer = undefined;
  }
}
