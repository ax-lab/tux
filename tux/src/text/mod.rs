//! General text utilities for tests.
//!
//! This module is enabled by the `text` feature (enabled by default).

mod join_lines;
pub use join_lines::*;

mod lines;
pub use lines::*;

mod trim;
pub use trim::*;
