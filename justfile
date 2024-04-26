import 'scripts/check_version_tag.just'
import 'scripts/test_coverage.just'
import 'scripts/util.just'
import 'scripts/fast_compile.just'


# Absolute path to the directory containing the utility recipes to invoke them from anywhere
## USAGE: `{{PRINT}} green "Hello world"`
PRINT := join(justfile_directory(), 'scripts/pretty_print.just')
## Usage: `{{PROMPT}} "Are you sure?"` (returns 0 if user answers "yes", 1 otherwise)
PROMPT := join(justfile_directory(), 'scripts/prompt.just') + " prompt"

[private]
@default:
    just --list

# Run Full checks and format
[no-exit-message]
full-check: run-pre-commit lint format check test

# Needs the rust toolchain
env:
    rustc --version
    cargo --version

# Lint the code
[no-exit-message]
lint *ARGS="-- -D warnings --no-deps":
    cargo clippy {{ ARGS }}

# Run pre-commit on all files
[no-exit-message]
run-pre-commit:
    pre-commit run --all-files

# Format the code
[no-exit-message]
format *ARGS:
    cargo fmt {{ ARGS }}

# Check if it compiles without compiling
[no-exit-message]
check *ARGS:
    cargo check {{ ARGS }}

# Run the tests
[no-exit-message]
test *ARGS:
    cargo test {{ ARGS }}

# Run tests and collect coverage
test-coverage: run-test-coverage
# Open the test report that comes out of the test-coverage recipe
coverage-report: open-coverage-report

# Build the application
build *ARGS:
    cargo build {{ ARGS }}

# Run the application (use `--` to pass arguments to the application)
run ARGS:
    cargo run {{ ARGS }}

# Clean the `target` directory
clean:
    cargo clean

# Build the documentation (use `--open` to open in the browser)
doc *ARGS:
    cargo doc {{ ARGS }}

# Publish the crate
publish:
    cargo publish

# List the dependencies
deps:
    cargo tree

# Update the dependencies
update:
    cargo update

# Audit Cargo.lock files for crates containing security vulnerabilities
audit *ARGS:
    #!/usr/bin/env bash
    if ! which cargo-audit >/dev/null; then
        {{PRINT}} yellow "cargo-audit not found"
        just prompt-install "cargo install cargo-audit"
    fi
    cargo audit {{ ARGS }}

# These tests require a token with public repo read access
test-github-auth-required *CARGO_TEST_ARGS:
    #!/usr/bin/env bash
    if [ -z "${GITHUB_TOKEN}" ]; then
        {{PRINT}} red "GITHUB_TOKEN not found, please export it as an environment variable"
        exit 1
    fi
    # Before adding tests here, confirm that it actually runs when you run
    # $ cargo test <test_name> -- --exact --ignored
    declare -a -r AUTH_REQUIRED_TESTS=(
        "create_issue_from_failed_run_yocto"
        "ci_provider::github::tests::test_download_workflow_run_logs"
        "ci_provider::github::tests::test_get_workflow_run_jobs"
    )
    for test in "${AUTH_REQUIRED_TESTS[@]}"; do
        cargo test {{CARGO_TEST_ARGS}} "${test}" -- --exact --ignored | grep "running 1 test"
    done

## CI specific recipes (run these to check if the code passes CI)
ci-lint: \
    (check "--verbose") \
    (lint "--verbose -- -D warnings --no-deps") \
    (format "-- --check --verbose") \
    (doc "--verbose --no-deps") \
    check-version \

ci-test: \
    (test "--verbose")

ci-auth-required-test: test-github-auth-required

# Pushes HEAD + latest tag atomically
[no-exit-message]
atomic-push-with-tags TAG=`git describe --tags --abbrev=0`:
    git push --atomic origin main {{TAG}}

annotated-tag-version VERSION MSG:
    git tag -a v{{VERSION}} -m "{{MSG}}"
