//! Contains the ErrorLog struct describing a failed job log from GitHub Actions.
use octocrab::models::JobId;

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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
