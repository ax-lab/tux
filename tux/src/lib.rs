//! This library provides miscellaneous utility functions for the testing
//! harness.
//!
//! # Note on error handling
//!
//! Since this library is designed to be used in a test harness, all fallible
//! code handle errors by panicking. This minimizes the code in unit tests by
//! removing the need to have error handling at the call site.

mod temp;
pub use temp::*;

mod server;
pub use server::*;

mod exe;
pub use exe::*;
