//! This library provides miscellaneous utility functions for unit and
//! integration tests in Rust.
//!
//! # Crate Features
//!
//! Most crate features are enabled by default. The only exception are
//! overly-specific features that are costly to build.
//!
//! Not enabled by default:
//!
//! - `server`:
//!   - Enables support for [`TestServer`].
//!   - Includes the [`warp`] and [`tokio`] re-exports.
//!
//! All other features are enabled by default:
//!
//! - `diff`: support for the text diff functions.
//! - `exec`: support for the binary execution functions.
//! - `temp`: helpers for managing temporary directories and files.
//! - `testdata`: support for file based tests.
//! - `text`: text utility functions.
//!
//! To disable the default features and opt into specific ones, change the
//! dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tux = { version = "...", default-features = false, features = ["..."] }
//! ```
//!

pub mod assert_panic;

#[cfg(feature = "exec")]
mod exec;

#[cfg(feature = "exec")]
pub use exec::*;

#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
pub use server::*;

#[cfg(feature = "temp")]
mod temp;

#[cfg(feature = "temp")]
pub use temp::*;

#[cfg(feature = "testdata")]
mod testdata;

#[cfg(feature = "testdata")]
pub use testdata::*;

#[cfg(feature = "text")]
pub mod text;

#[cfg(feature = "diff")]
pub mod diff;
