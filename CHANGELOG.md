# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-11

### Added
- Initial Rust rewrite of the `dejsonlz4` C tool.
- Library API: `encode`, `decode`, `parse_header`, `MAGIC`, `HEADER_SIZE`.
- CLI: `jsonlz4 [-d|-c] IN_FILE [OUT_FILE]` with `-` stdin/stdout support.
- Unit tests for the library and integration tests for the CLI.
- GitHub Actions CI: build, test, fmt, clippy across stable and MSRV (1.85).
- GitHub Actions release workflow: prebuilt binaries for Linux (x86_64,
  aarch64), macOS (x86_64, arm64), and Windows (x86_64) on tag push.

[Unreleased]: https://github.com/stvleung/jsonlz4/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/stvleung/jsonlz4/releases/tag/v0.1.0
