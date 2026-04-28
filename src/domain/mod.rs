//! Domain types shared across the CLI: stages, slugs.

pub mod slug;
pub mod stage;

pub use slug::validate_slug;
pub use stage::{Stage, WorkOsConfig};
