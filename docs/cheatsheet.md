# proj Cheat Sheet

Quick reference for daily use.

## The Basics

```bash
proj status                    # Start of day - see where you left off
proj session end "summary"     # End of day - summarize what you did
```

That's 90% of what you need.

---

## Session Commands

| Command | What It Does |
|---------|--------------|
| `proj status` | Show status, start/resume session |
| `proj resume` | Detailed "where I left off" |
| `proj session end "msg"` | End session with summary |
| `proj session list` | Show recent sessions |

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
| `proj context "topic"` | Search decisions/notes |
| `proj delta` | What changed since last check |
| `proj snapshot` | JSON dump for AI |

---

## Database Direct Access

Database: `.tracking/tracking.db`

```sql
-- Recent decisions
SELECT topic, decision FROM decisions WHERE status='active';

-- Open tasks
SELECT task_id, description FROM tasks WHERE status='pending';

-- Search everything
SELECT * FROM tracking_fts WHERE tracking_fts MATCH 'keyword';
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
| `proj upgrade` | Upgrade schema |
| `proj update` | Check for proj updates |

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
  "auto_commit_mode": "prompt"  // or "auto"
}
```

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

Init also:
- Adds session rules to global AGENTS.md
- Optionally enables auto-commit for git repos

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
proj context "authentication"         # Search
proj export --format md               # Full dump
```

---

## Files

| Location | What It Is |
|----------|------------|
| `.tracking/config.json` | Project settings |
| `.tracking/tracking.db` | All your data |
| `~/.proj/registry.json` | Global project list |
