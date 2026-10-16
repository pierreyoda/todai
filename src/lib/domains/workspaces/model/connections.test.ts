import { describe, expect, it } from "vitest";

import { attentionOf, isConnected } from "./connections";

describe("isConnected", () => {
  it("tells whether the workspace is connected to GitHub", () => {
    expect(isConnected({ githubLogin: "octocat" }, "github")).toBe(true);
    expect(isConnected({}, "github")).toBe(false);
    expect(isConnected(undefined, "github")).toBe(false);
  });
});

describe("attentionOf", () => {
  it("tells why the workspace's GitHub connection needs the user", () => {
    const signedOut = { kind: "signed_out", login: "octocat" } as const;
    expect(
      attentionOf(
        { githubLogin: "octocat", githubAttention: signedOut },
        "github",
      ),
    ).toEqual(signedOut);
    expect(attentionOf({ githubLogin: "octocat" }, "github")).toBeUndefined();
    expect(attentionOf(undefined, "github")).toBeUndefined();
  });
});
