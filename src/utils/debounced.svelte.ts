/**
 * Reactive copy of `getValue()` that only updates once the value has stopped changing for `delayMs`.
 *
 * Must be created during component initialization, as it relies on an effect (cleaned up on unmount, which also
 * drops any pending update).
 *
 * ```ts
 * const debouncedName = new Debounced(() => name, 500);
 * $effect(() => save(debouncedName.current));
 * ```
 */
export class Debounced<T> {
  #current = $state() as T;

  constructor(getValue: () => T, delayMs: number) {
    this.#current = getValue();
    $effect(() => {
      const value = getValue();
      const timeout = setTimeout(() => (this.#current = value), delayMs);
      return () => clearTimeout(timeout);
    });
  }

  get current(): T {
    return this.#current;
  }
}
