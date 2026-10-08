/**
 * The last segment of `path`, e.g. "todai.db" for "/Users/me/todai.db" or "C:\Users\me\todai.db".
 * Both "/" and "\" are separators, and trailing ones are ignored. Empty for an empty or root path.
 */
export const basename = (path: string): string => {
  const segments = path.split(/[/\\]+/).filter((segment) => segment !== "");
  return segments.at(-1) ?? "";
};

/** POSIX root ("/"), Windows drive with root ("C:\" or "C:/") or within a UNC share ("\\server\share\"). */
const ABSOLUTE_PATH = /^(\/|[a-zA-Z]:[/\\]|\\\\[^/\\]+[/\\][^/\\]+[/\\])/;

/**
 * Whether `path` is an absolute file path, as the backend expects for a database: not ending with a separator, nor
 * with "." or "..". Both POSIX and Windows paths are accepted, and the file system isn't checked.
 */
export const isValidPath = (path: string): boolean => {
  if (!ABSOLUTE_PATH.test(path) || /[/\\]$/.test(path)) {
    return false;
  }
  const name = basename(path);
  return name !== "" && name !== "." && name !== "..";
};
