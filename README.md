# proj

Your memory for AI-assisted work.

## What It Does

When you work with AI assistants, a lot happens: decisions get made, tasks pile up, things get done. Then the session ends, and you forget half of it.

**proj** remembers for you. It works for any type of project folder - software development, research, documentation, planning, or anything else you're working on with AI assistance. It tracks:
- What you decided and why
- What tasks are pending
- What's blocking you
- Where you left off
- Project documentation (architecture, API, concepts)

Next session, your AI assistant runs `proj status` automatically and picks up exactly where you stopped.

**Auto-updates:** proj keeps itself current automatically. When a new version is available, it downloads in the background and applies on your next command. No manual updating needed.

## Quick Start

```bash
# Install (pick one)
npx create-aiproj              # npm - downloads binary, no Rust needed
cargo install aiproject        # crates.io - compiles from source
brew install aiproject         # Homebrew (macOS/Linux)

# In any project directory
proj init

# Optional: enable automatic session tracking
proj shell install
```

The init wizard walks you through:
1. **Project type** - Software (Rust, Python, etc.), documentation, writing, or other
2. **Tracking database** - For sessions, decisions, tasks, blockers
3. **Documentation database** - Optional project docs with search
4. **Auto-commit** - Optionally commit changes when sessions end (git repos)
5. **AGENTS.md rules** - So AI assistants use proj automatically

After init, AI assistants will run `proj status` at the start of each conversation and pick up where you left off.

**Shell hook (optional):** `proj shell install` adds a hook so sessions start automatically when you cd into any tracked project. Run once and forget about it.

## The Simple Version

| When | Command |
|------|---------|
| Start working | `proj status` |
| Done for the day | `proj session end "summary"` |
| Forgot to end? | No problem - auto-closes after 8 hours |

## Does It Actually Help?

Yes. We ran a controlled test comparing AI agents working with and without proj:

| Metric | With proj | Without proj | Improvement |
|--------|-----------|--------------|-------------|
| Files read | 11 | 34 | **68% reduction** |
| Token usage | ~10K | ~20K | **50% reduction** |
| Context recovery | 100% | 0% | **Critical** |

The biggest finding: without tracking, AI agents literally cannot recover what the previous session was working on. With proj, they pick up exactly where you left off.

**[Read the full case study (PDF)](docs/CASE_STUDY.pdf)** | **[Markdown version](docs/CASE_STUDY.md)**

## Documentation

- **[Getting Started](docs/getting-started.md)** - First-time setup, your first project
- **[Concepts](docs/concepts.md)** - What are sessions, decisions, tasks?
- **[Command Reference](docs/manual.md)** - Every command explained
- **[Cheat Sheet](docs/cheatsheet.md)** - One-page quick reference
- **[Changelog](CHANGELOG.md)** - Version history and release notes

## Documentation Database (New in v1.6.0)

proj can also manage project documentation with full-text search:

```bash
# Set up docs (also available during init)
proj docs init

# Four options:
# - Generate: Analyze source code (Rust, Python, TypeScript, Go)
# - Import: Import existing markdown files
# - New Project: Answer questions to create skeleton docs
# - Skip: Set up later

# Search your docs
proj docs search "authentication"

# Show table of contents
proj docs show

# Add terminology
proj docs term add "API" --definition "Application Programming Interface"
```

The docs database supports staleness detection - it warns you when source files change so you can refresh generated documentation.

## For AI Assistants

When you run `proj init`, it automatically adds session management rules to your global AGENTS.md file. This tells AI assistants (Claude, Gemini, Codex) to:

1. Run `proj status` at the start of every conversation
2. Log decisions, blockers, and tasks as you work
3. End sessions with summaries

AI assistants can log things as you work:

```bash
proj log decision "database" "Using SQLite" "Simple, portable"
proj log blocker "Need API keys"
proj task add "Implement auth" --priority high
```

Or query the database directly:

```sql
SELECT topic, decision FROM decisions WHERE status = 'active';
```

Database location: `.tracking/tracking.db`

## Installation

### npm (Easiest - No Rust Required)

```bash
npx create-aiproj
```

Downloads a pre-built binary for your platform. Just needs Node.js.

### From crates.io

```bash
cargo install aiproject
```

Compiles from source. Requires Rust toolchain.

### Homebrew (macOS/Linux)

```bash
brew tap victorysightsound/tap
brew install aiproject
```

### Download Binary

Pre-built binaries for macOS (Intel & Apple Silicon), Linux, and Windows are available on the [Releases page](https://github.com/victorysightsound/aiproject/releases).

### Build from Source

```bash
git clone https://github.com/victorysightsound/aiproject
cd aiproject
cargo build --release
sudo cp target/release/proj /usr/local/bin/
```

## VS Code Extension

A VS Code extension integrates proj with GitHub Copilot for a visual, interactive experience.

**Install from VS Code Marketplace:**
1. Open VS Code → Extensions (Cmd+Shift+X)
2. Search "proj - AI Project Tracker"
3. Click Install

**Or search:** `victorysightsound.proj`

**Features:**
- **@proj chat participant** - Ask Copilot about your project: `@proj /status`, `@proj /tasks`
- **Automatic logging** - Copilot can log decisions, tasks, and blockers during conversation
- **Session notification** - See where you left off when opening a project
- **Status bar menu** - One-click access to view status, tasks, or end session
- **Auto-generate summaries** - Let Copilot write your session summary

**Requirements:**
- GitHub Copilot subscription
- proj CLI installed (see Installation above)
- Project initialized with `proj init`

**Quick example:**
```
You: "I decided to use Redis for caching"
Copilot: "Would you like me to log this decision?"
→ Click Allow → Decision saved to project history
```

See the [full VS Code documentation](vscode/README.md) for setup guide, all features, and troubleshooting.

## License

MIT
