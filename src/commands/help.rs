// Help command - show all commands

use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!(
        "
{} - Project Tracking Tool

{}
  proj init              Initialize new project (interactive)
  proj migrate           Convert existing project to proj format

{} (Tiered Verbosity)
  proj status            Minimal context (~50 tokens)
  proj status -q         Micro context (~10 tokens) - one line
  proj status -v         Working context (~200 tokens)
  proj status --full     Full context (~500+ tokens)
  proj resume            Detailed context for resuming work
  proj resume --for-ai   Compact JSON output for AI
  proj context <topic>   Search decisions/notes about a topic
  proj snapshot          Generate AI context snapshot (JSON)

{} (Token Optimization)
  proj delta             Show only changes since last check
  proj compress          Compress old sessions into summaries
  proj compress --auto   Auto-compress without prompts
  proj cleanup           Interactive review of stale items
  proj cleanup --auto    Auto-archive stale items
  proj cleanup --days N  Set staleness threshold (default: 30)

{}
  proj log decision <topic> <decision> [rationale]
  proj log note <category> <title> <content>
  proj log blocker <description>
  proj log question <question> [context]

{}
  proj task add <description> [--priority high]
  proj task update <id> --status <status>
  proj task list
  proj tasks             (shortcut for task list)

{}
  proj session start     Start new session explicitly
  proj session end <summary>

{}
  proj register          Add current project to global registry
  proj registered        List all registered projects
  proj dashboard         Multi-project overview

{}
  proj upgrade           Upgrade current project schema
  proj upgrade --info    Preview upgrade without applying
  proj upgrade --all     Upgrade all registered projects
  proj backup            Manual backup of tracking database
  proj check             Verify database integrity
  proj archive           Archive a completed project
  proj export --format md|json   Export session history

{}
  proj help              Show this help message

{}
  {}    Active session indicator
  {}    Completed/passing
  {}    Warning/needs attention
  {}    Error/missing

For more information: https://github.com/victorysightsound/proj
",
        "proj".bold(),
        "INITIALIZATION".cyan().bold(),
        "STATUS & CONTEXT".cyan().bold(),
        "EFFICIENCY".cyan().bold(),
        "LOGGING".cyan().bold(),
        "TASKS".cyan().bold(),
        "SESSIONS".cyan().bold(),
        "MULTI-PROJECT".cyan().bold(),
        "UTILITIES".cyan().bold(),
        "HELP".cyan().bold(),
        "STATUS INDICATORS".cyan().bold(),
        "●".green(),
        "✓".green(),
        "⚠".yellow(),
        "✗".red()
    );

    Ok(())
}
