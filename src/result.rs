//! The result alias used across the Weebly RSS client.

use crate::error::WeeblyError;

/// A convenient result alias for this crate.
pub type Result<T> = std::result::Result<T, WeeblyError>;
