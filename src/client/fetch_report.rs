//! The outcome of fetching the feeds of several sites.

use crate::error::WeeblyError;
use crate::types::WeeblyFeed;

/// The outcome of fetching the feeds of several sites.
#[derive(Debug, Default)]
pub struct FetchReport {
    /// The feeds, in the same order as the requested sites.
    pub feeds: Vec<WeeblyFeed>,
    /// The sites that could not be fetched, with the reason.
    pub errors: Vec<(String, WeeblyError)>,
}

impl FetchReport {
    /// Returns the number of feeds that were fetched successfully.
    pub fn success_count(&self) -> usize {
        self.feeds.len() - self.errors.len()
    }

    /// Returns whether every site was fetched successfully.
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty()
    }
}
