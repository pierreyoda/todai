export * from "./dates";
export * from "./estimates";
export * from "./paths";

export const isDefined = <T>(value: T | null | undefined): value is T =>
  value !== null && value !== undefined;
