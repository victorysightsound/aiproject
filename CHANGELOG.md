# Changelog

All notable changes to proj are documented here.

## [1.5.4] - 2026-01-25

### Changed
- **Session summary guidance**: Updated documentation and help text to encourage substantive 1-3 sentence summaries that answer "what was accomplished?" rather than generic summaries like "reviewed status"
- Updated CLI help text, docs (manual, getting-started, cheatsheet), and SESSION_RULE template for new projects

## [1.5.3] - 2026-01-25

### Added
- **crates.io installation**: Added `cargo install aiproject` as primary installation method

### Changed
- Reordered CLI installation options (cargo first, then Homebrew, GitHub releases, build from source)

## [1.5.2] - 2026-01-25

### Changed
- **VS Code Extension documentation**: Added multiple CLI installation methods (Homebrew, GitHub releases, build from source)
- Updated version references in documentation

## [1.5.1] - 2026-01-25

### Added
- **VS Code Extension icon**: Custom icon with clipboard and progress indicators

## [1.5.0] - 2026-01-25

### Added
- **VS Code Extension v1.5.0**: Major update to the VS Code extension
  - 9 Language Model Tools for automatic logging via GitHub Copilot
  - Session notification on workspace open with action buttons
  - Status bar quick menu (View Status, Tasks, End Session, Refresh)
  - Auto-generate session summaries using Copilot
  - End Session button in @proj /status output
- **Comprehensive VS Code documentation**: Complete novice-friendly guide
  - Quick Start Guide with step-by-step instructions
  - Detailed feature explanations with examples
  - Understanding Permissions section for Copilot tool approvals
  - CLI vs VS Code comparison
  - Troubleshooting guide

### Fixed
- VS Code extension field name mismatches (current_session vs session)
- Auto-generate summary flow now uses Language Model Tools directly
- Session notification delay (1.5s) for better visibility

## [1.4.0] - 2026-01-25

### Added
- **Automated changelog in `proj release`**: Prompts for changelog entries interactively, opens editor for each entry, auto-updates CHANGELOG.md
- **Changelog validation in release workflow**: Releases fail early if CHANGELOG.md is missing an entry for the version being released
- **Version argument for release**: `proj release 1.4.0` skips version selection prompt

### Changed
- Release command now checks for uncommitted changes before proceeding
- Release workflow validates changelog before building binaries

## [1.3.0] - 2026-01-25

### Added
- **Auto-commit on session end**: Optionally create git commits when ending sessions
  - Enable during `proj init` for git repositories
  - Two modes: "prompt" (ask each time) or "auto" (silent)
  - Commits use format: `[proj] <session summary>`
  - Configure via `auto_commit` and `auto_commit_mode` in config.json

### Changed
- Init wizard now asks about auto-commit for git repositories
- Documentation updated for all v1.3.0 features

## [1.2.0] - 2026-01-25

### Added
- **Automated release pipeline**: Bump version in Cargo.toml, push, and releases happen automatically
- **`proj update` command**: Check for newer versions from GitHub
- **`proj release` command**: Release management wizard for maintainers
- **`proj rollback` command**: Undo releases by deleting tags and GitHub releases
- **Performance case study**: Documented 68% reduction in files read, 50% token savings
- GitHub Actions CI/CD with automatic version detection and release creation
- Failure notifications create GitHub issues with fix instructions
- Homebrew formula auto-updates with correct SHA256 hashes

### Changed
- Release workflow supports both tag push and workflow_dispatch triggers

## [1.1.0] - 2026-01-25

### Added
- **8-hour stale session auto-close**: Sessions automatically close after 8 hours of inactivity
- **AGENTS.md integration**: `proj init` adds session rules to global AGENTS.md
- **Extension system refactor**: Removed book/sermon/course extensions, added schema/releases
- **FTS5 full-text search**: Fixed search functionality in tracking database
- **Registry bug fixes**: Improved project registration reliability

### Changed
- Sessions show "(auto-closed)" when closed due to staleness
- Improved status output with session age indicator

## [1.0.0] - 2026-01-22

### Added
- Initial release
- **Core tracking**: Sessions, decisions, tasks, blockers, questions, notes
- **Commands**: init, status, resume, session, log, task, context, delta
- **Database management**: check, upgrade, export, backup, compress, cleanup
- **Multi-project support**: register, registered, dashboard
- **Extensions**: extend command for specialized tracking tables
- SQLite database with FTS5 full-text search
- Colored terminal output
- Global project registry

---

## Version Format

proj follows [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes to commands or database schema
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, documentation updates
