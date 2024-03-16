use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    ci_manager::ci_provider::github::GitHub::init().unwrap();

    if let Err(e) = ci_manager::run().await {
        eprintln!("Error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
