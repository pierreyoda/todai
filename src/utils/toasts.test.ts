import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { MAX_TOASTS, Toasts } from "./toasts.svelte";

const DURATION = 1000;

describe("Toasts", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  const messages = (toasts: Toasts) => toasts.items.map(({ message }) => message);

  it("shows a toast for the duration", () => {
    const toasts = new Toasts(DURATION);
    toasts.show("Deleted");
    expect(toasts.items).toEqual([expect.objectContaining({ message: "Deleted", tone: "info" })]);

    vi.advanceTimersByTime(DURATION - 1);
    expect(messages(toasts)).toEqual(["Deleted"]);
    vi.advanceTimersByTime(1);
    expect(toasts.items).toEqual([]);
  });

  it("drops the oldest toasts beyond the maximum", () => {
    const toasts = new Toasts(DURATION);
    const shown = Array.from({ length: MAX_TOASTS + 1 }, (_, i) => `Toast ${i}`);
    for (const message of shown) toasts.show(message);
    expect(messages(toasts)).toEqual(shown.slice(1));
  });

  it("dismisses a toast before its time", () => {
    const toasts = new Toasts(DURATION);
    const id = toasts.show("First");
    toasts.show("Second");
    toasts.dismiss(id);
    expect(messages(toasts)).toEqual(["Second"]);
    // Its timer doesn't dismiss another one
    vi.advanceTimersByTime(DURATION - 1);
    expect(messages(toasts)).toEqual(["Second"]);
  });

  it("runs the action once, dismissing the toast", () => {
    const toasts = new Toasts(DURATION);
    const onClick = vi.fn();
    const id = toasts.show("Deleted", { action: { label: "Undo", onClick } });
    toasts.act(id);
    toasts.act(id);
    expect(onClick).toHaveBeenCalledTimes(1);
    expect(toasts.items).toEqual([]);
  });

  it("keeps the toasts while paused, then for what was left of their duration", () => {
    const toasts = new Toasts(DURATION);
    toasts.show("Older");
    vi.advanceTimersByTime(600);
    toasts.show("Newer");
    vi.advanceTimersByTime(100);

    toasts.pause();
    vi.advanceTimersByTime(10 * DURATION);
    expect(messages(toasts)).toEqual(["Older", "Newer"]);

    // Shown while paused: started on resume
    toasts.show("While paused");
    toasts.resume();
    vi.advanceTimersByTime(300);
    expect(messages(toasts)).toEqual(["Newer", "While paused"]);
    vi.advanceTimersByTime(600);
    expect(messages(toasts)).toEqual(["While paused"]);
    vi.advanceTimersByTime(100);
    expect(toasts.items).toEqual([]);
  });
});
