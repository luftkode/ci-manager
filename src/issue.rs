//! Contains the Issue struct and its associated methods.
//!
//! The Issue struct is used to represent a GitHub issue that will be created
//! in a repository. It contains a title, label, and body. The body is a
//! collection of FailedJob structs, which contain information about the failed
//! jobs in a GitHub Actions workflow run.
use std::fmt::{self, Display, Formatter, Write};

use crate::{ensure_https_prefix, err_parse::ErrorMessageSummary};

pub mod similarity;

#[derive(Debug)]
pub struct Issue {
    title: String,
    labels: Vec<String>,
    body: IssueBody,
}

impl Issue {
    pub fn new(
        title: String,
        run_id: String,
        mut run_link: String,
        failed_jobs: Vec<FailedJob>,
        label: String,
    ) -> Self {
        let mut labels = vec![label];
        failed_jobs.iter().for_each(|job| {
            if let Some(failure_label) = job.failure_label() {
                if !labels.contains(&failure_label) {
                    log::debug!("Adding failure label {failure_label} to issue");
                    labels.push(failure_label);
                }
            }
        });
        ensure_https_prefix(&mut run_link);
        Self {
            title,
            labels,
            body: IssueBody::new(run_id, run_link, failed_jobs),
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn labels(&self) -> &[String] {
        self.labels.as_slice()
    }

    pub fn body(&self) -> String {
        self.body.to_string()
    }
}

#[derive(Debug)]
pub struct IssueBody {
    run_id: String,
    run_link: String,
    failed_jobs: Vec<FailedJob>,
}

impl IssueBody {
    pub fn new(run_id: String, run_link: String, failed_jobs: Vec<FailedJob>) -> Self {
        Self {
            run_id,
            run_link,
            failed_jobs,
        }
    }
}

impl Display for IssueBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output_str = format!(
            "**Run ID**: {id} [LINK TO RUN]({run_url})

**{failed_jobs_list_title}**
{failed_jobs_name_list}",
            id = self.run_id,
            run_url = self.run_link,
            failed_jobs_list_title = format_args!(
                "{cnt} {job} failed:",
                cnt = self.failed_jobs.len(),
                job = if self.failed_jobs.len() == 1 {
                    "job"
                } else {
                    "jobs"
                }
            ),
            failed_jobs_name_list =
                self.failed_jobs
                    .iter()
                    .fold(String::new(), |mut s_out, job| {
                        let _ = writeln!(s_out, "- **`{}`**", job.name);
                        s_out
                    })
        );
        let output_len = output_str.len();
        write!(f, "{output_str}")?;

        let mut failed_jobs_str = String::new();
        for job in &self.failed_jobs {
            failed_jobs_str.push_str(&job.to_string());
            if output_len + failed_jobs_str.len() > 65535 {
                log::warn!("Issue body is too long, trimming");
                failed_jobs_str.truncate(65535 - output_len);
                break;
            }
        }
        write!(f, "{failed_jobs_str}")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct FailedJob {
    name: String,
    id: String,
    url: String,
    failed_step: String,
    error_message: ErrorMessageSummary,
}

impl FailedJob {
    pub fn new(
        name: String,
        id: String,
        mut url: String,
        failed_step: String,
        error_message: ErrorMessageSummary,
    ) -> Self {
        ensure_https_prefix(&mut url);
        Self {
            name,
            id,
            url,
            failed_step,
            error_message,
        }
    }

    pub fn failure_label(&self) -> Option<String> {
        self.error_message.failure_label()
    }
}

impl Display for FailedJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let summary = self.error_message.summary();
        let optional_log = match (self.error_message.logfile_name(), self.error_message.log()) {
            (Some(name), Some(contents)) => format!(
                "
<details>
<summary>{name}</summary>
<br>

```
{contents}
```
</details>"
            ),
            _ => String::from(""),
        };

        write!(
            f,
            "
### `{name}` (ID {id})
**Step failed:** `{failed_step}`
\\
**Log:** {url}
\\
*Best effort error summary*:
```
{error_message}```{optional_log}",
            name = self.name,
            id = self.id,
            failed_step = self.failed_step,
            url = self.url,
            error_message = summary,
            optional_log = optional_log
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const EXAMPLE_ISSUE_BODY: &str = r#"**Run ID**: 7858139663 [LINK TO RUN]( https://github.com/luftkode/distro-template/actions/runs/7850874958)

**2 jobs failed:**
- **`Test template xilinx`**
- **`Test template raspberry`**

### `Test template xilinx` (ID 21442749267)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749267
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```
### `Test template raspberry` (ID 21442749166)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749166
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```"#;

    #[test]
    fn test_issue_new() {
        let run_id = "7858139663".to_string();
        let run_link =
            "https://github.com/luftkode/distro-template/actions/runs/7850874958".to_string();
        let failed_jobs = vec![
            FailedJob::new(
                "Test template xilinx".to_string(),
                "21442749267".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749267".to_string(),
                "📦 Build yocto image".to_string(),
                ErrorMessageSummary::Other("Yocto error: ERROR: No recipes available for: ...
".to_string()),
            ),
            FailedJob::new(
                "Test template raspberry".to_string(),
                "21442749166".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749166".to_string(),
                "📦 Build yocto image".to_string(),
                ErrorMessageSummary::Other("Yocto error: ERROR: No recipes available for: ...
".to_string()),
            ),
        ];
        let label = "bug".to_string();
        let issue = Issue::new(
            "Scheduled run failed".to_string(),
            run_id,
            run_link,
            failed_jobs,
            label,
        );
        assert_eq!(issue.title, "Scheduled run failed");
        assert_eq!(issue.labels, ["bug"]);
        assert_eq!(issue.body.failed_jobs.len(), 2);
        assert_eq!(issue.body.failed_jobs[0].id, "21442749267");
    }

    #[test]
    fn test_issue_body_display() {
        let run_id = "7858139663".to_string();
        let run_link =
            " https://github.com/luftkode/distro-template/actions/runs/7850874958".to_string();
        let failed_jobs = vec![
            FailedJob::new(
                "Test template xilinx".to_string(),
                "21442749267".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749267".to_string(),
                "📦 Build yocto image".to_string(),
                ErrorMessageSummary::Other("Yocto error: ERROR: No recipes available for: ...
".to_string()),
            ),
            FailedJob::new(
                "Test template raspberry".to_string(),
                "21442749166".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749166".to_string(),
                "📦 Build yocto image".to_string(),
                ErrorMessageSummary::Other("Yocto error: ERROR: No recipes available for: ...
".to_string()),
            ),
            ];

        let issue_body = IssueBody::new(run_id, run_link, failed_jobs);
        assert_eq!(issue_body.to_string(), EXAMPLE_ISSUE_BODY);
        //std::fs::write("test2.md", issue_body.to_string()).unwrap();
    }
}
