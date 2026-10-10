import { describe, expect, it } from "vitest";

import type { GhAccount, GhStatus } from "../client/app";
import { githubConnectionState, preselectedGhAccount } from "./github";

const account = (login: string, { active = false, signedIn = true } = {}): GhAccount => ({ login, active, signedIn });

const status = (...accounts: GhAccount[]): GhStatus => ({ path: "/opt/homebrew/bin/gh", accounts });

describe("githubConnectionState", () => {
  it("is not connected without an account, whatever the GitHub CLI's status", () => {
    expect(githubConnectionState(undefined, undefined)).toBe("not_connected");
    expect(githubConnectionState(null, status(account("pierreyoda")))).toBe("not_connected");
    expect(githubConnectionState("", { path: null, accounts: [] })).toBe("not_connected");
  });

  it("is checking while the GitHub CLI's status is unknown", () => {
    expect(githubConnectionState("pierreyoda", undefined)).toBe("checking");
  });

  it("is connected through an account the GitHub CLI is signed in to", () => {
    expect(githubConnectionState("pierreyoda", status(account("acme-dev", { active: true }), account("pierreyoda")))).toBe(
      "connected",
    );
  });

  it("is signed out once the GitHub CLI isn't signed in to its account anymore", () => {
    expect(githubConnectionState("pierreyoda", status(account("pierreyoda", { signedIn: false })))).toBe("signed_out");
    expect(githubConnectionState("pierreyoda", status(account("acme-dev")))).toBe("signed_out");
    expect(githubConnectionState("pierreyoda", status())).toBe("signed_out");
  });

  it("is missing the GitHub CLI once it's not found", () => {
    expect(githubConnectionState("pierreyoda", { path: null, accounts: [] })).toBe("gh_missing");
  });
});

describe("preselectedGhAccount", () => {
  const work = account("acme-dev");
  const personal = account("pierreyoda", { active: true });
  const signedOut = account("old-login", { signedIn: false });

  it("keeps the current account while it's signed in", () => {
    expect(preselectedGhAccount([work, personal], "acme-dev")).toBe(work);
  });

  it("else picks the GitHub CLI's default account, else the first one signed in", () => {
    expect(preselectedGhAccount([work, personal], "old-login")).toBe(personal);
    expect(preselectedGhAccount([signedOut, work, personal])).toBe(personal);
    expect(preselectedGhAccount([signedOut, work])).toBe(work);
  });

  it("never picks an account the GitHub CLI isn't signed in to", () => {
    expect(preselectedGhAccount([signedOut], "old-login")).toBeUndefined();
    expect(preselectedGhAccount([account("active", { active: true, signedIn: false })])).toBeUndefined();
    expect(preselectedGhAccount([])).toBeUndefined();
  });
});
