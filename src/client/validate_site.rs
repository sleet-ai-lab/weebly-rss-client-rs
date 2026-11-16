//! Validates a Weebly site name.

use crate::error::WeeblyError;
use crate::result::Result;

/// Validates a Weebly site name and returns it unchanged.
///
/// A site name is the subdomain part of a Weebly address, without the
/// `.weebly.com` suffix. Both `example` and `example.weebly.com` are
/// accepted, and the returned value is always the bare subdomain.
///
/// # Errors
///
/// Returns [`WeeblyError::InvalidSite`] when the site name is empty, contains
/// whitespace, or contains a path separator.
///
/// ```
/// # use weebly_rss_client_rs::client::validate_site;
/// assert_eq!(validate_site("maerenfroespeaker").unwrap(), "maerenfroespeaker");
/// assert_eq!(validate_site("maerenfroespeaker.weebly.com").unwrap(), "maerenfroespeaker");
/// ```
pub fn validate_site(site: &str) -> Result<&str> {
    let trimmed = site.trim();

    if trimmed.is_empty() {
        return Err(WeeblyError::InvalidSite(site.to_string()));
    }

    if trimmed != site {
        return Err(WeeblyError::InvalidSite(site.to_string()));
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(WeeblyError::InvalidSite(site.to_string()));
    }

    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('?')
        || trimmed.contains('#')
    {
        return Err(WeeblyError::InvalidSite(site.to_string()));
    }

    // Accept both the bare subdomain and a full Weebly address.
    let subdomain = trimmed.strip_suffix(".weebly.com").unwrap_or(trimmed);

    if subdomain.is_empty() {
        return Err(WeeblyError::InvalidSite(site.to_string()));
    }

    if !subdomain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(WeeblyError::InvalidSite(site.to_string()));
    }

    Ok(subdomain)
}
