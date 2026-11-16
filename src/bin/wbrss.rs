//! `wbrss` — a general-purpose viewer for a Weebly blog's RSS feed.
//!
//! Fetches the feed and prints a human-readable overview of the blog, its
//! posts, their categories and the links they carry.
//!
//! ```text
//! wbrss [--limit <N>] [--full] [--json] <site> [<blog path>]
//! ```

use std::time::Duration;

use clap::Parser;
use weebly_rss_client_rs::{BlogPost, WeeblyClient, WeeblyFeed};

/// Truncate `text` to at most `max` characters, appending an ellipsis.
fn truncate(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(max).collect();
    out.push('…');
    out
}

fn print_post(post: &BlogPost, show_body: bool) {
    let date = post
        .pub_date
        .map(|date| date.to_rfc2822())
        .unwrap_or_else(|| "—".to_string());
    let category = post
        .category
        .as_ref()
        .map(|category| category.name.as_str())
        .unwrap_or("—");

    println!("  • {}", post.title);
    println!("      date:     {date}");
    println!("      category: {category}");
    println!("      link:     {}", post.permalink());

    if show_body {
        if let Some(text) = post.content_text.as_deref() {
            println!("      body:     {}", truncate(text, 300));
        }
    }

    if !post.links.is_empty() {
        println!("      links:    {}", post.links.len());
    }
    if !post.images.is_empty() {
        println!("      images:   {}", post.images.len());
    }
}

fn print_overview(feed: &WeeblyFeed, limit: usize, show_body: bool) {
    println!("Site:      {}", feed.site_name());
    println!("Blog:      {}", feed.blog_name());
    println!(
        "Subdomain: {}",
        feed.site_subdomain().unwrap_or("(unknown)")
    );
    println!("Feed:      {}", feed.link);
    println!();

    println!("Counts:");
    println!("  Total posts: {}", feed.items.len());
    println!("  Categories:  {}", feed.categories().len());
    println!();

    let categories = feed.categories();
    if !categories.is_empty() {
        println!("Categories:");
        for category in &categories {
            let count = feed.posts_in_category(&category.name).len();
            println!("  {category} ({count})");
        }
        println!();
    }

    println!("Posts ({} shown):", feed.items.len().min(limit));
    if feed.items.is_empty() {
        println!("  (none)");
    }
    for post in feed.items.iter().take(limit) {
        print_post(post, show_body);
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "wbrss",
    version,
    about = "View a Weebly blog's RSS feed: posts, categories and links"
)]
struct Args {
    /// Weebly site name, for example `maerenfroespeaker`.
    site: String,

    /// Path of the blog on the site, for example `blog`.
    ///
    /// Omit it for a blog at the root of the site.
    blog_path: Option<String>,

    /// Maximum number of posts to show.
    #[arg(long, default_value_t = 20)]
    limit: usize,

    /// Also print the body text of each post.
    #[arg(long)]
    full: bool,

    /// Output the raw feed as pretty JSON.
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

    if args.json {
        match serde_json::to_string_pretty(&feed) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    print_overview(&feed, args.limit, args.full);
}
