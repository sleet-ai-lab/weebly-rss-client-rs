//! A fully typed client for Weebly's public blog RSS feeds.
//!
//! Weebly publishes a feed for every blog at
//! `https://<site>.weebly.com/<blog path>/feed`. The feed carries the post
//! body twice: as a truncated plain-text summary in `<description>` and as
//! complete HTML in `content:encoded`. This crate turns that XML into a
//! typed, owned data model.
//!
//! # Example
//!
//! ```no_run
//! use weebly_rss_client_rs::{WeeblyClient, WeeblyError};
//!
//! # async fn run() -> Result<(), WeeblyError> {
//! let client = WeeblyClient::new();
//! let feed = client.fetch_feed("maerenfroespeaker", Some("blog")).await?;
//!
//! for post in &feed.items {
//!     println!(
//!         "{} ({}) — {}",
//!         post.title,
//!         post.pub_date.map(|date| date.year()).unwrap_or_default(),
//!         post.category.as_ref().map(|category| category.name.as_str()).unwrap_or("—"),
//!     );
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Layout
//!
//! The crate follows a strict one-item-per-file convention:
//!
//! * [`types`] holds the data model, one type per file.
//! * [`parse`] holds the XML parsers, one function per file.
//! * [`client`] holds the HTTP layer.
//! * [`util`] holds small shared helpers.
//!
//! # Scope
//!
//! This crate covers the RSS feed only. Weebly's ordinary site pages, blog
//! archives and comment threads are HTML, and are not parsed here. The feed
//! itself is a rolling window of the most recent posts — Weebly serves the
//! latest 25 by default — so a blog's complete history needs its archive
//! pages.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod client;
pub mod error;
pub mod parse;
pub mod result;
pub mod types;
pub mod util;

pub use client::{
    build_feed_url, build_site_url, FetchReport, WeeblyClient, DEFAULT_TIMEOUT, WEBLY_BASE_URL,
};
pub use error::WeeblyError;
pub use parse::{parse_feed, parse_item};
pub use result::Result;
pub use types::{BlogPost, Category, PostImage, PostLink, PublishedDate, WeeblyFeed};
pub use util::{collapse_whitespace, extract_images, extract_links, html_to_text};
