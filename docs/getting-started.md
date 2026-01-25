# Getting Started with proj

This guide walks you through setting up proj for the first time.

## What You Need

- A Mac, Linux, or Windows computer with a terminal
- Rust installed (for building from source)
- A project folder you want to track

## Installation

### Option 1: Build from Source

Open your terminal and run:

```bash
# Clone the repository
git clone https://github.com/victorysightsound/aiproject
cd aiproject

# Build it
cargo build --release

# Install it (makes 'proj' available everywhere)
sudo cp target/release/proj /usr/local/bin/
```

### Verify It Worked

```bash
proj --version
```

You should see the current version (e.g., `proj 1.3.0`).

## Your First Project

### Step 1: Go to Your Project Folder

```bash
cd ~/projects/my-project
```

### Step 2: Initialize proj

```bash
proj init
```

This starts an interactive setup. It will ask you:
1. What type of project is this? (rust, python, javascript, etc.)
2. What's the project name?
3. A short description (optional)
4. If it's a git repo: Enable auto-commit on session end? (optional)

**What happens during init:**
- Creates `.tracking/` folder with config and database
- Registers project in global registry
- Adds session rules to your global AGENTS.md (so AI assistants automatically run `proj status`)

**Note:** `proj init` requires a terminal. Run it directly, not through an AI assistant.

### Step 3: Check That It Worked

```bash
proj status
```

You should see something like:

```
============================================================
FULL PROJECT CONTEXT
============================================================

Project: my-project
Type: rust
Description: My awesome project
Schema Version: 1.2

----------------------------------------
CURRENT SESSION #1
Started: 2026-01-24 10:30:00

----------------------------------------
BLOCKERS:
  (none)

----------------------------------------
TASKS:
  (none)
```

That's it! Your project is now tracked.

## The Daily Workflow

### Starting Work

Every time you start working on your project:

```bash
proj status
```

This does two things:
1. Shows you where you left off (tasks, blockers, recent decisions)
2. Starts a new session (or resumes if you were recently working)

### During Work

As you work with an AI assistant, it can log things:

```bash
# Log a decision
proj log decision "auth" "Using JWT tokens" "Industry standard, stateless"

# Add a task
proj task add "Write login endpoint" --priority high

# Note something blocking you
proj log blocker "Waiting for API credentials"
```

You don't have to run these yourself - your AI assistant handles this.

### Ending Work

When you're done for the day:

```bash
proj session end "Added user authentication with JWT tokens. Fixed login bug where tokens weren't refreshing properly."
```

Write summaries that answer "what was accomplished?" - 1-3 substantive sentences work best. Avoid generic summaries like "worked on code" that don't help when resuming later.

### Auto-Commit (Optional)

If you enabled auto-commit during `proj init`, ending a session can also create a git commit:

```
$ proj session end "Fixed authentication bug"
✓ Session #5 ended. Summary: Fixed authentication bug
Commit changes with session summary? [Y/n] y
  ✓ Committed: [proj] Fixed authentication bug
```

Two modes are available:
- **Prompt** (default): Asks you each time
- **Auto**: Commits silently without asking

You can change this in `.tracking/config.json`.

### What If You Forget?

No problem. If you don't end your session and come back the next day (8+ hours later), proj automatically closes the old session and starts a new one:

```
⚠ Previous session #3 was stale (8+ hours). Auto-closed.
✓ Started new session #4
```

You don't lose any data. The old session just gets marked as "(auto-closed)".

## What Gets Tracked

proj creates a `.tracking` folder in your project:

```
my-project/
├── .tracking/
│   ├── config.json    # Project settings
│   └── tracking.db    # SQLite database with all your data
├── src/
└── ...
```

Inside `tracking.db`:
- **Sessions** - When you worked, summaries of what you did
- **Decisions** - What you decided and why
- **Tasks** - What needs to be done
- **Blockers** - What's in your way
- **Notes** - Anything else worth remembering

## Next Steps

- Read [Concepts](concepts.md) to understand sessions, decisions, and tasks
- Check the [Command Reference](manual.md) for all available commands
- Print the [Cheat Sheet](cheatsheet.md) for quick reference

## Troubleshooting

### "Not in a proj-tracked project"

You're not in a folder with proj initialized. Either:
- Run `proj init` to set it up
- `cd` to a folder that already has `.tracking/`

### "proj: command not found"

The binary isn't in your PATH. Make sure you copied it:
```bash
sudo cp target/release/proj /usr/local/bin/
```

### "Permission denied"

On Mac/Linux, you might need `sudo`:
```bash
sudo cp target/release/proj /usr/local/bin/
```
