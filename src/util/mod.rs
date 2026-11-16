//! Small helpers shared across the crate.

pub mod collapse_whitespace;
pub mod extract_images;
pub mod extract_links;
pub mod html_to_text;

pub use collapse_whitespace::collapse_whitespace;
pub use extract_images::extract_images;
pub use extract_links::extract_links;
pub use html_to_text::html_to_text;
