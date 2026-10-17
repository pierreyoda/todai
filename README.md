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

## Notion links

Todos can also link to Notion pages, including the pages of a database, or to a whole database. Until the workspace is
connected to Notion, they open in your browser, titled from their URL.

Connect a workspace to Notion and its links also show each page's Status, in its Notion color, with a ring, dot or check
for its To-do, In progress or Complete group. A page is shown by its database's unique ID (e.g. `TOD-31`) if it has one,
else by its title. Click it for its database, tags, assignees, due date and when it was last edited. A todo whose page is
in its Complete group offers "mark as done", but is never completed on its own. Pages in Notion's trash, or out of the
token's reach, show "In trash" or "No access".

### Connecting

todai uses a Notion personal access token, one per workspace:

1. In Notion's Developer portal, under **Personal access tokens**, select **New token**: capability **Notion API**,
   the Notion workspace the links point to, and the expiration you like. Copy it: Notion shows it only once.
2. In todai, open **Settings › Integrations**, pick the workspace, then **Connect Notion** and paste the token.

On Free and Business plans only Notion workspace owners can create tokens, unless an owner allows members (Notion's
**Settings › Connections**). Guests can't.

The token acts as you: todai sees what you see in that Notion workspace. It's kept in your macOS Keychain (service
`todai.notion`, the workspace's ID as account), never in the workspace's database or its backups: a backup restored as a
new workspace starts unconnected, and removing a workspace deletes its token. When Notion refuses the token (expired or
revoked) or it's missing from the Keychain, the **Integrations** link in the side panel shows a dot: paste a new one.

### What todai does with Notion

- It only reads, though the token itself could write (Notion's token capability covers both): one request per page,
  plus one per database for its title and status groups, kept an hour. Notion allows each token 180 requests a minute
  (600 on Business and Enterprise plans); when it asks to wait, todai waits.
- Pages are refreshed as GitHub's links are: complete or trashed pages, and those of completed todos, once a day.
- What Notion tells (titles, statuses, assignees' names, due dates) is kept in the workspace's database, so links show
  their last status offline. It's therefore in the workspace's backups too.
- A Status property's groups are recognized by their English names (To-do, In progress, Complete), else by their
  position.

### When a page is done

By default, a page is done once its Status is in its Complete group. A database can use another column instead, e.g. a
Stage select whose Released and Won't do values mean done, or a Shipped checkbox: click one of its pages' links, then
**Change…** next to "Done when". Its pages then show that column's value, and offer "mark as done" once they hold one.
The rules are kept in the workspace (and its backups), one per database, and listed on the Notion card of
**Settings › Integrations**, where **Reset** goes back to Notion's Status. They follow renamed columns and values;
a rule whose column was deleted is ignored.

