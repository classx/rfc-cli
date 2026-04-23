# rfc-cli

A CLI tool for managing RFC (Request for Comments) documents in a project.

RFCs are formal specifications that describe design decisions **before** implementation begins. This enables:

- capturing architectural decisions and their rationale
- discussing design before writing code
- giving AI assistants precise context instead of vague requests
- preserving a history of decisions and rejected alternatives

## Installation

```sh
cargo build --release
```

The binary will be at `target/release/rfc-cli`.

For convenience, you can copy it to your PATH:

```sh
cp target/release/rfc-cli ~/.local/bin/
```

## Shell Completion

Generate and install completion scripts:

- Bash: run `rfc-cli completions bash > /path/to/rfc-cli.bash`, then source it (e.g. `source /path/to/rfc-cli.bash`) or place it in your distro's completion directory (for macOS with Homebrew: `/usr/local/etc/bash_completion.d/`).
- Zsh: run `rfc-cli completions zsh > /path/to/_rfc-cli`, ensure the directory is in `$fpath`, then run `autoload -Uz compinit && compinit`.

## Quick Start

```sh
# Initialize the RFC structure in the project
rfc-cli init

# Create a new RFC
rfc-cli new "request caching"

# View the list
rfc-cli list

# Work on the RFC
rfc-cli edit 1
rfc-cli set 1 review

# Validation and diagnostics
rfc-cli check
rfc-cli doctor
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RFC_HOME` | Project root directory | Current directory |
| `EDITOR` | Editor for the `edit` command | — (required for `edit`) |

## Commands

### `init` — initialization

Creates the `docs/rfcs/` directory and an empty index file `.index.json`. Idempotent — safe to call repeatedly.

```sh
rfc-cli init
```

### `new <title>` — create an RFC

Creates a new RFC from a template with automatic numbering (0001, 0002, ...).

```sh
rfc-cli new "request caching"
# → Created docs/rfcs/0007.md
```

The new RFC is created with `draft` status and a template containing the required sections: Problem, Goal, Design, Alternatives, Voting, Migration.

### `list` — list RFCs

Displays a table of all RFCs sorted by number.

```sh
rfc-cli list
#  #      Status      Title
#  0001   implemented RFC-0001: RFC cli structure
#  0002   implemented RFC-0002: implement init and new commands
#  ...

# Filter by status
rfc-cli list --status draft
```

| Flag | Description |
|------|-------------|
| `--status <status>` | Show only RFCs with the specified status |

### `view <number>` — view contents

Prints the full RFC content to the terminal. The number can be specified without leading zeros.

```sh
rfc-cli view 1
# equivalent to: rfc-cli view 0001
```

### `status <number>` — current status

Displays the RFC status from the index (fast, without reading the file).

```sh
rfc-cli status 3
# RFC-0003: implemented
```

### `edit <number>` — editing

Opens the RFC in `$EDITOR`. Blocks editing of `accepted` and `implemented` RFCs without `--force`.

```sh
rfc-cli edit 1

# Force-edit an accepted RFC (with a warning)
rfc-cli edit 1 --force
```

| Flag | Description |
|------|-------------|
| `--force` | Allow editing accepted/implemented RFCs |

### `set <number> <status>` — change status

Changes the RFC status with transition validation. Automatically updates frontmatter, the index, and `content_hash`.

```sh
rfc-cli set 1 review
rfc-cli set 1 accepted
rfc-cli set 1 implemented

# Supersede with another RFC
rfc-cli set 3 superseded --by 7
```

| Flag | Description |
|------|-------------|
| `--by <number>` | Number of the superseding RFC (required for `superseded`) |

Allowed transitions:

```
draft → review → accepted → implemented
                    ↓            ↓
               deprecated    superseded
```

Also allowed: `review → draft`, `draft → deprecated`, `accepted → deprecated`, `implemented → deprecated`.

### `link <number> <path>` — link to code

Associates an RFC with a source code file. The path is stored relative to the project root.

```sh
rfc-cli link 2 src/commands/init.rs
# RFC-0002: linked src/commands/init.rs ✅

# Duplicate links are not added
rfc-cli link 2 src/commands/init.rs
# RFC-0002: link already exists: src/commands/init.rs
```

| Flag | Description |
|------|-------------|
| `--force` | Allow modifying accepted/implemented RFCs (recomputes `content_hash`) |

### `unlink <number> <path>` — remove link

Removes the association between an RFC and a file.

```sh
rfc-cli unlink 2 src/commands/init.rs
# RFC-0002: unlinked src/commands/init.rs ✅
```

| Flag | Description |
|------|-------------|
| `--force` | Allow modifying accepted/implemented RFCs |

### `deps <number>` — dependency tree

Shows forward or reverse dependencies of an RFC.

```sh
# Forward dependencies: what RFC-0005 depends on
rfc-cli deps 5
# RFC-0005 depends on:
#   - RFC-0001 (RFC cli structure) [implemented]
#   - RFC-0003 (list, view, status, edit commands) [implemented]

# Reverse dependencies: what depends on RFC-0001
rfc-cli deps 1 --reverse
# RFC-0001 is depended on by:
#   - RFC-0002 (implement init and new commands) [implemented]
#   - RFC-0003 (list, view, status, edit commands) [implemented]
#   - ...
```

| Flag | Description |
|------|-------------|
| `--reverse` | Show reverse dependencies (what depends on this RFC) |

### `check [<number>]` — format validation

Validates RFC correctness: frontmatter, required sections, links, dependencies, `content_hash` integrity. Without an argument, checks all RFCs.

```sh
rfc-cli check        # all RFCs
rfc-cli check 3      # only RFC-0003
```

Checks performed:
- Valid YAML frontmatter
- Non-empty required fields (`title`, `status`)
- Valid status value
- Required sections present (`## Problem`, `## Goal`, `## Design`, `## Alternatives`)
- Number in filename matches the title
- Dependencies exist
- Files listed in `links` exist
- `content_hash` integrity for accepted/implemented RFCs

Exit code: `0` — all OK, `1` — errors found.

### `doctor` — project health diagnostics

Analyzes semantic consistency between RFCs and the codebase. Unlike `check` (format), `doctor` looks for logical issues.

```sh
rfc-cli doctor
rfc-cli doctor --stale-days 14
```

| Flag | Description |
|------|-------------|
| `--stale-days <N>` | Threshold in days for "stale draft" detection (default: 30) |

Diagnostic checks:

| Check | Level | Description |
|-------|-------|-------------|
| Code drift | ❌ error | A file in `links` was modified after RFC acceptance |
| No implementation | ⚠️ warning | `accepted` RFC with no files in `links` |
| Dead links | ❌ error | A file in `links` does not exist on disk |
| Stale draft | ⚠️ warning | `draft` RFC not updated for more than N days |
| Unresolved dependencies | ⚠️ warning | `accepted` RFC depends on a non-accepted/implemented RFC |
| Circular dependencies | ❌ error | Cycle detected in the dependency graph |

Exit code: `1` if ❌ errors are present, `0` for ⚠️ warnings only or a clean project.

Example output:

```
RFC-0003 (cache):
  ❌ code drift: src/cache/mod.rs modified after RFC acceptance
  ❌ dead link: src/cache/old_store.rs (file not found)

RFC-0005 (logging):
  ⚠️  no linked files (status: accepted)

Summary: 3 error(s), 1 warning(s) across 2 RFC(s).
```

### `reindex` — rebuild index

Completely rebuilds `.index.json` from the RFC files on disk. Use when the index is corrupted or out of sync.

```sh
rfc-cli reindex
# Reindexed 6 RFCs.
```

## Quick Reference

| Command | Description |
|---------|-------------|
| `init` | Create `docs/rfcs/` and the index file |
| `new <title>` | Create an RFC from template |
| `list [--status S]` | List RFCs |
| `view <N>` | View RFC contents |
| `status <N>` | Show RFC status |
| `edit <N> [--force]` | Open in `$EDITOR` |
| `set <N> <S> [--by N]` | Change status |
| `link <N> <path> [--force]` | Link a file |
| `unlink <N> <path> [--force]` | Remove a link |
| `deps <N> [--reverse]` | Dependency tree |
| `check [N]` | Format validation |
| `doctor [--stale-days N]` | Health diagnostics |
| `reindex` | Rebuild index |

## RFC Process

### Rules

1. **No code without an RFC** — any non-trivial change starts with an RFC
2. **RFCs can only be changed through a new RFC** — direct editing of accepted RFCs is forbidden
3. **Statuses are mandatory** — every RFC goes through a defined lifecycle

### Lifecycle

```
draft → review → accepted → implemented
                    ↓            ↓
               deprecated    superseded
```

| Status | Meaning |
|--------|---------|
| `draft` | Work in progress |
| `review` | Ready for discussion |
| `accepted` | Decision approved, ready for implementation |
| `implemented` | Implementation complete |
| `superseded` | Replaced by a new RFC |
| `deprecated` | Obsolete or cancelled without replacement |

### RFC Document Format

Each RFC is a Markdown file in `docs/rfcs/` with YAML frontmatter:

```yaml
---
title: "RFC-0001: title"
status: draft
dependencies: [RFC-0003, RFC-0005]
superseded_by: null
links:
  - src/commands/init.rs
  - src/rfclib/rfc.rs
---

## Problem
## Goal
## Design
## Alternatives
## Voting
## Migration
```

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Title in the format `RFC-NNNN: description` |
| `status` | string | Current status |
| `dependencies` | list | RFC dependencies, e.g. `[RFC-0001]` |
| `superseded_by` | string/null | Number of the superseding RFC |
| `links` | list | Paths to associated source code files |

### Index File

Metadata for all RFCs is cached in `docs/rfcs/.index.json`. The index is updated automatically on every CLI invocation (based on `mtime`). If the index is corrupted, restore it with `rfc-cli reindex`.

For `accepted` and `implemented` RFCs, the index stores a `content_hash` (SHA-256) — protecting against unauthorized modification of approved documents.

## Development

```sh
# Build
make build          # debug
make release        # release

# Tests
make test           # run all tests (95 total)
cargo test <name>   # run a specific test

# Project validation
cargo run -- check
cargo run -- doctor
```

### Project Structure

```
src/
├── main.rs              # entry point, command routing
├── cli.rs               # CLI definition (clap derive)
├── commands/
│   ├── mod.rs
│   ├── init.rs          # rfc-cli init
│   ├── new.rs           # rfc-cli new
│   ├── list.rs          # rfc-cli list
│   ├── view.rs          # rfc-cli view
│   ├── status.rs        # rfc-cli status
│   ├── edit.rs          # rfc-cli edit
│   ├── set.rs           # rfc-cli set
│   ├── link.rs          # rfc-cli link
│   ├── unlink.rs        # rfc-cli unlink
│   ├── deps.rs          # rfc-cli deps
│   ├── check.rs         # rfc-cli check
│   ├── doctor.rs        # rfc-cli doctor
│   └── reindex.rs       # rfc-cli reindex
└── rfclib/
    ├── mod.rs
    ├── rfc.rs           # frontmatter parsing, normalization, field updates
    ├── index.rs         # index load/save, hashing, rebuild
    └── project.rs       # project root resolution (RFC_HOME)

tests/
└── integration_test.rs  # 95 integration tests

docs/rfcs/
├── .index.json          # index file (generated)
├── 0001.md              # RFC-0001: RFC cli structure
├── 0002.md              # RFC-0002: implement init and new commands
├── 0003.md              # RFC-0003: implement list, view, status, edit commands
├── 0004.md              # RFC-0004: implement set, check, reindex commands
├── 0005.md              # RFC-0005: implement link, unlink, deps commands
└── 0006.md              # RFC-0006: implement doctor command
```

## License

See [LICENSE](LICENSE).