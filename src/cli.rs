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
    Init {
        /// Directory to initialize (defaults to current directory, creates if doesn't exist)
        #[arg(long)]
        path: Option<String>,
        /// Project name (defaults to directory name)
        #[arg(long)]
        name: Option<String>,
        /// Project type: rust, python, javascript, web, documentation, other
        #[arg(long = "type")]
        project_type: Option<String>,
        /// Project description
        #[arg(long)]
        description: Option<String>,
        /// Skip documentation setup
        #[arg(long)]
        skip_docs: bool,
        /// Generate docs from source analysis
        #[arg(long)]
        docs_generate: bool,
        /// Import docs from markdown files
        #[arg(long)]
        docs_import: bool,
        /// Create skeleton documentation
        #[arg(long)]
        docs_new: bool,
        /// Documentation type: architecture, framework, guide, api, spec
        #[arg(long, default_value = "architecture")]
        docs_type: String,
        /// Enable auto-commit on session end (git repos only)
        #[arg(long)]
        auto_commit: bool,
        /// Auto-commit mode: prompt or auto
        #[arg(long, default_value = "prompt")]
        commit_mode: String,
        /// Skip AGENTS.md setup
        #[arg(long)]
        no_agents: bool,
        /// Install shell hook for automatic session tracking (non-interactive)
        #[arg(long)]
        shell_hook: bool,
    },
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
    /// Enter project - silent if session exists, shows context if new session
    Enter,
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
    /// Rollback a release or restore schema from backup
    Rollback {
        /// Version to rollback (defaults to latest release)
        version: Option<String>,
        /// Restore project schema from backup instead of release rollback
        #[arg(long)]
        schema: bool,
        /// List available schema backups
        #[arg(long)]
        list: bool,
    },
    /// Shell integration for automatic session tracking
    Shell(ShellCommands),
    /// Uninstall proj from projects
    Uninstall {
        /// Remove shell hook only, keep project data
        #[arg(long)]
        shell: bool,
        /// Remove .tracking/ from current project only
        #[arg(long)]
        project: bool,
        /// Remove shell hook + .tracking/ from ALL registered projects
        #[arg(long)]
        all: bool,
    },
    /// Project documentation database
    Docs(DocsCommands),
}

#[derive(Parser)]
pub struct ShellCommands {
    #[command(subcommand)]
    pub command: ShellSubcommand,
}

#[derive(Subcommand)]
pub enum ShellSubcommand {
    /// Install shell hook for automatic session tracking
    Install,
    /// Remove shell hook
    Uninstall,
    /// Show shell integration status
    Status,
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
        /// Force end even if no activity was logged
        #[arg(long)]
        force: bool,
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

#[derive(Parser)]
pub struct DocsCommands {
    #[command(subcommand)]
    pub command: DocsSubcommand,
}

#[derive(Subcommand)]
pub enum DocsSubcommand {
    /// Initialize documentation database
    Init {
        /// Generate from source analysis (non-interactive)
        #[arg(long)]
        generate: bool,
        /// Import from markdown files (non-interactive)
        #[arg(long)]
        import: bool,
        /// Create skeleton documentation (non-interactive)
        #[arg(long)]
        new: bool,
        /// Documentation type: architecture, framework, guide, api, spec
        #[arg(long, default_value = "architecture")]
        doc_type: String,
        /// Project name (defaults to directory name)
        #[arg(long)]
        name: Option<String>,
        /// Project description (for --new mode)
        #[arg(long)]
        description: Option<String>,
    },
    /// Show documentation database status
    Status,
    /// Refresh documentation from source analysis
    Refresh {
        /// Force refresh all sections, including manually edited ones
        #[arg(long)]
        force: bool,
    },
    /// Search documentation
    Search {
        /// Search query
        query: String,
    },
    /// Export documentation
    Export {
        /// Output format (md, html)
        #[arg(long, default_value = "md")]
        format: String,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<String>,
    },
    /// Display a section
    Show {
        /// Section ID to display (e.g., "1.2.3")
        section: Option<String>,
    },
    /// Manage terminology
    Term(DocsTermCommands),
}

#[derive(Parser)]
pub struct DocsTermCommands {
    #[command(subcommand)]
    pub command: DocsTermSubcommand,
}

#[derive(Subcommand)]
pub enum DocsTermSubcommand {
    /// Add a term to the glossary
    Add {
        /// The canonical form of the term
        term: String,
        /// Definition of the term
        #[arg(long)]
        def: String,
        /// Category (e.g., architecture, technology, workflow)
        #[arg(long)]
        category: Option<String>,
    },
    /// List all terms
    List,
    /// Search terms
    Search {
        /// Search query
        query: String,
    },
}
