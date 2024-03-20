# CI manager

# Purpose

Manage and automate CI in more complex scenarious such as automatic issue creation and triggering workflows in other repositories.

## Roadmap
- [x] (GitHub) Create issues from failed runs, with multiple configuration options, such as adding labels depending on the failed steps, and much more.
- [ ] (GitHub) Trigger workflows in another repository

# Installation

Install with `cargo install ci-manager` if you have the Rust toolchain installed.

On Windows or x86_64 linux, prebuilt binaries can be installed with:
```shell
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/luftkode/ci-manager/main/scripts/install.sh | bash -s -- --to ~/bin
```
In CI you might just want to install into something you know is in path, to save you the trouble e.g.
```shell
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/luftkode/ci-manager/main/scripts/install.sh | sudo bash -s -- --to $(dirname $(which curl)) --force
```
Or some variation there of.

# Usage
Run `ci-manager --help` to get started.

## Example

# Example

```shell
ci-manager \
    --ci=github \
    --verbosity=2 \
        create-issue-from-run \
            --repo=https://github.com/docker/buildx \
            --run-id=8302026485 \
            --title="CI scheduled build" \
            --label=bug \
            --kind=other \
            --trim-timestamp \
            --dry-run
```

## Example of a created issue's body
**Run ID**: 7945016152 [LINK TO RUN](https://github.com/luftkode/distro-template/actions/runs/7945016152)

**1 job failed:**
- **`Test template xilinx`**

### `Test template xilinx` (ID 21697280386)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7945016152/job/21697280386
\
*Best effort error summary*:
```
ERROR: sqlite3-native-3_3.43.2-r0 do_fetch: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')
ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21669
ERROR: Task (virtual:native:/app/yocto/build/../poky/meta/recipes-support/sqlite/sqlite3_3.43.2.bb:do_fetch) failed with exit code '1'

2024-02-18 09:08:45 - ERROR    - Command "/app/yocto/poky/bitbake/bin/bitbake -c build test-template-ci-xilinx-image package-index" failed with error 1
```
<details>
<summary>log.do_fetch</summary>
<br>

```
DEBUG: Executing python function extend_recipe_sysroot
NOTE: Direct dependencies are []
NOTE: Installed into sysroot: []
NOTE: Skipping as already exists in sysroot: []
DEBUG: Python function extend_recipe_sysroot finished
DEBUG: Executing python function fetcher_hashes_dummyfunc
DEBUG: Python function fetcher_hashes_dummyfunc finished
DEBUG: Executing python function do_fetch
DEBUG: Executing python function base_do_fetch
DEBUG: Trying PREMIRRORS
ERROR: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')
DEBUG: Python function base_do_fetch finished
DEBUG: Python function do_fetch finished

```
</details>
