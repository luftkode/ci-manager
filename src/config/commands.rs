//! The `commands` module contains the subcommands for the `gh-workflow-parser` CLI.

/// The maximum Levenshtein distance for issues to be considered similar.
///
/// Determined in tests at the bottom of this file.
pub const LEVENSHTEIN_THRESHOLD: usize = 100;

use crate::*;

pub mod locate_failure_log;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create an issue from a failed CI run
    CreateIssueFromRun {
        /// The repository to parse
        #[arg(long, value_hint = ValueHint::Url)]
        repo: String,
        /// The workflow run ID
        #[arg(short = 'r', long)]
        run_id: String,
        /// The issue label
        #[arg(short, long)]
        label: String,
        /// The kind of workflow (e.g. Yocto)
        #[arg(short, long)]
        kind: WorkflowKind,
        /// Title of the issue
        #[arg(short, long)]
        title: String,
        /// Don't create the issue if a similar issue already exists
        #[arg(short, long, default_value_t = true)]
        no_duplicate: bool,
    },

    /// Locate the specific failure log in a failed build/test/other
    LocateFailureLog {
        /// The kind of CI step (e.g. Yocto)
        #[arg(short, long)]
        kind: StepKind,
        /// Log file to search for the failure log (e.g. log.txt or read from stdin)
        /// File to operate on (if not provided, reads from stdin)
        #[arg(short = 'f', long, value_hint = ValueHint::FilePath)]
        input_file: Option<PathBuf>,
    },
}

/// The kind of workflow (e.g. Yocto)
#[derive(ValueEnum, Display, Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorkflowKind {
    Yocto,
    Other,
}

/// The kind of step in CI, e.g. Yocto, Pytest, Pre-commit, Docker build, etc.
///
/// This is used to take highly specific actions based on the kind of CI step that failed.
/// e.g. if a Yocto build fails, we can locate the specific log of the failed task and
/// create a GitHub issue with the log attached, or pass it to another tool for uploading it etc.
#[derive(ValueEnum, Display, EnumString, Copy, Clone, Debug, PartialEq, Eq)]
pub enum StepKind {
    Yocto,
    Other,
}
