//! Parsing error messages from the Yocto and other workflows
use crate::*;
use crate::{config::commands::WorkflowKind, err_parse::yocto::util::YoctoFailureKind};

use self::yocto::YoctoError;

/// Maximum size of a logfile we'll add to the issue body
///
/// The maximum size of a GitHub issue body is 65536
pub const LOGFILE_MAX_LEN: usize = 5000;

pub mod yocto;

#[derive(Debug)]
pub enum ErrorMessageSummary {
    Yocto(YoctoError),
    Other(String),
}

impl ErrorMessageSummary {
    pub fn summary(&self) -> &str {
        match self {
            ErrorMessageSummary::Yocto(err) => err.summary(),
            ErrorMessageSummary::Other(o) => o.as_str(),
        }
    }
    pub fn log(&self) -> Option<&str> {
        match self {
            ErrorMessageSummary::Yocto(err) => err.logfile().map(|log| log.contents.as_str()),
            ErrorMessageSummary::Other(_) => None, // Does not come with a log file
        }
    }
    pub fn logfile_name(&self) -> Option<&str> {
        match self {
            ErrorMessageSummary::Yocto(err) => err.logfile().map(|log| log.name.as_str()),
            ErrorMessageSummary::Other(_) => None, // Does not come with a log file
        }
    }

    pub fn failure_label(&self) -> Option<String> {
        match self {
            ErrorMessageSummary::Yocto(err) => Some(err.kind().to_string()),
            ErrorMessageSummary::Other(_) => None,
        }
    }
}

pub fn parse_error_message(
    err_msg: &str,
    workflow: WorkflowKind,
) -> anyhow::Result<ErrorMessageSummary> {
    let trim_timestamp = Config::global().trim_timestamp();
    let trim_ansi_codes = Config::global().trim_ansi_codes();
    log::debug!("Trim timestamp: {trim_timestamp}, Trim ansi codes: {trim_ansi_codes}");

    let err_msg = if trim_timestamp {
        log::info!("Trimming timestamp and ansi codes from the log");
        remove_timestamp_prefixes(err_msg)
    } else {
        err_msg.into()
    };
    let err_msg = if trim_ansi_codes {
        log::info!("Trimming ansi codes from the log");
        remove_ansi_codes(&err_msg)
    } else {
        err_msg
    };
    let err_msg = err_msg.to_string();

    let err_msg = match workflow {
        WorkflowKind::Yocto => {
            ErrorMessageSummary::Yocto(yocto::parse_yocto_error(&err_msg).unwrap_or_else(|e| {
                log::warn!("Failed to parse Yocto error, returning error message as is: {e}");
                YoctoError::new(err_msg, YoctoFailureKind::default(), None)
            }))
        }
        WorkflowKind::Other => ErrorMessageSummary::Other(err_msg.to_string()),
    };
    Ok(err_msg)
}
