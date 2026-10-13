/** Keys of the backend's timestamps (e.g. `createdAt`), serialized as ISO-8601 strings. */
const TIMESTAMP_KEY = /At$/;

const parseValue = (value: unknown, key?: string): unknown => {
  if (Array.isArray(value)) {
    return value.map((item) => parseValue(item));
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, parseValue(item, key)]),
    );
  }
  if (typeof value === "string" && key !== undefined && TIMESTAMP_KEY.test(key)) {
    return new Date(value);
  }
  return value;
};

/**
 * `response`, from the Rust backend, as typed in the frontend: timestamps (keys ending with "At") become `Date`s,
 * at any depth.
 */
export const parseResponse = <T>(response: unknown): T => parseValue(response) as T;
