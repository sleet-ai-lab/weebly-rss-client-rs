//! Builds the RSS feed URL of a Weebly site.

use crate::client::validate_site::validate_site;
use crate::result::Result;

/// Builds the RSS feed URL for a Weebly site.
///
/// Weebly publishes a feed for every blog at
/// `https://<site>.weebly.com/<blog path>/feed`. A blog sitting at the root
/// of the site has no path, and its feed lives at
/// `https://<site>.weebly.com/feed`.
///
/// The site may be given as a bare subdomain (`maerenfroespeaker`) or as a
/// full address (`maerenfroespeaker.weebly.com`).
///
/// # Errors
///
/// Returns [`WeeblyError::InvalidSite`](crate::error::WeeblyError::InvalidSite)
/// when the site name is empty or contains characters that are not valid in
/// a Weebly subdomain.
///
/// ```
/// # use weebly_rss_client_rs::client::build_feed_url;
/// assert_eq!(
///     build_feed_url("maerenfroespeaker", Some("blog")).unwrap(),
///     "https://maerenfroespeaker.weebly.com/blog/feed"
/// );
/// assert_eq!(
///     build_feed_url("maerenfroespeaker", None).unwrap(),
///     "https://maerenfroespeaker.weebly.com/feed"
/// );
/// ```
pub fn build_feed_url(site: &str, blog_path: Option<&str>) -> Result<String> {
    let subdomain = validate_site(site)?;

    let path = match blog_path.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => {
            let trimmed = path.trim_matches('/');
            if trimmed.is_empty() {
                String::new()
            } else {
                format!("/{trimmed}")
            }
        }
        None => String::new(),
    };

    Ok(format!("https://{subdomain}.weebly.com{path}/feed"))
}
