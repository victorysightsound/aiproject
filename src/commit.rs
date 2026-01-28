// Shared auto-commit logic - used by session end and task completion

use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

use crate::config::ProjectConfig;
use crate::paths::get_project_root;

/// Perform an auto-commit with the given message.
/// Returns Ok(true) if a commit was made, Ok(false) if skipped.
pub fn auto_commit(message: &str, config: &ProjectConfig) -> Result<bool> {
    // Check if we're in a git repo
    let project_root = get_project_root()?;
    if !project_root.join(".git").exists() {
        return Ok(false);
    }

    // Check if there are any changes to commit
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&project_root)
        .output()
        .with_context(|| "Failed to run git status")?;

    let has_changes = !status_output.stdout.is_empty();

    if !has_changes {
        println!("  {} No changes to commit", "ℹ".blue());
        return Ok(false);
    }

    // Determine if we should commit
    let should_commit = match config.auto_commit_mode.as_str() {
        "auto" => true,
        "prompt" | _ => {
            if atty::is(atty::Stream::Stdin) {
                Confirm::new()
                    .with_prompt("Commit changes?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            } else {
                println!("  {} Skipping commit (non-interactive)", "ℹ".blue());
                false
            }
        }
    };

    if !should_commit {
        return Ok(false);
    }

    // Stage all changes
    let add_result = Command::new("git")
        .args(["add", "-A"])
        .current_dir(&project_root)
        .output()
        .with_context(|| "Failed to run git add")?;

    if !add_result.status.success() {
        bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_result.stderr)
        );
    }

    // Create commit
    let commit_result = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(&project_root)
        .output()
        .with_context(|| "Failed to run git commit")?;

    if !commit_result.status.success() {
        let stderr = String::from_utf8_lossy(&commit_result.stderr);
        if stderr.contains("nothing to commit") {
            println!("  {} No changes to commit", "ℹ".blue());
            return Ok(false);
        }
        bail!("git commit failed: {}", stderr);
    }

    println!("  {} Committed: {}", "✓".green(), message);
    Ok(true)
}
