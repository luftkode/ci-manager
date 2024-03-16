use crate::*;

pub struct GitLab {
    client: gitlab::Gitlab,
}

impl GitLab {
    pub fn get() -> Self {
        // Grab the token from the CI_PAT environment variable
        let token = std::env::var("CI_PAT").unwrap();
        // Query the GitLab API
        let client = Gitlab::new("gitlab.com", token).unwrap();
        Self { client }
    }

    pub fn handle(&self, command: &commands::Command) -> Result<()> {
        let endpoint = projects::Project::builder()
            .project("CramBL/github-workflow-parser")
            .build()
            .unwrap();
        // Call the endpoint. The return type decides how to represent the value.
        let project: Project = endpoint.query(&self.client).unwrap();
        //let _: () = api::ignore(endpoint).query(&client).unwrap();
        println!("{project:?}");

        // List all open issues
        let endpoint = projects::issues::Issues::builder()
            .project("CramBL/github-workflow-parser")
            .state(projects::issues::IssueState::Opened)
            .label("bug")
            .build()
            .unwrap();

        let issues: Vec<Issue> = endpoint.query(&self.client).unwrap();
        println!("{issues:?}");

        // query pipeline status
        let endpoint = projects::pipelines::PipelineJobs::builder()
            .project("CramBL/github-workflow-parser")
            .pipeline(1180296622)
            .build()
            .unwrap();

        let pipeline_jobs: Vec<Job> = endpoint.query(&self.client).unwrap();

        println!("{pipeline_jobs:?}");

        // get log for failed job
        let failed_job = pipeline_jobs
            .iter()
            .find(|job| job.status == "failed")
            .unwrap();

        let endpoint = projects::jobs::Job::builder()
            .project("CramBL/github-workflow-parser")
            .job(6195815626)
            .build()
            .unwrap();

        let job: Job = endpoint.query(&self.client).unwrap();

        println!("{job:?}");

        let endpoint = projects::jobs::JobTrace::builder()
            .project("CramBL/github-workflow-parser")
            .job(6195815626)
            .build()
            .unwrap();

        let resp = api::raw(endpoint).query(&self.client).unwrap();

        println!("{}", String::from_utf8_lossy(&resp));

        // let failed_jobs: Vec<String> = pipeline_jobs
        //     .iter()
        //     .filter(|job| job.status == "failed")
        //     .map(|job| job.name.clone())
        //     .collect();

        // let endpoint = projects::issues::CreateIssue::builder()
        //     .project("CramBL/github-workflow-parser")
        //     .title("Failed pipeline")
        //     .description(format!(
        //         "The pipeline failed, these jobs failed: {}",
        //         failed_jobs.join(", ")
        //     ))
        //     .labels(["bug", "test"])
        //     .build()
        //     .unwrap();

        // let resp = api::raw(endpoint).query(&client).unwrap();

        // let resp_as_string = std::str::from_utf8(&resp).unwrap();

        // println!("{resp_as_string}");
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct Project {
    name: String,
    name_with_namespace: String,
}

#[derive(Debug, Deserialize)]
struct Issue {
    title: String,
    description: String,
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Job {
    id: u64,
    name: String,
    status: String,
    #[serde(rename = "ref")]
    ref_: String,
}
