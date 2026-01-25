# Core Concepts

This guide explains what proj tracks and why it matters.

## The Problem proj Solves

When you work with AI assistants on code, conversations look like this:

> "Let's use SQLite for the database"
> "Okay, and we decided JWT for auth, right?"
> "Wait, what was the blocker from yesterday?"
> "Didn't we already add that task?"

Sound familiar? AI sessions create a lot of information, but when the session ends, it's gone. Your brain is supposed to remember everything, but brains aren't great at that.

**proj is your external memory.** It captures what happened so you don't have to remember.

## Sessions

A **session** is a period of work on your project.

Think of it like clocking in and out:
- `proj status` = clock in (or see your current shift)
- `proj session end "summary"` = clock out

### Why Sessions Matter

Sessions give structure to your work history. Instead of one endless timeline, you have clear chunks:

```
Session #5: "Added user authentication"
  - 3 decisions made
  - 2 tasks completed
  - 1 blocker resolved

Session #4: "Fixed database migration bug"
  - 1 decision made
  - 1 task completed
```

When you come back later, you can see "what did I do last time?" at a glance.

### Session Summaries

When ending a session, write summaries that answer "what was accomplished?" - 1-3 substantive sentences work best:

```bash
# Good: Specific and actionable
proj session end "Implemented JWT authentication. Added login/logout endpoints. Fixed token refresh bug that was causing 401 errors."

# Bad: Generic and unhelpful
proj session end "Worked on auth"
```

Good summaries help future you (or another AI assistant) pick up exactly where you left off.

### Auto-Close

If you forget to end your session (power outage, got distracted, life happened), proj handles it. After 8 hours of inactivity, it automatically closes the session when you next run `proj status`:

```
⚠ Previous session #3 was stale (8+ hours). Auto-closed.
✓ Started new session #4
```

No data is lost. The old session just gets marked "(auto-closed)" instead of having a human-written summary.

## Decisions

A **decision** is a choice you made about your project.

```bash
proj log decision "database" "Using SQLite" "Simple, portable, no server needed"
```

This creates a record:
- **Topic**: "database" (what area this decision relates to)
- **Decision**: "Using SQLite" (what you decided)
- **Rationale**: "Simple, portable, no server needed" (why)

### Why Decisions Matter

Three weeks from now:
> "Why are we using SQLite instead of PostgreSQL?"

Without proj: You try to remember. Maybe you search through old chat logs.

With proj:
```bash
proj context "database"
```
```
Decisions
  #1 database (2026-01-24)
     Decision: Using SQLite
     Rationale: Simple, portable, no server needed
```

Decisions also help when you change your mind. You can supersede old decisions and keep a history of how your thinking evolved.

## Tasks

A **task** is something that needs to be done.

```bash
proj task add "Implement login endpoint" --priority high
proj task add "Write unit tests" --priority normal
```

Tasks have:
- **Description**: What needs to be done
- **Priority**: urgent, high, normal, low
- **Status**: pending, in_progress, completed, cancelled

### Task Workflow

```bash
# See all tasks
proj tasks

# Start working on one
proj task update 1 --status in_progress

# Finish it
proj task update 1 --status completed
```

### Why Tasks Matter

Tasks are your to-do list that persists across sessions. When you run `proj status`, you see what's pending:

```
Priority Tasks:
  ◐ [1] Implement login endpoint [!]
  ○ [2] Write unit tests
```

You (and your AI assistant) always know what's next.

## Blockers

A **blocker** is something preventing progress.

```bash
proj log blocker "Waiting for API credentials from client"
```

Blockers show up prominently in `proj status`:

```
Blockers (1):
  ✗ Waiting for API credentials from client
```

### Why Blockers Matter

Blockers are different from tasks. A task is "something to do." A blocker is "something in the way."

When you're blocked, you might context-switch to another task. But without tracking it, you forget to follow up. With proj:

```bash
proj resume
```
```
BLOCKERS (resolve these first!)
  ✗ Waiting for API credentials from client

Suggested Next Action
  Resolve blocker: Waiting for API credentials from client
```

proj reminds you what's stuck.

## Questions

A **question** is something you need to answer but haven't yet.

```bash
proj log question "Should we support Windows?" "Client hasn't confirmed requirements"
```

### Why Questions Matter

Questions are decision-debt. You'll need to answer them eventually, but not right now. Rather than forget, you track them:

```
Open Questions (2):
  ? Should we support Windows?
  ? What authentication method does the client prefer?
```

## Context Notes

A **context note** is anything else worth remembering.

```bash
proj log note "constraint" "API rate limit" "Max 100 requests per minute"
proj log note "assumption" "User count" "Expecting ~1000 daily users"
```

Categories:
- **goal** - What you're trying to achieve
- **constraint** - Limitations you're working within
- **assumption** - Things you're assuming are true
- **requirement** - Must-have features
- **note** - General information

### Why Notes Matter

Notes capture context that doesn't fit elsewhere. "Why did we build it this way?" Often the answer is in the constraints and assumptions you documented.

## How It All Connects

Everything ties to sessions:

```
Session #5
├── Decision: Use SQLite
├── Task: Implement database layer
├── Blocker: Need schema design approval
├── Question: Should we encrypt at rest?
└── Note: Client prefers PostgreSQL but we convinced them
```

When you end the session, you summarize what happened. When you start the next one, you see the full picture.

## The Tracking Database

All of this lives in a SQLite database at `.tracking/tracking.db`. You can query it directly:

```sql
-- Recent decisions
SELECT topic, decision FROM decisions
WHERE status = 'active'
ORDER BY created_at DESC LIMIT 5;

-- Open tasks
SELECT task_id, description, priority FROM tasks
WHERE status IN ('pending', 'in_progress');

-- Full-text search across all tracking data
SELECT * FROM tracking_fts WHERE tracking_fts MATCH 'authentication';
```

The database is the source of truth. The commands are just convenient ways to read and write it.

## Documentation Database (Optional)

In addition to tracking sessions and decisions, proj can manage project documentation. This is separate from the tracking database.

### Why a Docs Database?

Documentation lives in a separate database because:
- **Different lifespan**: Sessions are transient; docs are persistent
- **Different audience**: Sessions help AI assistants; docs help everyone
- **Different queries**: "What happened?" vs "How does this work?"

### What Gets Documented

The docs database can hold:
- **Architecture** - How the system is organized
- **Components** - Individual parts and their responsibilities
- **Terminology** - What your project-specific terms mean
- **Anything else** - Whatever helps understand the codebase

### Generated vs Manual

Documentation can be:
- **Generated** - Created by analyzing source code (Rust, Python, TypeScript, Go)
- **Imported** - Brought in from existing markdown files
- **Manual** - Written directly

proj tracks which sections are generated vs manual, so you know what's safe to regenerate.

### Staleness Detection

When you generate docs from source code, proj remembers which files were analyzed. If those files change, `proj docs status` warns you:

```
Documentation Status
  Created: 2026-01-24
  Sections: 12

⚠ Source has changed since last generation
  Modified: src/auth.rs, src/database.rs

Run 'proj docs refresh' to update generated sections
```

This keeps your docs in sync with your code.

### Full-Text Search

Both databases support full-text search:

```bash
# Search tracking data (sessions, decisions, tasks)
proj context "authentication"

# Search documentation
proj docs search "authentication"
```

This makes it easy for AI assistants to find relevant context quickly.
