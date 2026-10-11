/** What a toast offers to do about what it tells, e.g. undo it. */
export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface Toast {
  id: number;
  message: string;
  tone: "info" | "error";
  action?: ToastAction;
}

/** How long a toast is shown, in milliseconds, not counting while paused. */
export const TOAST_DURATION = 6000;

/** How many toasts are shown at most: showing another one drops the oldest. */
export const MAX_TOASTS = 3;

type Timer = {
  handle?: ReturnType<typeof setTimeout>;
  /** Left to show it for, when it was last (re)started. */
  remaining: number;
  startedAt: number;
};

/**
 * Short messages shown for a moment over the page, e.g. after deleting a todo with a way to undo it. They're paused
 * altogether while the user points at or focuses them, so that their actions stay in reach.
 *
 * ```ts
 * toasts.show("Deleted “Groceries”", { action: { label: "Undo", onClick: restore } });
 * ```
 */
export class Toasts {
  /** Oldest first. */
  items = $state<Toast[]>([]);

  readonly #duration: number;
  #timers = new Map<number, Timer>();
  #paused = false;
  #nextId = 1;

  constructor(duration = TOAST_DURATION) {
    this.#duration = duration;
  }

  /** Shows a toast for the duration, returning its ID. */
  show(message: string, { tone = "info", action }: { tone?: Toast["tone"]; action?: ToastAction } = {}): number {
    const id = this.#nextId++;
    this.items = [...this.items, { id, message, tone, action }];
    for (const { id: dropped } of this.items.slice(0, -MAX_TOASTS)) {
      this.dismiss(dropped);
    }
    const timer: Timer = { remaining: this.#duration, startedAt: Date.now() };
    this.#timers.set(id, timer);
    if (!this.#paused) this.#start(id, timer);
    return id;
  }

  dismiss(id: number) {
    clearTimeout(this.#timers.get(id)?.handle);
    this.#timers.delete(id);
    this.items = this.items.filter((toast) => toast.id !== id);
  }

  /** Runs the toast's action, then dismisses it. */
  act(id: number) {
    const toast = this.items.find((toast) => toast.id === id);
    this.dismiss(id);
    toast?.action?.onClick();
  }

  /** Keeps all the toasts shown until `resume`. */
  pause() {
    if (this.#paused) return;
    this.#paused = true;
    const now = Date.now();
    for (const timer of this.#timers.values()) {
      clearTimeout(timer.handle);
      timer.remaining -= now - timer.startedAt;
    }
  }

  /** Lets each toast go once shown for what was left of its duration. */
  resume() {
    if (!this.#paused) return;
    this.#paused = false;
    for (const [id, timer] of this.#timers) {
      this.#start(id, timer);
    }
  }

  #start(id: number, timer: Timer) {
    timer.startedAt = Date.now();
    timer.handle = setTimeout(() => this.dismiss(id), Math.max(timer.remaining, 0));
  }
}

/** The app's toasts, shown by `Toaster` in the layout. */
export const toasts = new Toasts();
