//! Contains the ErrorLog struct describing a failed job log from GitHub Actions.
use octocrab::models::{
    workflows::{Job, Step},
    JobId,
};

use super::JobLog;

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

    /// Returns the logs as a string
    pub fn logs_as_str(&self) -> String {
        let mut logs = String::new();
        for log in &self.failed_step_logs {
            logs.push_str(log.contents());
        }
        logs
    }
}

#[derive(Debug)]
pub struct StepErrorLog {
    pub step_name: String,
    pub contents: String,
}

impl StepErrorLog {
    pub fn new(step_name: String, error_log: String) -> Self {
        StepErrorLog {
            step_name,
            contents: error_log,
        }
    }

    pub fn contents(&self) -> &str {
        self.contents.as_str()
    }
}

pub fn repo_url_to_job_url(repo_url: &str, run_id: &str, job_id: &str) -> String {
    let run_url = repo_url_to_run_url(repo_url, run_id);
    run_url_to_job_url(&run_url, job_id)
}

pub fn repo_url_to_run_url(repo_url: &str, run_id: &str) -> String {
    format!("{repo_url}/actions/runs/{run_id}")
}

pub fn run_url_to_job_url(run_url: &str, job_id: &str) -> String {
    format!("{run_url}/job/{job_id}")
}

pub fn distance_to_other_issues(
    issue_body: &str,
    other_issues: &[octocrab::models::issues::Issue],
) -> usize {
    let other_issue_bodies: Vec<String> = other_issues
        .iter()
        .map(|issue| issue.body.as_deref().unwrap_or_default().to_string())
        .collect();

    crate::issue::similarity::issue_text_similarity(issue_body, &other_issue_bodies)
}

/// Logs the job error logs to the info log in a readable summary
pub fn log_info_downloaded_job_error_logs(job_error_logs: &[JobErrorLog]) {
    log::info!("Got {} job error log(s)", job_error_logs.len());
    for log in job_error_logs {
        log::info!(
            "\n\
                        \tName: {name}\n\
                        \tJob ID: {job_id}\
                        {failed_steps}",
            name = log.job_name,
            job_id = log.job_id,
            failed_steps = log
                .failed_step_logs
                .iter()
                .fold(String::new(), |acc, step| {
                    format!(
                        "{acc}\n\t Step: {step_name} | Log length: {log_len}",
                        acc = acc,
                        step_name = step.step_name,
                        log_len = step.contents().len()
                    )
                })
        );
    }
}

/// Extracts the error logs from the logs, failed jobs and failed steps
/// and returns a vector of [JobErrorLog].
///
/// The extraction is performed by taking the name of each failed step in each failed job
/// and searching for a log with a name that contains both the job name and the step name.
///
/// If a log is found, it is added to the [JobErrorLog] struct.
///
/// If a log is not found, an error is logged and the function continues.
pub fn job_error_logs_from_log_and_failed_jobs_and_steps(
    logs: &[JobLog],
    failed_jobs: &[&Job],
    failed_steps: &[&Step],
) -> Vec<JobErrorLog> {
    let mut job_error_logs: Vec<JobErrorLog> = Vec::new();
    for job in failed_jobs {
        log::info!("Extracting error logs for job: {}", job.name);
        let name = job.name.clone();
        let step_error_logs: Vec<StepErrorLog> =
            find_error_logs_for_job_steps(logs, &name, failed_steps);
        job_error_logs.push(JobErrorLog::new(job.id, name, step_error_logs));
    }
    job_error_logs
}

/// Finds the error logs for each step in the job and returns a vector of [StepErrorLog].
fn find_error_logs_for_job_steps(
    logs: &[JobLog],
    job_name: &str,
    steps: &[&Step],
) -> Vec<StepErrorLog> {
    steps
        .iter()
        .filter_map(|step| {
            let step_name = step.name.clone();
            let job_lob = match find_error_log(logs, job_name, &step_name) {
                Some(log) => log,
                None => {
                    log::error!("No log found for failed step: {step_name} in job: {job_name}. Continuing...");
                    return None;
                }
            };
            Some(StepErrorLog::new(step_name, job_lob.content.clone()))
        })
        .collect()
}

/// Finds the error log in the logs that contains the job name and the step name.
/// If no log is found, None is returned.
fn find_error_log<'j>(logs: &'j [JobLog], job_name: &str, step_name: &str) -> Option<&'j JobLog> {
    logs.iter()
        .find(|log| log.name.contains(step_name) && log.name.contains(job_name))
}
