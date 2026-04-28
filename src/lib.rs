//! Library crate exposing the internals of the `rw` CLI so integration tests
//! and future external consumers can reach them.
//!
//! `src/main.rs` is the binary entry point and is intentionally thin — all
//! logic lives in the modules declared here.

pub mod api;
pub mod auth_cache;
pub mod cli;
pub mod commands;
pub mod config;
pub mod http;
pub mod jsonapi;
pub mod migration;
pub mod output;
pub mod prompt;
pub mod version_check;
