# todai

A personal, day-by-day tasks-management and notes-taking application.

## GitHub links

Todos can link to GitHub pull requests and issues, and to any other web page. Links work on their own: they open in
your browser, shown as `repo#number` (e.g. `todai#58`) or by their domain.

Connect a workspace to GitHub and its links also show where each one stands: open, draft, merged or closed, with its
checks and review for a pull request, and "Completed" or "Not planned" for an issue. Click a link for its details. A
todo whose pull request is merged (or issue completed) offers "mark as done", but is never completed on its own.

### Connecting

todai goes through the [GitHub CLI](https://cli.github.com) (`gh`), and stores no token of its own:

1. Install it, e.g. `brew install gh`, then sign in with `gh auth login`.
2. In todai, open **Settings › Integrations**, pick the workspace, then **Connect GitHub** and one of `gh`'s accounts.

Each workspace has its own connection: one can use your own account, another your company's. todai only remembers which
account a workspace uses, and asks `gh` for its token when needed (kept in memory only). When todai finds `gh` missing,
or signed out of that account, the **Integrations** link in the side panel shows a dot: sign in again with
`gh auth login`, then **Check again**.

### What todai does with GitHub

- It only reads, through GitHub's GraphQL API: up to 50 pull requests and issues per request, which costs 1 or 2 of the
  5,000 points GitHub allows per hour (shown on the GitHub card).
- While todai is open, the day's links are refreshed every 10 minutes and when you come back to it: open ones once
  fetched more than 5 minutes ago, merged or closed ones (or those of completed todos) once a day. The calendar never
  fetches. **Behavior** settings, per workspace, can turn this off (links are then fetched when added, or refreshed from
  their details), along with "mark as done" and creating a todo from a pasted link.
- What GitHub tells is kept in the workspace's database, so links show their last status offline. It's therefore in the
  workspace's backups too, titles of private pull requests and issues included, as the todos' titles are. The
  connection itself isn't: a backup restored as a new workspace starts unconnected.
