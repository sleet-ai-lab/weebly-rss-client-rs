//! Extracts the images out of a post body.

use crate::types::blog_post::PostImage;
use crate::util::extract_links::attribute_value;

/// Extracts the images from an HTML fragment, in document order.
///
/// Weebly hosts post images under `https://<site>.weebly.com/uploads/...`
/// and wraps them in `<div class="wsite-image">`.
///
/// ```
/// # use weebly_rss_client_rs::util::extract_images;
/// let images = extract_images("<img src=\"https://example.weebly.com/uploads/1/a.png\" alt=\"Picture\"/>");
/// assert_eq!(images.len(), 1);
/// assert_eq!(images[0].url, "https://example.weebly.com/uploads/1/a.png");
/// ```
pub fn extract_images(html: &str) -> Vec<PostImage> {
    let mut images = Vec::new();
    let lower = html.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(offset) = lower[cursor..].find("<img") {
        let start = cursor + offset;
        let Some(end) = lower[start..].find('>') else {
            break;
        };
        let tag = &html[start..start + end];

        if let Some(url) = attribute_value(tag, "src") {
            let url = url.trim().to_string();
            if !url.is_empty() {
                images.push(PostImage {
                    url,
                    alt: attribute_value(tag, "alt").and_then(|alt| {
                        let trimmed = alt.trim().to_string();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed)
                        }
                    }),
                });
            }
        }

        cursor = start + end + 1;
    }

    images
}
