# proj-rs Agent Notes

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with arguments
cargo run -- status
cargo run -- help
```

## Project Structure

```
proj-rs/
├── Cargo.toml          # Dependencies and package config
├── src/
│   ├── main.rs         # Entry point and command dispatch
│   ├── cli.rs          # CLI definitions (clap derive)
│   ├── commands/       # Command implementations
│   │   ├── mod.rs
│   │   ├── status.rs
│   │   ├── session.rs
│   │   └── ...
│   ├── config.rs       # ProjectConfig, Registry models
│   ├── database.rs     # SQLite connection helpers
│   ├── models.rs       # Data models (Session, Task, etc.)
│   ├── paths.rs        # Path utilities
│   └── schema.rs       # SQL schema constants
```

## Gotchas

1. **Module system**: All modules referenced in main.rs must exist for compilation
2. **Dead code warnings**: Expected during foundation phase; modules used in later phases
3. **rusqlite bundled**: Uses bundled SQLite, no system dependency needed

## Testing Patterns

(To be added as tests are written)
