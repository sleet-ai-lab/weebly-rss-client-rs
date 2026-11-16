//! Builds the site URL of a Weebly site.

use crate::client::validate_site::validate_site;
use crate::result::Result;

/// Builds the home page URL for a Weebly site.
///
/// This is a convenience helper for linking to a site's page. The crate
/// itself only parses the RSS feed, and this URL is an ordinary HTML page,
/// not a feed.
///
/// ```
/// # use weebly_rss_client_rs::client::build_site_url;
/// assert_eq!(
///     build_site_url("maerenfroespeaker").unwrap(),
///     "https://maerenfroespeaker.weebly.com/"
/// );
/// ```
pub fn build_site_url(site: &str) -> Result<String> {
    let subdomain = validate_site(site)?;
    Ok(format!("https://{subdomain}.weebly.com/"))
}
