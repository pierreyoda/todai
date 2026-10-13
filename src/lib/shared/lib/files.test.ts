import { describe, expect, it } from "vitest";

import { formatFileSize } from "./files";

describe("formatFileSize", () => {
  it.each([
    [0, "0 B"],
    [512, "512 B"],
    [1023, "1023 B"],
    [1024, "1 KB"],
    [1536, "1.5 KB"],
    [48 * 1024 + 300, "48 KB"],
    [1024 * 1024, "1 MB"],
    [1.5 * 1024 * 1024, "1.5 MB"],
    [123 * 1024 * 1024, "123 MB"],
    [2 * 1024 ** 3, "2 GB"],
    [5 * 1024 ** 5, "5120 TB"],
  ])("formats %d bytes as %j", (bytes, label) => {
    expect(formatFileSize(bytes)).toBe(label);
  });
});
