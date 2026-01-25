# Changelog

All notable changes to proj are documented here.

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
