import { describe, expect, it } from "vitest";

import {
  continueList,
  findTasks,
  indent,
  outdent,
  toggleTask,
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

describe("findTasks", () => {
  it("finds the tasks of every kind of list, nested or quoted", () => {
    const text = [
      "- [ ] Bullet",
      "  - [x] Nested",
      "* [X] Star",
      "+ [ ] Plus",
      "1. [ ] Ordered",
      "2) [x] Parenthesis",
      "> - [ ] Quoted",
      "-\t[ ] Tab",
    ].join("\n");
    expect(findTasks(text).map(({ checked, text }) => [checked, text])).toEqual([
      [false, "Bullet"],
      [true, "Nested"],
      [true, "Star"],
      [false, "Plus"],
      [false, "Ordered"],
      [true, "Parenthesis"],
      [false, "Quoted"],
      [false, "Tab"],
    ]);
  });

  it("locates each box", () => {
    const text = "# Day\n\n- [ ] One\n  - [x] Two";
    for (const { offset } of findTasks(text)) {
      expect(text.slice(offset, offset + 3)).toMatch(/^\[[ x]\]$/);
    }
  });

  it("only keeps tasks with text, as the preview does", () => {
    expect(findTasks("- [ ]\n- [ ]   \n- [] No\n- [ ]No space\n-[ ] No space")).toEqual([]);
  });

  it("skips fenced code blocks", () => {
    const text = "```\n- [ ] Code\n```\n~~~\n- [ ] Code\n~~~\n- [ ] Task";
    expect(findTasks(text).map(({ text }) => text)).toEqual(["Task"]);
  });
});

describe("toggleTask", () => {
  /** `text` once its task shown `index`-th, `checked` and with `taskText`, is toggled; `null` if none matches. */
  const toggle = (text: string, index: number, checked: boolean, taskText: string) => {
    const edit = toggleTask(text, { index, checked, text: taskText });
    return edit && text.slice(0, edit.from) + edit.insert + text.slice(edit.to);
  };

  it("checks and unchecks a task, leaving the rest of the text", () => {
    const text = "- [ ] Milk\n- [x] Eggs\n- [ ] Bread";
    expect(toggle(text, 0, false, "Milk")).toBe("- [x] Milk\n- [x] Eggs\n- [ ] Bread");
    expect(toggle(text, 1, true, "Eggs")).toBe("- [ ] Milk\n- [ ] Eggs\n- [ ] Bread");
  });

  it("toggles nested and quoted tasks", () => {
    expect(toggle("- [ ] Call\n  - [ ] about the boiler", 1, false, "about the boiler")).toBe(
      "- [ ] Call\n  - [x] about the boiler",
    );
    expect(toggle("> 1. [X] Quoted", 0, true, "Quoted")).toBe("> 1. [ ] Quoted");
  });

  it("tells identical tasks apart by their position", () => {
    const text = "- [ ] Same\n- [ ] Same\n- [ ] Same";
    expect(toggle(text, 1, false, "Same")).toBe("- [ ] Same\n- [x] Same\n- [ ] Same");
  });

  it("skips what looks like a task but isn't shown as one", () => {
    // An indented code block: found, but not shown in the preview
    const text = "Code:\n\n    - [ ] Not a task\n\n- [ ] A task";
    expect(toggle(text, 0, false, "A task")).toBe("Code:\n\n    - [ ] Not a task\n\n- [x] A task");
  });

  it("changes nothing without a task in the same state and with the same text", () => {
    const text = "- [ ] Milk";
    expect(toggle(text, 0, true, "Milk")).toBeNull();
    expect(toggle(text, 0, false, "Eggs")).toBeNull();
    expect(toggle("", 0, false, "Milk")).toBeNull();
  });

  it("compares the text's first line only, trimmed", () => {
    expect(toggle("- [ ] Call the plumber  \n  about the boiler", 0, false, "Call the plumber\nabout the boiler")).toBe(
      "- [x] Call the plumber  \n  about the boiler",
    );
  });
});
