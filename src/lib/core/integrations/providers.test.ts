import { describe, expect, it } from "vitest";

import {
  keepProblems,
  needsUser,
  problemEntries,
  providerInfo,
} from "./providers";

describe("providerInfo", () => {
  it("names each provider", () => {
    expect(providerInfo("github")).toEqual({
      name: "GitHub",
      where: "on GitHub",
    });
    expect(providerInfo("notion")).toEqual({
      name: "Notion",
      where: "in Notion",
    });
  });
});

describe("needsUser", () => {
  it("tells the problems only the user can solve", () => {
    expect(needsUser({ kind: "signed_out", login: "octocat" })).toBe(true);
    expect(needsUser({ kind: "gh_missing" })).toBe(true);
    expect(needsUser({ kind: "token_refused" })).toBe(true);
    expect(needsUser({ kind: "token_missing" })).toBe(true);
    expect(needsUser({ kind: "offline" })).toBe(false);
    expect(needsUser({ kind: "failed", message: "Nope" })).toBe(false);
  });
});

describe("problemEntries", () => {
  it("lists the providers' problems, keeping those asked for", () => {
    const offline = { kind: "offline" } as const;
    expect(problemEntries({ github: offline })).toEqual([["github", offline]]);
    expect(
      problemEntries({ notion: { kind: "token_refused" }, github: offline }),
    ).toEqual([
      ["github", offline],
      ["notion", { kind: "token_refused" }],
    ]);
    expect(problemEntries({ github: offline }, () => false)).toEqual([]);
    expect(problemEntries({})).toEqual([]);
    expect(problemEntries(undefined)).toEqual([]);
  });
});

describe("keepProblems", () => {
  it("keeps the problems of the providers asked for", () => {
    const offline = { kind: "offline" } as const;
    expect(keepProblems({ github: offline }, () => true)).toEqual({
      github: offline,
    });
    expect(keepProblems({ github: offline }, () => false)).toEqual({});
    expect(keepProblems(undefined, () => true)).toEqual({});
  });
});
