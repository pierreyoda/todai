import type { Provider } from "#core/integrations";
import type { ParsedLink, TodoLink, TodoLinkKind } from "./types";

const KIND_NAMES: Record<TodoLinkKind, string> = {
  github_pull: "GitHub pull request",
  github_issue: "GitHub issue",
  notion: "Notion page",
  url: "Link",
};

/** What a link points to, e.g. "GitHub pull request". */
export const linkKindName = (kind: TodoLinkKind): string => KIND_NAMES[kind];

/** What a link points to, once its item is known: e.g. "Notion database" rather than "Notion page" for a whole one. */
export const linkItemName = ({
  kind,
  snapshot,
}: Pick<TodoLink, "kind"> & Partial<Pick<TodoLink, "snapshot">>): string =>
  snapshot?.type === "notion_database" ? "Notion database" : KIND_NAMES[kind];

const KIND_PROVIDERS: Record<TodoLinkKind, Provider | undefined> = {
  github_pull: "github",
  github_issue: "github",
  notion: "notion",
  url: undefined,
};

/** Where a link's item is fetched from, to show where it stands: `undefined` for any other web page. */
export const linkProvider = ({
  kind,
}: Pick<ParsedLink, "kind">): Provider | undefined => KIND_PROVIDERS[kind];

/**
 * Where a link's item is, as in "Looking up …": e.g. "pierreyoda/todai #58", or "“Roadmap” in Notion". `undefined` for
 * a plain web link.
 */
export const itemWhere = ({
  github,
  notion,
  label,
}: Pick<ParsedLink, "github" | "notion" | "label">): string | undefined => {
  if (github) return `${github.owner}/${github.repo} #${github.number}`;
  if (notion) return `“${label}” in Notion`;
  return undefined;
};

/** A link in full, e.g. "GitHub pull request pierreyoda/todai#58", "Notion page “Roadmap”", or "Link to docs.rs". */
export const describeLink = ({
  kind,
  label,
  github,
  notion,
  snapshot,
}: Pick<ParsedLink, "kind" | "label" | "github" | "notion"> &
  Partial<Pick<TodoLink, "snapshot">>): string => {
  if (github)
    return `${linkKindName(kind)} ${github.owner}/${github.repo}#${github.number}`;
  if (notion) return `${linkItemName({ kind, snapshot })} “${label}”`;
  return `Link to ${label}`;
};

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
 * address, e.g. "https://docs.rs", "github.com/pierreyoda/todai/pull/58" or "notion.so/Roadmap-8c1d…".
 */
export const mayBeLink = (text: string): boolean =>
  /^(https?:\/\/|(www\.)?github\.com\/|(www\.)?notion\.so\/|app\.notion\.com\/|[\w-]+\.notion\.site\/)\S+$/i.test(
    text.trim(),
  );
