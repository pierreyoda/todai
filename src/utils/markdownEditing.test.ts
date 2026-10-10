import { describe, expect, it } from "vitest";

import {
  continueList,
  indent,
  outdent,
  toggleWrap,
  type TextEdit,
  type TextSelection,
} from "./markdownEditing";

/*
 * Texts are written with their selection: `|` for a caret, `«` and `»` around a selected range.
 */

const parse = (marked: string): [string, TextSelection] => {
  if (marked.includes("|")) {
    const at = marked.indexOf("|");
    return [marked.replace("|", ""), { start: at, end: at }];
  }
  const start = marked.indexOf("«");
  const end = marked.indexOf("»") - 1;
  return [marked.replace("«", "").replace("»", ""), { start, end }];
};

const mark = (text: string, { start, end }: TextSelection): string =>
  start === end
    ? `${text.slice(0, start)}|${text.slice(start)}`
    : `${text.slice(0, start)}«${text.slice(start, end)}»${text.slice(end)}`;

/** `marked` once edited by `helper`, with its new selection; `null` if left to the browser. */
const run = (
  helper: (text: string, selection: TextSelection) => TextEdit | null,
  marked: string,
): string | null => {
  const [text, selection] = parse(marked);
  const edit = helper(text, selection);
  if (!edit) return null;
  return mark(text.slice(0, edit.from) + edit.insert + text.slice(edit.to), edit.selection);
};

describe("continueList", () => {
  it.each([
    ["- milk|", "- milk\n- |"],
    ["* milk|", "* milk\n* |"],
    ["+ milk|", "+ milk\n+ |"],
    ["eggs\n  - nested|", "eggs\n  - nested\n  - |"],
    ["1. one|", "1. one\n2. |"],
    ["9) nine|", "9) nine\n10) |"],
    ["-   wide|", "-   wide\n-   |"],
    ["- [x] done|", "- [x] done\n- [ ] |"],
    ["- [ ] todo|", "- [ ] todo\n- [ ] |"],
    ["3. [X] done|", "3. [X] done\n4. [ ] |"],
  ])("starts the next item after %j", (before, after) => {
    expect(run(continueList, before)).toBe(after);
  });

  it("moves the text after the caret to the next item", () => {
    expect(run(continueList, "- milk|eggs")).toBe("- milk\n- |eggs");
  });

  it.each([
    ["- |", "|"],
    ["- milk\n- |", "- milk\n|"],
    ["- milk\n  - [ ] |\nafter", "- milk\n|\nafter"],
    ["1.   |", "|"],
  ])("ends the list on an empty item: %j", (before, after) => {
    expect(run(continueList, before)).toBe(after);
  });

  it.each([
    ["plain text|"],
    ["# Heading|"],
    ["---|"],
    ["**bold**|"],
    ["-not a list|"],
    ["1.5 liters|"],
  ])("leaves a plain new line outside of lists: %j", (text) => {
    expect(run(continueList, text)).toBeNull();
  });

  it("leaves a plain new line within a marker", () => {
    expect(run(continueList, "|- milk")).toBeNull();
    expect(run(continueList, "-| milk")).toBeNull();
    expect(run(continueList, "- [ |] milk")).toBeNull();
  });

  it("leaves a plain new line replacing a selection", () => {
    expect(run(continueList, "- «milk»")).toBeNull();
  });

  it("ignores lists in fenced code blocks", () => {
    expect(run(continueList, "```\n- milk|")).toBeNull();
    expect(run(continueList, "~~~md\n- milk|\n~~~")).toBeNull();
    expect(run(continueList, "```\ncode\n```\n- milk|")).toBe("```\ncode\n```\n- milk\n- |");
  });
});

describe("indent", () => {
  it("inserts spaces at a caret outside of lists", () => {
    expect(run(indent, "milk|")).toBe("milk  |");
    expect(run(indent, "|")).toBe("  |");
  });

  it("replaces a selection within a line with spaces", () => {
    expect(run(indent, "mi«lk eg»gs")).toBe("mi  |gs");
  });

  it("nests a list item, wherever the caret is on it", () => {
    expect(run(indent, "- milk|")).toBe("  - milk|");
    expect(run(indent, "- m|ilk")).toBe("  - m|ilk");
    expect(run(indent, "|- milk")).toBe("  |- milk");
    expect(run(indent, "- milk\n- [ ] eggs|")).toBe("- milk\n  - [ ] eggs|");
  });

  it("indents every line touched by a selection, empty ones excepted", () => {
    expect(run(indent, "«a\nb»")).toBe("«  a\n  b»");
    expect(run(indent, "a«b\n\nc»d")).toBe("  a«b\n\n  c»d");
  });

  it("leaves out the line where the selection ends at its start", () => {
    expect(run(indent, "«a\n»b")).toBe("«  a\n»b");
  });

  it("changes nothing on empty lines only", () => {
    expect(run(indent, "a\n«\n»\nb")).toBeNull();
  });
});

describe("outdent", () => {
  it.each([
    ["  - milk|", "- milk|"],
    ["\t- milk|", "- milk|"],
    ["    deep|", "  deep|"],
    [" one space|", "one space|"],
    ["  |milk", "|milk"],
    [" | milk", "|milk"],
  ])("removes one indentation level from %j", (before, after) => {
    expect(run(outdent, before)).toBe(after);
  });

  it("outdents every line touched by a selection", () => {
    expect(run(outdent, "  a«b\n    c»d\ne")).toBe("a«b\n  c»d\ne");
  });

  it("changes nothing without indentation", () => {
    expect(run(outdent, "- milk|")).toBeNull();
    expect(run(outdent, "«a\nb»")).toBeNull();
  });
});

describe("toggleWrap", () => {
  it("wraps the selection", () => {
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "a «bold» b")).toBe("a **«bold»** b");
    expect(run((text, selection) => toggleWrap(text, selection, "_"), "an «italic» word")).toBe(
      "an _«italic»_ word",
    );
  });

  it("leaves the spaces at the selection's ends outside", () => {
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "«bold »next")).toBe("**«bold»** next");
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "a« bold» b")).toBe("a **«bold»** b");
  });

  it("unwraps a selection already wrapped, around or within it", () => {
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "a **«bold»** b")).toBe("a «bold» b");
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "a «**bold**» b")).toBe("a «bold» b");
  });

  it("inserts both markers around a caret, and removes them if empty", () => {
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "a |b")).toBe("a **|**b");
    expect(run((text, selection) => toggleWrap(text, selection, "**"), "a **|**b")).toBe("a |b");
  });

  it("keeps bold and italic apart", () => {
    expect(run((text, selection) => toggleWrap(text, selection, "_"), "**«bold»**")).toBe("**_«bold»_**");
  });
});
