use self::commands::locate_failure_log;

use super::*;

pub mod github;
pub mod util;

// Which CI provider is being used, determined from the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, ValueEnum)]
pub enum CIProvider {
    #[value(name = "GitHub", alias = "github")]
    GitHub,
}

impl CIProvider {
    fn env_is_github() -> bool {
        // Check if the GITHUB_ENV environment variable is set
        // might be a more appropriate one to check.. Or check several?
        // The dilemma is that it should return ok on GitHub runners, self-hosted or not
        // but also not trigger false positives by checking a variable some projects might set
        env::var("GITHUB_ENV").is_ok()
    }

    pub fn from_enviroment() -> Result<Self> {
        if Self::env_is_github() {
            Ok(Self::GitHub)
        } else {
            bail!("Could not determine CI provider from environment")
        }
    }

    pub async fn handle(&self, command: &commands::Command) -> Result<()> {
        use commands::Command;
        match command {
            // This is a command that is not specific to a CI provider
            Command::LocateFailureLog { kind, input_file } => {
                locate_failure_log::locate_failure_log(*kind, input_file.as_ref())
            }
            Command::CreateIssueFromRun {
                repo,
                run_id,
                label,
                kind,
                title,
                no_duplicate,
            } => match self {
                Self::GitHub => {
                    github::GitHub::get()
                        .create_issue_from_run(repo, run_id, label, kind, *no_duplicate, title)
                        .await
                }
            },
        }
    }
}
