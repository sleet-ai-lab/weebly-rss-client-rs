# weebly-rss-client-rs

A fully typed Rust client for Weebly's public blog RSS feeds. Weebly publishes a feed for every blog at `https://<site>.weebly.com/<blog path>/feed`, and that feed carries each post twice: as a truncated plain-text summary in `<description>` and as complete HTML in `content:encoded`. This crate fetches that feed over HTTP, parses the XML, and exposes it as an owned, strongly typed data model with no borrowed lifetimes, no `String` soup and no manual HTML scraping. Every type in the model derives `Serialize` and `Deserialize`, so a parsed feed can be written straight to JSON, and the crate ships two CLI binaries for inspecting a blog from the shell.

## Features

- **Typed data model.** `WeeblyFeed`, `BlogPost`, `Category` and `PublishedDate` are all concrete Rust types with documented fields.
- **Async client.** `WeeblyClient` wraps `reqwest` with rustls, connection pooling, a configurable timeout and a descriptive default `User-Agent`.
- **Full post bodies.** The complete post HTML from `content:encoded` is kept, and converted to plain text with the markup stripped.
- **Truncated summaries.** Weebly cuts the `<description>` summary at roughly 500 characters; the crate keeps both the summary and the flag that says it was cut.
- **Link and image extraction.** The links and images embedded in a post body are extracted in document order.
- **Category support.** Every post's category is parsed, and the feed can be filtered by it.
- **Site and blog identity.** The site name, blog name, subdomain and blog path are all derived from the channel metadata.
- **JSON-serializable types.** Every model type derives `serde::Serialize` and `serde::Deserialize`.
- **Two CLI binaries.** `wbrss` and `wbrss-summary` cover the overview and the statistics.

## Installation

```toml
[dependencies]
weebly-rss-client-rs = { git = "https://github.com/sleet-ai-lab/weebly-rss-client-rs" }
```

The client is async, so you also need a runtime:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Usage

```rust
use weebly_rss_client_rs::{WeeblyClient, WeeblyError};

#[tokio::main]
async fn main() -> Result<(), WeeblyError> {
    let client = WeeblyClient::new();
    let feed = client.fetch_feed("maerenfroespeaker", Some("blog")).await?;

    println!("{} ({})", feed.site_name(), feed.blog_name());

    for post in &feed.items {
        println!(
            "{} ({})  {}",
            post.title,
            post.pub_date.map(|date| date.year()).unwrap_or_default(),
            post.category.as_ref().map(|c| c.name.as_str()).unwrap_or("—"),
        );
    }

    Ok(())
}
```

The `blog_path` is the path the blog sits at on the site. Pass `None` for a blog at the root of the site.

The client is cheap to clone and pools connections, so create one and reuse it:

```rust
use std::time::Duration;
use weebly_rss_client_rs::WeeblyClient;

let client = WeeblyClient::new()
    .with_timeout(Duration::from_secs(10))
    .with_user_agent("my-tool/1.0");
```

Feeds can also be parsed straight from XML that you already have:

```rust
use weebly_rss_client_rs::parse_feed;

let xml = std::fs::read_to_string("feed.xml")?;
let feed = parse_feed(&xml)?;
println!("{} posts", feed.items.len());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Data model

`WeeblyFeed` holds the channel metadata and every post in document order:

| Field | Type | Description |
|---|---|---|
| `title` | `String` | The channel title, for example `Mae Renfroe - Blog`. |
| `link` | `String` | The channel link, pointing at the blog's page on the site. |
| `description` | `Option<String>` | The channel description. |
| `pub_date` | `Option<PublishedDate>` | The time the feed was generated. |
| `generator` | `Option<String>` | The software that generated the feed; Weebly writes `Weebly`. |
| `items` | `Vec<BlogPost>` | Every post in the feed, in document order. |

The feed offers helper methods: `len()`, `is_empty()`, `categories()` (the categories used, in first-seen order) and `posts_in_category(name)`. It also derives the site's identity from the channel metadata: `site_name()` and `blog_name()` split the channel title on ` - `, `site_subdomain()` extracts the subdomain from the channel link, and `blog_path()` extracts the path the blog sits at.

### BlogPost

| Field | Type | Description |
|---|---|---|
| `guid` | `String` | The GUID, which Weebly writes as the post's permalink URL. |
| `title` | `String` | The post title. |
| `link` | `String` | The post's permalink URL. |
| `comments_url` | `Option<String>` | The URL of the post's comment section. |
| `pub_date` | `Option<PublishedDate>` | The post's publication date. |
| `category` | `Option<Category>` | The category the post was filed under. |
| `summary` | `Option<String>` | The truncated plain-text summary from `<description>`. |
| `content_html` | `Option<String>` | The complete post body as HTML, from `content:encoded`. |
| `content_text` | `Option<String>` | The complete post body as plain text. |
| `links` | `Vec<PostLink>` | The links found in the post body, in document order. |
| `images` | `Vec<PostImage>` | The images found in the post body, in document order. |

Convenience methods: `permalink()` (the link, falling back to the GUID), `slug()` (the last path segment of the permalink), `is_summary_truncated()` (whether Weebly cut the summary short) and `content_length()`.

Each `PostLink` carries a `url` and an optional `text`. Each `PostImage` carries a `url` and an optional `alt`.

### Supporting types

| Type | Description |
|---|---|
| `Category` | A post category. Exposes `name()` and `is_uncategorized()`, which reports Weebly's `Uncategorized` placeholder. |
| `PublishedDate` | An RFC 2822 `pubDate` timestamp with the original UTC offset preserved. Exposes `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`, `offset()` and `to_rfc2822()`. |
| `WeeblyError` | The error type of the crate. The client maps a `404` to the `SiteNotFound` variant, and `is_site_not_found()` reports it. |

## CLI

The crate installs two binaries. All of them take the site name as a positional argument and the blog path as an optional second positional argument, and accept `--timeout <SECONDS>` (default `30`) and `--user-agent <STRING>`.

### `wbrss` — overview

Prints the blog, the post counts, the categories and a sample of the posts.

| Flag | Default | Description |
|---|---|---|
| `--limit <LIMIT>` | `20` | Maximum number of posts to show. |
| `--full` | off | Also print the body text of each post. |
| `--json` | off | Output the raw feed as pretty JSON. |
| `--timeout <TIMEOUT>` | `30` | HTTP timeout in seconds. |
| `--user-agent <USER_AGENT>` | — | Custom `User-Agent` header. |

```console
$ wbrss --limit 3 maerenfroespeaker blog
Site:      Mae Renfroe
Blog:      Blog
Subdomain: maerenfroespeaker
Feed:      https://maerenfroespeaker.weebly.com/blog

Counts:
  Total posts:  25
  Categories:   3

Categories:
  Uncategorized (3)
  video blog posts (11)
  walking with God (11)

Posts (3 shown):
  • Hope Again Ladies Retreat - Oct 3-5th
      date:     Sun, 03 Aug 2025 18:45:55 +0000
      category: Uncategorized
      link:     https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th
  • We visited Mount Rushmore - trip update! : Mae Renfroe
      date:     Wed, 12 Jun 2024 23:19:11 +0000
      category: video blog posts
      link:     https://maerenfroespeaker.weebly.com/blog/we-visited-mount-rushmore-trip-update-mae-renfroe
  • A Season of a Momma's Tears: w/ Bambi Lynn & Mae Renfroe :Thoughtful Thursdays
      date:     Sun, 21 Apr 2024 18:30:06 +0000
      category: walking with God
      link:     https://maerenfroespeaker.weebly.com/blog/a-season-of-a-mommas-tears-w-bambi-lynn-mae-renfroe-thoughtful-thursdays
```

### `wbrss-summary` — statistics

Prints aggregate counts, a per-year breakdown, the categories and the most linked-to hosts.

| Flag | Default | Description |
|---|---|---|
| `--json` | off | Output the summary as pretty JSON. |
| `--timeout <TIMEOUT>` | `30` | HTTP timeout in seconds. |
| `--user-agent <USER_AGENT>` | — | Custom `User-Agent` header. |

```console
$ wbrss-summary thesunshining sunshining-blog
Site:      THE SUNSHINING
Blog:      SunShining Blog
Subdomain: thesunshining
Feed:      https://thesunshining.weebly.com/sunshining-blog
Generator: Weebly

Posts:
  Total:            25
  With body:        25
  Truncated:        25
  Categories:       5
  Links:            152
  Images:           28
  Avg body length:  3053 chars

Posts per year:
  2021: 25

Categories:
    15x  Uncategorized
     6x  Daily SunShine
     2x  Book Spotlights
     1x  Nonresistant Book Reviews
     1x  Sparkling Christian Music

Most linked hosts:
    43x  thesunshining.weebly.com
    13x  medium.com
    12x  the-sunshining.github.io
     7x  dailysunshinewithncr.weebly.com
     7x  play.google.com
```

Both binaries support `--json`, which emits the parsed model rather than the text view:

```console
$ wbrss --json --limit 1 maerenfroespeaker blog
{
  "title": "Mae Renfroe - Blog",
  "link": "https://maerenfroespeaker.weebly.com/blog",
  "description": "Blog",
  "pub_date": {
    "offset_seconds": -28800,
    "year": 2026,
    "month": 11,
    "day": 15,
    "hour": 19,
    "minute": 1,
    "second": 30
  },
  "generator": "Weebly",
  "items": [
    {
      "guid": "https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th",
      "title": "Hope Again Ladies Retreat - Oct 3-5th",
      "link": "https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th",
      "comments_url": "https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th#comments",
      "pub_date": {
        "offset_seconds": 0,
        "year": 2025,
        "month": 8,
        "day": 3,
        "hour": 18,
        "minute": 45,
        "second": 55
      },
      "category": {
        "name": "Uncategorized"
      },
      "summary": "Hey, it;s been a long while since I posted on my blog. ...",
      "content_html": "<div class=\"paragraph\">Hey, it;s been a long while ...",
      "content_text": "Hey, it;s been a long while since I posted on my blog.\nI continue having amazing guests...",
      "links": [],
      "images": []
    }
  ]
}
```

## The feed format

Weebly serves one RSS 2.0 channel per blog. The channel header looks like this:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wfw="http://wellformedweb.org/CommentAPI/" xmlns:dc="http://purl.org/dc/elements/1.1/" >

<channel><title><![CDATA[Mae Renfroe - Blog]]></title><link><![CDATA[https://maerenfroespeaker.weebly.com/blog]]></link><description><![CDATA[Blog]]></description><pubDate>Sat, 14 Nov 2026 12:41:36 -0800</pubDate><generator>Weebly</generator>
```

Three namespace prefixes are declared, but only `content:` is actually used to carry data:

| Prefix | Namespace | Used for |
|---|---|---|
| `content:` | `http://purl.org/rss/1.0/modules/content/` | `encoded`, which holds the complete post body as HTML. |
| `wfw:` | `http://wellformedweb.org/CommentAPI/` | Declared, but no `wfw:` element appears in the feed. |
| `dc:` | `http://purl.org/dc/elements/1.1/` | Declared, but Weebly writes no `dc:creator`, so the author is not in the feed. |

A post item looks like this:

```xml
<item><title><![CDATA[Hope Again Ladies Retreat - Oct 3-5th]]></title><link><![CDATA[https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th]]></link><comments><![CDATA[https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th#comments]]></comments><pubDate>Sun, 03 Aug 2025 18:45:55 GMT</pubDate><category><![CDATA[Uncategorized]]></category><guid isPermaLink="false">https://maerenfroespeaker.weebly.com/blog/hope-again-ladies-retreat-oct-3-5th</guid><description><![CDATA[&nbsp;Hey, it;s been a long while since I posted on my blog. [...] ]]></description><content:encoded><![CDATA[<div class="paragraph">&nbsp;Hey, it;s been a long while since I posted on my blog.<br />&nbsp;I continue having amazing guests on my podcast via my YouTube channel once to twice a week.<br />...</div>]]></content:encoded></item>
```

Points worth knowing about the format:

- **The post body appears twice.** `<description>` holds a plain-text summary cut at roughly 500 characters and marked with a trailing ` [...] `, while `content:encoded` holds the complete body as HTML. The crate keeps both: `summary` for the short form, `content_html` and `content_text` for the full one.
- **The summary is not HTML.** It is the post body with the markup stripped, so it runs together without line breaks. Use `content_text` when you want the paragraph structure.
- **The `<guid>` is the permalink.** Weebly writes the post's URL into both `<link>` and `<guid>`, and sets `isPermaLink="false"` anyway. The crate exposes the GUID verbatim and offers `permalink()` and `slug()`.
- **Every post has a category.** The `<category>` element always holds a value, with `Uncategorized` as the placeholder on sites that have not set one up. `Category::is_uncategorized()` reports that case.
- **There is no author.** Unlike Letterboxd's `dc:creator`, Weebly writes no author element, so the author of a post is not in the feed.
- **The channel title is `<site> - <blog>`.** Weebly writes the site name and the blog name joined by ` - `, which is why `site_name()` and `blog_name()` split on that separator rather than using the title verbatim.
- **The markup is Weebly-specific.** Post bodies use `<div class="paragraph">` for prose, `<div class="wsite-multicol-table-wrap">` for multi-column layouts, `<div class="wsite-image">` for images and `<div class="wsite-youtube">` for embedded videos. The plain-text conversion treats `div`, `p`, `table`, `tr` and `br` as block-level, so columns come out as separate lines.
- **Non-breaking spaces are everywhere.** Weebly uses `&nbsp;` liberally at the start of lines and for indentation, so the plain-text conversion normalises them to ordinary spaces.
- **The feed is a rolling window.** Weebly serves the most recent posts only — 25 in the two captured feeds — and there is no pagination in the feed itself. A blog's complete history lives in its archive pages, which this crate does not parse.
- **A blog can be reached at more than one path.** The canonical feed is `<site>.weebly.com/<blog path>/feed`, but the same feed is also served under the site's numbered page path, such as `<site>.weebly.com/2/feed`. Both work and both are covered by the client.

## What this crate does not cover

The library is scoped to the RSS feed. It does not scrape Weebly's HTML pages:

| Route | Content | Supported |
|---|---|---|
| `<site>.weebly.com/<blog>/feed` | RSS 2.0 feed | yes — the whole point of this crate |
| `<site>.weebly.com/<blog>` | HTML: the blog page | no |
| `<site>.weebly.com/<blog>/archive` | HTML: the post archive, paginated | no |
| `<site>.weebly.com/<blog>/<post>` | HTML: a post and its comments | no |
| `<site>.weebly.com/2/feed` | RSS 2.0 feed, numbered page path | yes — it is the same feed |

This matters most for history. The feed carries only the most recent posts, so a blog's complete archive has to come from its archive pages.

## Project layout

The library follows a strict one-item-per-file convention: one type or one function per file, with a `mod.rs` that only re-exports. This keeps each file small and makes the public surface easy to read from the directory listing.

```text
src/
├── lib.rs                        # crate root and public re-exports
├── error.rs                      # WeeblyError
├── result.rs                     # Result<T>
├── bin/
│   ├── wbrss.rs                  # overview binary
│   └── wbrss_summary.rs          # summary binary
├── client/
│   ├── mod.rs
│   ├── build_feed_url.rs         # build_feed_url
│   ├── build_site_url.rs         # build_site_url
│   ├── default_timeout.rs        # DEFAULT_TIMEOUT
│   ├── fetch_report.rs           # FetchReport
│   ├── validate_site.rs          # validate_site
│   ├── weebly_base_url.rs        # WEBLY_BASE_URL
│   └── weebly_client.rs          # WeeblyClient
├── parse/
│   ├── mod.rs
│   ├── parse_feed.rs             # parse_feed
│   └── parse_item.rs             # parse_item
├── types/
│   ├── mod.rs
│   ├── blog_post.rs              # BlogPost, PostLink, PostImage
│   ├── category.rs               # Category
│   ├── published_date.rs         # PublishedDate
│   └── weebly_feed.rs            # WeeblyFeed
└── util/
    ├── mod.rs
    ├── collapse_whitespace.rs    # collapse_whitespace
    ├── extract_images.rs         # extract_images
    ├── extract_links.rs          # extract_links
    └── html_to_text.rs           # html_to_text
```

The `tests/` directory holds two integration test crates and the captured feeds:

```text
tests/
├── fixtures/                     # 2 real captured Weebly feeds
│   ├── maerenfroespeaker.xml
│   └── thesunshining.xml
├── live_api.rs                   # live tests, enabled with WEBLY_LIVE_TESTS=1
└── parse_fixtures.rs             # parses every fixture
```

The fixture test parses both captures and checks the invariants that hold across real feeds:

```console
$ cargo test --test parse_fixtures -- --nocapture
parsed 2 fixtures: 50 posts, 8 categories
  links: 155, images: 28, truncated summaries: 50, with body: 38
```

The live test is skipped by default so the suite does not depend on the network. Enable it with:

```console
$ WEBLY_LIVE_TESTS=1 cargo test --test live_api -- --nocapture
maerenfroespeaker: 25 posts, 13 with body, site "Mae Renfroe", blog "Blog"
missing site reported as: weebly site not found: zzz-not-a-real-weebly-site-9x8y7z
thesunshining/2: 25 posts, blog "SunShining Blog"
```

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
