use std::io::Read;

pub mod util;

use crate::{ci_provider::github::util::JobErrorLog, *};
use octocrab::{
    models::{
        issues::Issue,
        workflows::{Conclusion, Job, Run},
        RunId,
    },
    params::{workflows::Filter, State},
    Octocrab, *,
};

use super::util::*;
use anyhow::Result;

pub static GITHUB_CLIENT: OnceLock<GitHub> = OnceLock::new();

pub struct GitHub {
    client: Octocrab,
}

impl GitHub {
    /// Get a reference to the global config
    pub fn get() -> &'static GitHub {
        GITHUB_CLIENT
            .get()
            .expect("GITHUB_CLIENT is not initialized")
    }

    pub fn init() -> Result<()> {
        let github_client = match env::var("GITHUB_TOKEN") {
            Ok(token) => GitHub::new(&token)?,
            Err(e) => {
                log::debug!("{e:?}");
                log::warn!("GITHUB_TOKEN not set, using unauthenticated client");
                Self {
                    client: Octocrab::default(),
                }
            }
        };
        let _ = GITHUB_CLIENT.set(github_client);
        Ok(())
    }

    pub fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_owned())
            .build()?;
        Ok(Self { client })
    }

    pub async fn handle(&self, cmd: &commands::Command) -> Result<()> {
        use commands::Command;
        match cmd {
            Command::CreateIssueFromRun {
                repo,
                run_id,
                label,
                kind,
                no_duplicate,
                title,
            } => {
                let (owner, repo) = repo_to_owner_repo_fragments(repo)?;
                let run_id: u64 = run_id.parse()?;

                let workflow_run = self.workflow_run(&owner, &repo, RunId(run_id)).await?;
                log::debug!("{workflow_run:?}");

                if workflow_run.conclusion != Some("failure".to_string()) {
                    bail!("Expected run from a failed workflow, but workflow did not fail");
                }

                let jobs = self.workflow_run_jobs(&owner, &repo, RunId(run_id)).await?;
                log::debug!("{jobs:?}");

                let failed_jobs = jobs
                    .iter()
                    .filter(|job| job.conclusion == Some(Conclusion::Failure))
                    .collect::<Vec<_>>();

                log::info!(
                    "Found {} failed job(s): {}",
                    failed_jobs.len(),
                    failed_jobs
                        .iter()
                        .map(|j| j.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                failed_jobs.iter().for_each(|job| {
                    log::debug!("{job:?}");
                });

                let failed_steps = failed_jobs
                    .iter()
                    .flat_map(|job| job.steps.iter())
                    .filter(|step| step.conclusion == Some(Conclusion::Failure))
                    .collect::<Vec<_>>();
                log::info!(
                    "Found {} failed step(s): {}",
                    failed_steps.len(),
                    failed_steps
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                failed_steps.iter().for_each(|step| {
                    log::debug!("{step:?}");
                });

                let logs = self.download_job_logs(&owner, &repo, RunId(run_id)).await?;
                log::info!("Downloaded {} logs", logs.len());
                log::info!(
                    "Log names sorted by timestamp:\n{logs}",
                    logs = logs
                        .iter()
                        .map(|log| log.name.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                logs.iter().for_each(|log| {
                    log::debug!("{log:?}");
                });

                let mut job_error_logs: Vec<JobErrorLog> = Vec::new();

                for job in failed_jobs {
                    let name = job.name.clone();
                    let id = job.id.clone();
                    let mut step_error_logs: Vec<util::StepErrorLog> = Vec::new();
                    for steps in &failed_steps {
                        let step_name = steps.name.clone();
                        let step_log = logs
                            .iter()
                            .find(|log| {
                                log.name.contains(steps.name.as_str())
                                    && log.name.contains(job.name.as_str())
                            })
                            .unwrap();
                        step_error_logs
                            .push(util::StepErrorLog::new(step_name, step_log.content.clone()));
                    }
                    job_error_logs.push(JobErrorLog::new(id, name, step_error_logs));
                }

                log::info!("Got {} job error log(s)", job_error_logs.len());
                for log in &job_error_logs {
                    log::info!(
                        "Name: {name}\n\
                        Job ID: {job_id}",
                        name = log.job_name,
                        job_id = log.job_id
                    );
                    for step in &log.failed_step_logs {
                        log::info!(
                            "Step: {step_name}\n\
                            Log length: {log_len}",
                            step_name = step.step_name,
                            log_len = step.error_log.len()
                        );
                    }
                }

                Ok(())
            }
            Command::LocateFailureLog { kind, input_file } => {
                todo!("LocateFailureLog");
            }
        }
    }

    pub async fn open_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        self.issues(
            owner,
            repo,
            State::Open,
            DateFilter::None,
            LabelFilter::none(),
        )
        .await
    }

    pub async fn issues_at<I, S>(
        &self,
        owner: &str,
        repo: &str,
        date: DateFilter,
        state: State,
        labels: LabelFilter<I, S>,
    ) -> Result<Vec<Issue>>
    where
        S: AsRef<str> + fmt::Display + fmt::Debug,
        I: IntoIterator<Item = S> + Clone + fmt::Debug,
    {
        log::debug!("Getting issues for {owner}/{repo} with date={date:?}, state={state:?}, labels={labels:?}");
        self.issues(owner, repo, state, date, labels).await
    }

    // Utility function to get issues
    async fn issues<I, S>(
        &self,
        owner: &str,
        repo: &str,
        state: State,
        date: DateFilter,
        labels: LabelFilter<I, S>,
    ) -> Result<Vec<Issue>>
    where
        S: AsRef<str> + fmt::Display + fmt::Debug,
        I: IntoIterator<Item = S> + Clone,
    {
        let label_filter = labels.to_string();

        let date_filter = date.to_string();

        let issue_state = match state {
            State::Open => "is:open",
            State::Closed => "is:closed",
            State::All => "",
            _ => bail!("Invalid state"),
        };

        let query_str =
            format!("repo:{owner}/{repo} is:issue {issue_state} {date_filter} {label_filter}");
        log::debug!("Query string={query_str}");
        let issues = self
            .client
            .search()
            .issues_and_pull_requests(&query_str)
            .send()
            .await?;

        Ok(issues.items)
    }

    pub async fn workflow_run(&self, owner: &str, repo: &str, run_id: RunId) -> Result<Run> {
        log::debug!("Getting workflow run {run_id} for {owner}/{repo}");
        let run = self.client.workflows(owner, repo).get(run_id).await?;
        Ok(run)
    }

    pub async fn workflow_run_jobs(
        &self,
        owner: &str,
        repo: &str,
        run_id: RunId,
    ) -> Result<Vec<Job>> {
        log::debug!("Getting workflow run jobs for {run_id} for {owner}/{repo}");
        let jobs = self
            .client
            .workflows(owner, repo)
            .list_jobs(run_id)
            .page(1u8)
            .filter(Filter::All)
            .send()
            .await?;
        Ok(jobs.items)
    }

    pub async fn download_job_logs(
        &self,
        owner: &str,
        repo: &str,
        run_id: RunId,
    ) -> Result<Vec<JobLog>> {
        log::debug!("Downloading logs for {run_id} for {owner}/{repo}");
        let logs_zip = self
            .client
            .actions()
            .download_workflow_run_logs(owner, repo, run_id)
            .await?;

        log::debug!("Downloaded logs: {} bytes", logs_zip.len());
        let zip_reader = io::Cursor::new(logs_zip);
        let mut archive = zip::ZipArchive::new(zip_reader)?;

        log::info!(
            "Extracting {} log(s) from downloaded zip archive",
            archive.len()
        );

        let mut logs = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            log::info!("Extracting file: {} | size={}", file.name(), file.size());
            if file.size() == 0 {
                log::debug!("Skipping empty file: {}", file.name());
                continue;
            }

            let mut contents = String::with_capacity(1024);
            file.read_to_string(&mut contents)?;
            logs.push(JobLog::new(file.name().to_string(), contents));
        }

        log::debug!("Extracted logs: {} characters", logs.len());
        log::trace!("{logs:?}");

        // The logs are received in a random order, so we sort them by timestamp
        logs.sort_unstable_by(|a, b| {
            let a = timestamp_from_log(&a.content).unwrap();
            let b = timestamp_from_log(&b.content).unwrap();
            a.cmp(&b)
        });

        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octocrab::models::workflows::Conclusion;
    use pretty_assertions::{assert_eq, assert_ne};

    #[tokio::test]
    async fn test_get_issues() {
        GitHub::init().unwrap();
        let issues = GitHub::get()
            .issues_at(
                "docker",
                "buildx",
                DateFilter::Created(Date {
                    year: 2019,
                    month: 6,
                    day: 2,
                }),
                State::Closed,
                LabelFilter::none(),
            )
            .await
            .unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].title, "Building for ARM causes error often");
        assert_eq!(issues[0].number, 88);
    }

    #[tokio::test]
    async fn test_get_issues_by_label() {
        GitHub::init().unwrap();
        let issues = GitHub::get()
            .issues(
                "docker",
                "buildx",
                State::Open,
                DateFilter::None,
                LabelFilter::All(["kind/bug", "area/bake"]),
            )
            .await
            .unwrap();
        println!("{}", issues.len());
        assert_ne!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_get_workflow_run() {
        GitHub::init().unwrap();
        let run = GitHub::get()
            .workflow_run("gregerspoulsen", "artisan_tools", RunId(8172341325))
            .await
            .unwrap();
        //println!("{run:?}");
        assert_eq!(run.id, RunId(8172341325));
        assert_eq!(run.status, "completed");
    }

    #[tokio::test]
    async fn test_get_workflow_failed_run() {
        GitHub::init().unwrap();
        let run = GitHub::get()
            .workflow_run("gregerspoulsen", "artisan_tools", RunId(8172179418))
            .await
            .unwrap();
        println!("{run:?}");
        assert_eq!(run.id, RunId(8172179418));
        assert_eq!(run.status, "completed");
        assert_eq!(run.conclusion, Some("failure".to_string()));
    }

    #[tokio::test]
    async fn test_get_workflow_run_jobs() {
        GitHub::init().unwrap();
        let jobs = GitHub::get()
            .workflow_run_jobs("gregerspoulsen", "artisan_tools", RunId(8172179418))
            .await
            .unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].conclusion, Some(Conclusion::Failure));
        assert_eq!(jobs[0].steps.len(), 5);
        assert_eq!(jobs[0].steps[0].name, "Set up job");

        let failed_jobs = jobs
            .iter()
            .filter(|job| job.conclusion == Some(Conclusion::Failure))
            .collect::<Vec<_>>();
        let failed_steps = failed_jobs[0]
            .steps
            .iter()
            .filter(|step| step.conclusion == Some(Conclusion::Failure))
            .collect::<Vec<_>>();
        assert_eq!(failed_steps.len(), 1);
        assert_eq!(failed_steps[0].name, "Run tests");
    }

    #[tokio::test]
    #[ignore = "Downloading logs requires authentication"]
    async fn test_download_job_logs() {
        let owner = "CramBL";
        let repo = "moss_decoder";
        let token_val = "ghp_"; // Put your token here but don't commit it!
        let run_id = RunId(7554969653);
        env::set_var("GITHUB_TOKEN", token_val);
        GitHub::init().unwrap();
        let logs = GitHub::get()
            .download_job_logs(owner, repo, run_id)
            .await
            .unwrap();
        eprintln!("Got {} logs", logs.len());
        for log in &logs {
            eprintln!("{}\n{}", log.name, log.content);
        }
        assert_eq!(logs.len(), 10);
    }
}
