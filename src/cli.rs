// CLI module - Full implementation in Task #3

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "proj")]
#[command(author = "John Deaton <john@victorysightsound.com>")]
#[command(version)]
#[command(about = "Project tracking and context management for AI-assisted development")]
pub struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize new project
    Init,
    /// Migrate existing project to proj format
    Migrate,
    /// Show current project status
    Status {
        #[arg(short, long)]
        quiet: bool,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long)]
        full: bool,
    },
    /// Detailed context for resuming work
    Resume {
        #[arg(long)]
        for_ai: bool,
    },
    /// Session management
    Session(SessionCommands),
    /// Log decisions/notes/blockers/questions
    Log(LogCommands),
    /// Task management
    Task(TaskCommands),
    /// Shortcut for 'task list'
    Tasks,
    /// Search decisions and notes
    Context {
        topic: String,
        #[arg(long)]
        ranked: bool,
    },
    /// Show changes since last status
    Delta,
    /// Compress old sessions
    Compress {
        #[arg(long)]
        auto: bool,
    },
    /// Clean up stale items
    Cleanup {
        #[arg(long)]
        auto: bool,
        #[arg(long, default_value = "30")]
        days: u32,
    },
    /// Upgrade database schema
    Upgrade {
        #[arg(long)]
        info: bool,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        auto: bool,
    },
    /// Register project in global registry
    Register,
    /// List registered projects
    Registered,
    /// Overview of all projects
    Dashboard,
    /// Generate AI context snapshot
    Snapshot,
    /// Export session history
    Export {
        #[arg(long, default_value = "md")]
        format: String,
    },
    /// Manual backup
    Backup,
    /// Verify database integrity
    Check,
    /// Add extension tables
    Extend {
        #[arg(long = "type")]
        extension_type: String,
    },
    /// Archive completed project
    Archive,
    /// Check for updates
    Update,
    /// Release a new version (maintainer only)
    Release {
        /// Version to release (e.g., 1.4.0) - skips version selection prompt
        version: Option<String>,
        /// Check release status and update formulas
        #[arg(long)]
        check: bool,
    },
    /// Rollback a release (delete tag and GitHub release)
    Rollback {
        /// Version to rollback (defaults to latest)
        version: Option<String>,
    },
}

#[derive(Parser)]
pub struct SessionCommands {
    #[command(subcommand)]
    pub command: SessionSubcommand,
}

#[derive(Subcommand)]
pub enum SessionSubcommand {
    /// Start new session
    Start,
    /// End session with summary (1-3 sentences describing what was accomplished)
    End {
        /// What was accomplished this session (be specific, not generic)
        summary: String,
    },
    /// List recent sessions
    List,
}

#[derive(Parser)]
pub struct LogCommands {
    #[command(subcommand)]
    pub command: LogSubcommand,
}

#[derive(Subcommand)]
pub enum LogSubcommand {
    /// Log a decision
    Decision {
        topic: String,
        decision: String,
        rationale: Option<String>,
    },
    /// Log a note
    Note {
        category: String,
        title: String,
        content: String,
    },
    /// Log a blocker
    Blocker { description: String },
    /// Log a question
    Question {
        question: String,
        context: Option<String>,
    },
}

#[derive(Parser)]
pub struct TaskCommands {
    #[command(subcommand)]
    pub command: TaskSubcommand,
}

#[derive(Subcommand)]
pub enum TaskSubcommand {
    /// Add a new task
    Add {
        description: String,
        #[arg(long, default_value = "normal")]
        priority: String,
    },
    /// Update an existing task
    Update {
        id: i64,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        blocked_by: Option<String>,
    },
    /// List tasks
    List,
}
