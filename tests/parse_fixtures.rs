//! Integration test that parses every captured Weebly feed fixture.
//!
//! The fixtures in `tests/fixtures/` are real, unmodified captures of
//! `https://<site>.weebly.com/<blog path>/feed`. They are used to prove that
//! the parser copes with the shapes Weebly actually emits: posts with full
//! HTML bodies, truncated summaries, categories, embedded images and
//! multi-column layouts.

use std::fs;
use std::path::PathBuf;

use weebly_rss_client_rs::{parse_feed, WeeblyFeed};

/// The directory holding the captured feeds.
const FIXTURE_DIR: &str = "tests/fixtures";

/// Counts the `<item>` elements in a raw feed document.
fn count_raw_items(xml: &str) -> usize {
    xml.matches("<item>").count()
}

/// Returns the fixture file names in a deterministic order.
fn fixture_paths() -> Vec<PathBuf> {
    let entries = fs::read_dir(FIXTURE_DIR)
        .unwrap_or_else(|error| panic!("cannot read {FIXTURE_DIR}: {error}"));

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xml"))
        .collect();

    paths.sort();
    assert!(!paths.is_empty(), "no xml fixtures found in {FIXTURE_DIR}");

    paths
}

/// Asserts the invariants that hold for every parsed feed.
fn assert_feed_invariants(name: &str, feed: &WeeblyFeed, raw_items: usize) {
    assert!(
        !feed.title.is_empty(),
        "{name}: the channel title must not be empty"
    );
    assert!(
        feed.title.contains(" - "),
        "{name}: the channel title must have the \"<site> - <blog>\" shape, got {:?}",
        feed.title
    );
    assert!(
        feed.link.contains("weebly.com"),
        "{name}: the channel link must point at weebly.com, got {:?}",
        feed.link
    );
    assert_eq!(
        feed.generator.as_deref(),
        Some("Weebly"),
        "{name}: the generator must be Weebly"
    );
    assert!(
        feed.pub_date.is_some(),
        "{name}: the channel must carry a publication date"
    );

    let site_name = feed.site_name();
    assert!(
        !site_name.is_empty(),
        "{name}: the site name must not be empty"
    );
    assert_ne!(
        site_name,
        feed.title.as_str(),
        "{name}: the site name must have been split off the channel title"
    );

    let blog_name = feed.blog_name();
    assert!(
        !blog_name.is_empty(),
        "{name}: the blog name must not be empty"
    );
    assert_ne!(
        blog_name,
        feed.title.as_str(),
        "{name}: the blog name must have been split off the channel title"
    );

    let subdomain = feed
        .site_subdomain()
        .unwrap_or_else(|| panic!("{name}: the site subdomain must be present"));
    assert!(
        !subdomain.is_empty(),
        "{name}: the site subdomain must not be empty"
    );

    assert_eq!(
        feed.items.len(),
        raw_items,
        "{name}: the parsed item count must match the number of <item> elements"
    );

    for post in &feed.items {
        assert!(
            !post.guid.is_empty(),
            "{name}: every post must have a non-empty guid"
        );
        assert!(
            !post.title.is_empty(),
            "{name}: every post ({}) must have a non-empty title",
            post.guid
        );
        assert!(
            post.link.starts_with("https://"),
            "{name}: the link of {} must be an https url, got {:?}",
            post.guid,
            post.link
        );
        assert!(
            post.pub_date.is_some(),
            "{name}: the post {} must have a publication date",
            post.guid
        );
        assert!(
            post.category.is_some(),
            "{name}: the post {} must have a category",
            post.guid
        );
        assert!(
            !post.slug().is_empty(),
            "{name}: the post {} must have a slug",
            post.guid
        );
    }
}

/// Counts the `<item>` elements of every fixture and parses them all.
#[test]
fn every_fixture_parses_into_a_consistent_feed() {
    let paths = fixture_paths();

    let mut total_items = 0usize;
    let mut total_links = 0usize;
    let mut total_images = 0usize;
    let mut total_categories = 0usize;
    let mut truncated_summaries = 0usize;
    let mut posts_with_content = 0usize;

    let mut saw_truncated_summary = false;
    let mut saw_image = false;
    let mut saw_link = false;
    let mut saw_category = false;
    let mut saw_uncategorized = false;

    for path in &paths {
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        let xml = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        let raw_items = count_raw_items(&xml);
        assert!(
            raw_items > 0,
            "{name}: the fixture contains no <item> elements"
        );

        let feed = parse_feed(&xml)
            .unwrap_or_else(|error| panic!("{name}: the feed does not parse: {error}"));

        assert_feed_invariants(&name, &feed, raw_items);

        for post in &feed.items {
            total_links += post.links.len();
            total_images += post.images.len();

            if post.is_summary_truncated() {
                saw_truncated_summary = true;
                truncated_summaries += 1;
            }
            if post.content_text.is_some() {
                posts_with_content += 1;
            }
            if !post.images.is_empty() {
                saw_image = true;
            }
            if !post.links.is_empty() {
                saw_link = true;
            }
            if let Some(category) = &post.category {
                saw_category = true;
                if category.is_uncategorized() {
                    saw_uncategorized = true;
                }
            }
        }

        total_categories += feed.categories().len();
        total_items += feed.items.len();
    }

    // The fixtures must really contain data, otherwise the test would pass
    // vacuously.
    assert_eq!(
        paths.len(),
        2,
        "expected 2 fixtures in {FIXTURE_DIR}, found {}",
        paths.len()
    );
    assert!(
        total_items >= 50,
        "expected at least 50 items across the fixtures, got {total_items}"
    );
    assert!(
        total_categories > 0,
        "expected the fixtures to contain categories"
    );
    assert!(
        truncated_summaries > 0,
        "expected the fixtures to contain truncated summaries"
    );
    assert!(
        posts_with_content > 0,
        "expected the fixtures to contain post bodies"
    );
    assert!(
        saw_truncated_summary,
        "expected at least one truncated summary"
    );
    assert!(saw_image, "expected at least one post with an image");
    assert!(saw_link, "expected at least one post with a link");
    assert!(saw_category, "expected at least one post with a category");
    assert!(
        saw_uncategorized,
        "expected at least one Uncategorized post"
    );

    println!(
        "parsed {fixtures} fixtures: {items} posts, {categories} categories",
        fixtures = paths.len(),
        items = total_items,
        categories = total_categories,
    );
    println!(
        "  links: {links}, images: {images}, truncated summaries: {truncated}, with body: {with_body}",
        links = total_links,
        images = total_images,
        truncated = truncated_summaries,
        with_body = posts_with_content,
    );
}

/// The fixture file names are the site names the feeds were captured from.
#[test]
fn fixture_names_are_weebly_subdomains() {
    for path in fixture_paths() {
        let stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();

        assert!(
            !stem.is_empty(),
            "the fixture {} has no name",
            path.display()
        );
        assert!(
            stem.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "the fixture name {stem} is not a plausible Weebly subdomain"
        );
    }
}
