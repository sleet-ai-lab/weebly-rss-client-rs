//! Error types for the Weebly RSS client.
//!
//! Every fallible operation in this crate returns [`WeeblyError`].

/// The error type returned by all fallible operations in this crate.
#[derive(Debug, thiserror::Error)]
pub enum WeeblyError {
    /// The HTTP request to Weebly failed.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The HTTP response status was not a success status.
    #[error("unexpected http status {status} for {url}")]
    HttpStatus {
        /// The HTTP status code returned by Weebly.
        status: u16,
        /// The URL that was requested.
        url: String,
    },

    /// The response body could not be decoded as UTF-8 text.
    #[error("response body was not valid utf-8")]
    InvalidUtf8,

    /// The XML document could not be parsed.
    #[error("failed to parse xml: {0}")]
    Xml(#[from] quick_xml::Error),

    /// The XML document did not have the expected Weebly RSS shape.
    #[error("malformed weebly feed: {0}")]
    MalformedFeed(String),

    /// The requested site or blog could not be found.
    #[error("weebly site not found: {0}")]
    SiteNotFound(String),

    /// The site name was not in a usable form.
    #[error("invalid weebly site: {0}")]
    InvalidSite(String),

    /// A date in the feed could not be parsed.
    #[error("failed to parse date `{value}` as {expected}")]
    InvalidDate {
        /// The raw date text that failed to parse.
        value: String,
        /// A human description of the expected format.
        expected: &'static str,
    },
}

impl WeeblyError {
    /// Returns `true` when the error indicates the site does not exist.
    ///
    /// A 404 from the feed endpoint is how Weebly reports an unknown site, so
    /// this maps that case to [`WeeblyError::SiteNotFound`].
    pub fn is_site_not_found(&self) -> bool {
        matches!(
            self,
            WeeblyError::SiteNotFound(_) | WeeblyError::HttpStatus { status: 404, .. }
        )
    }
}
