//! This library provides miscellaneous utility functions for unit and
//! integration tests in Rust.

pub mod assert_panic;

mod exec;
pub use exec::*;

mod server;
pub use server::*;

mod temp;
pub use temp::*;

mod testdata;
pub use testdata::*;

pub mod text;

pub mod diff;
