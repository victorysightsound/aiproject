# proj-rs Agent Notes

## Current State (2026-01-21)

**Phases 1-5 complete (v0.0.1-v0.0.5)**

Next task: Phase 6, Task #43 - Implement cmd_delta (snapshot comparison)

## Build Commands

```bash
cd /Users/johndeaton/projects/global/tools/proj/proj-rs
cargo build        # Development build
cargo run -- help  # Test CLI
```

## Project Structure

```
proj-rs/
├── Cargo.toml
├── src/
│   ├── main.rs         # Entry point, command dispatch
│   ├── cli.rs          # CLI definitions (clap derive)
│   ├── commands/       # Command implementations
│   │   ├── status.rs   # DONE - all 4 tiers
│   │   ├── session.rs  # DONE - start/end/list
│   │   ├── log.rs      # DONE - decision/note/blocker/question
│   │   ├── task.rs     # DONE - add/update/list
│   │   ├── resume.rs   # DONE - human + JSON output
│   │   ├── context.rs  # DONE - search + ranked
│   │   ├── delta.rs    # STUB - needs implementation
│   │   ├── compress.rs # STUB
│   │   ├── cleanup.rs  # STUB
│   │   └── ...         # Other stubs
│   ├── config.rs       # ProjectConfig, Registry models
│   ├── database.rs     # SQLite connection helpers
│   ├── models.rs       # Data models
│   ├── paths.rs        # Path utilities
│   ├── schema.rs       # SQL schema constants
│   └── session.rs      # Session management functions
```

## Key Files for Resuming

- **fix_plan.md** - Task list with current focus
- **specs/*.md** - Detailed specifications for each command
- **PROMPT.md** - DIAL execution instructions

## Remaining Phases

- Phase 6: delta, compress, cleanup (efficiency)
- Phase 7: register, registered, dashboard (multi-project)
- Phase 8: upgrade system (migrations)
- Phase 9: init, migrate, extend, export, backup, check, archive, snapshot
- Phase 10: colors, --no-color, CHANGELOG, README, CI, release

## Git Tags

v0.0.1 through v0.0.5 created for recovery points.
