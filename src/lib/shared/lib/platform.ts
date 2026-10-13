const platform = typeof navigator === "undefined" ? "" : navigator.platform;

/** The label of an action revealing a file or folder in the platform's file manager. */
export const SHOW_IN_FILE_MANAGER = platform.startsWith("Mac")
  ? "Show in Finder"
  : platform.startsWith("Win")
    ? "Show in Explorer"
    : "Show in Files";
