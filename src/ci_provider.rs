use super::*;

pub mod github;

// Which CI provider is being used, determined from the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum CIProvider {
    GitHub,
    GitLab,
}

impl CIProvider {
    fn env_is_github() -> bool {
        // Check if the GITHUB_ENV environment variable is set
        // might be a more appropriate one to check.. Or check several?
        // The dilemma is that it should return ok on GitHub runners, self-hosted or not
        // but also not trigger false positives by checking a variable some projects might set
        env::var("GITHUB_ENV").is_ok()
    }
    fn env_is_gitlab() -> bool {
        env::var("GITLAB_CI").is_ok()
    }

    pub fn from_enviroment() -> Result<Self> {
        if Self::env_is_gitlab() {
            Ok(Self::GitLab)
        } else if Self::env_is_github() {
            Ok(Self::GitHub)
        } else {
            bail!("Could not determine CI provider from environment")
        }
    }
}
