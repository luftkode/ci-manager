use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    ci_manager::ci_provider::github::GitHub::init().unwrap();
    let issues = ci_manager::ci_provider::github::GitHub::get()
        .open_issues("CramBL", "mtgo-collection-manager");

    let is = issues.await.unwrap();

    println!("Got {} issues", is.len());

    for issue in is {
        println!("{}", issue.title);
    }

    return ExitCode::SUCCESS;

    if let Err(e) = ci_manager::run() {
        eprintln!("Error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
