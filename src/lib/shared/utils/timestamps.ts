import {
  format,
  formatDistanceStrict,
  isSameDay,
  isSameYear,
  subDays,
} from "date-fns";

/*
 * Moments (e.g. when a backup was made), unlike days, are shown in local time.
 */

/**
 * E.g. "Today, 14:32", "Yesterday, 08:47", "Oct 7, 21:15", or "Jun 30, 2025, 19:40" in another year than `now`'s.
 */
export const formatDateTime = (date: Date, now: Date = new Date()): string => {
  const time = format(date, "HH:mm");
  if (isSameDay(date, now)) return `Today, ${time}`;
  if (isSameDay(date, subDays(now, 1))) return `Yesterday, ${time}`;
  return format(
    date,
    isSameYear(date, now) ? "MMM d, HH:mm" : "MMM d, y, HH:mm",
  );
};

/** E.g. "14:32". */
export const formatTime = (date: Date): string => format(date, "HH:mm");

/** E.g. "Oct 9", or "Jun 30, 2025" in another year than `now`'s. */
export const formatShortDate = (date: Date, now: Date = new Date()): string =>
  format(date, isSameYear(date, now) ? "MMM d" : "MMM d, y");

/** E.g. "Saturday, October 10, 2026, 14:32:05". */
export const formatFullDateTime = (date: Date): string =>
  format(date, "EEEE, MMMM d, y, HH:mm:ss");

/** E.g. "Just now", "2 hours ago" or "3 months ago": from `now`, rounded to the largest unit. */
export const formatRelativeTime = (
  date: Date,
  now: Date = new Date(),
): string =>
  Math.abs(now.getTime() - date.getTime()) < 60 * 1000
    ? "Just now"
    : formatDistanceStrict(date, now, { addSuffix: true });
