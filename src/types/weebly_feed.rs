//! The top-level RSS feed for a Weebly blog.

use serde::{Deserialize, Serialize};

use crate::types::{BlogPost, Category};

/// A parsed Weebly RSS feed.
///
/// Every Weebly site publishes one RSS 2.0 channel per blog. A site with
/// several blogs exposes one channel each, and the channel link points at
/// the blog's page on the site.
///
/// [`WeeblyFeed::items`] holds every post in document order, which is the
/// order Weebly writes them: newest first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeeblyFeed {
    /// The channel title, for example `Mae Renfroe - Blog`.
    pub title: String,
    /// The channel link, pointing at the blog's page on the site.
    pub link: String,
    /// The channel description.
    pub description: Option<String>,
    /// The channel's publication date, which Weebly sets to the time the
    /// feed was generated.
    pub pub_date: Option<crate::types::PublishedDate>,
    /// The software that generated the feed.
    ///
    /// Weebly writes `Weebly` here.
    pub generator: Option<String>,
    /// Every post in the feed, in document order.
    pub items: Vec<BlogPost>,
}

impl WeeblyFeed {
    /// Returns an empty feed.
    ///
    /// This is used to keep the results of a multi-site fetch aligned with
    /// the requested sites when one of them fails.
    pub fn empty() -> Self {
        Self {
            title: String::new(),
            link: String::new(),
            description: None,
            pub_date: None,
            generator: None,
            items: Vec::new(),
        }
    }

    /// Returns the number of posts in the feed.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` when the feed holds no posts.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the categories used by the feed's posts, in first-seen order.
    ///
    /// The `Uncategorized` placeholder is included, because it is a real
    /// value in the feed.
    pub fn categories(&self) -> Vec<Category> {
        let mut seen: Vec<Category> = Vec::new();

        for post in &self.items {
            let Some(category) = &post.category else {
                continue;
            };
            if !seen.contains(category) {
                seen.push(category.clone());
            }
        }

        seen
    }

    /// Returns the posts filed under the given category.
    pub fn posts_in_category(&self, name: &str) -> Vec<&BlogPost> {
        self.items
            .iter()
            .filter(|post| {
                post.category
                    .as_ref()
                    .map(|category| category.name.eq_ignore_ascii_case(name))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Returns the site's name, derived from the channel title.
    ///
    /// Weebly writes channel titles as `<site name> - <blog name>`, so this
    /// strips the suffix. When the title does not have the expected shape the
    /// original title is returned unchanged.
    ///
    /// ```
    /// # use weebly_rss_client_rs::WeeblyFeed;
    /// # let feed = WeeblyFeed { title: "Mae Renfroe - Blog".into(), link: String::new(), description: None, pub_date: None, generator: None, items: Vec::new() };
    /// assert_eq!(feed.site_name(), "Mae Renfroe");
    /// ```
    pub fn site_name(&self) -> &str {
        let trimmed = self.title.trim();

        match trimmed.rsplit_once(" - ") {
            Some((site, blog)) if !site.is_empty() && !blog.is_empty() => site.trim(),
            _ => trimmed,
        }
    }

    /// Returns the blog's name, derived from the channel title.
    ///
    /// This is the part of the title after the ` - ` separator.
    pub fn blog_name(&self) -> &str {
        let trimmed = self.title.trim();

        match trimmed.rsplit_once(" - ") {
            Some((_, blog)) if !blog.is_empty() => blog.trim(),
            _ => trimmed,
        }
    }

    /// Returns the site's subdomain, derived from the channel link.
    ///
    /// Weebly sites live at `https://<subdomain>.weebly.com/<blog path>`, so
    /// this returns the subdomain without the `.weebly.com` suffix.
    ///
    /// ```
    /// # use weebly_rss_client_rs::WeeblyFeed;
    /// # let feed = WeeblyFeed { title: String::new(), link: "https://maerenfroespeaker.weebly.com/blog".into(), description: None, pub_date: None, generator: None, items: Vec::new() };
    /// assert_eq!(feed.site_subdomain().unwrap(), "maerenfroespeaker");
    /// ```
    pub fn site_subdomain(&self) -> Option<&str> {
        let rest = self.link.split("://").nth(1)?;
        let host = rest.split(['/', '?', '#']).next()?;
        let subdomain = host.strip_suffix(".weebly.com")?;
        if subdomain.is_empty() {
            None
        } else {
            Some(subdomain)
        }
    }

    /// Returns the blog's path on the site, derived from the channel link.
    ///
    /// For a channel link of `https://example.weebly.com/blog` this returns
    /// `blog`. A site whose blog sits at the root returns `None`.
    pub fn blog_path(&self) -> Option<&str> {
        let rest = self.link.split("://").nth(1)?;
        // Drop the host, keeping only the path.
        let path = rest.split_once('/').map(|(_, path)| path).unwrap_or("");
        let path = path.split(['?', '#']).next()?.trim_matches('/');
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    }
}
