import 'scripts/check_version_tag.just'
import 'scripts/test_coverage.just'
import 'scripts/util.just'

# Absolute path to the directory containing the utility recipes to invoke them from anywhere
## USAGE: `{{PRINT}} green "Hello world"`
PRINT := join(justfile_directory(), 'scripts/pretty_print.just')
## Usage: `{{PROMPT}} "Are you sure?"` (returns 0 if user answers "yes", 1 otherwise)
PROMPT := join(justfile_directory(), 'scripts/prompt.just') + " prompt"


[private]
@default:
    just --list


# Run Full checks and format
full-check: run-pre-commit lint format check test

# Needs the rust toolchain
env:
    rustc --version
    cargo --version

# Lint the code
lint *ARGS="-- -D warnings --no-deps":
    cargo clippy {{ ARGS }}

# Run pre-commit on all files
run-pre-commit:
    pre-commit run --all-files

# Format the code
format *ARGS:
    cargo fmt {{ ARGS }}

# Check if it compiles without compiling
check *ARGS:
    cargo check {{ ARGS }}


# Run the tests
test *ARGS:
    cargo test {{ ARGS }} -- --test-threads=1

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


## CI specific

_ci_lint: \
    (check "--verbose") \
    (lint "--verbose -- -D warnings --no-deps") \
    (format "-- --check --verbose") \
    (doc "--verbose") \
    check-version \