//! Contains the ErrorLog struct describing a failed job log from GitHub Actions.
use octocrab::models::JobId;
use once_cell::sync::Lazy;
use std::error::Error;

use regex::Regex;

#[derive(Debug)]
pub struct JobErrorLog {
    pub job_id: JobId,
    pub job_name: String,
    pub failed_step_logs: Vec<StepErrorLog>,
}

impl JobErrorLog {
    pub fn new(job_id: JobId, job_name: String, logs: Vec<StepErrorLog>) -> Self {
        JobErrorLog {
            job_id,
            job_name,
            failed_step_logs: logs,
        }
    }
}

#[derive(Debug)]
pub struct StepErrorLog {
    pub step_name: String,
    pub error_log: String,
}

impl StepErrorLog {
    pub fn new(step_name: String, error_log: String) -> Self {
        StepErrorLog {
            step_name,
            error_log,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
