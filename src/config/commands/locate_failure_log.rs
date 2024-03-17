use crate::*;
use crate::err_parse::yocto::util;
use std::io::Write;
use super::StepKind;

/// Locate the specific failure log in a failed build/test/other from a log file
///
/// # Arguments
///
/// * `kind` - The [BuildKind] (e.g. Yocto)
/// * `log_file` - Log file to search for the failure log (e.g. log.txt or read from stdin)
///
/// e.g. if you have the log of a failed Yocto build (stdout & stderr) stored in log.txt, you can run use
/// `gh-workflow-parser locate-failure-log --kind Yocto log.txt` to get an absolute path to the failure log
/// e.g. a log.do_fetch.1234 file
pub fn locate_failure_log(
    kind: StepKind,
    log_file: Option<&PathBuf>,
) -> Result<()> {
    let logfile_content: String = match log_file {
        Some(file) => {
            log::info!("Reading log file: {file:?}");
            if !file.exists() {
                bail!("File: {file:?} does not exist")
            }
            fs::read_to_string(file)?
        },
        None => {
            log::info!("Reading log from stdin");
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buf = String::new();
            io::Read::read_to_string(&mut handle, &mut buf)?;
            buf
        },
    };

    match kind {
        StepKind::Yocto => locate_yocto_failure_log(&logfile_content)?,
        StepKind::Other => todo!("This feature is not implemented yet!"),
    }

    Ok(())
}

/// Locate the specific failure log in a failed Yocto build from the contents of a log file
///
/// # Arguments
/// * `logfile_content` - The contents of the log file
///
/// # Returns
/// The absolute path to the failure log
///
/// # Errors
/// Returns an error if the log file does not contain a failure log
///
/// # Example
/// ```no_run
/// # use gh_workflow_parser::commands::locate_failure_log::locate_yocto_failure_log;
/// let logfile_content = r#"multi line
/// test string foo/bar/baz.txt and other
/// contents"#;
/// locate_yocto_failure_log(logfile_content).unwrap();
/// // Prints the absolute path to "foo/bar/baz.txt" to stdout
/// ```
///
pub fn locate_yocto_failure_log(logfile_content: &str) -> Result<()> {


    log::trace!("Finding failure log in log file contents: {logfile_content}");
    let error_summary = util::yocto_error_summary(logfile_content)?;
    let error_summary = util::trim_trailing_just_recipes(&error_summary)?;
    log::trace!("Trimmed error summary: {error_summary}");
    let log_file_line = util::find_yocto_failure_log_str(&error_summary)?;
    let path = logfile_path_from_str(log_file_line)?;
    // write to stdout
    pipe_print!("{}", path.to_string_lossy())?;

    Ok(())
}

/// Find the absolute path of the first path found in a string.
///
/// e.g. "foo yocto/test/bar.txt baz" returns the absolute path to "yocto/test/bar.txt"
///
/// Takes the following steps:
/// 1. Find a (unix) path in the string
/// 2. Check if the path exists then:
/// - **Path exists:** check that it is a file, then get the absolute path and return it
/// - **Path does not exist:** Attempt to find the file using the following steps:
///      1. Remove the first `/` from the string and try the remaining string as a path
///      2. Remove the next part of the string after the first `/` and try the remaining string as a path
///      3. Repeat step 1-2 until we find a path that exists or there are no more `/` in the string
///      4. If no path is found, return an error
pub fn logfile_path_from_str(s: &str) -> Result<PathBuf> {
    let path = first_path_from_str(s)?;
    log::debug!("Searching for logfile from path: {path:?}");
    if path.exists() {
        return canonicalize_if_file(path);
    }

    let mut parts = path.components().collect::<Vec<_>>();
    log::debug!("File not found, looking for file using parts: {parts:?}");
    for _ in 0..parts.len() {
        parts.remove(0);
        let tmp_path = parts.iter().collect::<PathBuf>();
        log::debug!("Looking for file at path: {tmp_path:?}");
        if tmp_path.exists() {
            return canonicalize_if_file(tmp_path);
        }
        // Then try the path from root (with '/' at the start)
        let tmp_path_from_root = PathBuf::from("/").join(tmp_path);
        log::debug!("Looking for file at path: {tmp_path_from_root:?}");
        if tmp_path_from_root.exists() {
            return canonicalize_if_file(tmp_path_from_root);
        }
    }

    bail!("No file found at path: {s}")
}

/// Checks if the path is a file and returns the absolute path if it is
/// # Errors
/// Returns an error if the path is not a file
fn canonicalize_if_file(path: PathBuf) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.canonicalize()?);
    }
    bail!("No file found at path: {path:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_dir::TempDir;

    #[test]
    fn test_logfile_path_from_str_simple() {
        // Create a temporary file and write the test string to it
        let dir = TempDir::new().unwrap();
        let dir_file = dir.child("test.log");
        let tmp_log_file = dir_file.as_path();
        // The test log string is formatted with the path to the temporary file
        let test_log_str = format!(
            "ERROR: Logfile of failure stored in: /app{real_location}",
            real_location = tmp_log_file.to_string_lossy()
        );
        std::fs::write(tmp_log_file, &test_log_str).unwrap();

        // Get the path from the test string
        let path = logfile_path_from_str(&test_log_str).unwrap();

        // Check that the path is the same as the temporary file
        assert_eq!(path, tmp_log_file);
    }

    #[test]
    fn test_logfile_path_from_str() {
        let dir = TempDir::new().unwrap();
        let real_path_str =
            r#"yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
        // Create the whole path in the temp dir
        let path_to_log = dir.path().join(real_path_str);
        // Make the whole path
        std::fs::create_dir_all(path_to_log.parent().unwrap()).unwrap();
        // The test log string is formatted with the path to the temporary file
        let test_log_str = format!(
            r"other contents
ERROR: Logfile of failure stored in: /app{real_location} other contents
other contents",
            real_location = &path_to_log.to_string_lossy()
        );
        // Create the file with the test string
        std::fs::write(&path_to_log, &test_log_str).unwrap();

        // Attempt to get the path from the test string
        let path = logfile_path_from_str(&test_log_str).unwrap();
        // Check that the path is the same as the temporary file
        assert_eq!(path, path_to_log);
    }
}
