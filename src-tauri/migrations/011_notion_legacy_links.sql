-- Notion links added before Notion was recognized, kept as plain URLs: now Notion links where their URL, on a Notion
-- host, ends with the page's ID after its title (`…/Title-<ID>`) or alone (`…/<ID>`). Their page is then fetched as a
-- new link's is, and their URL replaced by Notion's own. Any other stays a plain URL, as does one whose page the todo
-- already links to.
UPDATE OR IGNORE todo_links
SET
  kind = 'notion',
  notion_id = lower(substr(url, -32))
WHERE
  id IN (
    SELECT
      id
    FROM
      (
        SELECT
          id,
          url,
          substr(url, 9, instr(substr(url, 9), '/') - 1) AS host
        FROM
          todo_links
        WHERE
          kind = 'url'
          AND url GLOB 'https://*/*'
      )
    WHERE
      (
        host IN ('notion.so', 'www.notion.so', 'app.notion.com')
        OR (
          host GLOB '?*.notion.site'
          AND host NOT GLOB '*.*.notion.site'
        )
      )
      AND substr(url, -33, 1) IN ('/', '-')
      AND lower(substr(url, -32)) NOT GLOB '*[^0-9a-f]*'
  );
