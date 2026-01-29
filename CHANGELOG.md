# Changelog

All notable changes to proj are documented here.

## [Unreleased]

## [1.8.2] - 2026-01-29

### Changed
- **AI instructions for mid-conversation project switching**: SESSION_RULE now instructs AI agents to run `proj status` when switching to a different project directory mid-conversation, ensuring proper session tracking across projects.
- **Explicit commit instructions for AI agents**: SESSION_RULE now explicitly instructs AI to commit after completing tasks and before ending sessions. Previously only mentioned proj's auto-commit feature without telling AI to commit manually.

## [1.8.1] - 2026-01-29

### Changed
- **Session auto-start feedback**: When `proj log` or `proj task add` auto-creates a session (or closes a stale one), a message is now displayed so you know a new session started.
- **Proactive stale session warning**: Shell hook now checks for stale sessions on every prompt. If your session expired while the terminal was idle, you'll see a warning before you start typing. Warns once per stale session, then stays quiet until you run `proj status`.
- **`proj shell check` command**: New lightweight command used by the shell prompt hook to detect stale sessions.
- **`proj review` command**: Cleanup pass for missed logging. Shows git commits alongside logged items, helps identify decisions/tasks that weren't logged during the session.
- **Status nudge for review**: `proj status` now shows a nudge when there are commits but no decisions logged, prompting you to run `proj review`.

### Added
- **Project-local AGENTS.md creation**: `proj init` now creates an AGENTS.md file in the project directory with proj tracking instructions, plus CLAUDE.md and GEMINI.md symlinks pointing to it. If CLAUDE.md or GEMINI.md already exist as real files, they are promoted to AGENTS.md and the symlinks are created. This ensures all LLM platforms (Claude, Gemini, Codex) use unified project context.
- **Auto-create AGENTS.md on session start**: `proj status` now checks if AGENTS.md exists and creates it (with symlinks) if missing. This catches existing projects initialized before this feature was added.
- **Self-update with `--apply` flag**: `proj update --apply` downloads and applies updates in one command. When permission is denied (e.g., binary in `/usr/local/bin/`), shows clear instructions with the exact `sudo cp` command needed to complete the update.
- **`--check` flag for `proj update`**: Check for updates without starting background download.
- **VS Code: `/review` command**: Added `/review` slash command to the chat participant (`@proj /review`) for session review directly in Copilot Chat.

## [1.8.0] - 2026-01-28

### Added
- **Git history integration**: Recent git commits are synced into the tracking database and shown in `proj status` output (Tier 2 shows 3 commits, Tier 3 shows 10 with file stats). Commits are also searchable via `proj context`.
- **Structured session summaries**: When a session ends, a JSON-structured summary is built automatically containing decisions, tasks created/completed, blockers, notes, git commits, and files touched. Stored alongside the plain text summary for richer context on resume.
- **Session end review hints**: Before ending a session, proj now displays a review showing logged activity counts and git commit counts, with advisory hints when activity appears under-logged (e.g., "5 commits but no decisions logged").
- **`--recent` flag for `proj context`**: `proj context recent --recent` shows the last 10 items chronologically across decisions, tasks, notes, and git commits for quick "what happened recently?" recall.
- **Auto-commit on task completion**: New `auto_commit_on_task` config option (default: false). When enabled, completing a task via `proj task update <id> --status completed` auto-commits with message `[proj] Completed task #N: <description>`.
- **New `src/git.rs` module**: Git log parsing, commit syncing to SQLite, query helpers for recent commits, commits since session start, and commit message search.
- **New `src/commit.rs` module**: Shared auto-commit logic extracted from session end, now reused by both session end and task completion.
- **Mid-Session Context Recall instructions**: Updated AGENTS.md SESSION_RULE with guidance for LLMs to use `proj context` mid-session before making decisions that might duplicate or contradict previous ones.

### Changed
- **Schema version**: v1.3 -> v1.4
  - New `git_commits` table with indexes on hash and committed_at
  - New `structured_summary TEXT` column on sessions table
- **`proj status`**: Now syncs git commits on every run. Tier 2 (Working) shows recent commits section. Tier 3 (Full) shows GIT HISTORY section with file change stats and structured summary highlights from the last session.
- **`proj resume`**: Shows structured summary details (decisions list, commit count) from the last session when available.
- **`proj context`**: Now searches git commit messages in both basic and ranked modes.
- **`proj session end`**: Builds structured JSON summary before ending. Displays session review with hints about potentially missed logging.
- **`proj task update --status completed`**: Triggers auto-commit when `auto_commit_on_task: true` in config.
- **Session auto-commit**: Refactored into shared `commit.rs` module used by both session end and task completion.
- **AGENTS.md outdated-rules detection**: Now checks for `### Mid-Session Context Recall` marker (previously checked for `### Running proj Commands in LLM CLIs`).

### Configuration
- New field `auto_commit_on_task` (bool, default: false) in `.tracking/config.json`

## [1.7.28] - 2026-01-28

### Added
- **Comprehensive "@proj in Copilot Chat" guide in README**: New section covering three usage methods (slash commands, natural language, automatic tools) ranked by reliability, Copilot mode compatibility table, recommended workflow, quick reference, and troubleshooting
- **Honest expectations note**: Added "What to Expect" callout clarifying that @proj commands always work while automatic logging depends on Copilot mode and language model

### Changed
- **Quick Start Step 5 updated**: Renamed from "Let Copilot log automatically" to "Try automatic logging (bonus)" with honest language about mode/model dependency

## [1.7.27] - 2026-01-28

### Added
- **`@proj /end-auto` command**: Auto-generate session summary and end session directly in Copilot Chat
  - Uses Language Model API to generate summary from session activity
  - Falls back to default summary if no AI model available
- **Auto-detection of decisions/tasks/blockers**: When chatting with `@proj` (without a slash command), the participant analyzes messages using the LM API and automatically logs any decisions, tasks, or blockers it detects
- **Inline confirmations**: Logged items show as `> Logged decision:` or `> Added task:` directly in the chat

### Fixed
- **Restored Copilot Chat integration for all UI actions**: Status bar menu and startup notification buttons now open Copilot Chat instead of native popups
  - "View Status" opens `@proj /status` in Copilot Chat
  - "View Tasks" opens `@proj /tasks` in Copilot Chat
  - "End Session" opens `@proj /end-auto` in Copilot Chat
  - Startup notification buttons also route through Copilot Chat
  - Falls back to native UI if Copilot Chat is unavailable
- **Fixed invisible confirmation dialog**: Auto-summary confirmation changed from `showInputBox` (appears at top of window, easy to miss) to `showInformationMessage` (appears bottom-right, always visible)
- **Fixed chained QuickPick issue**: Added 150ms delay between status bar menu close and next UI element to prevent silent failures

### Changed
- **Removed inline Language Model API code from extension.ts**: Auto-summary delegates to the `@proj /end-auto` chat command instead of calling LM API directly

## [1.7.21] - 2026-01-28

### Added
- **`@proj /end-auto` command**: Auto-generate session summary and end session directly in Copilot Chat
  - Uses Language Model API to generate summary from session activity
  - Falls back to default summary if no AI model available
  - Shows generated summary in chat, no separate input box needed

### Fixed
- **Status bar "View Status" now shows visible feedback**: Output panel shows timestamp header and info notification
- **End Session from dropdown wrapped in error handler**: Uncaught errors now show as error messages instead of silently failing

## [1.7.20] - 2026-01-28

### Fixed
- **Fixed CLI argument passing in VS Code extension**: Switched from `exec`/`execSync` (shell command string) to `execFile`/`execFileSync` (array args)
  - Arguments with spaces (like session summaries) were being split by the shell
  - `@proj /end my summary here` now works correctly
  - Removed unnecessary double-quoting from all CLI wrapper functions
  - This likely also fixes the auto-summary flow which passes args through `runProjSync`

## [1.7.19] - 2026-01-28

### Fixed
- **VS Code auto-summary: visible progress and broader model search**
  - Added progress notification ("Generating session summary...") so user sees activity
  - Removed `{ family: 'gpt-4' }` filter from model selection - now finds ANY available Copilot model
  - Logs all available models with id, name, vendor, family for debugging
  - Uses `vscode.window.withProgress` for visible spinner during generation

## [1.7.18] - 2026-01-28

### Fixed
- **Dedicated debug output channel for VS Code**: Replaced console.log with dedicated `proj-debug` output channel
  - All log messages now written to "proj-debug" output panel in VS Code
  - Logs visible even when Extension Host console is not accessible
  - Timestamped messages for easier debugging of auto-summary flow

## [1.7.17] - 2026-01-28

### Fixed
- **More debug logging for End Session**: Added detailed logging to auto-summary flow
  - Logs each step: CLI call, LM API availability, model selection, request
  - Added 5s timeout for model selection, 15s timeout for LM request
  - Helps diagnose where the flow is failing

## [1.7.16] - 2026-01-28

### Added
- **VS Code extension debugging**: Added console logging to status bar menu commands
  - Logs when commands are triggered and executed
  - Check "Extension Host" output panel for debug info

## [1.7.15] - 2026-01-28

### Fixed
- **Documentation: changed `npx` to `npm install -g`**: `npx create-aiproj` runs temporarily and puts binary in cache where VS Code can't find it. Changed all docs to recommend `npm install -g create-aiproj` for permanent installation.

## [1.7.14] - 2026-01-28

### Fixed
- **npm installer now works on Windows**: Was just printing a message instead of installing
  - Uses PowerShell `Expand-Archive` to extract zip files
- **VS Code extension checks npm install location**: Added `%APPDATA%\npm` to path search on Windows

## [1.7.13] - 2026-01-28

### Fixed
- **VS Code extension now works on Windows**: Path detection was Unix-only
  - Now properly handles Windows paths (`%USERPROFILE%\.cargo\bin\proj.exe`)
  - Checks scoop and common Windows install locations
  - Uses correct path separator (`;` on Windows, `:` on Unix)

## [1.7.12] - 2026-01-28

### Fixed
- **VS Code extension now finds proj in cargo bin**: Extension was hardcoded to `/usr/local/bin/proj`
  - Now checks `~/.cargo/bin/proj` first (cargo install), then Homebrew paths
  - Configurable via `proj.cliPath` setting if needed
  - Fixes "commands do nothing" issue when Homebrew version is outdated

## [1.7.11] - 2026-01-28

### Fixed
- **VS Code auto-summary now actually ends the session**: v1.7.10 fix was incomplete - it generated a summary but didn't call `proj session end`
  - Now uses VS Code Language Model API to generate summary
  - Shows input box with AI-generated summary pre-filled for user review
  - Actually ends the session after user confirms

## [1.7.10] - 2026-01-28

### Fixed
- **VS Code auto-summary now works**: Fixed issue where AI couldn't access tools when generating session summaries
  - Previously opened Copilot Chat expecting it to call Language Model Tools
  - Now gets session activity directly and passes it to Copilot for summarization
  - Falls back to manual input if Copilot unavailable

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
