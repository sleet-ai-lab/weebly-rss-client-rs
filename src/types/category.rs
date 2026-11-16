//! The category a blog post was filed under.

use serde::{Deserialize, Serialize};

/// The category a Weebly blog post was filed under.
///
/// Weebly writes the category into a single `<category>` element. Sites
/// without a category structure still emit one, with the placeholder
/// `Uncategorized`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Category {
    /// The category name, exactly as Weebly wrote it.
    pub name: String,
}

impl Category {
    /// Creates a category from its name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the category name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns `true` when this is Weebly's `Uncategorized` placeholder.
    pub fn is_uncategorized(&self) -> bool {
        self.name.eq_ignore_ascii_case("uncategorized")
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
