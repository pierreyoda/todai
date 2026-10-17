import { format } from "date-fns";

import type { LinkStatus } from "./statuses";
import type {
  NotionColor,
  NotionPageSnapshot,
  NotionSnapshot,
  StatusGroup,
} from "./types";

/** The glyph shown before a Notion status, by the group it's filed under: like a checkbox's progress. */
const GROUP_GLYPHS: Record<StatusGroup, LinkStatus["glyph"]> = {
  to_do: "ring",
  in_progress: "dot",
  complete: "check",
};

/** A page's Status, as shown: in its Notion color, with its group's glyph. */
const statusOf = (page: NotionPageSnapshot): LinkStatus | null => {
  if (!page.status) return null;
  return {
    word: page.status.name,
    tone: "neutral",
    color: page.status.color,
    glyph: page.status.group ? GROUP_GLYPHS[page.status.group] : undefined,
  };
};

const IN_TRASH: LinkStatus = { word: "In trash", tone: "warn" };

/**
 * The one status word of a Notion page: "In trash" if it is, else its Status in its Notion color. `null` for a page
 * without a Status property, and for a whole database.
 */
export const notionStatus = (snapshot: NotionSnapshot): LinkStatus | null => {
  if (snapshot.inTrash) return IN_TRASH;
  return snapshot.type === "notion_page" ? statusOf(snapshot) : null;
};

/** The words telling where a Notion page stands, as in its details: its Status, and "In trash" if it is. */
export const notionStatusWords = (snapshot: NotionSnapshot): LinkStatus[] => {
  const status = snapshot.type === "notion_page" ? statusOf(snapshot) : null;
  return [status, snapshot.inTrash ? IN_TRASH : null].filter(
    (word): word is LinkStatus => word !== null,
  );
};

/** Why a todo linked to the Notion page `snapshot` (shown as `label`) could be marked as done, if so. */
export const notionDone = (
  snapshot: NotionSnapshot,
  label: string,
): { word: string; reason: string } | undefined => {
  if (
    snapshot.type !== "notion_page" ||
    snapshot.inTrash ||
    snapshot.status?.group !== "complete"
  )
    return undefined;
  return {
    word: snapshot.status.name,
    reason: `${label} is ${snapshot.status.name} in Notion`,
  };
};

/** A page's due date as shown, e.g. "Fri, Oct 16": `due` being a day (`YYYY-MM-DD`) or a date-time. */
export const formatDue = (due: string): string => {
  const day = /^(\d{4})-(\d{2})-(\d{2})$/.exec(due);
  if (day) {
    const [, year, month, date] = day.map(Number);
    return format(new Date(year, month - 1, date), "EEE, MMM d");
  }
  const at = new Date(due);
  return Number.isNaN(at.getTime()) ? due : format(at, "EEE, MMM d, HH:mm");
};

/** Notion's colors, as todai shows them on a dark background: text, then a pill's surface. */
export const NOTION_COLORS: readonly NotionColor[] = [
  "default",
  "gray",
  "brown",
  "orange",
  "yellow",
  "green",
  "blue",
  "purple",
  "pink",
  "red",
];
