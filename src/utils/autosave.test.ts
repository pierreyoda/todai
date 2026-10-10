import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Autosave, AUTOSAVE_DELAY } from "./autosave.svelte";

/** A save to settle by hand, recording what it was called with. */
const manualSave = () => {
  const calls: { content: string; resolve: () => void; reject: (error: unknown) => void }[] = [];
  let inProgress = 0;
  let maxInProgress = 0;
  const save = (content: string) =>
    new Promise<void>((resolve, reject) => {
      inProgress++;
      maxInProgress = Math.max(maxInProgress, inProgress);
      const settle = (settled: () => void) => () => {
        inProgress--;
        settled();
      };
      calls.push({ content, resolve: settle(resolve), reject: (error) => settle(() => reject(error))() });
    });
  return { save, calls, maxInProgress: () => maxInProgress };
};

describe("Autosave", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts from the saved content, with nothing to save", () => {
    const autosave = new Autosave("# Saturday", () => Promise.resolve());
    expect(autosave.draft).toBe("# Saturday");
    expect(autosave.dirty).toBe(false);
    expect(autosave.status).toBe("idle");
  });

  it("saves the draft once left unchanged for the delay", async () => {
    const save = vi.fn(() => Promise.resolve());
    const autosave = new Autosave("", save);

    autosave.draft = "#";
    expect(autosave.status).toBe("pending");
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY - 100);
    autosave.draft = "# Sat";
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY - 1);
    expect(save).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledExactlyOnceWith("# Sat");
    expect(autosave.status).toBe("idle");
    expect(autosave.dirty).toBe(false);
  });

  it("saves nothing once changed back to the saved content", async () => {
    const save = vi.fn(() => Promise.resolve());
    const autosave = new Autosave("Saved", save);

    autosave.draft = "Saved!";
    autosave.draft = "Saved";
    expect(autosave.status).toBe("idle");
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY * 2);
    expect(save).not.toHaveBeenCalled();
  });

  it("saves the changes made during a save right after it, one save at a time", async () => {
    const { save, calls, maxInProgress } = manualSave();
    const autosave = new Autosave("", save);

    autosave.draft = "a";
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY);
    expect(autosave.status).toBe("saving");
    autosave.draft = "ab";
    expect(autosave.status).toBe("saving");
    // Its delay ends while the first save is still in progress
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY);
    expect(calls).toHaveLength(1);

    calls[0].resolve();
    await vi.advanceTimersByTimeAsync(0);
    expect(calls.map(({ content }) => content)).toEqual(["a", "ab"]);
    expect(autosave.status).toBe("saving");
    calls[1].resolve();
    await vi.advanceTimersByTimeAsync(0);
    expect(autosave.status).toBe("idle");
    expect(maxInProgress()).toBe(1);
  });

  it("keeps the changes of a failed save, saved again with the next change", async () => {
    const { save, calls } = manualSave();
    const autosave = new Autosave("", save);

    autosave.draft = "Lost?";
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY);
    calls[0].reject("Database error: disk I/O error");
    await vi.advanceTimersByTimeAsync(0);
    expect(autosave.status).toBe("error");
    expect(autosave.error).toBe("Database error: disk I/O error");
    expect(autosave.dirty).toBe(true);

    autosave.draft = "Lost? No.";
    expect(autosave.status).toBe("pending");
    expect(autosave.error).toBeUndefined();
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY);
    calls[1].resolve();
    await vi.advanceTimersByTimeAsync(0);
    expect(calls[1].content).toBe("Lost? No.");
    expect(autosave.status).toBe("idle");
  });

  describe("flush", () => {
    it("saves right away, instead of after the delay", async () => {
      const save = vi.fn(() => Promise.resolve());
      const autosave = new Autosave("", save);

      autosave.draft = "Now";
      await expect(autosave.flush()).resolves.toBe(true);
      expect(save).toHaveBeenCalledExactlyOnceWith("Now");
      await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY);
      expect(save).toHaveBeenCalledOnce();
    });

    it("saves nothing without unsaved changes", async () => {
      const save = vi.fn(() => Promise.resolve());
      await expect(new Autosave("Saved", save).flush()).resolves.toBe(true);
      expect(save).not.toHaveBeenCalled();
    });

    it("waits for the save in progress, then saves the latest draft", async () => {
      const { save, calls } = manualSave();
      const autosave = new Autosave("", save);

      autosave.draft = "a";
      await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY);
      autosave.draft = "ab";
      const flushed = autosave.flush();
      calls[0].resolve();
      await vi.advanceTimersByTimeAsync(0);
      expect(calls.map(({ content }) => content)).toEqual(["a", "ab"]);
      calls[1].resolve();
      await expect(flushed).resolves.toBe(true);
      expect(autosave.status).toBe("idle");
    });

    it("retries a failed save, and tells whether it failed again", async () => {
      const save = vi.fn(() => Promise.reject(new Error("Database error")));
      const autosave = new Autosave("", save);

      autosave.draft = "Retried";
      await expect(autosave.flush()).resolves.toBe(false);
      await expect(autosave.flush()).resolves.toBe(false);
      expect(save).toHaveBeenCalledTimes(2);
      expect(autosave.status).toBe("error");
      expect(autosave.draft).toBe("Retried");
    });

    it("catches a save throwing right away", async () => {
      const autosave = new Autosave("", () => {
        throw new Error("Not even started");
      });
      autosave.draft = "Thrown";
      await expect(autosave.flush()).resolves.toBe(false);
      expect(autosave.status).toBe("error");
    });
  });
});
