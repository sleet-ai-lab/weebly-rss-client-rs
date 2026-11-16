//! Parses a single `<item>` element of a Weebly RSS feed.

use std::collections::BTreeMap;

use quick_xml::events::{BytesRef, Event};
use quick_xml::Reader;

use crate::error::WeeblyError;
use crate::result::Result;
use crate::types::{BlogPost, Category, PublishedDate};
use crate::util::{collapse_whitespace, extract_images, extract_links, html_to_text};

/// Parses a raw `<item>` XML fragment.
///
/// The fragment is the XML of a single `<item>` element, exactly as Weebly
/// writes it: a `<description>` holding a truncated plain-text summary and a
/// `<content:encoded>` holding the complete post body as HTML.
///
/// The fragment may be the inner XML of the element or the element itself,
/// both are accepted.
///
/// # Errors
///
/// Returns [`WeeblyError::Xml`] when the fragment is not well-formed XML.
///
/// ```
/// # use weebly_rss_client_rs::parse::parse_item::parse_item;
/// let raw = r#"<title>Hello</title><link>https://example.weebly.com/blog/hello</link>
/// <guid isPermaLink="false">https://example.weebly.com/blog/hello</guid>"#;
/// let post = parse_item(raw).expect("valid item").expect("a post with a guid");
/// assert_eq!(post.title, "Hello");
/// ```
pub fn parse_item(raw: &str) -> Result<Option<BlogPost>> {
    let fields = collect_fields(raw)?;

    // Weebly writes the permalink into both `<link>` and `<guid>`. A post
    // without either has no addressable identity and is skipped.
    let guid = match fields.get("guid").filter(|value| !value.is_empty()) {
        Some(value) => value.clone(),
        None => return Ok(None),
    };

    let link = fields
        .get("link")
        .filter(|value| !value.is_empty())
        .cloned()
        .unwrap_or_else(|| guid.clone());

    let title = fields.get("title").cloned().unwrap_or_default();

    let comments_url = fields
        .get("comments")
        .filter(|value| !value.is_empty())
        .cloned();

    let pub_date = fields
        .get("pubDate")
        .filter(|value| !value.is_empty())
        .and_then(|value| PublishedDate::parse(value).ok());

    let category = fields
        .get("category")
        .filter(|value| !value.is_empty())
        .map(|value| Category::new(value.clone()));

    let summary = fields
        .get("description")
        .map(|value| collapse_whitespace(value))
        .filter(|value| !value.is_empty());

    let content_html = fields
        .get("encoded")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let content_text = content_html
        .as_deref()
        .map(|html| collapse_whitespace(&html_to_text(html)))
        .filter(|value| !value.is_empty());

    let links = content_html
        .as_deref()
        .map(extract_links)
        .unwrap_or_default();
    let images = content_html
        .as_deref()
        .map(extract_images)
        .unwrap_or_default();

    Ok(Some(BlogPost {
        guid,
        title,
        link,
        comments_url,
        pub_date,
        category,
        summary,
        content_html,
        content_text,
        links,
        images,
    }))
}

/// Collects the text of every leaf element of the fragment, keyed by the
/// element's local name with any namespace prefix removed.
///
/// The fragment may or may not still be wrapped in its `<item>` element; both
/// the inner XML and the whole element are accepted.
fn collect_fields(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut reader = Reader::from_str(raw);
    reader.config_mut().trim_text(true);

    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut text = String::new();
    let mut depth = 0usize;
    // The depth at which the leaf elements of the fragment live. A wrapping
    // `<item>` element pushes them one level deeper.
    let mut base: Option<usize> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let name = local_name(event.name().as_ref());
                if base.is_none() {
                    // The first element decides where the leaves live.
                    base = Some(if name == "item" { 1 } else { 0 });
                }
                if depth == base.unwrap_or(0) {
                    current = Some(name);
                    text.clear();
                }
                depth += 1;
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
                if depth == base.unwrap_or(0) {
                    if let Some(key) = current.take() {
                        let value = decode_entities(text.trim());
                        fields.insert(key, value);
                    }
                    text.clear();
                }
            }
            Ok(Event::Empty(event)) => {
                if base.is_none() {
                    base = Some(0);
                }
                if depth == base.unwrap_or(0) {
                    let name = local_name(event.name().as_ref());
                    fields.insert(name, String::new());
                }
            }
            Ok(Event::CData(data)) => {
                if depth == base.unwrap_or(0) + 1 {
                    text.push_str(&String::from_utf8_lossy(data.as_ref()));
                }
            }
            Ok(Event::Text(value)) => {
                if depth == base.unwrap_or(0) + 1 {
                    let decoded = value.xml_content().unwrap_or_default().into_owned();
                    text.push_str(&decoded);
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                if depth == base.unwrap_or(0) + 1 {
                    text.push_str(&resolve_reference(&reference));
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(WeeblyError::Xml(error)),
            _ => {}
        }
    }

    Ok(fields)
}

/// Decodes HTML entities in a text field.
fn decode_entities(text: &str) -> String {
    html_escape::decode_html_entities(text).into_owned()
}

/// Resolves an XML entity or character reference such as `&#8203;` or `&amp;`.
///
/// `quick_xml` reports references as their own event instead of folding them
/// into the surrounding text, so they have to be resolved by hand.
fn resolve_reference(reference: &BytesRef<'_>) -> String {
    if let Ok(Some(ch)) = reference.resolve_char_ref() {
        return ch.to_string();
    }

    let raw = reference.decode().unwrap_or_default().into_owned();
    decode_entities(&raw)
}

/// Strips a namespace prefix from an element name.
fn local_name(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);
    match text.split_once(':') {
        Some((_, local)) => local.to_string(),
        None => text.into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POST_ITEM: &str = r#"<item><title><![CDATA[Hope Again Ladies Retreat]]></title><link><![CDATA[https://example.weebly.com/blog/hope-again]]></link><comments><![CDATA[https://example.weebly.com/blog/hope-again#comments]]></comments><pubDate>Sun, 03 Aug 2025 18:45:55 GMT</pubDate><category><![CDATA[Uncategorized]]></category><guid isPermaLink="false">https://example.weebly.com/blog/hope-again</guid><description><![CDATA[Hey, it&#8217;s been a long while since I posted on my blog. [...] ]]></description><content:encoded><![CDATA[<div class="paragraph">&nbsp;Hey, it&#8217;s been a long while since I posted on my blog.<br />&nbsp;I continue having amazing guests on my podcast.</div>]]></content:encoded></item>"#;

    #[test]
    fn parses_a_post_item() {
        let post = parse_item(POST_ITEM)
            .expect("valid xml")
            .expect("a post with a guid");

        assert_eq!(post.title, "Hope Again Ladies Retreat");
        assert_eq!(post.link, "https://example.weebly.com/blog/hope-again");
        assert_eq!(post.guid, "https://example.weebly.com/blog/hope-again");
        assert_eq!(
            post.comments_url.as_deref(),
            Some("https://example.weebly.com/blog/hope-again#comments")
        );
        assert_eq!(post.pub_date.map(|date| date.year()), Some(2025));
        assert_eq!(
            post.category
                .as_ref()
                .map(|category| category.name.as_str()),
            Some("Uncategorized")
        );
        assert!(post.is_summary_truncated());
        assert_eq!(post.slug(), "hope-again");
    }

    #[test]
    fn decodes_entities_in_the_summary() {
        let post = parse_item(POST_ITEM)
            .expect("valid xml")
            .expect("a post with a guid");

        let summary = post.summary.expect("a summary");
        assert!(summary.contains("it’s been a long while"));
        assert!(!summary.contains("&nbsp;"));
        assert!(!summary.contains("&#8217;"));
    }

    #[test]
    fn converts_the_content_html_into_text() {
        let post = parse_item(POST_ITEM)
            .expect("valid xml")
            .expect("a post with a guid");

        let text = post.content_text.expect("some text");
        assert!(text.contains("Hey, it’s been a long while"));
        assert!(text.contains("I continue having amazing guests"));
        assert!(!text.contains("<div"));
        assert!(!text.contains("&nbsp;"));
        assert!(!text.contains('\u{a0}'));
    }
    #[test]
    fn returns_none_without_a_guid() {
        let raw = r#"<title>Something</title><link>https://example.weebly.com/blog/x</link>"#;
        let post = parse_item(raw).expect("valid xml");
        assert!(post.is_none());
    }

    #[test]
    fn accepts_a_fragment_without_the_wrapping_item() {
        let raw = r#"<title>Hello</title><link>https://example.weebly.com/blog/hello</link><guid isPermaLink="false">https://example.weebly.com/blog/hello</guid>"#;
        let post = parse_item(raw).expect("valid xml").expect("a post");
        assert_eq!(post.title, "Hello");
    }

    #[test]
    fn extracts_links_and_images() {
        let raw = r#"<title>Media</title><link>https://example.weebly.com/blog/media</link><guid isPermaLink="false">https://example.weebly.com/blog/media</guid><content:encoded><![CDATA[<div class="wsite-image"><a><img src="https://example.weebly.com/uploads/1/a.png" alt="Picture" /></a></div><div class="paragraph"><a href="https://example.com">Example</a></div>]]></content:encoded>"#;
        let post = parse_item(raw).expect("valid xml").expect("a post");

        assert_eq!(post.images.len(), 1);
        assert_eq!(
            post.images[0].url,
            "https://example.weebly.com/uploads/1/a.png"
        );
        assert_eq!(post.images[0].alt.as_deref(), Some("Picture"));

        let urls: Vec<&str> = post.links.iter().map(|link| link.url.as_str()).collect();
        assert!(urls.contains(&"https://example.com"));
        assert!(urls.contains(&"https://example.weebly.com/uploads/1/a.png"));

        let example = post
            .links
            .iter()
            .find(|link| link.url == "https://example.com")
            .expect("the outbound link");
        assert_eq!(example.text.as_deref(), Some("Example"));
    }
}
