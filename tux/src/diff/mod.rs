//! Utilities for computing the difference between sequences.
//!
//! This module is enabled by the `diff` feature (enabled by default).

mod lines;
pub use lines::*;

mod lcs;
use lcs::*;
