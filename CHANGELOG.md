# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5]

### Fixed
- `list` is now strictly read-only and no longer rewrites `docs/rfcs/.index.json`
  on every invocation. The index is persisted only when a real change is
  detected during refresh.

## [0.1.4]

### Added
- `view` command now renders RFC content through an external Markdown renderer
  when the `RFC_VIEWER` environment variable is set (content is piped on stdin),
  mirroring how `edit` uses `$EDITOR` (RFC-0009).
- `--raw` flag for `view` to force raw Markdown output, ignoring `RFC_VIEWER`.

## [0.1.3]

### Added
- Git-based drift detection for the `doctor` command: `--drift git` (default)
  compares RFC and linked-file freshness using git commit time, with
  `--drift mtime` available as a fallback (RFC-0008).

## [0.1.2]

### Added
- Shell completion generation via the `completions <shell>` command.
- Status transition validation: `set` enforces the allowed RFC lifecycle
  (draft -> review -> accepted -> implemented, plus superseded/deprecated).

## [0.1.1]

### Changed
- Translated CLI help output to English.
- Added an English `README.md`; the Russian version moved to `README_ru.md`.

## [0.1.0]

### Added
- Initial RFC workflow and CLI bootstrap (RFC-0001).
- `init` and `new` commands to scaffold the RFC structure and create RFCs
  (RFC-0002).
- `list`, `view`, `status`, and `edit` commands for reading and navigating RFCs
  (RFC-0003).
- `set`, `check`, and `reindex` commands for status changes, validation, and
  index rebuilds (RFC-0004).
- `link`, `unlink`, and `deps` commands to connect RFCs with source files and
  inspect dependencies (RFC-0005).
- `doctor` command for repository health diagnostics (RFC-0006).
