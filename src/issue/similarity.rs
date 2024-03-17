use crate::*;

/// The maximum Levenshtein distance for issues to be considered similar.
pub const LEVENSHTEIN_THRESHOLD: usize = 100;

/// Calculate the smallest levenshtein distance between the issue body and the other issues with the same label
pub fn issue_text_similarity(issue_body: &str, other_issues: &[String]) -> usize {
    let issue_body_without_timestamps = util::remove_timestamps_and_ids(issue_body);

    let smallest_distance = other_issues
        .iter()
        .map(|other_issue_body| {
            distance::levenshtein(
                &issue_body_without_timestamps,
                &util::remove_timestamps_and_ids(other_issue_body),
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

    const EXAMPLE_ISSUE_BODY_0: &str = r#"**Run ID**: 7858139663 [LINK TO RUN]( https://github.com/luftkode/distro-template/actions/runs/7850874958)

**2 jobs failed:**
- **`Test template xilinx`**
- **`Test template raspberry`**

### `Test template xilinx` (ID 21442749267)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749267
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```
### `Test template raspberry` (ID 21442749166)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749166
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```"#;

    const EXAMPLE_ISSUE_BODY_1: &str = r#"**Run ID**: 7858139663 [LINK TO RUN]( https://github.com/luftkode/distro-template/actions/runs/7850874958)

**2 jobs failed:**
- **`Test template xilinx`**
- **`Test template raspberry`**

### `Test template xilinx` (ID 21442749267)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749267
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```
### `Test template raspberry` (ID 21442749166)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749166
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```"#;

    #[test]
    fn test_issue_body_distance() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let distance = issue_text_similarity(&issue_0, &[issue_1]);
        assert_eq!(distance, 0);
    }

    /// Identical except for very similar run and job IDs
    #[test]
    fn test_issue_body_distance_edit_minimal_diff() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let new_run_id = "7858139660";
        let new_job0_id = "21442749260";
        let new_job1_id = "21442749200";

        let issue_1 = issue_1.replace("7858139663", new_run_id);
        let issue_1 = issue_1.replace("21442749267", new_job0_id);
        let issue_1 = issue_1.replace("21442749166", new_job1_id);

        let distance = issue_text_similarity(&issue_0, &[issue_1]);
        assert_eq!(distance, 0); // No difference as IDs are now masked when comparing
    }

    /// Identical except for as different run and job IDs as possible
    #[test]
    fn test_issue_body_distance_edit_largest_similar() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let new_run_id = "0000000000";
        let new_job0_id = "00000000000";
        let new_job1_id = "33333333333";

        let issue_1 = issue_1.replace("7858139663", new_run_id);
        let issue_1 = issue_1.replace("21442749267", new_job0_id);
        let issue_1 = issue_1.replace("21442749166", new_job1_id);

        let distance = issue_text_similarity(&issue_0, &[issue_1]);
        assert_eq!(distance, 0); // No difference as IDs are now masked when comparing
    }

    /// Smallest difference in job and run IDs but different in other ways and should be treated as different.
    #[test]
    fn test_issue_body_distance_edit_minimal_but_different() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let new_run_id = "7858139660";
        let new_job0_id = "21442749260";
        let new_job1_id = "21442749200";

        let issue_1 = issue_1.replace("7858139663", new_run_id);
        let issue_1 = issue_1.replace("21442749267", new_job0_id);
        let issue_1 = issue_1.replace("21442749166", new_job1_id);
        let issue_1 = issue_1.replace(
            "Yocto error: ERROR: No recipes available for: ...",
            "ERROR: fetcher failure. malformed url. Attempting to fetch from ${SOURCE_MIRROR_URL}",
        );

        let distance = issue_text_similarity(&issue_0, &[issue_1]);
        assert_eq!(distance, 142);
    }

    // Regression test for https://github.com/luftkode/gh-workflow-parser/issues/9
    /// Large issue text with many timestamps doesn't make the issues dissimilar
    #[test]
    fn test_issue9_similar_with_frequest_timestamps() {
        let distance = issue_text_similarity(
            ISSUE_FREQUENT_TIMESTAMPS_TEXT1,
            &[ISSUE_FREQUENT_TIMESTAMPS_TEXT2.to_string()],
        );

        assert!(distance < LEVENSHTEIN_THRESHOLD, "Distance: {distance}");
    }

    const ISSUE_FREQUENT_TIMESTAMPS_TEXT1: &'static str = r#"**Run ID**: 8072883145 [LINK TO RUN](https://github.com/luftkode/distro-template/actions/runs/8072883145)

**1 job failed:**
- **`Test template xilinx`**

### `Test template xilinx` (ID 22055505284)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/8072883145/job/22055505284
\
*Best effort error summary*:
```
##[group]Run set -ou pipefail
[36;1mset -ou pipefail[0m
[36;1mjust --yes build-ci-image 2>&1 | tee "yocto_build.log"[0m
shell: /usr/bin/bash --noprofile --norc -e -o pipefail {0}
env:
  REGISTRY: ghcr.io
  CARGO_TERM_COLOR: always
  SSH_AUTH_SOCK: /tmp/ssh-XXXXXXJhUcQF/agent.2944549
  CLONE_TO_DIR: clone-to-dir
  YOCTO_BUILD_LOG: yocto_build.log
##[endgroup]
ABS_KAS_FRAG_CFG_DIR: /opt/github_runner_yocto/_work/distro-template/distro-template/clone-to-dir/test-template-ci-xilinx-distro/distro-builder-common/kas-config-fragments
[[BEFORE RUN]] - copying kas config: build_ci.yml from: /opt/github_runner_yocto/_work/distro-template/distro-template/clone-to-dir/test-template-ci-xilinx-distro/distro-builder-common/kas-config-fragments to: /opt/github_runner_yocto/_work/distro-template/distro-template/clone-to-dir/test-template-ci-xilinx-distro/main-image
---
[1;32mSetting bitbake environment variables: export BB_ENV_PASSTHROUGH_ADDITIONS=YOCTO_BRANCH; export YOCTO_BRANCH=nanbield;[0m
---
[1;32mRunning command(s): just in-container-build-ci-image[0m
---
[1;32mNot in container, running command(s) through: "docker compose" with image=image[0m
[1;32m---[0m
 Network distro-builder-common_default  Creating
 Network distro-builder-common_default  Created
~/kas/run-kas shell "main-image/image.yml:main-image/build_ci.yml" -c "bitbake -c cleansstate virtual/bootloader virtual/kernel"
2024-02-28 00:03:44 - INFO     - run-kas 4.2 started
2024-02-28 00:03:44 - INFO     - /app/main-image$ git rev-parse --show-toplevel
2024-02-28 00:03:44 - INFO     - /app/main-image$ git rev-parse --show-toplevel
2024-02-28 00:03:44 - INFO     - /app/main-image$ git rev-parse --show-toplevel
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q https://git.yoctoproject.org/poky /app/yocto/poky/
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q https://github.com/openembedded/meta-openembedded /app/yocto/layers/meta-openembedded
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q git@github.com:luftkode/meta-airborne.git /app/yocto/layers/meta-airborne
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q git@github.com:luftkode/meta-skytem-xilinx.git /app/yocto/layers/meta-skytem-xilinx
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q https://github.com/Xilinx/meta-xilinx /app/yocto/layers/meta-xilinx
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q https://github.com/Xilinx/meta-xilinx-tools /app/yocto/layers/meta-xilinx-tools
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q https://git.yoctoproject.org/meta-virtualization/ /app/yocto/layers/meta-virtualization
2024-02-28 00:03:44 - INFO     - /app/yocto$ git clone -q git@github.com:rust-embedded/meta-rust-bin.git /app/yocto/layers/meta-rust-bin
2024-02-28 00:03:45 - ERROR    - Warning: Permanently added 'github.com' (ED25519) to the list of known hosts.
2024-02-28 00:03:45 - ERROR    - Warning: Permanently added 'github.com' (ED25519) to the list of known hosts.
2024-02-28 00:03:45 - ERROR    - Warning: Permanently added 'github.com' (ED25519) to the list of known hosts.
2024-02-28 00:03:45 - INFO     - Repository meta-virtualization cloned
2024-02-28 00:03:45 - INFO     - /app/yocto/layers/meta-virtualization$ git remote set-url origin https://git.yoctoproject.org/meta-virtualization/
2024-02-28 00:03:45 - INFO     - /app/yocto/layers/meta-virtualization$ git cat-file -t ac125d881f34ff356390e19e02964f8980d4ec38
2024-02-28 00:03:45 - INFO     - Repository meta-virtualization already contains ac125d881f34ff356390e19e02964f8980d4ec38 as commit
2024-02-28 00:03:45 - INFO     - Repository meta-xilinx-tools cloned
2024-02-28 00:03:45 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git remote set-url origin https://github.com/Xilinx/meta-xilinx-tools
2024-02-28 00:03:45 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git cat-file -t 92b449c333e3a991735388f4cc0e38ec97e1f9ad
2024-02-28 00:03:45 - INFO     - Repository meta-xilinx-tools already contains 92b449c333e3a991735388f4cc0e38ec97e1f9ad as commit
2024-02-28 00:03:46 - INFO     - Repository meta-xilinx cloned
2024-02-28 00:03:46 - INFO     - /app/yocto/layers/meta-xilinx$ git remote set-url origin https://github.com/Xilinx/meta-xilinx
2024-02-28 00:03:46 - INFO     - /app/yocto/layers/meta-xilinx$ git cat-file -t a1c7db00727d02b8cd47d665fee86f75b0f83080
2024-02-28 00:03:46 - INFO     - Repository meta-xilinx already contains a1c7db00727d02b8cd47d665fee86f75b0f83080 as commit
2024-02-28 00:03:46 - INFO     - Repository meta-skytem-xilinx cloned
2024-02-28 00:03:46 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git remote set-url origin git@github.com:luftkode/meta-skytem-xilinx.git
2024-02-28 00:03:46 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git cat-file -t e8cf7aaa8c9d4d72a702fa0577421446aa38b223
2024-02-28 00:03:46 - INFO     - Repository meta-skytem-xilinx already contains e8cf7aaa8c9d4d72a702fa0577421446aa38b223 as commit
2024-02-28 00:03:46 - INFO     - Repository meta-rust-bin cloned
2024-02-28 00:03:46 - INFO     - /app/yocto/layers/meta-rust-bin$ git remote set-url origin git@github.com:rust-embedded/meta-rust-bin.git
2024-02-28 00:03:46 - INFO     - /app/yocto/layers/meta-rust-bin$ git cat-file -t 019e3b0073510e6f39fac23d9a3c2dd6d793a2fe
2024-02-28 00:03:46 - INFO     - Repository meta-rust-bin already contains 019e3b0073510e6f39fac23d9a3c2dd6d793a2fe as commit
2024-02-28 00:03:48 - INFO     - Repository meta-airborne cloned
2024-02-28 00:03:48 - INFO     - /app/yocto/layers/meta-airborne$ git remote set-url origin git@github.com:luftkode/meta-airborne.git
2024-02-28 00:03:48 - INFO     - /app/yocto/layers/meta-airborne$ git cat-file -t fa6e5f001067ffe6f19e038a8a87cc06d409cafa
2024-02-28 00:03:48 - INFO     - Repository meta-airborne already contains fa6e5f001067ffe6f19e038a8a87cc06d409cafa as commit
2024-02-28 00:03:49 - INFO     - Repository meta-openembedded cloned
2024-02-28 00:03:49 - INFO     - /app/yocto/layers/meta-openembedded$ git remote set-url origin https://github.com/openembedded/meta-openembedded
2024-02-28 00:03:50 - INFO     - /app/yocto/layers/meta-openembedded$ git cat-file -t da9063bdfbe130f424ba487f167da68e0ce90e7d
2024-02-28 00:03:50 - INFO     - Repository meta-openembedded already contains da9063bdfbe130f424ba487f167da68e0ce90e7d as commit
2024-02-28 00:04:00 - INFO     - Repository poky cloned
2024-02-28 00:04:00 - INFO     - /app/yocto/poky/$ git remote set-url origin https://git.yoctoproject.org/poky
2024-02-28 00:04:00 - INFO     - /app/yocto/poky/$ git cat-file -t 1a5c00f00c14cee3ba5d39c8c8db7a9738469eab
2024-02-28 00:04:00 - INFO     - Repository poky already contains 1a5c00f00c14cee3ba5d39c8c8db7a9738469eab as commit
2024-02-28 00:04:00 - INFO     - /app/yocto/poky/$ git status -s
2024-02-28 00:04:00 - INFO     - /app/yocto/poky/$ git checkout -q 1a5c00f00c14cee3ba5d39c8c8db7a9738469eab
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-openembedded$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-openembedded$ git checkout -q da9063bdfbe130f424ba487f167da68e0ce90e7d
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-airborne$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-airborne$ git checkout -q fa6e5f001067ffe6f19e038a8a87cc06d409cafa
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git checkout -q e8cf7aaa8c9d4d72a702fa0577421446aa38b223
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-xilinx$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-xilinx$ git checkout -q a1c7db00727d02b8cd47d665fee86f75b0f83080
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git checkout -q 92b449c333e3a991735388f4cc0e38ec97e1f9ad
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-virtualization$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-virtualization$ git checkout -q ac125d881f34ff356390e19e02964f8980d4ec38
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-rust-bin$ git status -s
2024-02-28 00:04:01 - INFO     - /app/yocto/layers/meta-rust-bin$ git checkout -q 019e3b0073510e6f39fac23d9a3c2dd6d793a2fe
2024-02-28 00:04:01 - INFO     - /app/yocto/poky/$ /tmp/tmpm4x_iz34/get_bb_env /app/yocto/build
2024-02-28 00:04:01 - INFO     - To start the default build, run: bitbake -c build test-template-ci-xilinx-image package-index
Loading cache...done.
Loaded 0 entries from dependency cache.
Parsing recipes...ERROR: ParseError at /app/yocto/build/../layers/meta-skytem-xilinx/recipes-bundles/zynq-update-bundle/zynq-update-bundle.bb:1: Could not inherit file classes/bundle.bbclass
ERROR: Parsing halted due to errors, see error messages above

Summary: There were 2 ERROR messages, returning a non-zero exit code.
2024-02-28 00:05:18 - ERROR    - Shell returned non-zero exit status
2024-02-28 00:05:18 - ERROR    - Command "/bin/bash -c 'bitbake -c cleansstate virtual/bootloader virtual/kernel'" failed with error 1
error: Recipe `in-container-build-ci-image` failed on line 30 with exit code 1
error: Recipe `run-in-docker` failed with exit code 1
error: Recipe `build-ci-image` failed with exit code 1
##[error]Process completed with exit code 1.
##[group]Run cargo install gh-workflow-parser --profile release-ci && gh-workflow-parser --version
[36;1mcargo install gh-workflow-parser --profile release-ci && gh-workflow-parser --version[0m
[36;1mfailure_log_abs_path=$( gh-workflow-parser locate-failure-log --input-file="yocto_build.log" --kind=yocto )[0m
[36;1mfailure_log_basename=$( basename "${failure_log_abs_path}" )[0m
[36;1mecho "failure_log_abs_path=${failure_log_abs_path}"[0m
[36;1mecho "failure_log_basename=${failure_log_basename}"[0m
[36;1mecho "YOCTO_FAILED_LOG_PATH=${failure_log_abs_path}" >> $GITHUB_ENV[0m
[36;1mecho "YOCTO_FAILED_LOG_BASENAME=${failure_log_basename}" >> $GITHUB_ENV[0m
shell: /usr/bin/bash --noprofile --norc -e -o pipefail {0}
env:
  REGISTRY: ghcr.io
  CARGO_TERM_COLOR: always
  SSH_AUTH_SOCK: /tmp/ssh-XXXXXXJhUcQF/agent.2944549
  CLONE_TO_DIR: clone-to-dir
  YOCTO_BUILD_LOG: yocto_build.log
##[endgroup]
[1m[32m    Updating[0m crates.io index
[1m[32m     Ignored[0m package `gh-workflow-parser v0.5.3` is already installed, use --force to override
gh-workflow-parser 0.5.3
INFO Locating failure log for kind: Yocto
INFO Reading log file: "yocto_build.log"
ERROR No log file line found
##[error]Process completed with exit code 1.
##[group]Run actions/upload-artifact@v4
with:
  retention-days: 7
  if-no-files-found: warn
  compression-level: 6
  overwrite: false
env:
  REGISTRY: ghcr.io
  CARGO_TERM_COLOR: always
  SSH_AUTH_SOCK: /tmp/ssh-XXXXXXJhUcQF/agent.2944549
  CLONE_TO_DIR: clone-to-dir
  YOCTO_BUILD_LOG: yocto_build.log
##[endgroup]
##[error]Input required and not supplied: path
```"#;

    const ISSUE_FREQUENT_TIMESTAMPS_TEXT2: &'static str = r#"**Run ID**: 8057183947 [LINK TO RUN](https://github.com/luftkode/distro-template/actions/runs/8057183947)

**1 job failed:**
- **`Test template xilinx`**

### `Test template xilinx` (ID 22007767507)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/8057183947/job/22007767507
\
*Best effort error summary*:
```
##[group]Run set -ou pipefail
[36;1mset -ou pipefail[0m
[36;1mjust --yes build-ci-image 2>&1 | tee "yocto_build.log"[0m
shell: /usr/bin/bash --noprofile --norc -e -o pipefail {0}
env:
  REGISTRY: ghcr.io
  CARGO_TERM_COLOR: always
  SSH_AUTH_SOCK: /tmp/ssh-XXXXXXooAvJr/agent.4134531
  CLONE_TO_DIR: clone-to-dir
  YOCTO_BUILD_LOG: yocto_build.log
##[endgroup]
ABS_KAS_FRAG_CFG_DIR: /opt/github_runner_yocto/_work/distro-template/distro-template/clone-to-dir/test-template-ci-xilinx-distro/distro-builder-common/kas-config-fragments
[[BEFORE RUN]] - copying kas config: build_ci.yml from: /opt/github_runner_yocto/_work/distro-template/distro-template/clone-to-dir/test-template-ci-xilinx-distro/distro-builder-common/kas-config-fragments to: /opt/github_runner_yocto/_work/distro-template/distro-template/clone-to-dir/test-template-ci-xilinx-distro/main-image
---
[1;32mSetting bitbake environment variables: export BB_ENV_PASSTHROUGH_ADDITIONS=YOCTO_BRANCH; export YOCTO_BRANCH=nanbield;[0m
---
[1;32mRunning command(s): just in-container-build-ci-image[0m
---
[1;32mNot in container, running command(s) through: "docker compose" with image=image[0m
[1;32m---[0m
 Network distro-builder-common_default  Creating
 Network distro-builder-common_default  Created
~/kas/run-kas shell "main-image/image.yml:main-image/build_ci.yml" -c "bitbake -c cleansstate virtual/bootloader virtual/kernel"
2024-02-26 23:44:33 - INFO     - run-kas 4.2 started
2024-02-26 23:44:33 - INFO     - /app/main-image$ git rev-parse --show-toplevel
2024-02-26 23:44:33 - INFO     - /app/main-image$ git rev-parse --show-toplevel
2024-02-26 23:44:33 - INFO     - /app/main-image$ git rev-parse --show-toplevel
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q https://git.yoctoproject.org/poky /app/yocto/poky/
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q https://github.com/openembedded/meta-openembedded /app/yocto/layers/meta-openembedded
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q git@github.com:luftkode/meta-airborne.git /app/yocto/layers/meta-airborne
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q git@github.com:luftkode/meta-skytem-xilinx.git /app/yocto/layers/meta-skytem-xilinx
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q https://github.com/Xilinx/meta-xilinx /app/yocto/layers/meta-xilinx
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q https://github.com/Xilinx/meta-xilinx-tools /app/yocto/layers/meta-xilinx-tools
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q https://git.yoctoproject.org/meta-virtualization/ /app/yocto/layers/meta-virtualization
2024-02-26 23:44:33 - INFO     - /app/yocto$ git clone -q git@github.com:rust-embedded/meta-rust-bin.git /app/yocto/layers/meta-rust-bin
2024-02-26 23:44:33 - ERROR    - Warning: Permanently added 'github.com' (ED25519) to the list of known hosts.
2024-02-26 23:44:33 - ERROR    - Warning: Permanently added 'github.com' (ED25519) to the list of known hosts.
2024-02-26 23:44:33 - ERROR    - Warning: Permanently added 'github.com' (ED25519) to the list of known hosts.
2024-02-26 23:44:34 - INFO     - Repository meta-virtualization cloned
2024-02-26 23:44:34 - INFO     - /app/yocto/layers/meta-virtualization$ git remote set-url origin https://git.yoctoproject.org/meta-virtualization/
2024-02-26 23:44:34 - INFO     - /app/yocto/layers/meta-virtualization$ git cat-file -t ac125d881f34ff356390e19e02964f8980d4ec38
2024-02-26 23:44:34 - INFO     - Repository meta-virtualization already contains ac125d881f34ff356390e19e02964f8980d4ec38 as commit
2024-02-26 23:44:34 - INFO     - Repository meta-xilinx-tools cloned
2024-02-26 23:44:34 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git remote set-url origin https://github.com/Xilinx/meta-xilinx-tools
2024-02-26 23:44:34 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git cat-file -t 92b449c333e3a991735388f4cc0e38ec97e1f9ad
2024-02-26 23:44:34 - INFO     - Repository meta-xilinx-tools already contains 92b449c333e3a991735388f4cc0e38ec97e1f9ad as commit
2024-02-26 23:44:34 - INFO     - Repository meta-xilinx cloned
2024-02-26 23:44:34 - INFO     - /app/yocto/layers/meta-xilinx$ git remote set-url origin https://github.com/Xilinx/meta-xilinx
2024-02-26 23:44:35 - INFO     - /app/yocto/layers/meta-xilinx$ git cat-file -t a1c7db00727d02b8cd47d665fee86f75b0f83080
2024-02-26 23:44:35 - INFO     - Repository meta-xilinx already contains a1c7db00727d02b8cd47d665fee86f75b0f83080 as commit
2024-02-26 23:44:35 - INFO     - Repository meta-skytem-xilinx cloned
2024-02-26 23:44:35 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git remote set-url origin git@github.com:luftkode/meta-skytem-xilinx.git
2024-02-26 23:44:35 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git cat-file -t e8cf7aaa8c9d4d72a702fa0577421446aa38b223
2024-02-26 23:44:35 - INFO     - Repository meta-skytem-xilinx already contains e8cf7aaa8c9d4d72a702fa0577421446aa38b223 as commit
2024-02-26 23:44:35 - INFO     - Repository meta-rust-bin cloned
2024-02-26 23:44:35 - INFO     - /app/yocto/layers/meta-rust-bin$ git remote set-url origin git@github.com:rust-embedded/meta-rust-bin.git
2024-02-26 23:44:35 - INFO     - /app/yocto/layers/meta-rust-bin$ git cat-file -t 019e3b0073510e6f39fac23d9a3c2dd6d793a2fe
2024-02-26 23:44:35 - INFO     - Repository meta-rust-bin already contains 019e3b0073510e6f39fac23d9a3c2dd6d793a2fe as commit
2024-02-26 23:44:37 - INFO     - Repository meta-airborne cloned
2024-02-26 23:44:37 - INFO     - /app/yocto/layers/meta-airborne$ git remote set-url origin git@github.com:luftkode/meta-airborne.git
2024-02-26 23:44:37 - INFO     - /app/yocto/layers/meta-airborne$ git cat-file -t fa6e5f001067ffe6f19e038a8a87cc06d409cafa
2024-02-26 23:44:37 - INFO     - Repository meta-airborne already contains fa6e5f001067ffe6f19e038a8a87cc06d409cafa as commit
2024-02-26 23:44:38 - INFO     - Repository meta-openembedded cloned
2024-02-26 23:44:38 - INFO     - /app/yocto/layers/meta-openembedded$ git remote set-url origin https://github.com/openembedded/meta-openembedded
2024-02-26 23:44:38 - INFO     - /app/yocto/layers/meta-openembedded$ git cat-file -t da9063bdfbe130f424ba487f167da68e0ce90e7d
2024-02-26 23:44:38 - INFO     - Repository meta-openembedded already contains da9063bdfbe130f424ba487f167da68e0ce90e7d as commit
2024-02-26 23:44:49 - INFO     - Repository poky cloned
2024-02-26 23:44:49 - INFO     - /app/yocto/poky/$ git remote set-url origin https://git.yoctoproject.org/poky
2024-02-26 23:44:49 - INFO     - /app/yocto/poky/$ git cat-file -t 1a5c00f00c14cee3ba5d39c8c8db7a9738469eab
2024-02-26 23:44:49 - INFO     - Repository poky already contains 1a5c00f00c14cee3ba5d39c8c8db7a9738469eab as commit
2024-02-26 23:44:49 - INFO     - /app/yocto/poky/$ git status -s
2024-02-26 23:44:49 - INFO     - /app/yocto/poky/$ git checkout -q 1a5c00f00c14cee3ba5d39c8c8db7a9738469eab
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-openembedded$ git status -s
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-openembedded$ git checkout -q da9063bdfbe130f424ba487f167da68e0ce90e7d
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-airborne$ git status -s
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-airborne$ git checkout -q fa6e5f001067ffe6f19e038a8a87cc06d409cafa
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git status -s
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-skytem-xilinx$ git checkout -q e8cf7aaa8c9d4d72a702fa0577421446aa38b223
2024-02-26 23:44:49 - INFO     - /app/yocto/layers/meta-xilinx$ git status -s
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-xilinx$ git checkout -q a1c7db00727d02b8cd47d665fee86f75b0f83080
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git status -s
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-xilinx-tools$ git checkout -q 92b449c333e3a991735388f4cc0e38ec97e1f9ad
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-virtualization$ git status -s
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-virtualization$ git checkout -q ac125d881f34ff356390e19e02964f8980d4ec38
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-rust-bin$ git status -s
2024-02-26 23:44:50 - INFO     - /app/yocto/layers/meta-rust-bin$ git checkout -q 019e3b0073510e6f39fac23d9a3c2dd6d793a2fe
2024-02-26 23:44:50 - INFO     - /app/yocto/poky/$ /tmp/tmpjxb_xapi/get_bb_env /app/yocto/build
2024-02-26 23:44:50 - INFO     - To start the default build, run: bitbake -c build test-template-ci-xilinx-image package-index
Loading cache...done.
Loaded 0 entries from dependency cache.
Parsing recipes...ERROR: ParseError at /app/yocto/build/../layers/meta-skytem-xilinx/recipes-bundles/zynq-update-bundle/zynq-update-bundle.bb:1: Could not inherit file classes/bundle.bbclass
ERROR: Parsing halted due to errors, see error messages above

Summary: There were 2 ERROR messages, returning a non-zero exit code.
2024-02-26 23:46:07 - ERROR    - Shell returned non-zero exit status
2024-02-26 23:46:07 - ERROR    - Command "/bin/bash -c 'bitbake -c cleansstate virtual/bootloader virtual/kernel'" failed with error 1
error: Recipe `in-container-build-ci-image` failed on line 30 with exit code 1
error: Recipe `run-in-docker` failed with exit code 1
error: Recipe `build-ci-image` failed with exit code 1
##[error]Process completed with exit code 1.
##[group]Run cargo install gh-workflow-parser --profile release-ci && gh-workflow-parser --version
[36;1mcargo install gh-workflow-parser --profile release-ci && gh-workflow-parser --version[0m
[36;1mfailure_log_abs_path=$( gh-workflow-parser locate-failure-log --input-file="yocto_build.log" --kind=yocto )[0m
[36;1mfailure_log_basename=$( basename "${failure_log_abs_path}" )[0m
[36;1mecho "failure_log_abs_path=${failure_log_abs_path}"[0m
[36;1mecho "failure_log_basename=${failure_log_basename}"[0m
[36;1mecho "YOCTO_FAILED_LOG_PATH=${failure_log_abs_path}" >> $GITHUB_ENV[0m
[36;1mecho "YOCTO_FAILED_LOG_BASENAME=${failure_log_basename}" >> $GITHUB_ENV[0m
shell: /usr/bin/bash --noprofile --norc -e -o pipefail {0}
env:
  REGISTRY: ghcr.io
  CARGO_TERM_COLOR: always
  SSH_AUTH_SOCK: /tmp/ssh-XXXXXXooAvJr/agent.4134531
  CLONE_TO_DIR: clone-to-dir
  YOCTO_BUILD_LOG: yocto_build.log
##[endgroup]
[1m[32m    Updating[0m crates.io index
[1m[32m     Ignored[0m package `gh-workflow-parser v0.5.3` is already installed, use --force to override
gh-workflow-parser 0.5.3
INFO Locating failure log for kind: Yocto
INFO Reading log file: "yocto_build.log"
ERROR No log file line found
##[error]Process completed with exit code 1.
##[group]Run actions/upload-artifact@v4
with:
  retention-days: 7
  if-no-files-found: warn
  compression-level: 6
  overwrite: false
env:
  REGISTRY: ghcr.io
  CARGO_TERM_COLOR: always
  SSH_AUTH_SOCK: /tmp/ssh-XXXXXXooAvJr/agent.4134531
  CLONE_TO_DIR: clone-to-dir
  YOCTO_BUILD_LOG: yocto_build.log
##[endgroup]
##[error]Input required and not supplied: path
```"#;
}
