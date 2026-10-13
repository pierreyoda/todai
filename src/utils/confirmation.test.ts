import { describe, expect, it } from "vitest";

import { Confirmation } from "./confirmation.svelte";

describe("Confirmation", () => {
  it("is shown while asked, then resolves with the answer", async () => {
    const confirmation = new Confirmation();
    expect(confirmation.shown).toBe(false);

    const answered = confirmation.ask();
    expect(confirmation.shown).toBe(true);

    confirmation.answer(true);
    expect(confirmation.shown).toBe(false);
    await expect(answered).resolves.toBe(true);
  });

  it("answers no once closed otherwise, e.g. by the modal's close button", async () => {
    const confirmation = new Confirmation();
    const answered = confirmation.ask();

    confirmation.shown = false;

    expect(confirmation.shown).toBe(false);
    await expect(answered).resolves.toBe(false);
  });

  it("answers no to a question asked again before being answered", async () => {
    const confirmation = new Confirmation();
    const first = confirmation.ask();
    const second = confirmation.ask();

    confirmation.answer(true);

    await expect(first).resolves.toBe(false);
    await expect(second).resolves.toBe(true);
  });

  it("ignores answers when not asked", () => {
    const confirmation = new Confirmation();
    expect(() => confirmation.answer(true)).not.toThrow();
    confirmation.shown = false;
    expect(confirmation.shown).toBe(false);
  });
});
