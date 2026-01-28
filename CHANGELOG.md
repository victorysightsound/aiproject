# Changelog

All notable changes to proj are documented here.

## [1.7.9] - 2026-01-28

### Changed
- **LLM CLI init wizard now includes documentation setup**: The AGENTS.md instructions for `proj init` now tell LLMs to ask about documentation database setup (skip/generate/import/new + doc type)
  - Matches what the interactive wizard already asks
  - Enables complete project setup through LLM CLI tools

## [1.7.8] - 2026-01-28

### Added
- **Non-interactive support for uninstall**: `proj uninstall --project --force` and `proj uninstall --all --force`
  - Skip confirmation prompts for LLM CLI tools (Claude Code, Codex, etc.)
  - Also available as `-y` short flag
- **Non-interactive support for shell install**: `proj shell install --force`
  - Installs for all available shells without prompting
  - Also available as `-y` short flag
- **LLM CLI interviewer pattern in AGENTS.md**: New section instructs LLMs to ask users wizard questions in chat, then use command-line flags
  - Covers: `proj init`, `proj uninstall`, `proj shell install`, `proj session end`, `proj upgrade`, `proj docs init`
  - Auto-propagates to existing projects via `proj upgrade`

### Changed
- Removed duplicate shell hook code from init.rs, now delegates to shell.rs

## [1.7.7] - 2026-01-28

### Added
- **Shell hook prompt in init wizard**: Interactive mode now asks if you want to enable automatic session tracking
- **`--shell-hook` flag**: Non-interactive init can install shell hook with this flag
- Shell hook installation is global (once installed, subsequent `proj init` calls won't ask again)
- **AGENTS.md auto-update during upgrade**: `proj upgrade` now updates outdated AI logging rules
  - Projects initialized before v1.7.4 had basic session rules without trigger-based logging
  - Upgrade detects outdated rules (missing "### Logging During Sessions" section)
  - Automatically updates global AGENTS.md with current instructions
  - Works even when database schema is already current

## [1.7.5] - 2026-01-27

### Added
- **Empty session handling**: `proj session end` now checks for logged activity
  - If no decisions, tasks, blockers, notes, or questions were logged, shows options
  - Options: add manually, AI review, or force-end with `--force` flag
  - Prevents sessions from ending without capturing important context
- **`--force` flag for session end**: `proj session end --force "summary"` skips empty check
- **AI review guidance**: AGENTS.md rules now include instructions for AI to review and log missed items when session is empty

### Improved
- Documentation updated with empty session handling details

## [1.7.4] - 2026-01-27

### Added
- **Session activity review**: `proj session end` now displays all logged activity before ending
  - Shows decisions, tasks, blockers, notes, and questions from the session
  - Helps verify nothing was missed before finalizing
  - Displays tip if no activity was logged
- **Enhanced AGENTS.md rules**: More explicit, trigger-based instructions for AI assistants
  - Specifies when to log decisions (after technical choices)
  - Specifies when to add tasks (todo, need to, should)
  - Specifies when to log blockers (blocked, waiting on, can't because)
- **Project directory prompt in init wizard**: Interactive mode now asks for project directory
  - Current directory shown as default (just press Enter)
  - Creates directory if it doesn't exist
  - Shows cd command if different directory was chosen

### Improved
- Documentation updated to clearly explain how AI logging works
- Added "How AI Logging Works" section to concepts.md

## [1.7.3] - 2026-01-27

### Added
- **`--path` flag for init**: Initialize a project in a different directory
  - `proj init --path ~/projects/new-project` creates directory if needed
  - Useful when not already in the target project folder

## [1.7.2] - 2026-01-27

### Added
- **Non-interactive init for LLM CLIs**: `proj init` now supports command-line flags
  - Enables initialization through Claude Code, Codex, and other LLM CLI tools
  - `proj init --name "project" --type rust` runs without prompts
  - Full control: `--description`, `--docs-generate`, `--auto-commit`, etc.
  - Auto-detects when not in a terminal and uses non-interactive mode

## [1.7.1] - 2026-01-27

### Added
- **Auto-update feature**: proj now automatically updates itself in the background
  - When an update is available, downloads new binary to staging area
  - On next run, atomically replaces binary and re-executes with same args
  - Brief notification: "Updated proj 1.7.0 → 1.7.1"
  - Supported platforms: macOS (Intel & Apple Silicon), Linux (x64 & ARM64)
  - Silent fallback if update fails (manual update still works)

## [1.7.0] - 2026-01-27

### Added
- **Autonomous shell integration**: `proj shell install` adds a shell hook that runs `proj enter` when you cd into any tracked project
  - `proj shell install` - Add hook to ~/.zshrc and/or ~/.bashrc
  - `proj shell uninstall` - Remove the hook
  - `proj shell status` - Check if hook is installed
- **Silent session start**: `proj enter` for shell hooks - silent if session exists, shows context only when starting new session
- **Uninstall command**: `proj uninstall` with options to cleanly remove proj
  - `--shell` - Remove shell hook only
  - `--project` - Remove .tracking from current project
  - `--all` - Remove everything (shell hook, current project tracking, global config)
- **Schema backup before upgrades**: Automatic backup to ~/.proj/backups/ before schema migrations
  - `proj rollback --schema` - Interactive restore from backup
  - `proj rollback --list` - List available schema backups
  - Only keeps 1 backup per project (no accumulation)
- **Schema upgrade prompts**: Status command now shows when a schema upgrade is available
- **Automated npm publishing**: Release workflow publishes to npm when NPM_PUBLISH_ENABLED=true
- **Automated VS Code Marketplace publishing**: Release workflow publishes extension when VSCODE_PUBLISH_ENABLED=true
- **Version sync across packages**: `proj release` now syncs versions in Cargo.toml, npm package, and VS Code extension

## [1.6.3] - 2026-01-25

### Changed
- **Comprehensive documentation update**: All documentation files updated for v1.6.x features and novice-friendliness
  - README: Added docs database section, VS Code extension reference, updated installation order
  - Getting Started: All installation methods, docs setup wizard, FTS5 troubleshooting
  - Concepts: Session summary guidance, documentation database explanation
  - Manual: Complete `proj docs` command reference, updated schema version
  - Cheat Sheet: Documentation database commands, updated workflows

## [1.6.2] - 2026-01-25

### Added
- **Documentation setup in init wizard**: `proj init` now includes documentation database setup with options to generate from source, import markdown, create skeleton, or skip

## [1.6.1] - 2026-01-25

### Fixed
- **FTS5 not created during init**: Full-text search table was defined but not created during project initialization, causing `proj context` searches to fail
- Added FTS5 creation to `proj migrate` command
- Added upgrade path (schema 1.2 → 1.3) for existing projects to add FTS5

## [1.6.0] - 2026-01-25

### Added
- **Documentation Database Feature**: New `proj docs` command for managing project documentation
  - `proj docs init` - Interactive wizard with multiple setup modes
  - `proj docs init --generate` - Auto-generate docs from source code analysis
  - `proj docs init --import` - Import existing markdown files
  - `proj docs init --new` - Create documentation skeleton from questions
  - `proj docs status` - Show database info with staleness detection
  - `proj docs refresh` - Update generated sections when source changes
  - `proj docs show` - Display table of contents or specific section
  - `proj docs search` - FTS5 full-text search across documentation
  - `proj docs export` - Export to markdown format
  - `proj docs term add/list/search` - Terminology glossary management

- **Multi-language Source Analysis**: Automatic documentation generation from code
  - Rust: modules, structs, enums, traits, functions, impl blocks
  - Python: classes, functions, async functions, docstrings
  - TypeScript: interfaces, classes, enums, types, functions, JSDoc
  - Go: structs, interfaces, functions, methods, constants

- **Change Detection**: Track source file modifications
  - Staleness warnings in `proj docs status` when source files change
  - `proj docs refresh` regenerates only generated sections
  - `--force` flag to regenerate all sections including manual edits

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
