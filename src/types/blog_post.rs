//! A single blog post in a Weebly RSS feed.

use serde::{Deserialize, Serialize};

use crate::types::{Category, PublishedDate};

/// A single blog post from a Weebly RSS feed.
///
/// A Weebly feed holds one item per published post. Each item carries a
/// truncated plain-text `description` and the complete post body in
/// `content:encoded`, wrapped in HTML.
///
/// Every field is owned; nothing borrows from the feed document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlogPost {
    /// The GUID, which Weebly writes as the post's permalink URL.
    pub guid: String,
    /// The post title.
    pub title: String,
    /// The post's permalink URL.
    pub link: String,
    /// The URL of the post's comment section.
    pub comments_url: Option<String>,
    /// The post's publication date.
    pub pub_date: Option<PublishedDate>,
    /// The category the post was filed under.
    pub category: Option<Category>,
    /// The truncated plain-text summary Weebly puts in `<description>`.
    ///
    /// Weebly cuts the summary at roughly 500 characters and appends
    /// ` [...]`. The full body is in [`BlogPost::content_html`].
    pub summary: Option<String>,
    /// The complete post body as HTML, from `content:encoded`.
    pub content_html: Option<String>,
    /// The complete post body as plain text.
    pub content_text: Option<String>,
    /// The links found in the post body, in document order.
    pub links: Vec<PostLink>,
    /// The images found in the post body, in document order.
    pub images: Vec<PostImage>,
}

impl BlogPost {
    /// Returns the post's permalink URL.
    ///
    /// Weebly writes the same URL into `<link>` and `<guid>`, so this prefers
    /// the `<link>` and falls back to the GUID.
    pub fn permalink(&self) -> &str {
        if self.link.is_empty() {
            &self.guid
        } else {
            &self.link
        }
    }

    /// Returns the post's slug, taken from the last path segment of its URL.
    ///
    /// ```
    /// # use weebly_rss_client_rs::BlogPost;
    /// # let post = BlogPost { guid: "https://example.weebly.com/blog/hello-world".into(), title: String::new(), link: String::new(), comments_url: None, pub_date: None, category: None, summary: None, content_html: None, content_text: None, links: Vec::new(), images: Vec::new() };
    /// assert_eq!(post.slug(), "hello-world");
    /// ```
    pub fn slug(&self) -> &str {
        let path = self
            .permalink()
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .trim_end_matches('/');

        path.rsplit('/').next().unwrap_or_default()
    }

    /// Returns `true` when the summary was cut short by Weebly.
    ///
    /// Weebly truncates the `<description>` summary and marks it with a
    /// trailing ` [...]`.
    pub fn is_summary_truncated(&self) -> bool {
        self.summary
            .as_deref()
            .map(|summary| summary.trim_end().ends_with("[...]"))
            .unwrap_or(false)
    }

    /// Returns the number of characters in the post body.
    pub fn content_length(&self) -> usize {
        self.content_text
            .as_deref()
            .map(str::len)
            .unwrap_or_default()
    }
}

/// A link found in a post body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostLink {
    /// The link target.
    pub url: String,
    /// The link's visible text, when it had any.
    pub text: Option<String>,
}

/// An image found in a post body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostImage {
    /// The image URL.
    pub url: String,
    /// The image's `alt` text, when it had any.
    pub alt: Option<String>,
}
