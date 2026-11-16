//! The asynchronous client that fetches Weebly RSS feeds.

use std::time::Duration;

use crate::client::build_feed_url::build_feed_url;
use crate::client::default_timeout::DEFAULT_TIMEOUT;
use crate::client::fetch_report::FetchReport;
use crate::error::WeeblyError;
use crate::parse::parse_feed::parse_feed;
use crate::result::Result;
use crate::types::WeeblyFeed;

/// An asynchronous client for Weebly's public RSS feeds.
///
/// Weebly publishes one feed per blog at
/// `https://<site>.weebly.com/<blog path>/feed`. A site with a single blog
/// may also expose the same feed under a numbered path such as `/2/feed`,
/// which is the form that appears in a blog's own navigation.
///
/// The client is cheap to clone and safe to share between tasks. It holds a
/// `reqwest::Client`, which pools connections, so create one and reuse it.
///
/// ```no_run
/// # use weebly_rss_client_rs::WeeblyClient;
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let client = WeeblyClient::new();
/// let feed = client.fetch_feed("maerenfroespeaker", Some("blog")).await?;
/// println!("{} posts", feed.items.len());
/// # Ok(())
/// # }
/// ```
///
/// # Builder
///
/// The settings are applied in the order they are called, and each one keeps
/// the others intact:
///
/// ```
/// # use weebly_rss_client_rs::WeeblyClient;
/// # use std::time::Duration;
/// let client = WeeblyClient::new()
///     .with_timeout(Duration::from_secs(10))
///     .with_user_agent("my-tool/1.0");
/// ```
#[derive(Debug, Clone)]
pub struct WeeblyClient {
    http: reqwest::Client,
    user_agent: String,
    timeout: Duration,
}

impl WeeblyClient {
    /// Creates a client with the default settings.
    ///
    /// The default timeout is 30 seconds.
    pub fn new() -> Self {
        let user_agent = default_user_agent();
        let http = build_http_client(&user_agent, DEFAULT_TIMEOUT);

        Self {
            http,
            user_agent,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Sets the request timeout.
    ///
    /// The timeout applies to each individual request.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.http = build_http_client(&self.user_agent, timeout);
        self
    }

    /// Sets the `User-Agent` header sent with every request.
    ///
    /// Weebly serves the public RSS feed to any client, but sending a
    /// descriptive user agent is good manners and makes the client easier to
    /// identify in server logs.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self.http = build_http_client(&self.user_agent, self.timeout);
        self
    }

    /// Returns the `User-Agent` the client sends.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Returns the request timeout of the client.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Fetches and parses the RSS feed of a blog.
    ///
    /// The `blog_path` is the path the blog sits at on the site, for example
    /// `blog` for `https://example.weebly.com/blog/feed`. Pass `None` for a
    /// blog at the root of the site.
    ///
    /// # Errors
    ///
    /// Returns [`WeeblyError::SiteNotFound`] when the site does not exist,
    /// [`WeeblyError::InvalidSite`] when the site name is not usable,
    /// [`WeeblyError::Http`] when the request fails,
    /// [`WeeblyError::HttpStatus`] when Weebly answers with a non-success
    /// status, and [`WeeblyError::MalformedFeed`] when the response is not a
    /// Weebly feed.
    pub async fn fetch_feed(&self, site: &str, blog_path: Option<&str>) -> Result<WeeblyFeed> {
        let xml = self.fetch_feed_xml(site, blog_path).await?;
        parse_feed(&xml)
    }

    /// Fetches the raw RSS XML of a blog without parsing it.
    ///
    /// # Errors
    ///
    /// Returns [`WeeblyError::SiteNotFound`] when Weebly answers with a 404,
    /// which is how it reports an unknown site or blog.
    /// [`WeeblyError::Http`] covers a failed request and
    /// [`WeeblyError::HttpStatus`] covers any other non-success status.
    pub async fn fetch_feed_xml(&self, site: &str, blog_path: Option<&str>) -> Result<String> {
        let url = build_feed_url(site, blog_path)?;
        let response = self.http.get(&url).send().await?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(WeeblyError::SiteNotFound(site.to_string()));
        }
        if !status.is_success() {
            return Err(WeeblyError::HttpStatus {
                status: status.as_u16(),
                url,
            });
        }

        response.text().await.map_err(WeeblyError::Http)
    }

    /// Fetches the feeds of several blogs concurrently.
    ///
    /// The blogs are fetched in parallel and the results are returned in the
    /// same order as the input. A blog whose feed cannot be fetched yields
    /// an empty feed in its slot and its error is collected, so one bad site
    /// does not lose the data of the others.
    ///
    /// # Errors
    ///
    /// Returns [`WeeblyError::SiteNotFound`] when at least one site does not
    /// exist. The successfully fetched feeds are still returned through
    /// [`FetchReport`].
    pub async fn fetch_feeds(
        &self,
        blogs: &[(impl AsRef<str>, Option<&str>)],
    ) -> Result<Vec<WeeblyFeed>> {
        Ok(self.fetch_feeds_report(blogs).await.feeds)
    }

    /// Fetches the feeds of several blogs concurrently, keeping the errors
    /// of individual blogs alongside the successful results.
    ///
    /// This never fails: a blog that cannot be fetched produces an error
    /// entry in [`FetchReport::errors`] instead.
    pub async fn fetch_feeds_report(
        &self,
        blogs: &[(impl AsRef<str>, Option<&str>)],
    ) -> FetchReport {
        let mut feeds = Vec::with_capacity(blogs.len());
        let mut errors = Vec::new();

        for (site, blog_path) in blogs {
            let site = site.as_ref();
            match self.fetch_feed(site, *blog_path).await {
                Ok(feed) => feeds.push(feed),
                Err(error) => {
                    errors.push((site.to_string(), error));
                    feeds.push(WeeblyFeed::empty());
                }
            }
        }

        FetchReport { feeds, errors }
    }
}

impl Default for WeeblyClient {
    fn default() -> Self {
        Self::new()
    }
}

fn build_http_client(user_agent: &str, timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(user_agent.to_string())
        .timeout(timeout)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn default_user_agent() -> String {
    format!(
        "weebly-rss-client-rs/{} (+https://github.com/sleet-ai-lab/weebly-rss-client-rs)",
        env!("CARGO_PKG_VERSION")
    )
}
