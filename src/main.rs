use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = ci_manager::ci_provider::github::GitHub::init() {
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }

    if let Err(e) = ci_manager::run().await {
        eprintln!("Error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
