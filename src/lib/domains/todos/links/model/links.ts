import type { ParsedLink, TodoLinkKind } from "./types";

const KIND_NAMES: Record<TodoLinkKind, string> = {
  github_pull: "GitHub pull request",
  github_issue: "GitHub issue",
  url: "Link",
};

/** What a link points to, e.g. "GitHub pull request". */
export const linkKindName = (kind: TodoLinkKind): string => KIND_NAMES[kind];

/** A link in full, e.g. "GitHub pull request pierreyoda/todai#58", or "Link to docs.rs". */
export const describeLink = ({
  kind,
  label,
  github,
}: Pick<ParsedLink, "kind" | "label" | "github">): string =>
  github
    ? `${linkKindName(kind)} ${github.owner}/${github.repo}#${github.number}`
    : `Link to ${label}`;

/** E.g. "1 link", "3 links". */
export const formatLinksCount = (count: number): string =>
  `${count} ${count === 1 ? "link" : "links"}`;

/** Whether `links` already has the normalized `url`, e.g. as recognized by `parse_link`. */
export const hasLink = (
  links: readonly Pick<ParsedLink, "url">[],
  url: string,
): boolean => links.some((link) => link.url === url);

/** Whether `url` is an http(s) one: the only kind of link opened, so that opening one never runs anything. */
export const isWebUrl = (url: string): boolean => /^https?:\/\/[^/]/i.test(url);

/**
 * Whether `text` may be a link pasted or typed, worth recognizing (`parse_link`): a single word starting like a web
 * address, e.g. "https://docs.rs" or "github.com/pierreyoda/todai/pull/58".
 */
export const mayBeLink = (text: string): boolean =>
  /^(https?:\/\/|(www\.)?github\.com\/)\S+$/i.test(text.trim());
