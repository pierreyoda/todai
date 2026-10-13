/**
 * A yes-or-no question asked through a modal, answered once: e.g. whether to leave although changes couldn't be saved.
 *
 * ```ts
 * const leaving = new Confirmation();
 * if (await leaving.ask()) leave();
 * // <UnsavedLeaveModal bind:show={leaving.shown} onLeave={() => leaving.answer(true)} />
 * ```
 */
export class Confirmation {
  #shown = $state(false);
  #resolve: ((confirmed: boolean) => void) | undefined;

  /** Whether it's being asked: the modal's `show`. Closing the modal without answering is answering no. */
  get shown(): boolean {
    return this.#shown;
  }

  set shown(shown: boolean) {
    if (shown) this.#shown = true;
    else this.answer(false);
  }

  /** Asks the question, resolving with its answer. Asked again before it's answered, the former one is answered no. */
  ask(): Promise<boolean> {
    this.#resolve?.(false);
    this.#shown = true;
    return new Promise((resolve) => (this.#resolve = resolve));
  }

  answer(confirmed: boolean) {
    this.#resolve?.(confirmed);
    this.#resolve = undefined;
    this.#shown = false;
  }
}
