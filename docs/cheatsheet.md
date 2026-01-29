# proj Cheat Sheet

Quick reference for daily use.

## The Basics

```bash
proj status                    # Start of day - see where you left off
proj session end "summary"     # End of day - summarize what you did
```

That's 90% of what you need.

**Make it automatic:** Run `proj shell install` once, and sessions start automatically when you cd into any tracked project.

---

## Shell Integration (Optional)

| Command | What It Does |
|---------|--------------|
| `proj shell install` | Add hook - auto-start sessions |
| `proj shell uninstall` | Remove the hook |
| `proj shell status` | Check if installed |

After install, just cd into your project and sessions start silently.

---

## Session Commands

| Command | What It Does |
|---------|--------------|
| `proj status` | Show status, start/resume session |
| `proj resume` | Detailed "where I left off" |
| `proj session end "msg"` | End session with summary |
| `proj session list` | Show recent sessions |

**Good summaries:** "Implemented JWT auth. Fixed token refresh bug." (specific)
**Bad summaries:** "Worked on code." (useless for resuming)

---

## Logging (AI Assistant Uses These)

| Command | Example |
|---------|---------|
| `proj log decision` | `proj log decision "db" "SQLite" "simple"` |
| `proj log blocker` | `proj log blocker "Need API keys"` |
| `proj log question` | `proj log question "Support Windows?"` |
| `proj log note` | `proj log note "note" "Setup" "Uses Rust 1.70"` |

**Note categories:** goal, constraint, assumption, requirement, note

---

## Tasks

| Command | Example |
|---------|---------|
| `proj task add` | `proj task add "Fix bug" --priority high` |
| `proj task update` | `proj task update 1 --status completed` |
| `proj tasks` | List all active tasks |

**Priorities:** urgent, high, normal, low

**Statuses:** pending, in_progress, completed, cancelled, blocked

---

## Search & Query

| Command | What It Does |
|---------|--------------|
| `proj context "topic"` | Search decisions, notes, and git commits |
| `proj context "topic" --ranked` | Relevance-scored search results |
| `proj context recent --recent` | Last 10 items across all tables |
| `proj delta` | What changed since last check |
| `proj snapshot` | JSON dump for AI |

---

## Documentation Database

| Command | What It Does |
|---------|--------------|
| `proj docs init` | Set up project documentation |
| `proj docs status` | Show docs info and staleness |
| `proj docs show` | Table of contents |
| `proj docs search "topic"` | Search documentation |
| `proj docs refresh` | Update generated docs |
| `proj docs export` | Export to markdown |
| `proj docs term add "X"` | Add terminology |

**Init modes:**
- `--generate` - Analyze source code (Rust, Python, TypeScript, Go)
- `--import path/` - Import markdown files
- `--new` - Create documentation skeleton

---

## Database Direct Access

**Tracking database:** `.tracking/tracking.db`

```sql
-- Recent decisions
SELECT topic, decision FROM decisions WHERE status='active';

-- Open tasks
SELECT task_id, description FROM tasks WHERE status='pending';

-- Recent git commits
SELECT short_hash, message, files_changed FROM git_commits ORDER BY committed_at DESC LIMIT 10;

-- Search everything (decisions, notes, tasks, commit messages)
SELECT * FROM tracking_fts WHERE tracking_fts MATCH 'keyword';
```

**Documentation database:** `<project>_docs.db`

```sql
-- Table of contents
SELECT section_id, title FROM sections ORDER BY sort_order;

-- Search docs
SELECT * FROM sections_fts WHERE sections_fts MATCH 'keyword';

-- Get terminology
SELECT canonical, definition FROM terminology;
```

---

## Multi-Project

| Command | What It Does |
|---------|--------------|
| `proj registered` | List all tracked projects |
| `proj register` | Add current project to registry |

---

## Utilities

| Command | What It Does |
|---------|--------------|
| `proj check` | Verify database integrity |
| `proj backup` | Manual backup |
| `proj export --format md` | Export as markdown |
| `proj upgrade` | Upgrade schema (auto-backs up first) |
| `proj migrate` | Fix schema issues (FTS5, etc.) |
| `proj update` | Check for proj updates (auto-updates enabled) |

---

## Schema Backup & Restore

| Command | What It Does |
|---------|--------------|
| `proj rollback --list` | List available backups |
| `proj rollback --schema` | Restore from backup |

Backups are created automatically before schema upgrades.

---

## Uninstall

| Command | What It Does |
|---------|--------------|
| `proj uninstall --shell` | Remove shell hook |
| `proj uninstall --project` | Remove tracking from current project |
| `proj uninstall --all` | Remove everything |

---

## Auto-Commit (Optional)

If enabled during `proj init`:

```bash
proj session end "summary"   # Also creates git commit
```

Config in `.tracking/config.json`:
```json
{
  "auto_commit": true,
  "auto_commit_mode": "prompt",
  "auto_commit_on_task": false
}
```

| Field | What It Does |
|-------|--------------|
| `auto_commit` | Commit on session end |
| `auto_commit_mode` | "prompt" (ask) or "auto" (silent) |
| `auto_commit_on_task` | Commit when a task is marked completed |

---

## Forgot to End Session?

No problem. After 8 hours, proj auto-closes stale sessions:

```
⚠ Previous session #3 was stale (8+ hours). Auto-closed.
✓ Started new session #4
```

---

## Common Workflows

### Starting a New Project
```bash
cd my-project
proj init          # Interactive setup (run in terminal)
proj status        # Verify it worked
```

Init wizard walks through:
- Project info (name, type, description)
- Documentation setup (generate/import/new/skip)
- Auto-commit for git repos
- AGENTS.md rules for AI assistants

### Daily Work
```bash
proj status                           # Morning
# ... work with AI assistant ...
proj session end "Implemented X, fixed Y, updated Z"  # End of day (be specific!)
```

### Quick Task Management
```bash
proj task add "Fix login bug" --priority high
proj tasks                            # See the list
proj task update 1 --status completed # Done
```

### Finding Old Decisions
```bash
proj context "authentication"         # Search tracking data
proj docs search "authentication"     # Search documentation
proj export --format md               # Full dump
```

### Setting Up Documentation Later
```bash
proj docs init                        # Interactive wizard
# Or non-interactive:
proj docs init --generate             # Analyze source code
proj docs init --import docs/         # Import markdown files
```

---

## Files

| Location | What It Is |
|----------|------------|
| `.tracking/config.json` | Project settings |
| `.tracking/tracking.db` | Sessions, decisions, tasks, git commits |
| `<project>_docs.db` | Documentation (optional) |
| `~/.proj/registry.json` | Global project list |
| `~/.proj/backups/` | Schema backups (1 per project) |
