//! Extracts the links out of a post body.

use crate::types::blog_post::PostLink;
use crate::util::collapse_whitespace::collapse_whitespace;
use crate::util::html_to_text::html_to_text;

/// Extracts the links from an HTML fragment, in document order.
///
/// Both `<a href="...">` links and bare `<img src="...">` targets are
/// collected, but a link that is already wrapped around an image is only
/// reported once, as the `<a>`.
///
/// Weebly wraps images in `<a>` elements and uses `<a>` for outbound links,
/// so this covers both shapes of link a Weebly post can carry. The visible
/// text of an anchor is captured as well, when it has any.
///
/// ```
/// # use weebly_rss_client_rs::util::extract_links;
/// let links = extract_links("<p><a href=\"https://example.com\">Example</a></p>");
/// assert_eq!(links.len(), 1);
/// assert_eq!(links[0].url, "https://example.com");
/// assert_eq!(links[0].text.as_deref(), Some("Example"));
/// ```
pub fn extract_links(html: &str) -> Vec<PostLink> {
    let mut links = Vec::new();
    let mut cursor = 0usize;

    while let Some((start, end)) = next_tag(html, cursor, "a") {
        cursor = end;

        let tag = &html[start..end];
        let Some(url) = attribute_value(tag, "href") else {
            continue;
        };

        let url = url.trim().to_string();
        if url.is_empty() {
            continue;
        }

        // An image wrapped in a link must not be reported twice.
        if links.iter().any(|link: &PostLink| link.url == url) {
            continue;
        }

        links.push(PostLink {
            url,
            text: anchor_text(html, end),
        });
    }

    // Images that are not already wrapped in a link are links too.
    cursor = 0usize;
    while let Some((start, end)) = next_tag(html, cursor, "img") {
        cursor = end;

        let tag = &html[start..end];
        let Some(url) = attribute_value(tag, "src") else {
            continue;
        };

        let url = url.trim().to_string();
        if url.is_empty() || links.iter().any(|link: &PostLink| link.url == url) {
            continue;
        }

        links.push(PostLink { url, text: None });
    }

    links
}

/// Returns the visible text of an anchor, given the offset just past its
/// opening tag.
///
/// The text is the content up to the matching `</a>`. Weebly often leaves
/// the visible text in nested `<span>` elements, so the markup is stripped
/// before the text is read.
fn anchor_text(html: &str, after_open: usize) -> Option<String> {
    let rest = &html[after_open..];
    let close = rest.to_ascii_lowercase().find("</a")?;
    let inner = &rest[..close];

    let text = collapse_whitespace(&html_to_text(inner));
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Finds the next start tag with the given name, returning its byte range.
///
/// The returned range covers the whole tag, including the angle brackets.
fn next_tag(html: &str, from: usize, name: &str) -> Option<(usize, usize)> {
    let mut cursor = from;

    while let Some(offset) = html[cursor..].find('<') {
        let start = cursor + offset;
        let rest = &html[start..];

        // The offset of the closing `>` within `rest`.
        let tag_end = rest.find('>')?;

        // Read the tag name, skipping the leading slash of a closing tag.
        let inner = rest[1..tag_end].trim_start_matches('/');
        let tag_name: String = inner
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();

        if tag_name.eq_ignore_ascii_case(name) {
            return Some((start, start + tag_end + 1));
        }

        cursor = start + tag_end + 1;
    }

    None
}

/// Reads an attribute value out of an HTML tag.
pub(crate) fn attribute_value(tag: &str, attribute: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(offset) = lower[cursor..].find(attribute) {
        let position = cursor + offset;
        // The attribute must be preceded by whitespace to avoid matching
        // substrings such as `data-src` when looking for `src`.
        let preceded_by_space =
            position > 0 && lower.as_bytes()[position - 1].is_ascii_whitespace();
        let followed_by_equals = lower[position + attribute.len()..].starts_with('=');

        if preceded_by_space && followed_by_equals {
            let rest = &tag[position + attribute.len() + 1..];
            let rest_trimmed = rest.trim_start();
            let quote = rest_trimmed.chars().next()?;

            if quote == '"' || quote == '\'' {
                let body = &rest_trimmed[1..];
                let end = body.find(quote)?;
                return Some(body[..end].to_string());
            }

            let end = rest_trimmed
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(rest_trimmed.len());
            return Some(rest_trimmed[..end].to_string());
        }

        cursor = position + attribute.len();
    }

    None
}
