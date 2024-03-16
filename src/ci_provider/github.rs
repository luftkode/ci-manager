use std::fmt::Write;

use crate::*;
use anyhow::Result;
use octocrab::{models::issues::Issue, params::State, Octocrab, *};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Date { year, month, day } = self;
        write!(f, "{year}-{month:02}-{day:02}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFilter {
    Created(Date),
    Updated(Date),
    Any,
}

impl fmt::Display for DateFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateFilter::Created(date) => write!(f, "created:{date}"),
            DateFilter::Updated(date) => write!(f, "updated:{date}"),
            DateFilter::Any => f.write_str(""), // No date filter
        }
    }
}

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

    pub async fn open_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        self.issues(owner, repo, State::Open, DateFilter::Any, [""])
            .await
    }

    pub async fn issues_at(
        &self,
        owner: &str,
        repo: &str,
        date: DateFilter,
        state: State,
    ) -> Result<Vec<Issue>> {
        self.issues(owner, repo, state, date, [""]).await
    }

    async fn issues<I, S>(
        &self,
        owner: &str,
        repo: &str,
        state: State,
        date: DateFilter,
        labels: I,
    ) -> Result<Vec<Issue>>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let label_filter = labels
            .into_iter()
            .map(|l| {
                if l.as_ref().is_empty() {
                    "".to_string()
                } else {
                    format!("label:\"{}\"", l.as_ref())
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        let date_filter = date.to_string();

        let issue_state = match state {
            State::Open => "is:open",
            State::Closed => "is:closed",
            State::All => "",
            _ => bail!("Invalid state"),
        };

        let query_str =
            format!("repo:{owner}/{repo} is:issue {issue_state} {date_filter} {label_filter}");
        log::debug!("Query string: {query_str}");
        let issues = self
            .client
            .search()
            .issues_and_pull_requests(&query_str)
            .send()
            .await?;

        Ok(issues.items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_date_display() {
        let date = Date {
            year: 2021,
            month: 6,
            day: 2,
        };
        assert_eq!(date.to_string(), "2021-06-02");
    }

    #[test]
    fn test_date_filter_display() {
        let date = Date {
            year: 2021,
            month: 6,
            day: 2,
        };
        let date_filter = DateFilter::Created(date);
        assert_eq!(date_filter.to_string(), "created:2021-06-02");
    }

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
            )
            .await
            .unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].title, "Building for ARM causes error often");
        assert_eq!(issues[0].number, 88);
    }
}
