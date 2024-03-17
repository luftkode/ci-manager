#![allow(dead_code, unused_imports)]
use std::fmt::Display;

/// Re-export some common utilities for system tests
pub use assert_cmd::prelude::*; // Add methods on commands
pub use assert_cmd::Command;
pub use assert_fs::fixture::ChildPath;
// Get the methods for the Commands struct
pub use assert_fs::prelude::*;
pub use assert_fs::TempDir;
pub use predicates::prelude::*; // Used for writing assertions // Create temporary directories
pub use pretty_assertions::assert_eq as pretty_assert_eq;
pub use pretty_assertions::assert_ne as pretty_assert_ne;
pub use pretty_assertions::assert_str_eq as pretty_assert_str_eq;

pub use std::{error::Error, fs, process::Output};
