/**
 * The last segment of `path`, e.g. "todai.db" for "/Users/me/todai.db" or "C:\Users\me\todai.db".
 * Both "/" and "\" are separators, and trailing ones are ignored. Empty for an empty or root path.
 */
export const basename = (path: string): string => {
  const segments = path.split(/[/\\]+/).filter((segment) => segment !== "");
  return segments.at(-1) ?? "";
};
