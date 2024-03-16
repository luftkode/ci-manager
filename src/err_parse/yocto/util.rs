use crate::*;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Display, EnumString, EnumIter,
)]
pub enum YoctoFailureKind {
    /// The 6 standard tasks in Yocto <https://docs.yoctoproject.org/ref-manual/tasks.html>
    #[strum(serialize = "do_build")]
    DoBuild,
    #[strum(serialize = "do_compile")]
    DoCompile,
    #[strum(serialize = "do_compile_ptest_base")]
    DoCompilePtestBase,
    #[strum(serialize = "do_configure")]
    DoConfigure,
    #[strum(serialize = "do_configure_ptest_base")]
    DoConfigurePtestBase,
    #[strum(serialize = "do_deploy")]
    DoDeploy,
    /// Other tasks
    #[strum(serialize = "do_fetch")]
    DoFetch,
    /// If it's a type of failure we're not familiar with or parsing fails, default to this
    #[default]
    #[strum(serialize = "misc")]
    Misc,
}

impl YoctoFailureKind {
    /// Takes in a yocto logfile filename such as `log.do_fetch.21616` and attempts to determine the type
    /// of yocto task the the logfile is associated with.
    ///
    /// # Example
    /// ```
    /// # use gh_workflow_parser::err_msg_parse::yocto_err::util::YoctoFailureKind;
    /// let kind = YoctoFailureKind::parse_from_logfilename("log.do_fetch.21616").unwrap();
    /// assert_eq!(kind, YoctoFailureKind::DoFetch);
    ///
    /// // Infallible if you're sure the filename is a yocto log but it might not be a known task
    /// let kind = YoctoFailureKind::parse_from_logfilename("log.some_custom_task.21616").unwrap_or_default();
    /// assert_eq!(kind, YoctoFailureKind::Misc);
    /// ```
    pub fn parse_from_logfilename(fname: &str) -> Result<Self> {
        for variant in YoctoFailureKind::iter() {
            let variant_as_str = variant.to_string();
            if fname.contains(&variant_as_str) {
                return Ok(variant);
            }
        }
        bail!("Could not determine task from input: {fname}")
    }
}

/// Find the `--- Error summary ---` section in the log and return the rest of the log.
pub fn yocto_error_summary(log: &str) -> Result<String> {
    const YOCTO_ERROR_SUMMARY_SIGNATURE: &str = "--- Error summary ---";
    let error_summary = log
        .split(YOCTO_ERROR_SUMMARY_SIGNATURE)
        .collect::<Vec<&str>>()
        .pop()
        .context("No error summary found")?;
    Ok(error_summary.trim().to_string())
}

/// Trim the trailing `error: Recipe` lines from the error summary
/// This is to remove the noise of just recipe failures
pub fn trim_trailing_just_recipes(log: &str) -> Result<String> {
    let trimmed = log
        .lines()
        .rev()
        .skip_while(|line| {
            line.starts_with("error: Recipe ")
                // Also skip the last line that looks like `##[error]Process completed with exit code 2.`
                || line.starts_with("##[error]Process completed with exit code")
        })
        .collect::<Vec<&str>>()
        .iter()
        .rev()
        .fold(String::with_capacity(log.len()), |acc, line| {
            acc + line + "\n"
        });
    Ok(trimmed)
}

/// Find the kind of yocto failure in the string e.g. this would be `do_fetch`
/// ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616
///
/// # Example
/// ```
/// use gh_workflow_parser::err_msg_parse::yocto_err::util::find_yocto_failure_log_str;
/// let log = r#"ERROR: Some error message
/// ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616
/// ERROR: Some other error message"#;
///
/// let failure_log_str = find_yocto_failure_log_str(log).unwrap();
///
/// assert_eq!(failure_log_str, "ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616");
/// ```
///
///
pub fn find_yocto_failure_log_str(log: &str) -> Result<&str> {
    let log_file_line = log
        .lines()
        .find(|line| line.contains("Logfile of failure stored in"))
        .context("No log file line found")?;

    Ok(log_file_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    const ERROR_SUMMARY_TEST_STR: &str = r#"ERROR: sqlite3-native-3_3.43.2-r0 do_fetch: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')
    ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616
    ERROR: Task (virtual:native:/app/yocto/build/../poky/meta/recipes-support/sqlite/sqlite3_3.43.2.bb:do_fetch) failed with exit code '1'

    2024-02-11 00:09:04 - ERROR    - Command "/app/yocto/poky/bitbake/bin/bitbake -c build test-template-ci-xilinx-image package-index" failed with error 1"#;

    #[test]
    fn test_determine_yocto_error_kind() {
        let task = "do_build";
        assert_eq!(
            YoctoFailureKind::from_str(task).unwrap(),
            YoctoFailureKind::DoBuild
        );
    }

    #[test]
    fn test_yocto_error_from_error_message() {
        // find the part of the string after
        let log_file_line = ERROR_SUMMARY_TEST_STR
            .lines()
            .find(|line| line.contains("Logfile of failure stored in"))
            .ok_or("No log file line found")
            .unwrap();
        dbg!(log_file_line);
        // Get the path stored after `Logfile of failure stored in: `
        let logfile_path = log_file_line
            .split("Logfile of failure stored in: ")
            .collect::<Vec<&str>>()
            .pop()
            .ok_or("No log file found");

        let path = std::path::PathBuf::from(logfile_path.unwrap());
        let fname = path.file_stem().unwrap().to_str().unwrap();

        let yocto_failure = YoctoFailureKind::parse_from_logfilename(fname).unwrap();
        assert_eq!(yocto_failure, YoctoFailureKind::DoFetch);
    }

    const TEST_NOT_TRIMMED_YOCTO_ERROR_SUMMARY: &str = r#"ERROR: sqlite3-native-3_3.43.2-r0 do_fetch: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')
ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21665
ERROR: Task (virtual:native:/app/yocto/build/../poky/meta/recipes-support/sqlite/sqlite3_3.43.2.bb:do_fetch) failed with exit code '1'

2024-02-16 12:45:43 - ERROR    - Command "/app/yocto/poky/bitbake/bin/bitbake -c build test-template-ci-xilinx-image package-index" failed with error 1
error: Recipe `in-container-build-ci-image` failed on line 31 with exit code 2
error: Recipe `run-in-docker` failed with exit code 2
error: Recipe `build-ci-image` failed with exit code 2"#;

    const TEST_EXPECT_TRIMMED_YOCTO_ERROR_SUMMARY: &str = r#"ERROR: sqlite3-native-3_3.43.2-r0 do_fetch: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')
ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21665
ERROR: Task (virtual:native:/app/yocto/build/../poky/meta/recipes-support/sqlite/sqlite3_3.43.2.bb:do_fetch) failed with exit code '1'

2024-02-16 12:45:43 - ERROR    - Command "/app/yocto/poky/bitbake/bin/bitbake -c build test-template-ci-xilinx-image package-index" failed with error 1
"#;

    #[test]
    pub fn test_trim_yocto_error_summary() {
        let trimmed = trim_trailing_just_recipes(TEST_NOT_TRIMMED_YOCTO_ERROR_SUMMARY).unwrap();
        eprintln!("{trimmed}");
        assert_eq!(trimmed, TEST_EXPECT_TRIMMED_YOCTO_ERROR_SUMMARY);
    }
}