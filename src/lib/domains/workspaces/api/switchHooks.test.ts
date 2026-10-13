import { afterEach, describe, expect, it, vi } from "vitest";

import {
  afterWorkspaceSwitch,
  beforeWorkspaceSwitch,
  canSwitchWorkspace,
  runAfterWorkspaceSwitch,
} from "./switchHooks";

describe("canSwitchWorkspace", () => {
  const stops: (() => void)[] = [];
  const register = (hook: () => Promise<boolean>) => {
    stops.push(beforeWorkspaceSwitch(hook));
    return hook;
  };
  afterEach(() => {
    for (const stop of stops.splice(0)) stop();
  });

  it("switches without any hook", async () => {
    expect(await canSwitchWorkspace()).toBe(true);
  });

  it("asks every hook in turn", async () => {
    const order: string[] = [];
    register(async () => (order.push("first"), true));
    register(async () => (order.push("second"), true));
    expect(await canSwitchWorkspace()).toBe(true);
    expect(order).toEqual(["first", "second"]);
  });

  it("stops at the first refusal", async () => {
    register(() => Promise.resolve(false));
    const after = register(vi.fn(() => Promise.resolve(true)));
    expect(await canSwitchWorkspace()).toBe(false);
    expect(after).not.toHaveBeenCalled();
  });

  it("no longer asks a hook once stopped", async () => {
    const hook = vi.fn(() => Promise.resolve(false));
    beforeWorkspaceSwitch(hook)();
    expect(await canSwitchWorkspace()).toBe(true);
    expect(hook).not.toHaveBeenCalled();
  });
});

describe("runAfterWorkspaceSwitch", () => {
  const stops: (() => void)[] = [];
  afterEach(() => {
    for (const stop of stops.splice(0)) stop();
  });

  it("calls every hook, resolving once they're all done", async () => {
    const done: string[] = [];
    let finishFirst = () => {};
    const first = new Promise<void>((resolve) => (finishFirst = resolve));
    stops.push(afterWorkspaceSwitch(() => first.then(() => done.push("first"))));
    stops.push(afterWorkspaceSwitch(async () => done.push("second")));

    const running = runAfterWorkspaceSwitch();
    await Promise.resolve();
    expect(done).toEqual(["second"]);
    finishFirst();
    await running;
    expect(done).toEqual(["second", "first"]);
  });

  it("no longer calls a hook once stopped", async () => {
    const hook = vi.fn(() => Promise.resolve());
    afterWorkspaceSwitch(hook)();
    await runAfterWorkspaceSwitch();
    expect(hook).not.toHaveBeenCalled();
  });
});
