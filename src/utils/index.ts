export * from "./dates";
export * from "./estimates";
export * from "./files";
export * from "./paths";
export * from "./platform";

export const isDefined = <T>(value: T | null | undefined): value is T =>
  value !== null && value !== undefined;
