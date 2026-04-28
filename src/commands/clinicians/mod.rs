//! Clinician commands.
//!
//! Each public command lives in its own submodule; shared types and the
//! legacy reqwest helpers used by the migration-pending commands live in
//! `data`, `output`, and `client`.

mod assign;
mod client;
mod data;
mod enable;
mod grant;
mod output;
mod prepare;
mod register;
mod show;
mod update;

#[cfg(test)]
mod testing;

pub use assign::assign;
pub use enable::{disable, enable};
pub use grant::grant;
pub use prepare::prepare;
pub use register::register;
pub use show::show;
pub use update::update;
