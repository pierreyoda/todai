import { describe, expect, it } from "vitest";
import {
  checkName,
  closingPullRequest,
  describeChecks,
  describeReview,
  itemState,
  notableChecks,
} from "./github";
import {
  describeProblem,
  doneSuggestion,
  linkStatus,
  linkStatusWords,
  statusesFetchedAt,
} from "./statuses";
import type {
  Checks,
  GitHubSnapshot,
  IssueSnapshot,
  NotionPageSnapshot,
  PullRequestSnapshot,
  RelatedItem,
  TodoLink,
} from "./types";

const checks = (overrides: Partial<Checks> = {}): Checks => ({
  state: "success",
  passed: 3,
  failed: 0,
  running: 0,
  skipped: 0,
  runs: [],
  ...overrides,
});

const pull = (
  overrides: Partial<PullRequestSnapshot> = {},
): PullRequestSnapshot => ({
  type: "pull_request",
  title: "Add todo_links table and link chips",
  state: "open",
  isDraft: false,
  additions: 412,
  deletions: 37,
  changedFiles: 14,
  headRef: "feature/todo-links",
  baseRef: "main",
  author: "pierreyoda",
  updatedAt: new Date("2026-10-10T12:00:00Z"),
  closes: [],
  ...overrides,
});

const issue = (overrides: Partial<IssueSnapshot> = {}): IssueSnapshot => ({
  type: "issue",
  title: "Link todos to GitHub pull requests and issues",
  state: "open",
  updatedAt: new Date("2026-10-09T12:00:00Z"),
  assignees: [],
  labels: [],
  comments: 4,
  closedBy: [],
  ...overrides,
});

const link = (
  snapshot?: GitHubSnapshot,
  overrides: Partial<TodoLink> = {},
): TodoLink => ({
  id: "0199d0c0-0000-7000-8000-000000000001",
  todoId: "0199d0c0-0000-7000-8000-000000000002",
  kind: snapshot?.type === "issue" ? "github_issue" : "github_pull",
  url: "https://github.com/pierreyoda/todai/pull/58",
  label: "todai#58",
  github: { owner: "pierreyoda", repo: "todai", number: 58 },
  createdAt: new Date("2026-10-10T08:00:00Z"),
  snapshot,
  fetchedAt: snapshot && new Date("2026-10-10T14:02:00Z"),
  ...overrides,
});

describe("linkStatus", () => {
  it.each([
    [
      "merged",
      pull({ state: "merged", checks: checks({ state: "failure" }) }),
      "Merged",
      "merged",
    ],
    [
      "closed",
      pull({ state: "closed", reviewDecision: "approved" }),
      "Closed",
      "neutral",
    ],
    [
      "a draft",
      pull({ isDraft: true, checks: checks({ state: "failure" }) }),
      "Draft",
      "neutral",
    ],
    [
      "failing its checks",
      pull({
        checks: checks({ state: "failure" }),
        reviewDecision: "approved",
      }),
      "Checks failing",
      "bad",
    ],
    [
      "with changes requested",
      pull({
        reviewDecision: "changes_requested",
        checks: checks({ state: "pending" }),
      }),
      "Changes requested",
      "bad",
    ],
    [
      "approved",
      pull({
        reviewDecision: "approved",
        checks: checks({ state: "pending" }),
      }),
      "Approved",
      "good",
    ],
    [
      "waiting for a review",
      pull({
        reviewDecision: "review_required",
        checks: checks({ state: "pending" }),
      }),
      "Review required",
      "pending",
    ],
    [
      "running its checks",
      pull({ checks: checks({ state: "pending" }) }),
      "Checks running",
      "pending",
    ],
    ["green, needing no review", pull({ checks: checks() }), "Open", "open"],
    ["without checks", pull(), "Open", "open"],
  ] as const)(
    "picks one word for a pull request %s",
    (_, snapshot, word, tone) => {
      expect(linkStatus(link(snapshot))).toEqual({ word, tone });
    },
  );

  it.each([
    ["open", issue(), "Open", "open"],
    [
      "completed",
      issue({ state: "closed", stateReason: "completed" }),
      "Completed",
      "merged",
    ],
    [
      "not planned",
      issue({ state: "closed", stateReason: "not_planned" }),
      "Not planned",
      "neutral",
    ],
    [
      "a duplicate",
      issue({ state: "closed", stateReason: "duplicate" }),
      "Duplicate",
      "neutral",
    ],
    [
      "closed long ago, without a reason",
      issue({ state: "closed" }),
      "Completed",
      "merged",
    ],
  ] as const)("picks one word for an issue %s", (_, snapshot, word, tone) => {
    expect(linkStatus(link(snapshot))).toEqual({ word, tone });
  });

  it("warns when GitHub can't tell, whatever was known before", () => {
    expect(
      linkStatus(link(pull({ state: "merged" }), { fetchError: "no_access" })),
    ).toEqual({
      word: "No access",
      tone: "warn",
    });
    expect(linkStatus(link(undefined, { fetchError: "sso_required" }))).toEqual(
      {
        word: "SSO required",
        tone: "warn",
      },
    );
  });

  it("has nothing to tell about links never fetched", () => {
    expect(linkStatus(link())).toBeNull();
  });
});

describe("linkStatusWords", () => {
  it("tells where the item stands, then its status if it tells more", () => {
    expect(
      linkStatusWords(
        link(pull({ checks: checks({ state: "pending", running: 1 }) })),
      ),
    ).toEqual([
      { word: "Open", tone: "open" },
      { word: "Checks running", tone: "pending" },
    ]);
    expect(linkStatusWords(link(pull({ state: "merged" })))).toEqual([
      { word: "Merged", tone: "merged" },
    ]);
    expect(linkStatusWords(link(issue(), { fetchError: "no_access" }))).toEqual(
      [
        { word: "Open", tone: "open" },
        { word: "No access", tone: "warn" },
      ],
    );
  });

  it("has nothing to tell about links never fetched", () => {
    expect(linkStatusWords(link())).toEqual([]);
    expect(
      linkStatusWords(link(undefined, { fetchError: "no_access" })),
    ).toEqual([]);
  });
});

describe("itemState", () => {
  it.each([
    [pull(), "open"],
    [pull({ isDraft: true }), "draft"],
    [pull({ state: "merged" }), "merged"],
    [pull({ state: "closed", isDraft: true }), "closed"],
    [issue(), "open"],
    [issue({ state: "closed", stateReason: "not_planned" }), "not_planned"],
    [issue({ state: "closed" }), "completed"],
  ] as const)("tells where %j stands", (snapshot, expected) => {
    expect(itemState(snapshot)).toBe(expected);
  });
});

const notionLink = (
  status: NotionPageSnapshot["status"],
  overrides: Partial<NotionPageSnapshot> = {},
): TodoLink => ({
  id: "0199d0c0-0000-7000-8000-000000000003",
  todoId: "0199d0c0-0000-7000-8000-000000000002",
  kind: "notion",
  url: "https://www.notion.so/Write-the-release-notes-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
  label: "TOD-27",
  notion: { id: "8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5" },
  createdAt: new Date("2026-10-10T08:00:00Z"),
  snapshot: {
    type: "notion_page",
    title: "Write the release notes",
    uniqueId: { prefix: "TOD", number: 27 },
    status,
    assignees: [],
    tags: [],
    lastEditedAt: new Date("2026-10-10T12:00:00Z"),
    inTrash: false,
    ...overrides,
  },
  fetchedAt: new Date("2026-10-10T14:02:00Z"),
});

describe("doneSuggestion", () => {
  it("suggests marking a todo as done once one of its Notion pages is complete", () => {
    const shipped = notionLink({
      name: "Shipped",
      color: "purple",
      group: "complete",
    });
    expect(
      doneSuggestion({
        completed: false,
        links: [
          notionLink({
            name: "In review",
            color: "blue",
            group: "in_progress",
          }),
          shipped,
        ],
      }),
    ).toEqual({
      link: shipped,
      word: "Shipped",
      reason: "TOD-27 is Shipped in Notion",
    });
  });

  it("doesn't for Notion pages not complete, trashed, or of an unknown group", () => {
    expect(
      doneSuggestion({
        completed: false,
        links: [
          notionLink({ name: "Not started", color: "default", group: "to_do" }),
          notionLink(
            { name: "Done", color: "green", group: "complete" },
            { inTrash: true },
          ),
          notionLink({ name: "Done", color: "green" }),
          notionLink(undefined),
        ],
      }),
    ).toBeUndefined();
  });

  it("suggests marking a todo as done once one of its pull requests is merged or issues completed", () => {
    const merged = link(pull({ state: "merged" }));
    expect(
      doneSuggestion({ completed: false, links: [link(pull()), merged] }),
    ).toEqual({
      link: merged,
      word: "Merged",
      reason: "todai#58 was merged",
    });
    const completed = link(
      issue({ state: "closed", stateReason: "completed" }),
    );
    expect(doneSuggestion({ completed: false, links: [completed] })).toEqual({
      link: completed,
      word: "Completed",
      reason: "todai#58 was completed",
    });
  });

  it("doesn't otherwise", () => {
    const notPlanned = link(
      issue({ state: "closed", stateReason: "not_planned" }),
    );
    expect(
      doneSuggestion({
        completed: false,
        links: [
          link(pull()),
          link(pull({ state: "closed" })),
          link(),
          notPlanned,
        ],
      }),
    ).toBeUndefined();
    expect(
      doneSuggestion({
        completed: true,
        links: [link(pull({ state: "merged" }))],
      }),
    ).toBeUndefined();
  });
});

describe("closingPullRequest", () => {
  const related = (
    number: number,
    state: RelatedItem["state"],
  ): RelatedItem => ({
    kind: "github_pull",
    url: `https://github.com/pierreyoda/todai/pull/${number}`,
    label: `todai#${number}`,
    github: { owner: "pierreyoda", repo: "todai", number },
    title: `Pull request ${number}`,
    state,
  });

  it("offers the first pull request closing an issue, unless closed unmerged or already linked", () => {
    const closedBy = [
      related(50, "closed"),
      related(58, "open"),
      related(61, "draft"),
    ];
    expect(closingPullRequest(issue({ closedBy }), [])?.label).toBe("todai#58");
    expect(
      closingPullRequest(issue({ closedBy }), [{ url: closedBy[1].url }])
        ?.label,
    ).toBe("todai#61");
    expect(
      closingPullRequest(issue({ closedBy: [related(57, "merged")] }), [])
        ?.label,
    ).toBe("todai#57");
  });

  it("offers none otherwise", () => {
    expect(
      closingPullRequest(issue({ closedBy: [related(50, "closed")] }), []),
    ).toBeUndefined();
    expect(closingPullRequest(issue(), [])).toBeUndefined();
    expect(closingPullRequest(pull(), [])).toBeUndefined();
    expect(closingPullRequest(undefined, [])).toBeUndefined();
  });
});

describe("statusesFetchedAt", () => {
  it("is when the most recently fetched link was", () => {
    const latest = new Date("2026-10-10T14:02:00Z");
    expect(
      statusesFetchedAt([
        { fetchedAt: new Date("2026-10-10T13:00:00Z") },
        { fetchedAt: latest },
        { fetchedAt: undefined },
      ]),
    ).toEqual(latest);
    expect(statusesFetchedAt([{ fetchedAt: undefined }])).toBeUndefined();
  });
});

describe("describeChecks", () => {
  it.each([
    [checks({ passed: 2, running: 1, skipped: 4 }), "2 of 3 passed, 1 running"],
    [
      checks({ passed: 11, failed: 1, running: 1, skipped: 13 }),
      "11 of 13 passed, 1 failing, 1 running",
    ],
    [checks({ passed: 3 }), "3 of 3 passed"],
    [checks({ passed: 0, skipped: 4 }), "4 skipped"],
  ])("sums up %j as %j", (value, expected) => {
    expect(describeChecks(value)).toBe(expected);
  });
});

describe("notableChecks", () => {
  it("lists failing, then running, then passed checks, without the skipped ones", () => {
    const runs = [
      { name: "lint", outcome: "passed" },
      { name: "triage", outcome: "skipped" },
      { name: "test", outcome: "running" },
      { name: "build", outcome: "failed" },
      { name: "check", outcome: "passed" },
    ] as const;
    const names = (max?: number) =>
      notableChecks(checks({ runs: [...runs] }), max).map(({ name }) => name);
    expect(names()).toEqual(["build", "test", "lint", "check"]);
    expect(names(2)).toEqual(["build", "test"]);
  });
});

describe("checkName", () => {
  it("names GitHub Actions jobs after their workflow first", () => {
    expect(checkName({ name: "svelte-check", workflow: "check" })).toBe(
      "check · svelte-check",
    );
    expect(checkName({ name: "ci/circleci" })).toBe("ci/circleci");
  });
});

describe("describeReview", () => {
  it.each([
    [undefined, "Not required"],
    ["approved", "Approved"],
    ["changes_requested", "Changes requested"],
    ["review_required", "Required, not given yet"],
  ] as const)("describes %j as %j", (reviewDecision, expected) => {
    expect(describeReview({ reviewDecision })).toBe(expected);
  });
});

describe("describeProblem", () => {
  const from = new Date(2026, 9, 10, 14, 2);

  it("tells since when the statuses are when offline", () => {
    expect(describeProblem("github", { kind: "offline" }, from)).toEqual({
      label: "Offline · link statuses from 14:02",
      detail:
        "GitHub could not be reached: the links show their statuses from 14:02.",
    });
    expect(describeProblem("github", { kind: "offline" }).label).toBe(
      "Offline",
    );
  });

  it("tells what to do about the GitHub CLI", () => {
    expect(
      describeProblem(
        "github",
        { kind: "signed_out", login: "pierreyoda" },
        from,
      ).detail,
    ).toContain("isn't signed in to pierreyoda anymore: run gh auth login");
    expect(describeProblem("github", { kind: "gh_missing" }).label).toBe(
      "GitHub CLI not found",
    );
  });

  it("tells what to do about a Notion token", () => {
    expect(describeProblem("notion", { kind: "token_refused" }, from)).toEqual({
      label: "Notion: paste a new token",
      detail:
        "Notion refused this workspace's token: it expired, or was revoked. Paste a new personal access token in Settings › Integrations. Until then, the links show their statuses from 14:02.",
    });
    expect(describeProblem("notion", { kind: "token_missing" }).label).toBe(
      "Notion: token missing",
    );
    expect(describeProblem("notion", { kind: "offline" }).detail).toBe(
      "Notion could not be reached: the links show what their URL tells.",
    );
  });

  it("tells until when GitHub asked to wait", () => {
    expect(
      describeProblem("github", {
        kind: "rate_limited",
        resetAt: new Date(2026, 9, 10, 15, 30),
      }).label,
    ).toBe("GitHub rate limit · until 15:30");
  });

  it("gives GitHub's error as is", () => {
    expect(
      describeProblem("github", {
        kind: "failed",
        message: "GitHub answered with HTTP 418.",
      }).detail,
    ).toBe(
      "GitHub answered with HTTP 418. Until then, the links show what their URL tells.",
    );
  });
});
