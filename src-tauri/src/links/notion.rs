//! Notion pages (and whole databases), as recognized from their URL.

use serde::{Deserialize, Serialize};
use url::Url;

use super::{LinkKind, ParsedLink};

/// A page, or a database, in Notion.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct NotionRef {
    /// Its ID: 32 lowercase hexadecimal digits, as in its URL.
    pub id: String,
}

/// What a page is called when its URL has no slug and it wasn't fetched yet.
pub const UNTITLED: &str = "Notion page";

/// The page or database `url` points to, if any, as a link: on `notion.so`, `app.notion.com`, or a published
/// `*.notion.site`, by the ID ending its path, or the one of the page shown over a database (`?p=`).
pub fn recognize(url: &Url) -> Option<ParsedLink> {
    if !is_notion_host(url.host_str()?) {
        return None;
    }
    // A page opened over another one (or a database): the one shown
    let peeked = url
        .query_pairs()
        .find(|(name, _)| name == "p")
        .and_then(|(_, value)| page_id(&value).map(|id| (id, None)));
    let (id, slug) = match peeked {
        Some(peeked) => peeked,
        None => {
            let segment = url.path_segments()?.rfind(|segment| !segment.is_empty())?;
            let (slug, id) = split_slug(segment)?;
            (id, slug)
        }
    };
    Some(ParsedLink {
        kind: LinkKind::Notion,
        url: self::url(&id, slug),
        label: slug.map_or_else(|| UNTITLED.into(), slug_title),
        github: None,
        notion: Some(NotionRef { id }),
    })
}

fn is_notion_host(host: &str) -> bool {
    matches!(host, "notion.so" | "www.notion.so" | "app.notion.com")
        || host
            .strip_suffix(".notion.site")
            .is_some_and(|workspace| !workspace.is_empty() && !workspace.contains('.'))
}

/// `Slug-<id>` as `(Some(Slug), id)`, or `<id>` as `(None, id)`: the ID being 32 hex digits, dashed as a UUID or not.
fn split_slug(segment: &str) -> Option<(Option<&str>, String)> {
    let dashed = segment.len() >= 36 && segment.as_bytes()[segment.len() - 36] != b'-'
        || segment.len() == 36;
    let (slug, raw) = if dashed && page_id(&segment[segment.len() - 36..]).is_some() {
        (
            &segment[..segment.len() - 36],
            &segment[segment.len() - 36..],
        )
    } else if segment.len() >= 32 {
        (
            &segment[..segment.len() - 32],
            &segment[segment.len() - 32..],
        )
    } else {
        return None;
    };
    let id = page_id(raw)?;
    let slug = slug.strip_suffix('-').filter(|slug| !slug.is_empty());
    Some((slug, id))
}

/// `raw` as a page ID, if it's one: 32 hex digits, dashed as a UUID or not, in any case.
fn page_id(raw: &str) -> Option<String> {
    let id: String = raw
        .chars()
        .filter(|c| *c != '-')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    (id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()) && raw.len() <= 36).then_some(id)
}

/// The normalized URL of the page `id`, with its `slug` if known: Notion's own form, without the workspace.
pub fn url(id: &str, slug: Option<&str>) -> String {
    match slug {
        Some(slug) => format!("https://www.notion.so/{slug}-{id}"),
        None => format!("https://www.notion.so/{id}"),
    }
}

/// A page's title as its URL's slug tells it, e.g. "Define the 0.3 scope" from `Define-the-0-3-scope`: dots and other
/// punctuation are lost, so it's only shown until the page is fetched.
pub fn slug_title(slug: &str) -> String {
    slug.replace('-', " ")
}

/// A page's label from its normalized `url`, as `recognize` gave it.
pub fn label_from_url(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|url| recognize(&url))
        .map_or_else(|| UNTITLED.into(), |link| link.label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::links::parse;

    const ID: &str = "8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5";

    #[test]
    fn recognizes_pages_on_every_notion_host() {
        for input in [
            "https://www.notion.so/acme/Define-the-0-3-scope-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
            "https://notion.so/Define-the-0-3-scope-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5?pvs=4#block",
            "https://app.notion.com/p/Define-the-0-3-scope-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
            "https://acme.notion.site/Define-the-0-3-scope-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
            "https://www.notion.so/acme/Define-the-0-3-scope-8C1D2E3F-4A5B-46C7-D8E9-F0A1B2C3D4E5",
            "notion.so/Define-the-0-3-scope-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
        ] {
            assert_eq!(
                parse(input).unwrap(),
                ParsedLink {
                    kind: LinkKind::Notion,
                    url: format!("https://www.notion.so/Define-the-0-3-scope-{ID}"),
                    label: "Define the 0 3 scope".into(),
                    github: None,
                    notion: Some(NotionRef { id: ID.into() }),
                },
                "{input:?}"
            );
        }
    }

    #[test]
    fn pages_without_a_slug_are_untitled_until_fetched() {
        let link = parse(&format!("https://www.notion.so/{ID}")).unwrap();
        assert_eq!(link.url, format!("https://www.notion.so/{ID}"));
        assert_eq!(link.label, UNTITLED);
        assert_eq!(link.notion.unwrap().id, ID);
        assert_eq!(label_from_url(&link.url), UNTITLED);
        assert_eq!(
            label_from_url(&format!("https://www.notion.so/Roadmap-{ID}")),
            "Roadmap"
        );
    }

    #[test]
    fn a_page_shown_over_a_database_is_the_page() {
        let link = parse(&format!(
            "https://www.notion.so/acme/Tickets-2b7c9d0e1f2a43b5c6d7e8f9a0b1c2d3?v=4e5f6a7b8c9d40e1f2a3b4c5d6e7f8a9&p={ID}&pm=s"
        ))
        .unwrap();
        assert_eq!(link.notion.unwrap().id, ID);
        assert_eq!(link.label, UNTITLED);
        // A database view: the database itself
        let database = parse(
            "https://www.notion.so/acme/2b7c9d0e1f2a43b5c6d7e8f9a0b1c2d3?v=4e5f6a7b8c9d40e1f2a3b4c5d6e7f8a9",
        )
        .unwrap();
        assert_eq!(
            database.notion.unwrap().id,
            "2b7c9d0e1f2a43b5c6d7e8f9a0b1c2d3"
        );
    }

    #[test]
    fn other_notion_pages_are_plain_links() {
        for input in [
            "https://www.notion.so/",
            "https://www.notion.so/acme",
            "https://www.notion.so/product",
            "https://www.notion.so/help/create-integrations-with-the-notion-api",
            "https://developers.notion.com/reference/page",
            "https://www.notion.so/acme/Not-an-id-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4g5",
            "https://evil.example.com/x.notion.site/Page-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
        ] {
            let link = parse(input).unwrap();
            assert_eq!(link.kind, LinkKind::Url, "{input:?}");
            assert_eq!(link.notion, None, "{input:?}");
        }
    }
}
