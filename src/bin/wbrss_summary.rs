//! `wbrss-summary` — compact statistics for a Weebly blog's RSS feed.
//!
//! Prints aggregate information about the feed — post counts, a per-year
//! breakdown, the most-used categories and the links that appear in the
//! posts — without listing every individual post.
//!
//! ```text
//! wbrss-summary [--json] <site> [<blog path>]
//! ```

use std::collections::BTreeMap;
use std::time::Duration;

use clap::Parser;
use serde::Serialize;
use weebly_rss_client_rs::{WeeblyClient, WeeblyFeed};

/// The number of posts published in one year.
#[derive(Debug, Serialize)]
struct YearCount {
    year: i32,
    count: usize,
}

/// A category and how many posts it holds.
#[derive(Debug, Serialize)]
struct CategoryCount {
    category: String,
    count: usize,
}

/// The most linked-to host in the post bodies.
#[derive(Debug, Serialize)]
struct HostCount {
    host: String,
    count: usize,
}

/// The full statistics summary.
#[derive(Debug, Serialize)]
struct Summary {
    site_name: String,
    blog_name: String,
    subdomain: Option<String>,
    feed_link: String,
    generator: Option<String>,
    total_posts: usize,
    total_categories: usize,
    total_links: usize,
    total_images: usize,
    truncated_summaries: usize,
    posts_with_content: usize,
    average_content_length: Option<usize>,
    years: Vec<YearCount>,
    categories: Vec<CategoryCount>,
    top_hosts: Vec<HostCount>,
}

fn build_summary(feed: &WeeblyFeed) -> Summary {
    let mut year_counts: BTreeMap<i32, usize> = BTreeMap::new();
    let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut host_counts: BTreeMap<String, usize> = BTreeMap::new();

    let mut total_links = 0usize;
    let mut total_images = 0usize;
    let mut truncated_summaries = 0usize;
    let mut posts_with_content = 0usize;
    let mut content_length_sum = 0usize;

    for post in &feed.items {
        if let Some(date) = post.pub_date {
            *year_counts.entry(date.year()).or_insert(0) += 1;
        }

        if let Some(category) = &post.category {
            *category_counts.entry(category.name.clone()).or_insert(0) += 1;
        }

        total_links += post.links.len();
        total_images += post.images.len();

        if post.is_summary_truncated() {
            truncated_summaries += 1;
        }

        if let Some(text) = post.content_text.as_deref() {
            posts_with_content += 1;
            content_length_sum += text.len();
        }

        for link in &post.links {
            if let Some(host) = host_of(&link.url) {
                *host_counts.entry(host).or_insert(0) += 1;
            }
        }
    }

    let years = year_counts
        .into_iter()
        .map(|(year, count)| YearCount { year, count })
        .collect();

    let mut categories: Vec<CategoryCount> = category_counts
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();
    categories.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.category.cmp(&b.category))
    });

    let mut top_hosts: Vec<HostCount> = host_counts
        .into_iter()
        .map(|(host, count)| HostCount { host, count })
        .collect();
    top_hosts.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.host.cmp(&b.host)));
    top_hosts.truncate(10);

    let average_content_length = content_length_sum
        .checked_div(posts_with_content)
        .filter(|_| posts_with_content > 0);

    Summary {
        site_name: feed.site_name().to_string(),
        blog_name: feed.blog_name().to_string(),
        subdomain: feed.site_subdomain().map(|name| name.to_string()),
        feed_link: feed.link.clone(),
        generator: feed.generator.clone(),
        total_posts: feed.items.len(),
        total_categories: feed.categories().len(),
        total_links,
        total_images,
        truncated_summaries,
        posts_with_content,
        average_content_length,
        years,
        categories,
        top_hosts,
    }
}

/// Returns the host of a URL, when it has one.
fn host_of(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    let host = rest.split(['/', '?', '#']).next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

fn print_summary(summary: &Summary) {
    println!("Site:      {}", summary.site_name);
    println!("Blog:      {}", summary.blog_name);
    println!(
        "Subdomain: {}",
        summary.subdomain.as_deref().unwrap_or("(unknown)")
    );
    println!("Feed:      {}", summary.feed_link);
    if let Some(generator) = &summary.generator {
        println!("Generator: {generator}");
    }
    println!();

    println!("Posts:");
    println!("  Total:            {}", summary.total_posts);
    println!("  With body:        {}", summary.posts_with_content);
    println!("  Truncated:        {}", summary.truncated_summaries);
    println!("  Categories:       {}", summary.total_categories);
    println!("  Links:            {}", summary.total_links);
    println!("  Images:           {}", summary.total_images);
    match summary.average_content_length {
        Some(length) => println!("  Avg body length:  {length} chars"),
        None => println!("  Avg body length:  n/a"),
    }
    println!();

    println!("Posts per year:");
    if summary.years.is_empty() {
        println!("  (none)");
    }
    for year in &summary.years {
        println!("  {}: {}", year.year, year.count);
    }
    println!();

    println!("Categories:");
    if summary.categories.is_empty() {
        println!("  (none)");
    }
    for category in &summary.categories {
        println!("  {:>4}x  {}", category.count, category.category);
    }
    println!();

    println!("Most linked hosts:");
    if summary.top_hosts.is_empty() {
        println!("  (none)");
    }
    for host in &summary.top_hosts {
        println!("  {:>4}x  {}", host.count, host.host);
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "wbrss-summary",
    version,
    about = "Print statistics summary for a Weebly blog's RSS feed"
)]
struct Args {
    /// Weebly site name, for example `maerenfroespeaker`.
    site: String,

    /// Path of the blog on the site, for example `blog`.
    ///
    /// Omit it for a blog at the root of the site.
    blog_path: Option<String>,

    /// Output the summary as pretty JSON.
    #[arg(long)]
    json: bool,

    /// HTTP timeout in seconds.
    #[arg(long, default_value_t = 30)]
    timeout: u64,

    /// Custom User-Agent header.
    #[arg(long)]
    user_agent: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut client = WeeblyClient::new().with_timeout(Duration::from_secs(args.timeout));
    if let Some(user_agent) = args.user_agent.clone() {
        client = client.with_user_agent(user_agent);
    }

    let feed: WeeblyFeed = match client
        .fetch_feed(&args.site, args.blog_path.as_deref())
        .await
    {
        Ok(feed) => feed,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    };

    let summary = build_summary(&feed);

    if args.json {
        match serde_json::to_string_pretty(&summary) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    print_summary(&summary);
}
