// Shell integration commands - install/uninstall shell hooks for automatic tracking

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

/// Shell hook marker - used to detect if hook is already installed
const HOOK_MARKER_START: &str = "# >>> proj shell integration >>>";
const HOOK_MARKER_END: &str = "# <<< proj shell integration <<<";

/// Zsh hook code - uses precmd (every prompt) and chpwd (directory change)
const ZSH_HOOK: &str = r#"# >>> proj shell integration >>>
# Runs proj enter on directory change, checks for stale sessions on every prompt
_proj_auto_enter() {
    if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
        proj enter
    fi
}
_proj_check_stale() {
    if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
        proj shell check 2>/dev/null
    fi
}
# Run enter on directory change
if [[ -z "${chpwd_functions[(r)_proj_auto_enter]}" ]]; then
    chpwd_functions+=(_proj_auto_enter)
fi
# Check for stale session on every prompt
if [[ -z "${precmd_functions[(r)_proj_check_stale]}" ]]; then
    precmd_functions+=(_proj_check_stale)
fi
# <<< proj shell integration <<<"#;

/// Bash hook code - uses PROMPT_COMMAND for both directory change and stale check
const BASH_HOOK: &str = r#"# >>> proj shell integration >>>
# Runs proj enter on directory change, checks for stale sessions on every prompt
_proj_last_dir=""
_proj_prompt_hook() {
    # Check for stale session on every prompt
    if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
        proj shell check 2>/dev/null
    fi
    # Run enter on directory change
    if [[ "$PWD" != "$_proj_last_dir" ]]; then
        _proj_last_dir="$PWD"
        if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
            proj enter
        fi
    fi
}
# Add to PROMPT_COMMAND if not already present
if [[ "$PROMPT_COMMAND" != *"_proj_prompt_hook"* ]]; then
    PROMPT_COMMAND="_proj_prompt_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
# <<< proj shell integration <<<"#;

pub fn install(force: bool) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    // Check what shells are available
    let zsh_exists = zshrc.exists();
    let bash_exists = bashrc.exists();

    if !zsh_exists && !bash_exists {
        println!(
            "{} No shell config files found (.zshrc or .bashrc)",
            "⚠".yellow()
        );
        println!("Please create ~/.zshrc or ~/.bashrc and run this command again.");
        return Ok(());
    }

    // Check if already installed
    let zsh_installed = zsh_exists && is_hook_installed(&zshrc)?;
    let bash_installed = bash_exists && is_hook_installed(&bashrc)?;

    if zsh_installed || bash_installed {
        println!("{} Shell integration already installed:", "✓".green());
        if zsh_installed {
            println!("  • ~/.zshrc");
        }
        if bash_installed {
            println!("  • ~/.bashrc");
        }
        if !force {
            println!();
            println!("To reinstall, first run: proj shell uninstall");
        }
        return Ok(());
    }

    if !force {
        println!("{}", "Shell Integration Setup".bold());
        println!();

        // Show what we're about to do
        println!("This will add a hook to your shell configuration that:");
        println!("  • Detects when you cd into a directory with .tracking/");
        println!("  • Automatically runs 'proj enter' to start/continue session");
        println!("  • Shows project context only when starting a new session");
        println!();
    }

    let mut installed_shells = Vec::new();

    // Install for zsh
    if zsh_exists && !zsh_installed {
        let should_install = force
            || Confirm::new()
                .with_prompt("Install for zsh (~/.zshrc)?")
                .default(true)
                .interact()?;

        if should_install {
            install_hook(&zshrc, ZSH_HOOK)?;
            installed_shells.push("zsh");
            println!("{} Installed in ~/.zshrc", "✓".green());
        }
    }

    // Install for bash
    if bash_exists && !bash_installed {
        let should_install = force
            || Confirm::new()
                .with_prompt("Install for bash (~/.bashrc)?")
                .default(true)
                .interact()?;

        if should_install {
            install_hook(&bashrc, BASH_HOOK)?;
            installed_shells.push("bash");
            println!("{} Installed in ~/.bashrc", "✓".green());
        }
    }

    if installed_shells.is_empty() {
        println!("No changes made.");
        return Ok(());
    }

    println!();
    println!("{}", "Setup complete!".bold().green());
    println!();
    println!("To activate, either:");
    println!("  • Open a new terminal window, or");
    println!("  • Run: source ~/.{}rc", installed_shells[0]);
    println!();
    println!("Then cd into any project with proj tracking - session will start automatically.");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    println!("{}", "Removing Shell Integration".bold());
    println!();

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    let mut removed = false;

    // Remove from zsh
    if zshrc.exists() && is_hook_installed(&zshrc)? {
        remove_hook(&zshrc)?;
        println!("{} Removed from ~/.zshrc", "✓".green());
        removed = true;
    }

    // Remove from bash
    if bashrc.exists() && is_hook_installed(&bashrc)? {
        remove_hook(&bashrc)?;
        println!("{} Removed from ~/.bashrc", "✓".green());
        removed = true;
    }

    if removed {
        println!();
        println!("Shell integration removed. Open a new terminal for changes to take effect.");
    } else {
        println!("Shell integration was not installed.");
    }

    Ok(())
}

pub fn status() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    println!("{}", "Shell Integration Status".bold());
    println!();

    let zsh_installed = zshrc.exists() && is_hook_installed(&zshrc)?;
    let bash_installed = bashrc.exists() && is_hook_installed(&bashrc)?;

    if zsh_installed || bash_installed {
        println!("{} Shell integration is installed:", "✓".green());
        if zsh_installed {
            println!("  • ~/.zshrc");
        }
        if bash_installed {
            println!("  • ~/.bashrc");
        }
    } else {
        println!("{} Shell integration is not installed.", "○".white());
        println!();
        println!("Run 'proj shell install' to enable automatic session tracking.");
    }

    Ok(())
}

/// Check if hook is already installed in a file
fn is_hook_installed(path: &PathBuf) -> Result<bool> {
    let content = fs::read_to_string(path).unwrap_or_default();
    Ok(content.contains(HOOK_MARKER_START))
}

/// Install hook at the end of a shell config file
fn install_hook(path: &PathBuf, hook: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_default();

    // Add a newline if the file doesn't end with one
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(hook);
    content.push('\n');

    fs::write(path, content).with_context(|| format!("Failed to write to {:?}", path))?;
    Ok(())
}

/// Remove hook from a shell config file
fn remove_hook(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)?;

    // Find and remove the hook block
    let mut new_content = String::new();
    let mut in_hook_block = false;

    for line in content.lines() {
        if line.contains(HOOK_MARKER_START) {
            in_hook_block = true;
            continue;
        }
        if line.contains(HOOK_MARKER_END) {
            in_hook_block = false;
            continue;
        }
        if !in_hook_block {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Clean up any double blank lines
    while new_content.contains("\n\n\n") {
        new_content = new_content.replace("\n\n\n", "\n\n");
    }

    fs::write(path, new_content).with_context(|| format!("Failed to write to {:?}", path))?;
    Ok(())
}

/// Check for stale session - used by shell prompt hook
/// This runs on every prompt, so it must be fast and only print once per stale session
pub fn check() -> Result<()> {
    use crate::database::open_database;
    use crate::paths::get_tracking_db_path;
    use crate::session::get_active_session;
    use chrono::{Duration, Utc};

    // Quick exit if not in a proj directory
    let tracking_dir = std::path::Path::new(".tracking");
    if !tracking_dir.exists() {
        return Ok(());
    }

    // Open database
    let db_path = match get_tracking_db_path() {
        Ok(p) => p,
        Err(_) => return Ok(()), // Silent fail
    };
    let conn = match open_database(&db_path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // Silent fail
    };

    // Check for active session
    let session = match get_active_session(&conn) {
        Ok(Some(s)) => s,
        _ => return Ok(()), // No active session, nothing to warn about
    };

    // Check if session is stale (24+ hours)
    let stale_hours: i64 = 24;
    let now = Utc::now();
    let session_age = now - session.started_at;

    if session_age <= Duration::hours(stale_hours) {
        return Ok(()); // Session is fine, exit silently
    }

    // Session is stale - check if we've already warned
    let warned_marker = tracking_dir.join(format!(".warned_stale_{}", session.session_id));
    if warned_marker.exists() {
        return Ok(()); // Already warned for this session
    }

    // Get last session summary for context
    let summary = session.summary.as_deref().unwrap_or("(no summary)");
    let hours_stale = session_age.num_hours();

    // Print warning
    eprintln!();
    eprintln!(
        "{} Session #{} expired (inactive {} hours)",
        "⚠ proj:".yellow().bold(),
        session.session_id,
        hours_stale
    );
    if summary != "(no summary)" && summary != "(auto-closed)" {
        eprintln!("  Last: \"{}\"", truncate_str(summary, 60));
    }
    eprintln!("  Run '{}' to start a new session.", "proj status".cyan());
    eprintln!();

    // Create marker so we don't warn again
    let _ = fs::write(&warned_marker, "");

    Ok(())
}

/// Truncate a string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Check if shell integration is installed (for use by other commands)
pub fn is_installed() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    is_hook_installed(&zshrc).unwrap_or(false) || is_hook_installed(&bashrc).unwrap_or(false)
}
