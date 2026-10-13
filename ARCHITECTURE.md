# todai

## Frontend architecture

The frontend (`src/`) is organized by domain. `src/lib/architecture.test.ts` checks these rules at each `pnpm test`.

- `src/lib/shared/`: what knows nothing of todai's domains. `ui/` (UI kit, icons), `utils/` (toasts, autosave,
  timestamps…), `ipc/` (`createInvoker`, `queryClient`, `workspaceDataKey`, `MutationHooks`).
- `src/lib/core/<module>/`: building blocks. `time` (`Day`, `Month`, `Week`, the date picker), `markdown`, `github`.
- `src/lib/domains/<domain>/`: `tags`, `workspaces` (with `backups/`), `notes`, `todos` (with `links/`), `agenda`,
  `integrations`.
- `src/lib/shell/`: the app's frame, e.g. `SidePanel`, `ActiveWorkspaceGuard`.
- `src/routes/`: thin pages composing the above. The pages of the `(workspace)` group only show once the active
  workspace is usable.

### Layers

A layer never imports a higher one: `shared`, then `core` (importing only `shared`), then the domains, then `shell`, then
`routes`. A domain only imports the lower domains listed in `DOMAINS` of the architecture test: `notes` → `workspaces`;
`todos` → `workspaces`, `tags`; `agenda` → `todos`, `notes`, `tags`; `integrations` → `workspaces`, `todos`.

### Imports

- Across modules, through the aliases of `package.json`: `#shared/<file, with its extension>`, `#core/<module>`,
  `#domains/<domain>`, `#shell/<file, with its extension>`. `#core/…` and `#domains/…` only reach the module's
  `index.ts`, its public surface: export something there to use it elsewhere.
- Within a module: relatively, leaving out `.ts`.

### Within a domain

- `model/`: types and pure rules, with their tests. No Svelte, TanStack Query or Tauri.
- `api/`: `commands.ts` (its backend commands, behind an `invoke…` from `createInvoker`), `queries.ts` (query keys,
  under `workspaceDataKey` when the data comes from the active workspace's database, and query options),
  `mutations.ts`.
- `state/`: client state, in `*.svelte.ts`.
- `ui/`: components. They read through queries, but never call `createMutation`, `useQueryClient` or an `invoke…`.

### Mutations

Each mutation is a `createXxxMutation` factory in its domain's `api/mutations.ts` ("Add" for a creation). It owns the
backend call, the cache's refresh or update, toasts and undo. A component passes its own reactions (e.g. closing its
modal) as hooks, `{ onSuccess, onError }` (see `MutationHooks`): the factory calls them first, before returning the
cache's refresh. A `mutate(..., { onSuccess })` callback would only run once the cache is refetched, and not at all once
the component is gone.

### Editor

The aliases of `package.json` are mirrored in the `paths` of `tsconfig.json` (the architecture test checks it): change
both together. The Svelte language server doesn't reread `package.json`, only `tsconfig.json`, so without them it would
keep reporting the imports as missing (e.g. after switching branches) until restarted.
