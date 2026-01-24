# Changelog

All notable changes to proj will be documented in this file.

## [0.9.0] - 2026-01-21

### Added
- Complete rewrite from Python to Rust for improved performance
- All commands from Python version now available in Rust

### Commands Implemented

#### Core Commands
- `proj status` - Show project status with multiple verbosity levels (quiet, verbose, full)
- `proj resume` - Detailed context for resuming work with `--for-ai` JSON output
- `proj session start` - Start a new work session
- `proj session end <summary>` - End session with summary
- `proj session list` - List recent sessions

#### Logging Commands
- `proj log decision <topic> <decision> [rationale]` - Log architectural decisions
- `proj log note <category> <title> <content>` - Log context notes
- `proj log blocker <description>` - Log blockers
- `proj log question <question> [context]` - Log questions

#### Task Management
- `proj task add <description> [--priority]` - Add tasks
- `proj task update <id> [--status] [--notes] [--priority]` - Update tasks
- `proj task list` / `proj tasks` - List tasks

#### Context & Search
- `proj context <topic> [--ranked]` - Search decisions and notes with relevance scoring
- `proj delta` - Show changes since last status check (token efficiency)
- `proj compress [--auto]` - Compress old sessions for token savings
- `proj cleanup [--auto] [--days N]` - Archive stale items

#### Multi-Project Support
- `proj register` - Register project in global registry
- `proj registered` - List all registered projects
- `proj dashboard` - Interactive multi-project overview

#### Schema Management
- `proj upgrade [--info] [--all] [--auto]` - Upgrade database schema

#### Initialization & Utilities
- `proj init` - Interactive project initialization
- `proj migrate` - Convert existing project to proj format
- `proj extend --type <type>` - Add extension tables (api, schema, releases)
- `proj export [--format md|json]` - Export session history
- `proj backup` - Manual database backup
- `proj check` - Verify database integrity
- `proj archive` - Archive completed project
- `proj snapshot` - Generate AI context snapshot (JSON)

### Changed
- Performance improvements from Rust implementation
- TTY detection for automatic color disable when piped
- Global `--no-color` flag support
- Respects NO_COLOR environment variable

## [0.0.8] - 2026-01-21
- Schema upgrade system with migration registry
- Compatibility checking and dry-run mode

## [0.0.7] - 2026-01-21
- Multi-project support (register, registered, dashboard)

## [0.0.6] - 2026-01-21
- Efficiency features (delta, compress, cleanup)

## [0.0.5] - 2026-01-21
- Context and search commands (resume, context with ranked scoring)

## [0.0.4] - 2026-01-21
- Task management commands (add, update, list)

## [0.0.3] - 2026-01-21
- Logging commands (decision, note, blocker, question)

## [0.0.2] - 2026-01-21
- Core commands (status, session management)

## [0.0.1] - 2026-01-21
- Initial Rust foundation (CLI, database, models, schemas)
