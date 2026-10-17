import type {
  NotionColumnKind,
  NotionDoneRule,
  NotionSchemaColumn,
} from "./types";

const KIND_NAMES: Record<NotionColumnKind, string> = {
  status: "Status",
  select: "Select",
  multi_select: "Multi-select",
  checkbox: "Checkbox",
};

/** What kind of column it is, e.g. "Multi-select". */
export const columnKindName = (kind: NotionColumnKind): string =>
  KIND_NAMES[kind];

/** E.g. "Released, Won't do or Archived". */
const either = (names: readonly string[]): string =>
  names.length > 1
    ? `${names.slice(0, -1).join(", ")} or ${names[names.length - 1]}`
    : (names[0] ?? "");

/** When a database's pages are done, as in "Done when …": e.g. "Stage is Released or Won't do". */
export const describeRule = (
  rule: Pick<NotionDoneRule, "propertyName" | "propertyKind" | "doneOptions">,
): string => {
  if (rule.propertyKind === "checkbox")
    return `${rule.propertyName} is checked`;
  const names = either(rule.doneOptions.map((option) => option.name));
  return rule.propertyKind === "multi_select"
    ? `${rule.propertyName} has ${names}`
    : `${rule.propertyName} is ${names}`;
};

/** When a database's pages are done without a rule: as Notion's Status groups tell. */
export const DEFAULT_RULE = "its Status is in its Complete group";

/**
 * The column to pick first when editing a database's rule: its rule's, if still there, else its Status, else its
 * first one.
 */
export const preselectedColumn = (
  columns: readonly NotionSchemaColumn[],
  rule?: Pick<NotionDoneRule, "propertyId">,
): NotionSchemaColumn | undefined =>
  columns.find((column) => column.propertyId === rule?.propertyId) ??
  columns.find((column) => column.kind === "status") ??
  columns[0];

/** The options of `column` meaning done, by ID, preselected when editing: `rule`'s still offered, if it's its column. */
export const preselectedOptions = (
  column: NotionSchemaColumn,
  rule?: Pick<NotionDoneRule, "propertyId" | "doneOptions">,
): string[] =>
  rule?.propertyId === column.propertyId
    ? rule.doneOptions
        .map((option) => option.id)
        .filter((id) => column.options.some((option) => option.id === id))
    : [];

/** The rule `column` with the options `optionIds` would set, if it's one: a checkbox needs none, others at least one. */
export const ruleFor = (
  dataSourceId: string,
  databaseTitle: string,
  column: NotionSchemaColumn,
  optionIds: readonly string[],
): NotionDoneRule | undefined => {
  const doneOptions = column.options
    .filter((option) => optionIds.includes(option.id))
    .map(({ id, name }) => ({ id, name }));
  if (column.kind !== "checkbox" && doneOptions.length === 0) return undefined;
  return {
    dataSourceId,
    databaseTitle,
    propertyId: column.propertyId,
    propertyName: column.name,
    propertyKind: column.kind,
    doneOptions: column.kind === "checkbox" ? [] : doneOptions,
  };
};
