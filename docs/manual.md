# Command Reference

Complete reference for all proj commands.

## Global Options

These work with any command:

| Option | Description |
|--------|-------------|
| `--no-color` | Disable colored output |
| `--help` | Show help for any command |
| `--version` | Show version |

Example:
```bash
proj --no-color status
proj task --help
```

---

## Initialization

### proj init

Initialize a new project with proj tracking.

```bash
proj init
```

**Interactive** - asks for:
- Project type (rust, python, javascript, web, documentation, other)
- Project name
- Description (optional)
- Auto-commit on session end? (if git repo detected)
- Auto-commit mode: prompt or auto

Creates `.tracking/` folder with `config.json` and `tracking.db`.

**Also:**
- Registers project in global registry
- Adds session rules to global AGENTS.md (for AI assistants)

**Note:** Run this in a terminal, not through an AI assistant.

---

### proj migrate

Convert an existing project to use proj tracking.

```bash
proj migrate
```

For projects that have content but no proj tracking. Interactive.

---

## Status & Context

### proj status

Show current project status and start/resume a session.

```bash
proj status              # Normal output
proj status --quiet      # Minimal output (one line)
proj status --verbose    # More detail
proj status --full       # Everything
```

**Behavior:**
- First run in a session shows full context
- Subsequent runs show minimal context
- Auto-closes stale sessions (8+ hours old)

**Output includes:**
- Current session info
- Active blockers
- Pending tasks
- Recent decisions
- Open questions

---

### proj resume

Detailed context for resuming work.

```bash
proj resume              # Human-readable
proj resume --for-ai     # JSON format for AI consumption
```

Similar to `proj status` but focused on "where did I leave off?"

---

### proj context

Search decisions and notes.

```bash
proj context "database"           # Basic search
proj context "auth" --ranked      # Results sorted by relevance
```

Searches:
- Decision topics and content
- Note titles and content
- Full-text search index

---

### proj snapshot

Generate AI context snapshot in JSON.

```bash
proj snapshot
```

Outputs structured JSON with:
- Project info
- Current session
- Active tasks
- Blockers
- Recent decisions
- Open questions

Useful for programmatic access.

---

### proj delta

Show what changed since last status check.

```bash
proj delta
```

Shows new/changed items since you last checked. Useful for AI assistants to see only what's new.

---

## Sessions

### proj session start

Manually start a new session.

```bash
proj session start
```

Usually not needed - `proj status` handles this automatically.

---

### proj session end

End the current session with a summary.

```bash
proj session end "What we accomplished"
```

**Writing Good Summaries:**

The summary should answer "what was accomplished?" so future sessions (or another AI) can understand where you left off. Aim for 1-3 substantive sentences that capture the actual work done.

**Good summaries (substantive):**
```bash
proj session end "Implemented automated release pipeline with version bump triggering GitHub Actions. Added rollback command. Released v1.1.0 and v1.2.0."
proj session end "Fixed authentication bug in login flow - token refresh was failing silently. Added error logging."
proj session end "Refactored database layer to use connection pooling. Updated tests to use mock connections."
```

**Avoid generic summaries (not helpful):**
```bash
# These tell you nothing useful for resuming work
proj session end "Reviewed project status"
proj session end "Worked on the codebase"
proj session end "Made some changes"
```

**Auto-commit:** If enabled in config, also creates a git commit with the session summary:
```
✓ Session #5 ended. Summary: Added user authentication
Commit changes with session summary? [Y/n] y
  ✓ Committed: [proj] Added user authentication
```

---

### proj session list

List recent sessions.

```bash
proj session list
```

Output:
```
Recent Sessions:
------------------------------------------------------------
#5    2026-01-24 10:30 - Added user authentication
#4    2026-01-23 14:15 - Fixed database migration bug
#3    2026-01-22 09:00 - (auto-closed)
```

---

## Logging

### proj log decision

Log an architectural decision.

```bash
proj log decision <topic> <decision> [rationale]
```

**Examples:**
```bash
proj log decision "database" "Using SQLite" "Simple, portable"
proj log decision "auth" "JWT tokens" "Stateless, industry standard"
proj log decision "framework" "Chose Actix Web"
```

---

### proj log note

Log a context note.

```bash
proj log note <category> <title> <content>
```

**Categories:** goal, constraint, assumption, requirement, note

**Examples:**
```bash
proj log note "constraint" "API limit" "Max 100 requests per minute"
proj log note "assumption" "Users" "Expecting ~1000 daily users"
proj log note "goal" "Performance" "Page load under 2 seconds"
```

---

### proj log blocker

Log something blocking progress.

```bash
proj log blocker <description>
```

**Examples:**
```bash
proj log blocker "Waiting for API credentials"
proj log blocker "Need design approval for new UI"
proj log blocker "CI pipeline broken"
```

---

### proj log question

Log an open question.

```bash
proj log question <question> [context]
```

**Examples:**
```bash
proj log question "Should we support Windows?"
proj log question "What auth method?" "Client hasn't specified"
```

---

## Tasks

### proj task add

Add a new task.

```bash
proj task add <description> [--priority <level>]
```

**Priorities:** urgent, high, normal (default), low

**Examples:**
```bash
proj task add "Implement login endpoint"
proj task add "Fix memory leak" --priority urgent
proj task add "Update documentation" --priority low
```

---

### proj task update

Update an existing task.

```bash
proj task update <id> [--status <status>] [--priority <priority>] [--notes <notes>]
```

**Statuses:** pending, in_progress, completed, cancelled, blocked

**Examples:**
```bash
proj task update 1 --status in_progress
proj task update 1 --status completed
proj task update 2 --notes "Blocked by API issue"
proj task update 3 --priority urgent --status in_progress
```

---

### proj tasks

List all active tasks.

```bash
proj tasks
```

Shortcut for `proj task list`.

Output:
```
Active Tasks:
------------------------------------------------------------
◐ #1    [high] Implement login endpoint
○ #2    [normal] Write unit tests
○ #3    [low] Update README
```

Icons:
- `○` pending
- `◐` in progress
- `✗` blocked

---

## Database Management

### proj extend

Add extension tables for specialized tracking.

```bash
proj extend --type <type>
```

**Types:**
- `api` - API endpoints and models
- `schema` - Database tables, columns, migrations
- `releases` - Version releases and deployment tracking

**Examples:**
```bash
proj extend --type api
proj extend --type schema
proj extend --type releases
```

---

### proj check

Verify database integrity.

```bash
proj check
```

Checks:
- Database file exists and is valid
- Schema version
- Config file validity

---

### proj upgrade

Upgrade database schema to latest version.

```bash
proj upgrade              # Upgrade current project
proj upgrade --info       # Show what would be upgraded
proj upgrade --all        # Upgrade all registered projects
```

---

### proj export

Export session history.

```bash
proj export --format md     # Markdown
proj export --format json   # JSON
```

Exports all sessions, decisions, tasks, etc.

---

### proj backup

Create a manual backup.

```bash
proj backup
```

Copies database to `~/.proj/backups/`.

---

### proj compress

Compress old sessions to save space.

```bash
proj compress             # Interactive
proj compress --auto      # Automatic (sessions older than 7 days)
```

Combines old sessions into compressed summaries.

---

### proj cleanup

Archive stale items.

```bash
proj cleanup              # Default: 30 days
proj cleanup --days 60    # Custom threshold
proj cleanup --auto       # Non-interactive
```

---

## Multi-Project

### proj register

Register current project in global registry.

```bash
proj register
```

Adds project to `~/.proj/registry.json` for cross-project commands.

---

### proj registered

List all registered projects.

```bash
proj registered
```

Output:
```
Registered Projects (3):

  ✓ my-app
      Type: rust
      Path: /Users/me/projects/my-app
      Schema: v1.2

  ✓ website
      Type: web
      Path: /Users/me/projects/website
      Schema: v1.2
```

---

### proj dashboard

Interactive overview of all projects.

```bash
proj dashboard
```

**Note:** Interactive - run in terminal, not through AI assistant.

---

## Project Lifecycle

### proj archive

Archive a completed project.

```bash
proj archive
```

Marks project as archived. Interactive confirmation.

**Note:** Interactive - run in terminal, not through AI assistant.

---

## Updates & Releases

### proj update

Check for updates to proj.

```bash
proj update
```

Compares installed version against latest GitHub release. Shows update instructions if a newer version is available.

---

### proj release

Release management (for maintainers).

```bash
proj release              # Interactive release wizard
proj release --check      # Verify/update Homebrew formula
```

Used for managing proj releases. The `--check` flag updates the Homebrew formula with correct SHA256 hashes after a release.

---

### proj rollback

Undo a release (for maintainers).

```bash
proj rollback             # Rollback latest release
proj rollback 1.2.0       # Rollback specific version
```

Deletes GitHub release and tags (local and remote). Interactive confirmation required.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NO_COLOR` | Disable colored output (set to any value) |
| `PROJ_HOME` | Override global config directory (default: `~/.proj`) |

---

## File Locations

| Path | Description |
|------|-------------|
| `.tracking/config.json` | Project configuration |
| `.tracking/tracking.db` | Project database |
| `~/.proj/registry.json` | Global project registry |
| `~/.proj/backups/` | Backup storage |

---

## Configuration File

The `.tracking/config.json` file contains project settings:

```json
{
  "name": "my-project",
  "project_type": "rust",
  "description": "My awesome project",
  "schema_version": "1.2",
  "auto_backup": true,
  "auto_session": true,
  "auto_commit": false,
  "auto_commit_mode": "prompt"
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | - | Project name |
| `project_type` | string | - | rust, python, javascript, web, documentation, other |
| `description` | string | null | Optional description |
| `schema_version` | string | "1.2" | Database schema version |
| `auto_backup` | bool | true | Auto-backup on session end |
| `auto_session` | bool | true | Auto-start sessions on status |
| `auto_commit` | bool | false | Git commit on session end |
| `auto_commit_mode` | string | "prompt" | "prompt" (ask) or "auto" (silent) |
