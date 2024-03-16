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
/// # use ci_manager::util::remove_timestamps;
/// # use pretty_assertions::assert_eq;
/// let test_str = r"ID 21442749267 ";
/// let modified = remove_timestamps(test_str);
/// assert_eq!(modified, "ID"); // Note that the space is removed
///
///
/// let test_str = r#"ID 21442749267
/// date: 2024-02-28 00:03:46
/// other text"#;
/// let modified = remove_timestamps(test_str);
/// assert_eq!(modified, "IDdate: \nother text");
/// ```
pub fn remove_timestamps(text: &str) -> borrow::Cow<str> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use temp_dir::TempDir;

    // Output from `gh run --repo=github.com/luftkode/distro-template view 7858139663`
    const TEST_OUTPUT_VIEW_RUN: &str = r#"
    X master Use template and build image · 7858139663
    Triggered via schedule about 10 hours ago

    JOBS
    ✓ enable-ssh-agent in 5s (ID 21442747661)
    ✓ Test template raspberry in 19m20s (ID 21442749166)
    X Test template xilinx in 5m41s (ID 21442749267)
      ✓ Set up job
      ✓ Log in to the Container registry
      ✓ Cleanup build folder before start
      ✓ Run actions/checkout@v4
      ✓ Setup Rust and Just
      ✓ 🗻 Make a templated project
      ✓ ⚙️ Run new project setup steps
      ✓ ⚒️ Build docker image
      X 📦 Build yocto image
      - 📩 Deploy image artifacts
      ✓ Docker down
      ✓ Cleanup build folder after done
      ✓ Create issue on failure
      ✓ Post Run actions/checkout@v4
      ✓ Post Log in to the Container registry
      ✓ Complete job

    ANNOTATIONS
    X Process completed with exit code 2.
    Test template xilinx: .github#3839


    To see what failed, try: gh run view 7858139663 --log-failed
    View this run on GitHub: https://github.com/luftkode/distro-template/actions/runs/7858139663
"#;

    #[test]
    fn test_take_lines_with_failed_jobs() {
        let failed_jobs = take_lines_with_failed_jobs(TEST_OUTPUT_VIEW_RUN.to_string());
        assert_eq!(failed_jobs.len(), 1, "Failed jobs: {:?}", failed_jobs);
        assert_eq!(
            failed_jobs[0],
            "X Test template xilinx in 5m41s (ID 21442749267)"
        );
    }

    #[test]
    fn test_id_from_job_lines() {
        let job_lines = vec![
            "✓ Test template raspberry in 19m20s (ID 21442749166)".to_string(),
            "X Test template xilinx in 5m41s (ID 21442749267)".to_string(),
            "X Test template other in 5m1s (ID 01449267)".to_string(),
        ];
        let ids = id_from_job_lines(&job_lines);
        assert_eq!(ids.len(), 3, "Job IDs: {:?}", ids);
        assert_eq!(ids[0], "21442749166");
        assert_eq!(ids[1], "21442749267");
        assert_eq!(ids[2], "01449267");
    }

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
    pub fn test_remove_timestamps() {
        let test_str = "ID 8072883145 ";
        let modified = remove_timestamps(test_str);
        assert_eq!(modified, "ID");
    }

    #[test]
    pub fn test_remove_timestamps_log_text() {
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

        let modified = remove_timestamps(LOG_TEXT);
        assert_eq!(
            modified, EXPECTED_MODIFIED,
            "Expected: {EXPECTED_MODIFIED}\nGot: {modified}"
        );
    }
}
