/*
 * Keyboard helpers of the Markdown editor, as pure functions: from a text and its selection, the edit to make.
 */

/** A selection in a text, as offsets: `start` equals `end` for a caret. */
export type TextSelection = { start: number; end: number };

/** Replaces `text.slice(from, to)` with `insert`, then selects `selection` (offsets in the edited text). */
export type TextEdit = {
  from: number;
  to: number;
  insert: string;
  selection: TextSelection;
};

/** What Tab adds to the start of a line, and Shift+Tab removes. */
const INDENT = "  ";

/** A list item's marker after its indentation: `- `, `* `, `+ `, `1. ` or `1) `, then `[ ] ` or `[x] ` for a task. */
const LIST_ITEM = /^(\s*)(?:([-*+])|(\d{1,9})([.)]))(\s+)(\[[ xX]\]\s+)?/;

/** Opening and closing lines of fenced code blocks. */
const CODE_FENCE = /^ {0,3}(?:```|~~~)/;

const caret = (offset: number): TextSelection => ({ start: offset, end: offset });

/** The bounds of the line containing `offset`, without its line break. */
const lineAt = (text: string, offset: number): { start: number; end: number } => {
  const start = offset === 0 ? 0 : text.lastIndexOf("\n", offset - 1) + 1;
  const lineBreak = text.indexOf("\n", offset);
  return { start, end: lineBreak === -1 ? text.length : lineBreak };
};

/** Whether the line starting at `lineStart` is within a fenced code block, where Markdown isn't interpreted. */
const isInCodeBlock = (text: string, lineStart: number): boolean =>
  text
    .slice(0, lineStart)
    .split("\n")
    .filter((line) => CODE_FENCE.test(line)).length %
    2 ===
  1;

/**
 * Enter on a list item: starts the next item, with the same indentation and marker (numbered ones incremented, tasks
 * unchecked). On an empty item, removes its marker instead, ending the list.
 *
 * `null` for a plain new line: outside of lists or code blocks, within a marker, or replacing a selection.
 */
export const continueList = (text: string, { start, end }: TextSelection): TextEdit | null => {
  if (start !== end) return null;
  const line = lineAt(text, start);
  const match = LIST_ITEM.exec(text.slice(line.start, line.end));
  if (!match || isInCodeBlock(text, line.start)) return null;
  const markerEnd = line.start + match[0].length;
  if (start < markerEnd) return null;

  if (text.slice(markerEnd, line.end).trim() === "") {
    return { from: line.start, to: line.end, insert: "", selection: caret(line.start) };
  }
  const [, indent, bullet, number, delimiter, spacing, task] = match;
  const marker = bullet ?? `${Number(number) + 1}${delimiter}`;
  const insert = `\n${indent}${marker}${spacing}${task ? "[ ] " : ""}`;
  return { from: start, to: start, insert, selection: caret(start + insert.length) };
};

/**
 * Shifts the lines touched by the selection by one indentation level, empty ones excepted: `+1` adds it, `-1` removes
 * up to one (two spaces, or a tab). The selection follows its text.
 *
 * `null` if no line changes.
 */
const shiftLines = (text: string, { start, end }: TextSelection, direction: 1 | -1): TextEdit | null => {
  const from = lineAt(text, start).start;
  // A selection ending at the start of a line leaves that line out
  const to = lineAt(text, end > start && text[end - 1] === "\n" ? end - 1 : end).end;

  let lineStart = from;
  let mappedStart = start;
  let mappedEnd = end;
  const lines = text
    .slice(from, to)
    .split("\n")
    .map((line) => {
      const added = direction > 0 && line !== "" ? INDENT.length : 0;
      const removed = direction < 0 ? (/^(?:\t| {1,2})/.exec(line)?.[0].length ?? 0) : 0;
      // How much an offset moves with this line's change: a caret at its start moves with its text
      const shift = (offset: number) =>
        added > 0
          ? offset > lineStart || (offset === lineStart && start === end)
            ? added
            : 0
          : -Math.min(Math.max(offset - lineStart, 0), removed);
      mappedStart += shift(start);
      mappedEnd += shift(end);
      lineStart += line.length + 1;
      return added > 0 ? INDENT + line : line.slice(removed);
    });

  const insert = lines.join("\n");
  if (insert === text.slice(from, to)) return null;
  return { from, to, insert, selection: { start: mappedStart, end: mappedEnd } };
};

/**
 * Tab: indents the lines touched by the selection, when it spans several lines or is on a list item (to nest it).
 * Otherwise, replaces the selection with spaces.
 *
 * `null` if nothing changes, e.g. on empty lines only.
 */
export const indent = (text: string, selection: TextSelection): TextEdit | null => {
  const { start, end } = selection;
  const line = lineAt(text, start);
  const multiline = end > line.end;
  if (!multiline && !LIST_ITEM.test(text.slice(line.start, line.end))) {
    return { from: start, to: end, insert: INDENT, selection: caret(start + INDENT.length) };
  }
  return shiftLines(text, selection, 1);
};

/**
 * Shift+Tab: removes one indentation level from the lines touched by the selection.
 *
 * `null` if none is indented.
 */
export const outdent = (text: string, selection: TextSelection): TextEdit | null =>
  shiftLines(text, selection, -1);

/**
 * Wraps the selection with `marker`, e.g. `**` to make it bold, or unwraps it if already wrapped (`marker` either
 * around or within it). Spaces at the selection's ends stay out, as Markdown ignores emphasis starting or ending with
 * one. With a caret, inserts both markers around it.
 */
export const toggleWrap = (text: string, selection: TextSelection, marker: string): TextEdit => {
  let { start, end } = selection;
  while (start < end && /\s/.test(text[start])) start++;
  while (end > start && /\s/.test(text[end - 1])) end--;
  const length = marker.length;
  const selected = text.slice(start, end);

  if (start >= length && text.slice(start - length, start) === marker && text.startsWith(marker, end)) {
    return {
      from: start - length,
      to: end + length,
      insert: selected,
      selection: { start: start - length, end: end - length },
    };
  }
  if (selected.length >= 2 * length && selected.startsWith(marker) && selected.endsWith(marker)) {
    return {
      from: start,
      to: end,
      insert: selected.slice(length, -length),
      selection: { start, end: end - 2 * length },
    };
  }
  return {
    from: start,
    to: end,
    insert: marker + selected + marker,
    selection: { start: start + length, end: end + length },
  };
};
