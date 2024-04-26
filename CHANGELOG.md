# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
