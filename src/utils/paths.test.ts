import { describe, expect, it } from "vitest";

import { basename } from "./paths";

describe("basename", () => {
  it.each([
    ["/Users/me/todai.db", "todai.db"],
    ["/Users/me/.todai", ".todai"],
    ["/Users/me/archive.tar.gz", "archive.tar.gz"],
    ["/Users/me/My Todos.db", "My Todos.db"],
    ["relative/path/todai.db", "todai.db"],
    ["todai.db", "todai.db"],
  ])("returns the file name of the POSIX path %j", (path, expected) => {
    expect(basename(path)).toBe(expected);
  });

  it.each([
    ["C:\\Users\\me\\todai.db", "todai.db"],
    ["\\\\server\\share\\todai.db", "todai.db"],
    ["C:/Users/me\\todai.db", "todai.db"],
  ])("returns the file name of the Windows path %j", (path, expected) => {
    expect(basename(path)).toBe(expected);
  });

  it("returns the last directory of a path with trailing separators", () => {
    expect(basename("/Users/me/")).toBe("me");
    expect(basename("C:\\Users\\me\\\\")).toBe("me");
  });

  it("ignores repeated separators", () => {
    expect(basename("/Users//me///todai.db")).toBe("todai.db");
  });

  it.each(["", "/", "//", "\\"])("is empty for the empty or root path %j", (path) => {
    expect(basename(path)).toBe("");
  });
});
