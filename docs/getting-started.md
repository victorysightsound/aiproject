# Getting Started with proj

This guide walks you through setting up proj for the first time.

## What You Need

- A Mac, Linux, or Windows computer with a terminal
- A project folder you want to track
- (Optional) Rust toolchain if installing from source

## Installation

Pick whichever method works best for you:

### Option 1: npm (Easiest)

If you have Node.js installed:

```bash
npx create-aiproj
```

This downloads a pre-built binary for your platform. No Rust required.

### Option 2: From crates.io

If you have Rust installed:

```bash
cargo install aiproject
```

This compiles from source and installs the `proj` command globally.

### Option 3: Homebrew (macOS/Linux)

```bash
brew tap victorysightsound/tap
brew install aiproject
```

### Option 4: Download Binary

Pre-built binaries are available on the [Releases page](https://github.com/victorysightsound/aiproject/releases).

Download the right one for your system:
- **macOS Apple Silicon**: `proj-aarch64-apple-darwin.tar.gz`
- **macOS Intel**: `proj-x86_64-apple-darwin.tar.gz`
- **Linux ARM**: `proj-aarch64-unknown-linux-gnu.tar.gz`
- **Linux x86**: `proj-x86_64-unknown-linux-gnu.tar.gz`

Extract and move to your PATH:

```bash
tar -xzf proj-*.tar.gz
sudo mv proj /usr/local/bin/
```

### Option 5: Build from Source

```bash
git clone https://github.com/victorysightsound/aiproject
cd aiproject
cargo build --release
sudo cp target/release/proj /usr/local/bin/
```

### Verify Installation

```bash
proj --version
```

You should see the current version (e.g., `proj 1.7.0`).

### Optional: Enable Automatic Session Tracking

After installation, you can make session tracking completely automatic:

```bash
proj shell install
```

This adds a shell hook that runs `proj enter` whenever you cd into a tracked project. Sessions start silently when one exists, or show full context when starting fresh. Run this once and forget about it.

## Your First Project

### Step 1: Go to Your Project Folder

```bash
cd ~/projects/my-project
```

### Step 2: Initialize proj

```bash
proj init
```

This starts an interactive setup wizard. It walks you through:

1. **Project info** - Name, type (rust, python, javascript, documentation, other), description
2. **Tracking database** - Creates `.tracking/` folder with session tracking
3. **Documentation database** - Optional project docs with full-text search:
   - **Skip** - Set up documentation later
   - **Generate** - Analyze your source code (Rust, Python, TypeScript, Go) and create docs automatically
   - **Import** - Import existing markdown files into the docs database
   - **New Project** - Answer questions to create a documentation skeleton
4. **Auto-commit** (git repos only) - Optionally commit changes when sessions end
5. **AGENTS.md rules** - Adds session rules so AI assistants automatically use proj

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
Schema Version: 1.3

----------------------------------------
CURRENT SESSION #1
Started: 2026-01-25 10:30:00

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

All of this data is full-text searchable using `proj context <query>`.

## Documentation Database (Optional)

If you chose to set up documentation during init (or want to add it later), proj creates a separate docs database:

```
my-project/
├── .tracking/
│   └── tracking.db        # Session/decision tracking
├── my-project_docs.db     # Documentation database
└── ...
```

Useful commands:

```bash
# Show documentation status
proj docs status

# Search documentation
proj docs search "authentication"

# Show table of contents
proj docs show

# Refresh if source files changed
proj docs refresh

# Add terminology
proj docs term add "API" --definition "Application Programming Interface"
```

## Next Steps

- Read [Concepts](concepts.md) to understand sessions, decisions, and tasks
- Check the [Command Reference](manual.md) for all available commands
- Print the [Cheat Sheet](cheatsheet.md) for quick reference

## Staying Updated

proj updates itself automatically. When a new version is available:
1. It downloads the update in the background
2. On your next command, it applies the update
3. You see a brief message: "Updated proj 1.7.0 → 1.7.1"

That's it - no action needed on your part. If auto-update fails for any reason (permissions, network issues), running `proj update` will show manual update instructions.

## Troubleshooting

### "Not in a proj-tracked project"

You're not in a folder with proj initialized. Either:
- Run `proj init` to set it up
- `cd` to a folder that already has `.tracking/`

### "proj: command not found"

The binary isn't in your PATH. Depending on how you installed:

**crates.io**: Make sure `~/.cargo/bin` is in your PATH
**Homebrew**: Run `brew link aiproject`
**Manual**: Run `sudo cp proj /usr/local/bin/`

### "Permission denied"

On Mac/Linux, you might need `sudo`:
```bash
sudo cp proj /usr/local/bin/
```

### "FTS5 not available" or search not working

If you upgraded from an older version, run:
```bash
proj migrate
```

This adds the full-text search tables if they're missing.
