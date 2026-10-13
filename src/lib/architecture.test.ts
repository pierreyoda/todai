import { readdirSync, readFileSync } from "node:fs";
import { join, posix, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

/*
 * The frontend's architecture, checked on its imports. `src/lib` holds its modules, by layer:
 * - `shared`: what knows nothing of todai's domains (UI kit, toasts, IPC), importing no other module;
 * - `core/<module>`: todai's building blocks (days, Markdown, GitHub), importing only `shared`;
 * - `domains/<domain>`: its business areas, importing `shared`, `core`, and the lower domains listed in `DOMAINS`;
 * - `shell`: the app's frame (side panel, workspace guard), importing any of them;
 * then `src/routes` composes them. Other folders of `src` hold no source files.
 *
 * Across modules, imports go through the aliases of `package.json`: `#core/<module>` and `#domains/<domain>` only reach
 * their `index.ts`, their public surface. Within a module, they're relative.
 *
 * Within a module:
 * - `model/` holds pure rules, free of Svelte, TanStack Query and Tauri;
 * - `api/` alone calls the backend and writes to the cache: components and routes use its queries and mutations.
 */

/** Each domain, with the other domains it may import: none of them may import it back. */
const DOMAINS: Readonly<Record<string, readonly string[]>> = {
  tags: [],
  workspaces: [],
  notes: ["workspaces"],
  todos: ["workspaces", "tags"],
  agenda: ["todos", "notes", "tags"],
  integrations: ["workspaces", "todos"],
};

/** For `api/` only, along with the `invoke…` functions of its commands. */
const API_ONLY_IMPORTS = new Set(["createMutation", "useQueryClient"]);

const FRAMEWORK_PACKAGE = /^(svelte($|\/)|@tanstack\/|@tauri-apps\/)/;

/** A file of `src`, at `path` relative to it (e.g. `lib/domains/todos/ui/TodoPanel.svelte`). */
type SourceFile = { path: string; source: string };

type Import = { specifier: string; names: string[] };

/**
 * The module of `path` (relative to `src`): `shared`, `core/<module>`, `domains/<domain>` or `shell` in `src/lib`, else
 * `routes`; `null` for any other file (e.g. `app.css`, or one of `src/lib` outside of the modules).
 */
const moduleOf = (path: string): string | null => {
  const [root, layer, name, ...rest] = path.split("/");
  if (root === "routes") return "routes";
  if (root !== "lib" || name === undefined) return null;
  if (layer === "shared" || layer === "shell") return layer;
  if ((layer === "core" || layer === "domains") && rest.length > 0)
    return `${layer}/${name}`;
  return null;
};

/** Whether module `from` may import module `to`, another one: only from a lower layer. */
const mayImport = (from: string, to: string | null): boolean => {
  if (to === null || to === "routes" || from === "shared") return false;
  if (to === "shared" || from === "shell") return to !== "shell";
  if (from.startsWith("core/")) return false;
  // From a domain
  if (to.startsWith("core/")) return true;
  if (to.startsWith("domains/")) {
    return (
      DOMAINS[from.slice("domains/".length)]?.includes(
        to.slice("domains/".length),
      ) ?? false
    );
  }
  return false;
};

/**
 * What `specifier`, imported from `path`, reaches: a module (or `null` for another file of `src`), `undefined` for a
 * package (or a SvelteKit module, e.g. `$app/state`), or why it can't be imported so.
 */
const targetOf = (
  path: string,
  specifier: string,
):
  | { module: string | null; alias: boolean; resolved?: string }
  | { error: string }
  | undefined => {
  const bare = specifier.replace(/\?.*$/, "");
  if (bare.startsWith(".")) {
    const resolved = posix.normalize(posix.join(posix.dirname(path), bare));
    return {
      module: resolved.startsWith("../") ? null : moduleOf(resolved),
      alias: false,
      resolved,
    };
  }
  if (!bare.startsWith("#")) return undefined;
  const [alias, name, ...deeper] = bare.slice(1).split("/");
  if (alias === "shared" || alias === "shell")
    return { module: alias, alias: true };
  if (alias !== "core" && alias !== "domains")
    return { error: "an unknown alias" };
  if (name === undefined || deeper.length > 0)
    return { error: `past its index.ts: import "#${alias}/${name}"` };
  return { module: `${alias}/${name}`, alias: true };
};

/** The code of `source`, without comments: only its `<script>` blocks for a component. */
const codeOf = ({ path, source }: SourceFile): string =>
  (path.endsWith(".svelte")
    ? [...source.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)]
        .map(([, script]) => script)
        .join("\n")
    : source
  )
    .replace(/\/\*[\s\S]*?\*\//g, "")
    // Not the slashes of URLs, nor of strings
    .replace(/(^|[^:"'`\\])\/\/.*$/gm, "$1");

/** The imports (and re-exports) of `code`, with the names they bring in when listed. */
const importsOf = (code: string): Import[] => [
  ...[
    ...code.matchAll(
      /\b(?:import|export)\s+([^;]*?)\s*from\s*["']([^"']+)["']/g,
    ),
  ].map(([, clause, specifier]) => ({
    specifier,
    names:
      clause
        .match(/\{([^}]*)\}/)?.[1]
        .split(",")
        .map(
          (name) =>
            name
              .trim()
              .replace(/^type\s+/, "")
              .split(/\s+as\s+/)[0],
        )
        .filter(Boolean) ?? [],
  })),
  // For their side effects (e.g. a stylesheet), or dynamic
  ...[...code.matchAll(/\bimport\s*\(?\s*["']([^"']+)["']/g)].map(
    ([, specifier]) => ({ specifier, names: [] }),
  ),
];

/** How the imports of `file` break the architecture, if they do. */
const violationsOf = (file: SourceFile): string[] => {
  const { path } = file;
  const from = moduleOf(path);
  const violations: string[] = [];
  const flag = (reason: string) => violations.push(`src/${path}: ${reason}`);

  // Unless right in `src`, like SvelteKit's own files (e.g. `app.d.ts`)
  if (from === null && path.startsWith("lib/"))
    flag("outside of shared, core/<module>, domains/<domain> and shell");
  else if (from === null && path.includes("/"))
    flag("outside of src/lib and src/routes");
  if (from === null) return violations;
  const domain = from.startsWith("domains/")
    ? from.slice("domains/".length)
    : undefined;
  if (domain !== undefined && !(domain in DOMAINS))
    flag(`an unknown domain: add "${domain}" to DOMAINS`);

  const folders = path.split("/").slice(0, -1);
  const inModel = folders.includes("model");
  const inComponents = folders.includes("ui") || from === "routes";
  if (inModel && /\.svelte(\.ts)?$/.test(path))
    flag("a Svelte file in model/, which holds pure rules");

  for (const { specifier, names } of importsOf(codeOf(file))) {
    const imports = `imports "${specifier}"`;
    if (inComponents) {
      const apiOnly = names.filter(
        (name) => API_ONLY_IMPORTS.has(name) || name.startsWith("invoke"),
      );
      if (apiOnly.length > 0)
        flag(
          `${imports}: ${apiOnly.join(", ")}, for api/ only: use its queries and mutations`,
        );
    }
    if (inModel && FRAMEWORK_PACKAGE.test(specifier))
      flag(`${imports}: a framework in model/, which holds pure rules`);

    const target = targetOf(path, specifier);
    if (target === undefined) continue;
    if ("error" in target) {
      flag(`${imports}: ${target.error}`);
      continue;
    }
    const to = target.module;
    if (to === from) {
      if (
        target.alias &&
        (from.startsWith("core/") || from.startsWith("domains/"))
      ) {
        flag(
          `${imports}: its own module's index.ts: import its files relatively`,
        );
      }
      if (inModel && /\/(api|state|ui)\//.test(`${target.resolved}/`)) {
        flag(`${imports}: model/ doesn't depend on the rest of its module`);
      }
    } else if (from !== "routes" && !mayImport(from, to)) {
      flag(
        `${imports}: ${from} may not depend on ${to ?? "files outside of the modules"}`,
      );
    } else if (!target.alias && to !== null) {
      flag(
        `${imports}: another module, relatively: import it through its alias (#shared/…, #core/…, #domains/…)`,
      );
    }
  }
  return violations;
};

describe("the architecture's rules", () => {
  const svelte = (script: string) =>
    `<script lang="ts">\n${script}\n</script>\n\n<p>Some markup</p>\n`;

  it.each([
    [
      "a domain to import a lower one, through its index.ts",
      "lib/domains/todos/ui/TodoLineItem.svelte",
      svelte(`import { TagUpsertModal } from "#domains/tags";`),
    ],
    [
      "a domain to import shared files and core modules",
      "lib/domains/notes/ui/NotePanel.svelte",
      svelte(
        `import Modal from "#shared/ui/Modal.svelte";\nimport { formatFullDay } from "#core/time";`,
      ),
    ],
    [
      "a module's files to import each other relatively",
      "lib/domains/todos/api/mutations.ts",
      `import { todoKeys } from "./queries";\nimport type { Todo } from "../model/types";`,
    ],
    [
      "components to read through queries",
      "lib/domains/tags/ui/TagList.svelte",
      svelte(
        `import { createQuery } from "@tanstack/svelte-query";\nimport { tagsQueryOptions } from "../api/queries";`,
      ),
    ],
    [
      "api/ to call the backend and write to the cache",
      "lib/domains/tags/api/mutations.ts",
      `import { createMutation, useQueryClient } from "@tanstack/svelte-query";\nimport { invokeTags } from "./commands";`,
    ],
    [
      "the shell to import domains",
      "lib/shell/SidePanel.svelte",
      svelte(
        `import { page } from "$app/state";\nimport { NoteMarker } from "#domains/notes";`,
      ),
    ],
    [
      "routes to import their own files relatively, and modules through their aliases",
      "routes/(workspace)/calendar/notes/[day]/+page.svelte",
      svelte(
        `import "../../../../../app.css";\nimport type { PageProps } from "./$types";\n` +
          `import { NotePanel } from "#domains/notes";\nimport SidePanel from "#shell/SidePanel.svelte";`,
      ),
    ],
    ["SvelteKit's own files, right in src", "app.d.ts", "declare global {}"],
    [
      "commented out imports",
      "lib/shared/utils/toasts.svelte.ts",
      `// import type { Day } from "../../../client/types";\n/* import { addDays } from "#core/time"; */\nexport const x = 1;`,
    ],
  ])("allows %s", (_, path, source) => {
    expect(violationsOf({ path, source })).toEqual([]);
  });

  it.each([
    [
      "a domain importing a higher one",
      "lib/domains/tags/ui/TagForm.svelte",
      svelte(`import { TodoPanel } from "#domains/todos";`),
    ],
    [
      "a domain importing one it doesn't list",
      "lib/domains/notes/api/queries.ts",
      `import { todoKeys } from "#domains/todos";`,
    ],
    [
      "a domain's files reached through its alias",
      "lib/domains/agenda/ui/CalendarWeek.svelte",
      svelte(`import { todoKeys } from "#domains/todos/api/queries.ts";`),
    ],
    [
      "another module imported relatively",
      "lib/domains/agenda/ui/CalendarWeek.svelte",
      svelte(
        `import TodoSummaryItem from "../../todos/ui/TodoSummaryItem.svelte";`,
      ),
    ],
    [
      "a module importing its own index.ts",
      "lib/domains/todos/ui/TodoPanel.svelte",
      svelte(`import { todoKeys } from "#domains/todos";`),
    ],
    [
      "src/lib importing files outside of its modules",
      "lib/core/time/days.ts",
      `import type { Day } from "../../../client/types";`,
    ],
    [
      "a source file outside of src/lib and src/routes",
      "utils/dates.ts",
      "export const x = 1;",
    ],
    [
      "shared importing a module",
      "lib/shared/ui/DatePicker.svelte",
      svelte(`import { addDays } from "#core/time";`),
    ],
    [
      "a core module importing another one",
      "lib/core/markdown/model/editing.ts",
      `import { addDays } from "#core/time";`,
    ],
    [
      "a domain importing the shell",
      "lib/domains/notes/ui/NotePanel.svelte",
      svelte(`import SidePanel from "#shell/SidePanel.svelte";`),
    ],
    [
      "routes reaching into src/lib relatively",
      "routes/calendar/+page.svelte",
      svelte(
        `import VerticalCalendar from "../../lib/domains/agenda/ui/VerticalCalendar.svelte";`,
      ),
    ],
    [
      "a component creating a mutation",
      "lib/domains/tags/ui/TagForm.svelte",
      svelte(
        `import {\n  createQuery,\n  createMutation,\n} from "@tanstack/svelte-query";`,
      ),
    ],
    [
      "a component calling the backend",
      "lib/domains/tags/ui/TagForm.svelte",
      svelte(`import { invokeTags } from "../api/commands";`),
    ],
    [
      "a route writing to the cache",
      "routes/+page.svelte",
      svelte(`import { useQueryClient } from "@tanstack/svelte-query";`),
    ],
    [
      "model/ importing a framework",
      "lib/domains/todos/model/order.ts",
      `import { untrack } from "svelte";`,
    ],
    [
      "model/ importing the rest of its module",
      "lib/domains/todos/model/order.ts",
      `import { todoKeys } from "../api/queries";`,
    ],
    [
      "a Svelte file in model/",
      "lib/domains/notes/model/drafts.svelte.ts",
      `export const drafts = $state({});`,
    ],
    ["an unknown domain", "lib/domains/trash/ui/TrashList.svelte", svelte("")],
    [
      "a file of src/lib outside of the modules",
      "lib/helpers.ts",
      "export const x = 1;",
    ],
    [
      "an unknown alias",
      "lib/shared/ui/Button.svelte",
      svelte(`import { x } from "#lib/foo.ts";`),
    ],
  ])("flags %s", (_, path, source) => {
    expect(violationsOf({ path, source })).toHaveLength(1);
  });
});

describe("the frontend's architecture", () => {
  const SRC = fileURLToPath(new URL("..", import.meta.url));
  const SELF = "lib/architecture.test.ts";

  it("follows its rules", () => {
    const files = readdirSync(SRC, { recursive: true, encoding: "utf8" })
      .map((path) => path.split(sep).join("/"))
      .filter((path) => /\.(ts|js|svelte)$/.test(path) && path !== SELF)
      .map((path) => ({ path, source: readFileSync(join(SRC, path), "utf8") }));
    expect(files.length).toBeGreaterThan(0);
    expect(files.flatMap(violationsOf)).toEqual([]);
  });

  // The editor's Svelte language server only rereads `tsconfig.json`, which takes precedence over `package.json`
  it("mirrors its aliases in tsconfig.json", () => {
    const read = (file: string) =>
      JSON.parse(readFileSync(join(SRC, "..", file), "utf8"));
    const imports: Record<string, string> = read("package.json").imports;
    const paths: Record<string, string[]> =
      read("tsconfig.json").compilerOptions.paths;
    expect(paths).toEqual(
      Object.fromEntries(
        Object.entries(imports).map(([alias, target]) => [alias, [target]]),
      ),
    );
  });
});
