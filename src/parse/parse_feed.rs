//! Parses a Weebly RSS document into a [`WeeblyFeed`].

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::WeeblyError;
use crate::parse::parse_item::parse_item;
use crate::result::Result;
use crate::types::{BlogPost, PublishedDate, WeeblyFeed};

/// Parses a Weebly RSS document into a [`WeeblyFeed`].
///
/// # Errors
///
/// Returns [`WeeblyError::Xml`] when the document is not well-formed XML, and
/// [`WeeblyError::MalformedFeed`] when it is not an RSS document with a
/// `<channel>`.
///
/// ```
/// # use weebly_rss_client_rs::parse::parse_feed;
/// let xml = r#"<rss version="2.0"><channel><title>Example - Blog</title>
/// <link>https://example.weebly.com/blog</link><generator>Weebly</generator></channel></rss>"#;
/// let feed = parse_feed(xml).expect("valid feed");
/// assert_eq!(feed.site_name(), "Example");
/// assert_eq!(feed.blog_name(), "Blog");
/// ```
pub fn parse_feed(xml: &str) -> Result<WeeblyFeed> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut title = String::new();
    let mut link = String::new();
    let mut description = None;
    let mut pub_date = None;
    let mut generator = None;
    let mut items: Vec<BlogPost> = Vec::new();

    // The name of the channel-level element currently being read, if any.
    let mut current: Option<String> = None;
    let mut current_text = String::new();

    // Depth of the element nesting, used to recognise the direct children of
    // `<channel>`, which are the elements holding the channel's metadata.
    let mut depth = 0usize;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let raw_name = event.name();
                let name = local_name(raw_name.as_ref());

                if name == "item" {
                    // Capture the raw XML of the item so that it can be handed
                    // to `parse_item` unchanged.
                    let span = reader.read_to_end(raw_name).map_err(WeeblyError::Xml)?;
                    let bytes = xml.as_bytes();
                    let start = (span.start as usize).min(bytes.len());
                    let end = (span.end as usize).min(bytes.len());
                    let raw = String::from_utf8_lossy(&bytes[start..end]);
                    if let Some(item) = parse_item(&raw)? {
                        items.push(item);
                    }
                    continue;
                }

                // Channel-level elements are the direct children of `<channel>`;
                // their own children are not channel fields.
                if depth == 2 && current.is_none() {
                    current = Some(name);
                    current_text.clear();
                }

                depth += 1;
            }
            Ok(Event::End(event)) => {
                let name = local_name(event.name().as_ref());
                depth = depth.saturating_sub(1);

                if depth == 2 {
                    if let Some(expected) = current.take() {
                        if name == expected {
                            let value = std::mem::take(&mut current_text);
                            apply_channel_field(
                                &expected,
                                &value,
                                &mut title,
                                &mut link,
                                &mut description,
                                &mut pub_date,
                                &mut generator,
                            );
                        }
                        current_text.clear();
                    }
                }
            }
            Ok(Event::CData(data)) => {
                let text = String::from_utf8_lossy(data.as_ref()).into_owned();
                current_text.push_str(&text);
            }
            Ok(Event::Text(text)) => {
                let value = text.xml_content().unwrap_or_default().into_owned();
                current_text.push_str(&value);
            }
            Ok(Event::GeneralRef(reference)) => {
                if let Ok(Some(ch)) = reference.resolve_char_ref() {
                    current_text.push(ch);
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(WeeblyError::Xml(error)),
            _ => {}
        }
    }

    if title.is_empty() && link.is_empty() && items.is_empty() {
        return Err(WeeblyError::MalformedFeed(
            "document has no <channel> contents".to_string(),
        ));
    }

    Ok(WeeblyFeed {
        title,
        link,
        description,
        pub_date,
        generator,
        items,
    })
}

fn apply_channel_field(
    name: &str,
    value: &str,
    title: &mut String,
    link: &mut String,
    description: &mut Option<String>,
    pub_date: &mut Option<PublishedDate>,
    generator: &mut Option<String>,
) {
    match name {
        "title" => *title = value.to_string(),
        "link" => {
            if link.is_empty() {
                *link = value.to_string();
            }
        }
        "description" => {
            if description.is_none() {
                *description = Some(value.to_string());
            }
        }
        "pubDate" => {
            if pub_date.is_none() {
                *pub_date = PublishedDate::parse(value).ok();
            }
        }
        "generator" if generator.is_none() => *generator = Some(value.to_string()),
        _ => {}
    }
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

    const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/">
<channel><title><![CDATA[Mae Renfroe - Blog]]></title><link><![CDATA[https://maerenfroespeaker.weebly.com/blog]]></link><description><![CDATA[Blog]]></description><pubDate>Sat, 14 Nov 2026 12:41:36 -0800</pubDate><generator>Weebly</generator><item><title><![CDATA[First Post]]></title><link><![CDATA[https://maerenfroespeaker.weebly.com/blog/first-post]]></link><comments><![CDATA[https://maerenfroespeaker.weebly.com/blog/first-post#comments]]></comments><pubDate>Sun, 03 Aug 2025 18:45:55 GMT</pubDate><category><![CDATA[Uncategorized]]></category><guid isPermaLink="false">https://maerenfroespeaker.weebly.com/blog/first-post</guid><description><![CDATA[Hello there [...] ]]></description><content:encoded><![CDATA[<div class="paragraph">Hello there</div>]]></content:encoded></item></channel></rss>"#;

    #[test]
    fn parses_the_channel_metadata() {
        let feed = parse_feed(FEED).expect("valid feed");

        assert_eq!(feed.title, "Mae Renfroe - Blog");
        assert_eq!(feed.link, "https://maerenfroespeaker.weebly.com/blog");
        assert_eq!(feed.description.as_deref(), Some("Blog"));
        assert_eq!(feed.generator.as_deref(), Some("Weebly"));
        assert_eq!(feed.pub_date.map(|date| date.year()), Some(2026));
    }

    #[test]
    fn parses_the_items() {
        let feed = parse_feed(FEED).expect("valid feed");

        assert_eq!(feed.items.len(), 1);
        let post = &feed.items[0];
        assert_eq!(post.title, "First Post");
        assert_eq!(post.slug(), "first-post");
        assert_eq!(
            post.category
                .as_ref()
                .map(|category| category.name.as_str()),
            Some("Uncategorized")
        );
    }

    #[test]
    fn derives_the_site_and_blog_identity() {
        let feed = parse_feed(FEED).expect("valid feed");

        assert_eq!(feed.site_name(), "Mae Renfroe");
        assert_eq!(feed.blog_name(), "Blog");
        assert_eq!(feed.site_subdomain(), Some("maerenfroespeaker"));
        assert_eq!(feed.blog_path(), Some("blog"));
    }

    #[test]
    fn rejects_a_document_without_a_channel() {
        let xml = r#"<rss version="2.0"></rss>"#;
        let error = parse_feed(xml).expect_err("a document without a channel must fail");
        assert!(matches!(error, WeeblyError::MalformedFeed(_)));
    }
}
