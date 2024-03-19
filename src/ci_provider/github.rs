use std::io::Read;

pub mod util;

use crate::{
    ci_provider::github::util::{
        distance_to_other_issues, repo_url_to_run_url, run_url_to_job_url, JobErrorLog,
    },
    err_parse::parse_error_message,
    issue::FailedJob,
    *,
};
use hyper::body;
use octocrab::{
    models::{
        issues::Issue,
        workflows::{Conclusion, Job, Run},
        Label, RunId,
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
        GITHUB_CLIENT.get_or_init(|| Self::init().expect("Failed to initialize GitHub client"))
    }

    fn init() -> Result<GitHub> {
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
        Ok(github_client)
    }

    fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_owned())
            .build()?;
        Ok(Self { client })
    }

    pub async fn create_issue_from_run(
        &self,
        repo: &String,
        run_id: &String,
        label: &String,
        kind: &commands::WorkflowKind,
        no_duplicate: bool,
        title: &String,
    ) -> Result<()> {
        log::debug!(
            "Creating issue from:\n\
            \trepo: {repo}\n\
            \trun_id: {run_id}\n\
            \tlabel: {label}\n\
            \tkind: {kind}\n\
            \tno_duplicate: {no_duplicate}\n\
            \ttitle: {title}",
        );
        let (owner, repo) = repo_to_owner_repo_fragments(repo)?;
        let run_url = repo_url_to_run_url(&format!("github.com/{owner}/{repo}"), run_id);
        let run_id: u64 = run_id.parse()?;

        let workflow_run = self.workflow_run(&owner, &repo, RunId(run_id)).await?;
        log::debug!("{workflow_run:?}");

        if workflow_run.conclusion != Some("failure".to_string()) {
            log::info!(
                "Workflow run didn't fail, but has conclusion: {:?}. Continuing...",
                workflow_run.conclusion
            );
        }

        let mut jobs = self.workflow_run_jobs(&owner, &repo, RunId(run_id)).await?;
        // Take only jobs from the most recent attempt
        let mut max_attempt = 0;
        for job in &jobs {
            if job.run_attempt > max_attempt {
                max_attempt = job.run_attempt;
            }
        }
        jobs.retain(|job| job.run_attempt == max_attempt);
        let jobs = jobs; // Make immutable again

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

        let logs = self
            .download_workflow_run_logs(&owner, &repo, RunId(run_id))
            .await?;
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
            log::info!("Extracting error logs for job: {}", job.name);
            let name = job.name.clone();
            let id = job.id;
            let mut step_error_logs: Vec<util::StepErrorLog> = Vec::new();
            for steps in &failed_steps {
                log::info!("\tExtracting error logs for step: {}", steps.name);
                let step_name = steps.name.clone();
                let step_log = logs.iter().find(|log| {
                    log.name.contains(steps.name.as_str()) && log.name.contains(job.name.as_str())
                });
                match step_log {
                    Some(step_log) => step_error_logs
                        .push(util::StepErrorLog::new(step_name, step_log.content.clone())),
                    None => log::error!(
                        "No log found for failed step: {step_name} in job: {job_name}. Continuing...",
                        job_name = job.name
                    ),
                }
            }
            job_error_logs.push(JobErrorLog::new(id, name, step_error_logs));
        }

        util::log_info_downloaded_job_error_logs(&job_error_logs);

        // Parse to a github issue
        // Map the GitHub Job to a `FailedJob`
        let failed_jobs = job_error_logs
            .iter()
            .map(|job| {
                let job_id_str = job.job_id.to_string();
                let job_url = run_url_to_job_url(&run_url, &job_id_str);
                let continuous_errorlog_msgs = job.logs_as_str();
                let first_failed_step = job.failed_step_logs.first().unwrap().step_name.to_owned();
                let parsed_msg = parse_error_message(&continuous_errorlog_msgs, *kind).unwrap();
                FailedJob::new(
                    job.job_name.to_owned(),
                    job_id_str,
                    job_url,
                    first_failed_step,
                    parsed_msg,
                )
            })
            .collect();

        let issue = issue::Issue::new(
            title.to_owned(),
            run_id.to_string(),
            run_url,
            failed_jobs,
            label.to_owned(),
        );
        log::debug!("generic issue instance: {issue:?}");
        // Check if-no-duplicate is set
        if no_duplicate {
            log::info!("No-duplicate flag is set, checking for similar issues");
            // Then check if a similar issue exists
            let open_issues = self
                .issues_at(
                    &owner,
                    &repo,
                    DateFilter::None,
                    State::Open,
                    LabelFilter::All([label]),
                )
                .await?;
            log::info!(
                "Found {num_issues} open issue(s) with label {label}",
                num_issues = open_issues.len()
            );
            let min_distance = distance_to_other_issues(&issue.body(), &open_issues);
            log::info!("Minimum distance to similar issue: {min_distance}");
            match min_distance {
                0 => {
                    log::warn!("An issue with the exact same body already exists. Exiting...");
                    return Ok(());
                }
                _ if min_distance < issue::similarity::LEVENSHTEIN_THRESHOLD => {
                    log::warn!("An issue with a similar body already exists. Exiting...");
                    return Ok(());
                }
                _ => log::info!("No similar issue found. Continuing..."),
            }
        }

        // Get all labels for the repo, and create the ones that don't exist
        let all_labels = self.get_all_labels(&owner, &repo).await?;
        log::info!("Got {num_labels} label(s)", num_labels = all_labels.len());
        let labels_to_create: Vec<String> = issue
            .labels()
            .iter()
            .filter(|label| !all_labels.iter().any(|l| l.name.eq(*label)))
            .cloned()
            .collect();
        if !labels_to_create.is_empty() {
            log::info!(
                "{} label(s) determined for the issue-to-be-created do not yet exist on the repo, and will be created: {labels_to_create:?}",
                labels_to_create.len()
            );
        }

        // Check if dry-run is set
        if Config::global().dry_run() {
            // Then print the issue to be created instead of creating it
            println!("####################################");
            println!("DRY RUN MODE! The following issue would be created:");
            println!("==== ISSUE TITLE ==== \n{}", issue.title());
            println!("==== ISSUE LABEL(S) ==== \n{}", issue.labels().join(","));
            println!("==== START OF ISSUE BODY ==== \n{}", issue.body());
            println!("==== END OF ISSUE BODY ====");
        } else {
            // Create the labels that don't exist
            for issue_label in labels_to_create {
                log::info!("Creating label: {issue_label}");
                self.client
                    .issues(&owner, &repo)
                    .create_label(issue_label, "FF0000", "")
                    .await?; // Await the completion of the create_label future
            }
            self.create_issue(&owner, &repo, issue).await?;
        }

        Ok(())
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

    /// Create an issue
    pub async fn create_issue(&self, owner: &str, repo: &str, issue: issue::Issue) -> Result<()> {
        log::debug!(
            "Creating issue for {owner}/{repo} with\n\
        \ttitle:  {title}\n\
        \tlabels: {labels:?}\n\
        \tbody:   {body}",
            title = issue.title(),
            body = issue.body(),
            labels = issue.labels()
        );
        // The maximum size of a GitHub issue body is 65536
        if issue.body().len() > 65536 {
            log::error!(
                "Issue body is too long: {len} characters. Maximum for GitHub issues is 65536. Exiting...",
                len = issue.body().len()
            );
            bail!("Issue body is too long");
        }

        self.client
            .issues(owner, repo)
            .create(issue.title())
            .body(issue.body())
            .labels(issue.labels().to_vec())
            .send()
            .await?;
        Ok(())
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

    pub async fn get_all_labels(&self, owner: &str, repo: &str) -> Result<Vec<Label>> {
        let label_page = self
            .client
            .issues(owner, repo)
            .list_labels_for_repo()
            .send()
            .await?;
        Ok(label_page.items)
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

    /// Get the entire raw log for a job
    ///
    /// # Note
    /// The log does not contain the name of the workflow steps, only the output of the steps. It is
    /// therefore not feasible to parse the log to find the step that failed.
    /// Instead use [`download_workflow_run_logs`][GitHub::download_workflow_run_logs] to get the logs for the entire workflow run.
    pub async fn download_job_logs(&self, owner: &str, repo: &str, job_id: u64) -> Result<String> {
        use http_body_util::BodyExt;
        use hyper::Uri;
        log::debug!("Downloading logs for job {job_id} for {owner}/{repo}");
        // Workaround until https://github.com/XAMPPRocky/octocrab/issues/394 is fixed
        // adapted from: https://github.com/XAMPPRocky/octocrab/issues/394#issuecomment-1586054876

        // route: https://docs.github.com/en/rest/actions/workflow-jobs?apiVersion=2022-11-28#download-job-logs-for-a-workflow-run
        let route = format!("/repos/{owner}/{repo}/actions/jobs/{job_id}/logs");
        let uri = Uri::builder().path_and_query(route).build()?;
        // The endpoint returns a link to the logs, so configure the client to follow the redirect and return the data
        let data_response = self
            .client
            .follow_location_to_data(self.client._get(uri).await?)
            .await?;
        let boxbody = data_response.into_body();
        // Read the streaming body into a byte vector
        let body_bytes = BodyExt::collect(boxbody).await?.to_bytes().to_vec();
        log::debug!("Downloaded {} bytes", body_bytes.len());
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();
        Ok(body_str)
    }

    /// Download the logs for a workflow run as a zip file, and extract the logs into a vector of [`JobLog`]s
    /// sorted by the timestamp appearing in the logs.
    ///
    /// # Note
    /// The logs are from the entire workflow run and all attempts, not just the most recent attempt.
    pub async fn download_workflow_run_logs(
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
    #[ignore = "Needs a valid GITHUB_TOKEN with read access to public repos"]
    async fn test_get_workflow_run_jobs() {
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
    #[ignore = "Might fail when running with `cargo test` (If another test sets the GITHUB_TOKEN env var)"]
    async fn test_download_workflow_run_logs() {
        let owner = "docker";
        let repo = "buildx";
        let run_id = RunId(8302026485);
        GitHub::init().unwrap();
        let logs = GitHub::get()
            .download_workflow_run_logs(owner, repo, run_id)
            .await
            .unwrap();
        for log in &logs {
            eprintln!("{}\n{}", log.name, log.content);
        }
        assert_eq!(logs.len(), 37);
    }
}
