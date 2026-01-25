# proj

Your memory for AI coding sessions.

## What It Does

When you work with AI assistants on code, a lot happens: decisions get made, tasks pile up, bugs get fixed. Then the session ends, and you forget half of it.

**proj** remembers for you. It tracks:
- What you decided and why
- What tasks are pending
- What's blocking you
- Where you left off

Next session, your AI assistant runs `proj status` automatically and picks up exactly where you stopped.

## Quick Start

```bash
# Install (see Installation section for options)
cargo install --path .

# In any project directory
proj init              # First time setup (interactive)
```

That's it. After init, proj automatically:
- Adds session rules to your global AGENTS.md (for Claude, Gemini, etc.)
- AI assistants will run `proj status` at the start of each conversation
- Optionally commits your changes when you end a session

Everything else happens automatically.

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

### Homebrew (macOS/Linux)

```bash
brew tap victorysightsound/tap
brew install aiproject
```

### From Source

```bash
git clone https://github.com/victorysightsound/aiproject
cd aiproject
cargo build --release
sudo cp target/release/proj /usr/local/bin/
```

### Download Binary

Pre-built binaries for macOS, Linux, and Windows are available on the [Releases page](https://github.com/victorysightsound/aiproject/releases).

## License

MIT
