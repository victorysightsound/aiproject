# proj

Project tracking and context management for AI-assisted development.

## Overview

`proj` is a command-line tool that tracks project context, decisions, tasks, and sessions in a local SQLite database. It is designed to provide AI assistants with efficient access to project history without requiring expensive context window reloads.

## Installation

### From Source

```bash
git clone https://github.com/victorysightsound/aiproject
cd aiproject
cargo build --release
# Binary at target/release/proj
```

### Homebrew (coming soon)

```bash
brew install victorysightsound/tap/aiproject
```

### npm (coming soon)

```bash
npm install -g aiproject
```

## Quick Start

```bash
# Initialize a new project
proj init

# Check project status (auto-starts session)
proj status

# Log a decision
proj log decision "database" "Using SQLite for local storage" "Simple, portable, no server required"

# Add a task
proj task add "Implement user authentication" --priority high

# End session with summary
proj session end "Set up project structure and made initial architecture decisions"
```

## Commands

### Core Commands

| Command | Description |
|---------|-------------|
| `proj status` | Show current project status |
| `proj resume` | Detailed context for resuming work |
| `proj session start` | Start a new work session |
| `proj session end <summary>` | End session with summary |
| `proj session list` | List recent sessions |

### Logging

| Command | Description |
|---------|-------------|
| `proj log decision <topic> <decision> [rationale]` | Log an architectural decision |
| `proj log note <category> <title> <content>` | Log a context note |
| `proj log blocker <description>` | Log a blocker |
| `proj log question <question> [context]` | Log a question |

### Task Management

| Command | Description |
|---------|-------------|
| `proj task add <description>` | Add a new task |
| `proj task update <id> --status <status>` | Update task status |
| `proj tasks` | List all tasks |

### Context & Search

| Command | Description |
|---------|-------------|
| `proj context <topic>` | Search decisions and notes |
| `proj delta` | Show changes since last check |
| `proj compress` | Compress old sessions |
| `proj cleanup` | Archive stale items |

### Multi-Project

| Command | Description |
|---------|-------------|
| `proj register` | Register in global registry |
| `proj registered` | List all registered projects |
| `proj dashboard` | Interactive project overview |

### Utilities

| Command | Description |
|---------|-------------|
| `proj init` | Initialize new project |
| `proj migrate` | Convert existing project |
| `proj export --format md|json` | Export history |
| `proj backup` | Manual database backup |
| `proj check` | Verify database integrity |
| `proj snapshot` | Generate AI context JSON |
| `proj upgrade` | Upgrade database schema |
| `proj archive` | Archive completed project |

## For AI Assistants

### Token-Efficient Context Loading

```bash
# Get JSON snapshot for AI context
proj resume --for-ai

# Check if anything changed since last load
proj delta

# Compress old sessions to reduce tokens
proj compress --auto
```

### Direct Database Access

For maximum efficiency, AI assistants can query the SQLite database directly:

```sql
-- Get recent decisions
SELECT topic, decision, rationale
FROM decisions
WHERE status = 'active'
ORDER BY created_at DESC LIMIT 10;

-- Search all content
SELECT * FROM tracking_fts
WHERE tracking_fts MATCH 'authentication';

-- Get active tasks
SELECT description, status, priority
FROM tasks
WHERE status NOT IN ('completed', 'cancelled');
```

Database location: `.tracking/tracking.db`

## Project Structure

```
.tracking/
  config.json     # Project configuration
  tracking.db     # SQLite database
  backups/        # Automatic backups
```

## Extension Tables

Add domain-specific tables for specialized tracking:

```bash
proj extend --type api       # API endpoints and models
proj extend --type schema    # Database tables, columns, migrations
proj extend --type releases  # Version releases and deployment tracking
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NO_COLOR` | Disable colored output |
| `PROJ_HOME` | Override global config directory |

## Global Options

```bash
proj --no-color status  # Disable colors for this command
proj --version          # Show version
proj --help             # Show help
```

## License

MIT
