import { afterEach, describe, expect, it, vi } from "vitest";

import { beforeWorkspaceSwitch, canSwitchWorkspace } from "./workspaceSwitch";

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
