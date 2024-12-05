#![allow(unused_imports, dead_code)]
use super::*;

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

pub async fn run() -> Result<()> {
    config::init()?;
    // Generate completion script and exit
    if Config::global().generate_completion_script() {
        return Ok(());
    }

    let ci_provider = if let Some(ci_provider) = Config::global().no_ci() {
        ci_provider
    } else {
        ci_provider::CIProvider::from_environment()?
    };

    log::info!("CI provider: {ci_provider}");

    ci_provider.handle(Config::global().subcmd()).await?;

    Ok(())
}
