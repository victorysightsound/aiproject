// proj - Project tracking and context management for AI-assisted development

mod auto_update;
mod cli;
mod commands;
mod config;
mod database;
mod docs_db;
mod models;
mod paths;
mod schema;
mod schema_docs;
mod session;
mod source_analyzer;

use anyhow::Result;
use atty::Stream;
use clap::Parser;
use cli::{Cli, Commands};
use colored::control;

/// Version constants
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SCHEMA_VERSION: &str = "1.3";
pub const MIN_SCHEMA_VERSION: &str = "1.0";

fn main() -> Result<()> {
    // Check for pending updates FIRST (before parsing args)
    // If update is applied, this will re-exec and not return
    if let Ok(true) = auto_update::check_and_apply_pending() {
        // Should not reach here (process exits after re-exec)
        return Ok(());
    }

    let cli = Cli::parse();

    // Configure color output:
    // 1. Disable if --no-color flag is set
    // 2. Disable if not a TTY (piped/redirected output)
    // 3. Respect NO_COLOR environment variable (handled by colored crate)
    if cli.no_color || !atty::is(Stream::Stdout) {
        control::set_override(false);
    }

    match cli.command {
        Commands::Init {
            path,
            name,
            project_type,
            description,
            skip_docs,
            docs_generate,
            docs_import,
            docs_new,
            docs_type,
            auto_commit,
            commit_mode,
            no_agents,
            shell_hook,
        } => commands::init::run(
            path,
            name,
            project_type,
            description,
            skip_docs,
            docs_generate,
            docs_import,
            docs_new,
            docs_type,
            auto_commit,
            commit_mode,
            no_agents,
            shell_hook,
        ),
        Commands::Migrate => commands::migrate::run(),
        Commands::Status {
            quiet,
            verbose,
            full,
        } => commands::status::run(quiet, verbose, full),
        Commands::Enter => commands::enter::run(),
        Commands::Resume { for_ai } => commands::resume::run(for_ai),
        Commands::Session(cmd) => commands::session::run(cmd),
        Commands::Log(cmd) => commands::log::run(cmd),
        Commands::Task(cmd) => commands::task::run(cmd),
        Commands::Tasks => commands::task::list(),
        Commands::Context { topic, ranked } => commands::context::run(&topic, ranked),
        Commands::Delta => commands::delta::run(),
        Commands::Compress { auto } => commands::compress::run(auto),
        Commands::Cleanup { auto, days } => commands::cleanup::run(auto, days),
        Commands::Upgrade { info, all, auto } => commands::upgrade::run(info, all, auto),
        Commands::Register => commands::register::run(),
        Commands::Registered => commands::registered::run(),
        Commands::Dashboard => commands::dashboard::run(),
        Commands::Snapshot => commands::snapshot::run(),
        Commands::Export { format } => commands::export::run(format),
        Commands::Backup => commands::backup::run(),
        Commands::Check => commands::check::run(),
        Commands::Extend { extension_type } => commands::extend::run(extension_type),
        Commands::Archive => commands::archive::run(),
        Commands::Update => commands::update_check::run(),
        Commands::Release { version, check } => commands::release::run(version, check),
        Commands::Rollback {
            version,
            schema,
            list,
        } => commands::rollback::run(version, schema, list),
        Commands::Shell(cmd) => {
            use cli::ShellSubcommand;
            match cmd.command {
                ShellSubcommand::Install { force } => commands::shell::install(force),
                ShellSubcommand::Uninstall => commands::shell::uninstall(),
                ShellSubcommand::Status => commands::shell::status(),
            }
        }
        Commands::Uninstall {
            shell,
            project,
            all,
            force,
        } => commands::uninstall::run(shell, project, all, force),
        Commands::Docs(cmd) => commands::docs::run(cmd),
    }
}
