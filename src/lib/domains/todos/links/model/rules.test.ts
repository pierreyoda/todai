import { describe, expect, it } from "vitest";

import {
  describeRule,
  preselectedColumn,
  preselectedOptions,
  ruleFor,
} from "./rules";
import type { NotionSchemaColumn } from "./types";

const stage: NotionSchemaColumn = {
  propertyId: "st",
  name: "Stage",
  kind: "select",
  options: [
    { id: "dev", name: "Dev", color: "blue" },
    { id: "released", name: "Released", color: "green" },
    { id: "wontdo", name: "Won't do", color: "gray" },
  ],
};
const status: NotionSchemaColumn = {
  propertyId: "s",
  name: "Status",
  kind: "status",
  options: [{ id: "done", name: "Done", color: "green" }],
};
const shipped: NotionSchemaColumn = {
  propertyId: "sh",
  name: "Shipped",
  kind: "checkbox",
  options: [],
};

describe("describeRule", () => {
  it("tells when a database's pages are done", () => {
    expect(
      describeRule({
        propertyName: "Stage",
        propertyKind: "select",
        doneOptions: [
          { id: "released", name: "Released" },
          { id: "wontdo", name: "Won't do" },
          { id: "archived", name: "Archived" },
        ],
      }),
    ).toBe("Stage is Released, Won't do or Archived");
    expect(
      describeRule({
        propertyName: "Tags",
        propertyKind: "multi_select",
        doneOptions: [{ id: "x", name: "shipped" }],
      }),
    ).toBe("Tags has shipped");
    expect(
      describeRule({
        propertyName: "Shipped",
        propertyKind: "checkbox",
        doneOptions: [],
      }),
    ).toBe("Shipped is checked");
  });
});

describe("preselectedColumn", () => {
  it("is the rule's column, else the Status, else the first one", () => {
    const columns = [shipped, stage, status];
    expect(preselectedColumn(columns, { propertyId: "st" })).toBe(stage);
    expect(preselectedColumn(columns, { propertyId: "gone" })).toBe(status);
    expect(preselectedColumn(columns)).toBe(status);
    expect(preselectedColumn([shipped])).toBe(shipped);
    expect(preselectedColumn([])).toBeUndefined();
  });
});

describe("preselectedOptions", () => {
  it("keeps the rule's options its column still offers", () => {
    const rule = {
      propertyId: "st",
      doneOptions: [
        { id: "released", name: "Released" },
        { id: "deleted", name: "Deleted since" },
      ],
    };
    expect(preselectedOptions(stage, rule)).toEqual(["released"]);
    expect(preselectedOptions(status, rule)).toEqual([]);
    expect(preselectedOptions(stage)).toEqual([]);
  });
});

describe("ruleFor", () => {
  it("needs a value meaning done, unless it's a checkbox", () => {
    expect(ruleFor("ds1", "Tickets", stage, [])).toBeUndefined();
    expect(ruleFor("ds1", "Tickets", stage, ["released", "unknown"])).toEqual({
      dataSourceId: "ds1",
      databaseTitle: "Tickets",
      propertyId: "st",
      propertyName: "Stage",
      propertyKind: "select",
      doneOptions: [{ id: "released", name: "Released" }],
    });
    expect(ruleFor("ds1", "Tickets", shipped, [])?.doneOptions).toEqual([]);
  });
});
