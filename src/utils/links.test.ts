import { describe, expect, it } from "vitest";

import type { ParsedLink } from "../client/types";
import { describeLink, formatLinksCount, hasLink, isWebUrl, linkKindName } from "./links";

const pull: ParsedLink = {
  kind: "github_pull",
  url: "https://github.com/pierreyoda/todai/pull/58",
  label: "todai#58",
  github: { owner: "pierreyoda", repo: "todai", number: 58 },
};
const issue: ParsedLink = {
  kind: "github_issue",
  url: "https://github.com/pierreyoda/todai/issues/12",
  label: "todai#12",
  github: { owner: "pierreyoda", repo: "todai", number: 12 },
};
const page: ParsedLink = {
  kind: "url",
  url: "https://docs.rs/keyring/latest/keyring/",
  label: "docs.rs",
};

describe("linkKindName", () => {
  it.each([
    ["github_pull", "GitHub pull request"],
    ["github_issue", "GitHub issue"],
    ["url", "Link"],
  ] as const)("names %j links %j", (kind, expected) => {
    expect(linkKindName(kind)).toBe(expected);
  });
});

describe("describeLink", () => {
  it("names GitHub links with their owner and repository", () => {
    expect(describeLink(pull)).toBe("GitHub pull request pierreyoda/todai#58");
    expect(describeLink(issue)).toBe("GitHub issue pierreyoda/todai#12");
  });

  it("names other links by their domain", () => {
    expect(describeLink(page)).toBe("Link to docs.rs");
  });
});

describe("formatLinksCount", () => {
  it.each([
    [0, "0 links"],
    [1, "1 link"],
    [2, "2 links"],
    [12, "12 links"],
  ])("formats %j as %j", (count, expected) => {
    expect(formatLinksCount(count)).toBe(expected);
  });
});

describe("hasLink", () => {
  it("compares normalized URLs", () => {
    expect(hasLink([pull, page], pull.url)).toBe(true);
    expect(hasLink([pull, page], issue.url)).toBe(false);
    expect(hasLink([], pull.url)).toBe(false);
  });
});

describe("isWebUrl", () => {
  it.each(["https://github.com/pierreyoda/todai/pull/58", "http://localhost:1420/", "HTTPS://EXAMPLE.ORG"])(
    "accepts %j",
    (url) => {
      expect(isWebUrl(url)).toBe(true);
    },
  );

  it.each([
    "",
    "github.com/pierreyoda/todai",
    "javascript:alert(1)",
    "file:///etc/passwd",
    "mailto:someone@example.org",
    "https://",
    "https:///path",
  ])("rejects %j", (url) => {
    expect(isWebUrl(url)).toBe(false);
  });
});
