use crate::*;
use crate::{
    config::commands::locate_failure_log::logfile_path_from_str, err_parse::LOGFILE_MAX_LEN,
};

use self::util::YoctoFailureKind;

pub mod util;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct YoctoError {
    summary: String,
    kind: YoctoFailureKind,
    logfile: Option<YoctoFailureLog>,
}

impl YoctoError {
    pub fn new(summary: String, kind: YoctoFailureKind, logfile: Option<YoctoFailureLog>) -> Self {
        YoctoError {
            summary,
            kind,
            logfile,
        }
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
    pub fn kind(&self) -> YoctoFailureKind {
        self.kind
    }
    pub fn logfile(&self) -> Option<&YoctoFailureLog> {
        self.logfile.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct YoctoFailureLog {
    pub name: String,
    pub contents: String,
}

/// Parse a log from a Yocto build and return a [YoctoError] containing error
/// summary, error kind, and logfile contents if it exists and is not too large.
pub fn parse_yocto_error(log: &str) -> anyhow::Result<YoctoError> {
    let error_summary = util::yocto_error_summary(log)?;
    log::debug!(
        "Yocto error before trimming just recipe failures: \n{}",
        error_summary
    );

    let error_summary = util::trim_trailing_just_recipes(&error_summary)?;
    log::debug!("Yocto error: \n{}", error_summary);

    // Find the kind of yocto failure in the string e.g. this would be `do_fetch`
    // ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616

    // Find the line with the `Logfile of failure stored in` and get the path
    let log_file_line = util::find_yocto_failure_log_str(&error_summary)?;
    let path = first_path_from_str(log_file_line)?;
    let fname = path
        .file_stem()
        .with_context(|| format!("No file stem in {path:?}"))?
        .to_str()
        .context("Could not convert file stem to string")?;
    let yocto_failure_kind = match YoctoFailureKind::parse_from_logfilename(fname) {
        Ok(kind) => kind,
        Err(e) => {
            log::error!("{e}");
            log::warn!("Could not determine yocto failure kind, continuing with default kind");
            YoctoFailureKind::default()
        }
    };

    let failure_log: Option<YoctoFailureLog> = match logfile_path_from_str(path.to_str().unwrap()) {
        Ok(p) => {
            let contents = fs::read_to_string(p)?;
            if contents.len() > LOGFILE_MAX_LEN {
                log::warn!("Logfile of yocto failure exceeds maximum length of {LOGFILE_MAX_LEN}. It will not be added to the issue body.");
                None
            } else {
                Some(YoctoFailureLog {
                    name: fname.to_owned(),
                    contents,
                })
            }
        }
        Err(e) => {
            log::trace!("{e}");
            log::error!("Logfile from error summary does not exist at: {path:?}");
            log::warn!("Continuing without attempting to attach logfile to issue");
            None
        }
    };

    let yocto_error = YoctoError {
        summary: error_summary,
        kind: yocto_failure_kind,
        logfile: failure_log,
    };

    Ok(yocto_error)
}
