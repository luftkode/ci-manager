use crate::util::*;
mod util;

/// System test for the CI manager.

const KEY_WITH_PUBLIC_REPO_ACCESS: &str = "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";
fn set_github_token_for_pub_repo_access() {
    std::env::set_var("GITHUB_TOKEN", KEY_WITH_PUBLIC_REPO_ACCESS);
}

#[test]
#[ignore = "Not reliably authenticating with GitHub"]
fn create_issue_from_failed_run_yocto() -> Result<(), Box<dyn Error>> {
    set_github_token_for_pub_repo_access();

    let mut cmd = Command::cargo_bin("ci-manager")?;

    cmd.arg("--ci=github")
        .arg("--verbosity=3")
        .arg("--dry-run")
        .arg("create-issue-from-run")
        .arg("--repo=https://github.com/docker/buildx")
        .arg("--run-id=8302026485")
        .arg("--title=\"Scheduled run failed\"")
        .arg("--label=\"CI scheduled build\"")
        .arg("--kind=yocto");

    let Output {
        status,
        stdout,
        stderr,
    } = cmd.output()?;

    let stderr = String::from_utf8(stderr)?;
    let stdout = String::from_utf8(stdout)?;

    assert!(
        status.success(),
        "Command failed with status: {status}\n - stdout: {stdout}\n - stderr: {stderr}"
    );

    let stderr_contains_fn = predicate::str::contains(
        "Failed to parse Yocto error, returning error message as is: No log file line found",
    );
    assert!(stderr_contains_fn.eval(&stderr), "stderr: {stderr}");

    Ok(())
}

const EXPECT_FAILURE_LOG_CONTENTS: &str = "foobar";
const REL_PATH_TO_FAILURE_LOG: &str =
    r#"yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;

#[test]
fn locate_failure_log_from_file() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory with a long path down to a text file
    let dir = TempDir::new()?;
    // Create the whole path in the temp dir
    let path_to_log = dir.path().join(REL_PATH_TO_FAILURE_LOG);
    fs::create_dir_all(path_to_log.parent().unwrap())?;
    // Create the file with the test string
    fs::write(&path_to_log, EXPECT_FAILURE_LOG_CONTENTS)?;

    // Now create the yocto build failure log string that should contain the path to the file
    // The test log string is formatted with the path to the temporary file
    let test_log_str = format!(
        r"other contents
ERROR: Logfile of failure stored in: /app{real_location} other contents
other contents",
        real_location = &path_to_log.to_string_lossy()
    );
    let test_log_file = dir.child("test.log");
    test_log_file.write_str(&test_log_str)?;

    // Now we should be able to retrieve the `foobar` string from the file by locating it through the log string
    let mut cmd = Command::cargo_bin("ci-manager")?;
    cmd.arg("--ci=gitlab")
        .arg("locate-failure-log")
        .arg("--input-file")
        .arg(test_log_file.path())
        .arg("--kind=yocto");

    let Output {
        status,
        stdout,
        stderr,
    } = cmd.output()?;

    let stdout = String::from_utf8(stdout)?;
    let stderr = String::from_utf8(stderr)?;

    assert!(
        status.success(),
        "Command failed with status: {status}\n - stdout: {stdout}\n - stderr: {stderr}"
    );
    pretty_assert_eq!(stdout, path_to_log.to_str().unwrap());
    // Read the file and check that the contents are as expected
    let contents = fs::read_to_string(&stdout)?;
    pretty_assert_eq!(contents, EXPECT_FAILURE_LOG_CONTENTS);

    Ok(())
}

#[test]
fn locate_failure_log_from_stdin() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory with a long path down to a text file
    let dir = TempDir::new()?;
    // Create the whole path in the temp dir
    let path_to_log = dir.path().join(REL_PATH_TO_FAILURE_LOG);
    fs::create_dir_all(path_to_log.parent().unwrap())?;
    // Create the file with the test string
    fs::write(&path_to_log, EXPECT_FAILURE_LOG_CONTENTS)?;

    // Now create the yocto build failure log string that should contain the path to the file
    // The test log string is formatted with the path to the temporary file
    let test_log_str = format!(
        r"other contents
ERROR: Logfile of failure stored in: /app{real_location} other contents
other contents",
        real_location = &path_to_log.to_string_lossy()
    );
    let test_log_file = dir.child("test.log");
    test_log_file.write_str(&test_log_str)?;

    // Now we should be able to retrieve the `foobar` string from the file by locating it through the log string
    let mut cmd = Command::cargo_bin("ci-manager")?;
    cmd.pipe_stdin(test_log_file)?
        .arg("--ci=github")
        .arg("locate-failure-log")
        .arg("--kind=yocto");

    let Output {
        status,
        stdout,
        stderr,
    } = cmd.output()?;

    let stdout = String::from_utf8(stdout)?;
    let stderr = String::from_utf8(stderr)?;

    assert!(
        status.success(),
        "Command failed with status: {status}\n - stdout: {stdout}\n - stderr: {stderr}"
    );
    pretty_assert_eq!(stdout, path_to_log.to_str().unwrap());
    // Read the file and check that the contents are as expected
    let contents = fs::read_to_string(&stdout)?;
    pretty_assert_eq!(contents, EXPECT_FAILURE_LOG_CONTENTS);

    Ok(())
}
