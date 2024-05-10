# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2024-05-10

### Added

- Handling of and *issue formatting* for failed jobs that timed out because they were never picked up by a runner.

### Fixed

- The new feature fixes [issue 4](https://github.com/luftkode/ci-manager/issues/4)

## [0.4.1] - 2024-05-06

### Added

- Allow quiet installations in GitHub CI (default is to echo all script commands)

## [0.4.0] - 2024-04-26

### Added

- Smarter markdown truncating on issue bodies exceeding max content length

### Changed

- Update dependencies
- Update dependencies to `rustls` vulnerability: [RUSTSEC-2024-0336](https://rustsec.org/advisories/RUSTSEC-2024-0336)

## [0.3.1] - 2024-04-26

### Added

- Yocto failures now recognizes the two additional tasks `do_rootfs` & `do_image`

## [0.3.0] - 2024-03-27

### Added

- The `--trim-ansi-codes` flag removes ansi-codes from the log error message when creating issues from a failed run.

## [0.2.1] - 2024-03-19

### Added

- packaging and installation script

## [0.2.0] - 2024-03-19

### Changed

- Allow creating issues from workflows that didn't conclude in failure

## [0.1.1] - 2024-03-18

### Fixed

- crash on unable to find logs for a failed step, now logs an error and continues.

## [0.1.0] - 2024-03-18

Feature parity with [GitHub Workflow Parser](https://crates.io/crates/gh-workflow-parser)
