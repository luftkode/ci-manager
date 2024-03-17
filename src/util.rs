//! Utility functions for parsing and working with GitHub CLI output and other utility functions.
use super::*;

/// Parse a path from a string
/// # Example
/// ```
/// # use ci_manager::util::first_path_from_str;
/// use std::path::PathBuf;
///
/// let haystack = r#"multi line
/// test string with/path/file.txt is
/// valid"#;
/// let path = first_path_from_str(haystack).unwrap();
/// assert_eq!(path, PathBuf::from("with/path/file.txt"));
///
/// // No path in string is an error
/// let haystack = "Random string with no path";
/// assert!(first_path_from_str(haystack).is_err());
///
/// // Path with no leading '/' and no file extension is OK
/// let haystack = "foo app/3-_2/t/3 bar";
/// let path = first_path_from_str(haystack).unwrap();
/// assert_eq!(path, PathBuf::from("app/3-_2/t/3"));
///
/// // More realistic example
/// let haystack = r#" ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
/// let path = first_path_from_str(haystack).unwrap();
/// assert_eq!(
///   path,
///  PathBuf::from("/app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616")
/// );
/// ```
/// # Errors
/// This function returns an error if no valid path is found in the string
pub fn first_path_from_str(s: &str) -> Result<PathBuf> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"[a-zA-Z0-9-_.\/]+\/[a-zA-Z0-9-_.]+").unwrap());

    let path_str = RE.find(s).context("No path found in string")?.as_str();
    Ok(PathBuf::from(path_str))
}

/// Take the lines with failed jobs from the output of `gh run view`
pub fn take_lines_with_failed_jobs(output: String) -> Vec<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"X.*ID [0-9]*\)").unwrap());

    RE.find_iter(&output)
        .map(|m| m.as_str().to_owned())
        .collect()
}

/// Extract the job IDs from the lines with job information
pub fn id_from_job_lines(lines: &[String]) -> Vec<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"ID (?<JOB_ID>[0-9]*)").unwrap());

    lines
        .iter()
        .map(|line| {
            RE.captures(line)
                .unwrap_or_else(|| {
                    panic!("Expected a line with a Job ID, but no ID found in line: {line}")
                })
                .name("JOB_ID")
                .expect("Expected a Job ID")
                .as_str()
                .to_owned()
        })
        .collect()
}

/// Parse text for timestamps and IDs and remove them, returning the modified text without making a copy.
///
/// Some compromises are made to be able to remove timestamps in between other symbols e.g. '/83421321/'.
/// but still avoid removing commit SHAs. That means that these symbols are also removed (any non-letter character
/// preceding and following an ID).
///
/// # Example
/// ```
/// # use ci_manager::util::remove_timestamps_and_ids;
/// # use pretty_assertions::assert_eq;
/// let test_str = r"ID 21442749267 ";
/// let modified = remove_timestamps_and_ids(test_str);
/// assert_eq!(modified, "ID"); // Note that the space is removed
///
///
/// let test_str = r#"ID 21442749267
/// date: 2024-02-28 00:03:46
/// other text"#;
/// let modified = remove_timestamps_and_ids(test_str);
/// assert_eq!(modified, "IDdate: \nother text");
/// ```
pub fn remove_timestamps_and_ids(text: &str) -> borrow::Cow<str> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?x)
            # Timestamps like YYYY-MM-DD HH:MM:SS
            ([0-9]{4}-[0-9]{2}-[0-9]{2}\x20[0-9]{2}:[0-9]{2}:[0-9]{2})
            |
            # IDs like 21442749267 but only if they are preceded and followed by non-letter characters
            (?:[^[a-zA-Z]])([0-9]{10,11})(?:[^[a-zA-Z]])
        ",
        )
        .unwrap()
    });

    RE.replace_all(text, "")
}

/// Parse a log and remove line-prefixed timestamps in the format `YYYY-MM-DDTHH:MM:SS.0000000Z` (ISO 8601).
/// # Example
/// ```
/// # use ci_manager::util::remove_timestamp_prefixes;
/// # use pretty_assertions::assert_eq;
/// let test_str = "2024-02-28T00:03:46.0000000Z [INFO] This is a log message";
/// let modified = remove_timestamp_prefixes(test_str);
/// assert_eq!(modified, "[INFO] This is a log message");
/// ```
/// ## Multiple lines
/// ```
/// # use ci_manager::util::remove_timestamp_prefixes;
/// # use pretty_assertions::assert_eq;
/// let test_str = "\
/// 2024-02-28T00:03:46.0000000Z [INFO] This is a log message
/// 2024-03-15T20:35:48.9824182Z [ERROR] This is another log message";
/// let modified = remove_timestamp_prefixes(test_str);
/// assert_eq!(modified, "\
/// [INFO] This is a log message
/// [ERROR] This is another log message");
///
pub fn remove_timestamp_prefixes(log: &str) -> borrow::Cow<str> {
    // The fist group matches 0 or more newlines, and uses that group to replace the timestamp
    // this way the newlines are preserved (making it agnostic to the type of newline used in the log)
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([\r\n]*)\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{7}Z\s").unwrap());

    RE.replace_all(log, "$1")
}

/// Parse an absolute path from a string. This assumes that the the first '/' found in the string is the start
/// of the path.
/// # Example
/// ```
/// # use ci_manager::util::first_abs_path_from_str;
/// use std::path::PathBuf;
///
/// let test_str = r#" ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
/// let path = first_abs_path_from_str(test_str).unwrap();
/// assert_eq!(
///    path,
///   PathBuf::from("/app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616")
/// );
/// ```
///
/// # Errors
/// Returns an error if no '/' is found in the string or
/// if the path is not a valid path.
pub fn first_abs_path_from_str(s: &str) -> Result<PathBuf> {
    let start = s.find('/').context("Path not found, no '/' in string")?;
    let path = PathBuf::from(&s[start..]);
    Ok(path)
}

/// Add https:// to a URL if it is not already present
/// # Example
/// ```
/// # use ci_manager::util::ensure_https_prefix;
/// # use pretty_assertions::assert_eq;
/// // If the URL does not have the https prefix, it is added
/// let mut url = String::from("github.com/docker/buildx/issues");
/// ensure_https_prefix(&mut url);
/// assert_eq!(url, "https://github.com/docker/buildx/issues");
///
/// // If the URL already has the https prefix, it is not modified
/// let mut url = String::from("https://gitlab.com/foo-org/foo-repo");
/// ensure_https_prefix(&mut url);
/// assert_eq!(url, String::from("https://gitlab.com/foo-org/foo-repo"));
/// ```
pub fn ensure_https_prefix(url: &mut String) {
    if url.starts_with("https://") {
        return;
    }
    url.insert_str(0, "https://");
}

/// Canonicalize a repository URL to the form `https://{host}/{repo}`
///
/// # Arguments
/// * `repo` - The repository URL e.g. `user1/user1-repo`
/// * `host` - The host for the repository e.g. `github.com`
///
/// # Example
/// ```
/// # use ci_manager::util::canonicalize_repo_url;
/// let repo = "bob/bobbys-repo";
/// let canonicalized = canonicalize_repo_url(repo, "github");
/// assert_eq!(canonicalized, "https://github.com/bob/bobbys-repo");
///
/// // If the host is already in the URL, only the protocol is added
/// let repo = "github.com/lisa/lisas-repo";
/// let canonicalized = canonicalize_repo_url(repo, "github.com");
/// assert_eq!(canonicalized, "https://github.com/lisa/lisas-repo");
///
/// // If the URL is already in the canonical form, it is returned as is
/// let repo = "https://gitlab.com/foo-org/foo-repo";
/// let canonicalized = canonicalize_repo_url(repo, "gitlab.com");
/// assert_eq!(canonicalized, repo);
/// ```
pub fn canonicalize_repo_url(repo: &str, host: &str) -> String {
    // Check if the host argument has a top-level domain and add it `.com` if it doesn't
    let host = if host.contains('.') {
        host.to_string()
    } else {
        format!("{host}.com")
    };
    let canonical_prefix: String = format!("https://{host}/");
    if repo.starts_with("https://") {
        if repo.starts_with(&canonical_prefix) {
            repo.to_string()
        } else {
            repo.replace("https://", &canonical_prefix)
        }
    } else if repo.starts_with(&format!("{host}/")) {
        repo.replace(&format!("{host}/"), &canonical_prefix)
    } else {
        format!("{canonical_prefix}{repo}")
    }
}

/// Parse a repository URL/identifier to owner and repo fragments
/// # Example
/// ```
/// # use pretty_assertions::assert_eq;
/// # use ci_manager::util::repo_to_owner_repo_fragments;
/// let repo_url = "github.com/luftkode/distro-template";
/// let (owner, repo) = repo_to_owner_repo_fragments(repo_url).unwrap();
/// assert_eq!((owner.as_str(), repo.as_str()), ("luftkode", "distro-template"));
///
/// let repo_url = "luftkode/bifrost-app";
/// let (owner, repo) = repo_to_owner_repo_fragments(repo_url).unwrap();
/// assert_eq!((owner.as_str(), repo.as_str()), ("luftkode", "bifrost-app"));
/// ```
///
/// # Errors
/// Returns an error if the URL cannot be parsed
/// # Example
/// ```
/// # use ci_manager::util::repo_to_owner_repo_fragments;
/// let repo_url = "github.com/luftkode";
/// let result = repo_to_owner_repo_fragments(repo_url);
/// assert!(result.is_err());
/// ```
pub fn repo_to_owner_repo_fragments(repo_url: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = repo_url.split('/').collect();
    // reverse the order of the parts and take the first two
    let repo_and_owner = parts.into_iter().rev().take(2).collect::<Vec<&str>>();
    // Check that there are 2 parts and that neither are empty or contain spaces or dots
    if repo_and_owner.len() != 2
        || repo_and_owner
            .iter()
            .any(|s| s.is_empty() || s.contains(' ') || s.contains('.'))
    {
        bail!("Could not parse owner and repo from URL: {repo_url}");
    }
    let (repo, owner) = (repo_and_owner[0], repo_and_owner[1]);
    Ok((owner.to_string(), repo.to_string()))
}

/// Calculate the smallest levenshtein distance between an issue body and other issue bodies
pub fn issue_text_similarity(issue_body: &str, other_issues: &[String]) -> usize {
    let issue_body_without_timestamps = remove_timestamps_and_ids(issue_body);

    let smallest_distance = other_issues
        .iter()
        .map(|other_issue_body| {
            distance::levenshtein(
                &issue_body_without_timestamps,
                &remove_timestamps_and_ids(other_issue_body),
            )
        })
        .min()
        .unwrap_or(usize::MAX);

    smallest_distance
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_absolute_path_from_str() {
        let test_str = r#" ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
        let path = first_abs_path_from_str(test_str).unwrap();
        assert_eq!(
            path,
            PathBuf::from("/app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616")
        );
    }

    #[test]
    pub fn test_canonicalize_repo_url() {
        let repo = "luftkode/distro-template";
        let canonicalized = canonicalize_repo_url(repo, "github.com");
        assert_eq!(canonicalized, "https://github.com/luftkode/distro-template");
    }

    #[test]
    pub fn test_remove_timestamps_and_ids() {
        let test_str = "ID 8072883145 ";
        let modified = remove_timestamps_and_ids(test_str);
        assert_eq!(modified, "ID");
    }

    #[test]
    pub fn test_remove_timestamps_and_ids_log_text() {
        const LOG_TEXT: &'static str = r#"**Run ID**: 8072883145 [LINK TO RUN](https://github.com/luftkode/distro-template/actions/runs/8072883145)

        **1 job failed:**
        - **`Test template xilinx`**

        ### `Test template xilinx` (ID 22055505284)
        **Step failed:** `📦 Build yocto image`
        \
        **Log:** https://github.com/luftkode/distro-template/actions/runs/8072883145/job/22055505284
        "#;

        const EXPECTED_MODIFIED: &'static str = r#"**Run ID**:[LINK TO RUN](https://github.com/luftkode/distro-template/actions/runs

        **1 job failed:**
        - **`Test template xilinx`**

        ### `Test template xilinx` (ID
        **Step failed:** `📦 Build yocto image`
        \
        **Log:** https://github.com/luftkode/distro-template/actions/runsjob        "#;

        let modified = remove_timestamps_and_ids(LOG_TEXT);
        assert_eq!(
            modified, EXPECTED_MODIFIED,
            "Expected: {EXPECTED_MODIFIED}\nGot: {modified}"
        );
    }
}
