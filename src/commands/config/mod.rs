//! `rw config` subcommands.
//!
//! Each top-level subcommand (`profile`, `updates`, `default`) lives in its
//! own file. Interactive prompt helpers are in `prompts`.

mod default;
mod profile;
mod prompts;
mod updates;

pub use default::{default_get, default_list, default_rm, default_set};
pub use profile::{
    profile_add, profile_auth, profile_list, profile_rm, profile_set, profile_show, profile_use,
};
pub use updates::{updates_disable, updates_enable, updates_show};
