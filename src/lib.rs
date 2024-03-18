#![allow(unused_imports)]

pub(crate) use {
    crate::util::*, ci_provider::CIProvider,
    config::commands::locate_failure_log::locate_failure_log, config::Config,
};

pub(crate) use {
    anyhow::{bail, Context, Result},
    clap::{
        builder::styling::{AnsiColor, Effects, Styles},
        *,
    },
    config::commands,
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Serialize},
    std::{
        borrow, env,
        error::Error,
        fmt, fs, io,
        marker::PhantomData,
        path::{Path, PathBuf},
        process::{Command, ExitCode},
        sync::OnceLock,
    },
    strum::*,
};
// Imports for the Gitlab API v3
pub(crate) use gitlab::{
    api::{
        self,
        issues::ProjectIssues,
        projects::{self, issues, jobs},
        Query,
    },
    Gitlab,
};

/// Module containing macros related to protocol words.
pub mod macros {
    #[macro_export]
    // These macros are needed because the normal ones panic when there's a broken pipe.
    // This is especially problematic for CLI tools that are frequently piped into `head` or `grep -q`
    macro_rules! pipe_println {
        () => (print!("\n"));
        ($fmt:expr) => ({
            writeln!(io::stdout(), $fmt)
        });
        ($fmt:expr, $($arg:tt)*) => ({
            writeln!(io::stdout(), $fmt, $($arg)*)
        })
    }
    pub use pipe_println;

    #[macro_export]
    macro_rules! pipe_print {
        () => (print!("\n"));
        ($fmt:expr) => ({
            write!(io::stdout(), $fmt)
        });
        ($fmt:expr, $($arg:tt)*) => ({
            write!(io::stdout(), $fmt, $($arg)*)
        })
    }

    pub use pipe_print;
}

pub mod ci_provider;
pub mod config;
pub mod err_parse;
pub mod issue;
pub mod util;

pub use crate::run::run;
pub mod run;
