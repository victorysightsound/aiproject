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

**Interactive mode** (default in terminal) - wizard asks for:
1. **Project info** - type (rust, python, javascript, web, documentation, other), name, description
2. **Documentation database** - choose how to set up project docs:
   - **Skip** - Set up documentation later with `proj docs init`
   - **Generate** - Analyze source code (Rust, Python, TypeScript, Go) to create docs
   - **Import** - Import existing markdown files into the docs database
   - **New Project** - Answer questions to create documentation skeleton
3. **Auto-commit** (git repos only) - Optionally commit changes when sessions end
4. **AGENTS.md rules** - Adds session rules so AI assistants automatically use proj

Creates `.tracking/` folder with `config.json` and `tracking.db`.

**Non-interactive mode** (for LLM CLIs like Claude Code, Codex):

```bash
# Minimal - uses defaults and auto-detection
proj init --name "my-project" --type rust

# Full options
proj init --name "my-project" --type python --description "My app" \
  --docs-generate --docs-type architecture --auto-commit --commit-mode prompt
```

| Flag | Description |
|------|-------------|
| `--path <dir>` | Directory to initialize (creates if doesn't exist) |
| `--name <name>` | Project name (defaults to directory name) |
| `--type <type>` | rust, python, javascript, web, documentation, other |
| `--description <desc>` | Optional project description |
| `--skip-docs` | Skip documentation setup |
| `--docs-generate` | Generate docs from source analysis |
| `--docs-import` | Import docs from markdown files |
| `--docs-new` | Create skeleton documentation |
| `--docs-type <type>` | architecture, framework, guide, api, spec |
| `--auto-commit` | Enable git auto-commit on session end |
| `--commit-mode <mode>` | prompt (ask each time) or auto (silent) |
| `--no-agents` | Skip AGENTS.md setup |

When `--name` and `--type` are provided, init runs non-interactively. This allows LLM CLIs to gather the information through their own interface and then run `proj init` with the appropriate flags.

---

### proj migrate

Update an existing project's database schema and fix issues.

```bash
proj migrate
```

Use this if:
- You upgraded proj and need the latest schema
- Full-text search isn't working (adds FTS5 tables if missing)
- You get schema version errors

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
proj session end "Summary" --force   # Skip empty session check
```

| Flag | Description |
|------|-------------|
| `--force` | End session even if no activity was logged |

**Session Activity Review:**

Before ending, proj displays all decisions, tasks, blockers, and notes logged during the session:

```
Session Activity:
──────────────────────────────────────────────────

◆ Decisions (2)
  • database: Using SQLite
  • auth: JWT tokens for authentication

◆ Tasks Added (1)
  ○ Write unit tests [!]

◆ Blockers (1)
  ✗ Waiting for API credentials

──────────────────────────────────────────────────
```

**Empty Session Handling:**

If no activity was logged during the session, proj shows options instead of ending immediately:

```
⚠ No activity was logged this session.

Before ending, you can capture what happened:

  1. Add manually - Run proj log/task commands yourself
  2. AI review - AI analyzes conversation and logs items
  3. End anyway - Run: proj session end --force "summary"

Session not ended. Choose an option above.
```

To force-end an empty session: `proj session end --force "summary"`

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

## Documentation Database

These commands manage project documentation with full-text search.

### proj docs init

Initialize the documentation database.

```bash
proj docs init                    # Interactive wizard
proj docs init --generate         # Non-interactive: analyze source code
proj docs init --import <path>    # Non-interactive: import markdown files
proj docs init --new              # Non-interactive: create skeleton
```

**Four setup modes:**
1. **Generate** - Analyze source code (Rust, Python, TypeScript, Go) and create documentation
2. **Import** - Import existing markdown files into the database
3. **New Project** - Answer questions to create documentation skeleton
4. **Manual** - Start with empty database

Creates `<project-name>_docs.db` in the project root.

---

### proj docs status

Show documentation database status.

```bash
proj docs status
```

Output:
```
Documentation Status
  Database: my-project_docs.db
  Created: 2026-01-24
  Sections: 12
  Terms: 5

⚠ Source has changed since last generation
  Modified: src/auth.rs, src/database.rs

Run 'proj docs refresh' to update generated sections
```

Shows staleness warnings if source files have changed since documentation was generated.

---

### proj docs show

Display documentation contents.

```bash
proj docs show              # Table of contents
proj docs show <section>    # Specific section
```

---

### proj docs search

Full-text search across documentation.

```bash
proj docs search "authentication"
proj docs search "database schema"
```

---

### proj docs refresh

Update documentation when source files change.

```bash
proj docs refresh           # Update generated sections only
proj docs refresh --force   # Regenerate everything including manual edits
```

Only affects sections that were auto-generated from source code. Manual sections are preserved unless `--force` is used.

---

### proj docs export

Export documentation to markdown.

```bash
proj docs export                    # Export to stdout
proj docs export --output docs.md   # Export to file
```

---

### proj docs term

Manage terminology glossary.

```bash
# Add a term
proj docs term add "API" --definition "Application Programming Interface"
proj docs term add "JWT" --definition "JSON Web Token" --category security

# List all terms
proj docs term list

# Search terms
proj docs term search "token"
```

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
      Schema: v1.3

  ✓ website
      Type: web
      Path: /Users/me/projects/website
      Schema: v1.3
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

**Auto-update:** proj automatically updates itself in the background. When an update is detected:
1. Downloads the new binary to a staging area (`~/.proj/pending_update/`)
2. On your next command, atomically replaces the binary
3. Re-executes your command with the new version
4. Shows a brief notification: "Updated proj 1.7.0 → 1.7.1"

This happens seamlessly - you don't need to do anything. If auto-update fails for any reason (permissions, network, etc.), manual update instructions are shown instead.

**Supported platforms:** macOS (Intel & Apple Silicon), Linux (x64 & ARM64). Windows users should update manually.

---

### proj release

Release management (for maintainers).

```bash
proj release              # Interactive release wizard
proj release 1.5.0        # Skip version selection
proj release --check      # Verify/update Homebrew formula
```

Used for managing proj releases. The `--check` flag updates the Homebrew formula with correct SHA256 hashes after a release.

---

### proj rollback

Undo a release or restore from schema backup.

```bash
proj rollback             # Rollback latest release
proj rollback 1.2.0       # Rollback specific version
proj rollback --schema    # Restore schema from backup
proj rollback --list      # List available schema backups
```

**Release rollback:** Deletes GitHub release and tags (local and remote). Interactive confirmation required.

**Schema rollback:** Restores .tracking/ from a backup created before a schema upgrade. Backups are stored in `~/.proj/backups/` and only 1 backup is kept per project.

---

## Shell Integration

### proj shell install

Install shell hook for automatic session tracking.

```bash
proj shell install
```

Adds a hook to your shell (zsh and/or bash) that runs `proj enter` when you cd into any directory with a `.tracking/` folder. This makes session tracking completely automatic.

**What it does:**
- For zsh: Adds to `~/.zshrc`
- For bash: Adds to `~/.bashrc`
- Uses `chpwd` hook (zsh) or `PROMPT_COMMAND` (bash)

---

### proj shell uninstall

Remove the shell hook.

```bash
proj shell uninstall
```

---

### proj shell status

Check if shell hook is installed.

```bash
proj shell status
```

---

### proj enter

Silent session start (used by shell hook).

```bash
proj enter
```

**Behavior:**
- If there's an active, non-stale session: exits silently (no output)
- If there's no session or session is stale (8+ hours): shows full context like `proj status`

This command is designed for shell hooks - it keeps your terminal clean when you're just changing directories, but shows context when you actually need it.

---

### proj uninstall

Cleanly remove proj from your system.

```bash
proj uninstall            # Interactive - asks what to remove
proj uninstall --shell    # Remove shell hook only
proj uninstall --project  # Remove .tracking from current project
proj uninstall --all      # Remove everything
```

**Options:**

| Flag | What it removes |
|------|-----------------|
| `--shell` | Shell hook from ~/.zshrc and ~/.bashrc |
| `--project` | .tracking/ folder from current project |
| `--all` | Shell hook + current project + global config (~/.proj) |

Interactive confirmation required for destructive operations.

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
| `.tracking/tracking.db` | Session/decision tracking database |
| `<project>_docs.db` | Documentation database (optional) |
| `~/.proj/registry.json` | Global project registry |
| `~/.proj/backups/` | Schema backups (1 per project, created before upgrades) |
| `~/.proj/pending_update/` | Staged update binary (auto-cleaned after update) |
| `~/.proj/version_cache.json` | Cached version check (refreshes every 24h) |

---

## Configuration File

The `.tracking/config.json` file contains project settings:

```json
{
  "name": "my-project",
  "project_type": "rust",
  "description": "My awesome project",
  "schema_version": "1.3",
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
| `schema_version` | string | "1.3" | Database schema version |
| `auto_backup` | bool | true | Auto-backup on session end |
| `auto_session` | bool | true | Auto-start sessions on status |
| `auto_commit` | bool | false | Git commit on session end |
| `auto_commit_mode` | string | "prompt" | "prompt" (ask) or "auto" (silent) |
